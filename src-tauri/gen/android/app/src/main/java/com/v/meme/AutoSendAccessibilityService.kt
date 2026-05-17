package com.v.meme

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class AutoSendAccessibilityService : AccessibilityService() {

    private val debounceMs = 3000L
    private var lastProcessedTime = 0L
    private var retryCount = 0
    private val maxRetries = 15
    private var handler: Handler? = null
    private var servicePollRunnable: Runnable? = null

    companion object {
        private const val TAG = "AutoSendA11y"
        private const val PREFS_NAME = "auto_send_prefs"
        private const val KEY_ENABLED = "auto_send_enabled"
        private const val SEND_FLOW_TIMEOUT_MS = 30000L
        private const val POLL_INTERVAL_MS = 800L

        private const val WECHAT_PACKAGE = "com.tencent.mm"
        private const val QQ_PACKAGE = "com.tencent.mobileqq"

        @Volatile
        var isRunning = false
            private set

        @Volatile
        var sendFlowActive = false
            private set

        @Volatile
        var inDialogContext = false
            private set

        @Volatile
        private var lastTargetWindowEvent = 0L
        private const val STALE_EVENT_TIMEOUT_MS = 1500L

        private var flowTimeoutHandler: Handler? = null
        private var flowTimeoutRunnable: Runnable? = null

        fun isEnabled(context: Context): Boolean {
            val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            return prefs.getBoolean(KEY_ENABLED, false)
        }

        fun setEnabled(context: Context, enabled: Boolean) {
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putBoolean(KEY_ENABLED, enabled)
                .apply()
        }

        fun activateSendFlow() {
            Log.d(TAG, "activateSendFlow called, setting sendFlowActive = true")
            sendFlowActive = true
            startFlowTimeout()
        }

        fun deactivateSendFlow() {
            sendFlowActive = false
            inDialogContext = false
            lastTargetWindowEvent = 0L
            Log.d(TAG, "deactivateSendFlow called, sendFlowActive = false")
            cancelFlowTimeout()
        }

        private fun startFlowTimeout() {
            cancelFlowTimeout()
            if (flowTimeoutHandler == null) {
                flowTimeoutHandler = Handler(Looper.getMainLooper())
            }
            flowTimeoutRunnable = Runnable {
                Log.d(TAG, "Send flow timeout (30s), deactivating")
                sendFlowActive = false
                inDialogContext = false
                lastTargetWindowEvent = 0L
            }
            flowTimeoutHandler?.postDelayed(flowTimeoutRunnable!!, SEND_FLOW_TIMEOUT_MS)
        }

        private fun cancelFlowTimeout() {
            flowTimeoutRunnable?.let { flowTimeoutHandler?.removeCallbacks(it) }
            flowTimeoutRunnable = null
        }
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        isRunning = true
        Log.d(TAG, "Service connected, starting continuous polling")

        val info = AccessibilityServiceInfo().apply {
            eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED or
                    AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            flags = AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS or
                    AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS
            notificationTimeout = 100
        }
        serviceInfo = info

        startContinuousPolling()
    }

    private fun startContinuousPolling() {
        handler = Handler(Looper.getMainLooper())
        servicePollRunnable = Runnable {
            try {
                if (sendFlowActive) {
                    Log.d(TAG, "Poll tick: sendFlowActive=true, retryCount=$retryCount")
                    val now = System.currentTimeMillis()
                    if (now - lastProcessedTime >= debounceMs) {
                        if (findAndClickSendButton()) {
                            Log.d(TAG, "Send button clicked via polling, send flow complete")
                            deactivateSendFlow()
                            lastProcessedTime = now
                            retryCount = 0
                        } else {
                            retryCount++
                            if (retryCount >= maxRetries) {
                                Log.d(TAG, "Max retries ($maxRetries) reached, deactivating send flow")
                                deactivateSendFlow()
                                retryCount = 0
                            }
                        }
                    }
                } else {
                    retryCount = 0
                }
            } catch (e: Exception) {
                Log.e(TAG, "Poll tick exception: ${e.message}", e)
                retryCount = 0
            }
            handler?.postDelayed(servicePollRunnable!!, POLL_INTERVAL_MS)
        }
        handler?.post(servicePollRunnable!!)
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event == null) return

        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        if (!prefs.getBoolean(KEY_ENABLED, false)) return

        if (!sendFlowActive) return

        val packageName = event.packageName?.toString() ?: return
        if (packageName != WECHAT_PACKAGE && packageName != QQ_PACKAGE) return

        val now = System.currentTimeMillis()
        if (now - lastProcessedTime < debounceMs) return

        when (event.eventType) {
            AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                val eventClassName = event.className?.toString() ?: ""
                Log.d(TAG, "Event window state changed: $eventClassName, package: $packageName")

                lastTargetWindowEvent = now

                val isDialogEvent = eventClassName.contains("Dialog") ||
                    eventClassName.contains("BottomSheet") || eventClassName.contains("Popup")

                val isWechatShareActivity = packageName == WECHAT_PACKAGE && (
                    eventClassName.contains("ShareImgUI") ||
                    eventClassName.contains("MsgRetransmitUI") ||
                    eventClassName.contains("MvvmContactListUI") ||
                    eventClassName.contains("HalfScreenTransparentActivity")
                )

                val isWechatMainChat = packageName == WECHAT_PACKAGE &&
                    eventClassName.contains("LauncherUI")

                if (isDialogEvent || isWechatShareActivity) {
                    if (!inDialogContext) {
                        inDialogContext = true
                        Log.d(TAG, "inDialogContext set to true")
                    }
                } else if (isWechatMainChat) {
                    if (inDialogContext) {
                        inDialogContext = false
                        Log.d(TAG, "inDialogContext set to false (main chat)")
                    }
                } else if (packageName == QQ_PACKAGE && !isDialogEvent) {
                    if (inDialogContext) {
                        inDialogContext = false
                        Log.d(TAG, "inDialogContext set to false")
                    }
                }
            }
            AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED -> {
                Log.d(TAG, "Event content changed: ${event.className}")
            }
        }
    }

    private fun findAndClickSendButton(): Boolean {
        if (!inDialogContext) {
            if (!checkDialogFromEventFallback()) {
                Log.d(TAG, "Not in dialog context, skipping send button search")
                return false
            }
        }

        if (System.currentTimeMillis() - lastTargetWindowEvent > STALE_EVENT_TIMEOUT_MS) {
            Log.d(TAG, "No window events from target app for ${STALE_EVENT_TIMEOUT_MS}ms, treating context as stale, clearing inDialogContext")
            inDialogContext = false
            return false
        }

        val root = rootInActiveWindow
        if (root == null) {
            Log.d(TAG, "rootInActiveWindow is null, cannot search")
            return false
        }

        try {
            Log.d(TAG, "Searching for send button in window... (package=${root.packageName})")
            dumpNodeTreeSafe(root, 0)

            val sendButton = findSendButtonInTree(root)
            if (sendButton != null) {
                val hasCancel = hasButtonWithTextInTree(root, "取消")
                if (!hasCancel) {
                    Log.d(TAG, "Found send button but no '取消' in tree, likely main chat, NOT clicking")
                    sendButton.recycle()
                    return false
                }
                lastProcessedTime = System.currentTimeMillis()
                Log.d(TAG, "Found send button, performing click")
                sendButton.performAction(AccessibilityNodeInfo.ACTION_CLICK)
                sendButton.recycle()
                return true
            }

            Log.d(TAG, "Send button not found")
            return false
        } catch (e: Exception) {
            Log.e(TAG, "Exception in findAndClickSendButton: ${e.message}", e)
            return false
        } finally {
            try { root.recycle() } catch (e: Exception) { Log.e(TAG, "Error recycling root: ${e.message}") }
        }
    }

    private fun checkDialogFromEventFallback(): Boolean {
        val root = rootInActiveWindow ?: return false
        try {
            val hasCancel = hasButtonWithTextInTree(root, "取消")
            val hasSend = hasButtonWithTextInTree(root, "发送")

            if (hasCancel && hasSend) {
                Log.d(TAG, "Fallback: tree has both '取消' and '发送', in share dialog context")
                inDialogContext = true
                return true
            }

            val hasDialogLike = hasDialogContainerInTree(root)
            if (hasDialogLike) {
                Log.d(TAG, "Fallback: tree contains dialog-like container, setting inDialogContext=true")
                inDialogContext = true
                return true
            }

            Log.d(TAG, "Fallback: no share dialog indicators found")
            return false
        } catch (e: Exception) {
            Log.e(TAG, "Exception in checkDialogFromEventFallback: ${e.message}")
            return false
        } finally {
            try { root.recycle() } catch (e: Exception) { Log.e(TAG, "Error recycling root: ${e.message}") }
        }
    }

    private fun hasButtonWithTextInTree(node: AccessibilityNodeInfo, targetText: String): Boolean {
        try {
            if (node.isVisibleToUser) {
                val text = node.text?.toString()?.trim()
                val contentDesc = node.contentDescription?.toString()?.trim()
                if (text == targetText || contentDesc == targetText) {
                    return true
                }
            }
            for (i in 0 until node.childCount) {
                val child = node.getChild(i) ?: continue
                val result = hasButtonWithTextInTree(child, targetText)
                if (result) {
                    child.recycle()
                    return true
                }
                child.recycle()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception in hasButtonWithTextInTree: ${e.message}")
        }
        return false
    }

    private fun hasDialogContainerInTree(node: AccessibilityNodeInfo): Boolean {
        try {
            val className = node.className?.toString() ?: ""
            if (node.isVisibleToUser && (className.contains("Dialog") ||
                    className.contains("BottomSheet") || className.contains("Popup"))) {
                return true
            }
            for (i in 0 until node.childCount) {
                val child = node.getChild(i) ?: continue
                val result = hasDialogContainerInTree(child)
                if (result) {
                    child.recycle()
                    return true
                }
                child.recycle()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception in hasDialogContainerInTree: ${e.message}")
        }
        return false
    }

    private fun findSendButtonInTree(node: AccessibilityNodeInfo): AccessibilityNodeInfo? {
        try {
            if (node.isVisibleToUser) {
                val text = node.text?.toString()?.trim()
                val contentDesc = node.contentDescription?.toString()?.trim()

                if ((text == "发送" || contentDesc == "发送") &&
                    (node.isClickable || node.parent?.isClickable == true)) {
                    if (node.isClickable) {
                        return AccessibilityNodeInfo.obtain(node)
                    }
                    val parent = node.parent
                    if (parent != null && parent.isClickable) {
                        val result = AccessibilityNodeInfo.obtain(parent)
                        parent.recycle()
                        return result
                    }
                    parent?.recycle()
                }

                if (text == "发送" && node.isEnabled) {
                    return AccessibilityNodeInfo.obtain(node)
                }
            }

            for (i in 0 until node.childCount) {
                val child = node.getChild(i) ?: continue
                val result = findSendButtonInTree(child)
                if (result != null) return result
                child.recycle()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception in findSendButtonInTree: ${e.message}")
        }
        return null
    }

    private fun dumpNodeTreeSafe(node: AccessibilityNodeInfo, depth: Int) {
        if (depth > 4) return
        try {
            if (!node.isVisibleToUser) return

            val text = node.text?.toString()?.let { " text='$it'" } ?: ""
            val desc = node.contentDescription?.toString()?.let { " desc='$it'" } ?: ""
            val clickable = if (node.isClickable) " [clickable]" else ""
            val className = node.className?.toString()?.substringAfterLast('.') ?: "?"

            if (text.isNotEmpty() || desc.isNotEmpty() || node.isClickable) {
                val indent = "  ".repeat(depth)
                Log.d(TAG, "${indent}$className$text$desc$clickable")
            }

            for (i in 0 until node.childCount) {
                val child = node.getChild(i) ?: continue
                dumpNodeTreeSafe(child, depth + 1)
                child.recycle()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Exception in dumpNodeTree: ${e.message}")
        }
    }

    override fun onInterrupt() {
        Log.d(TAG, "Service interrupted")
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
        sendFlowActive = false
        inDialogContext = false
        lastTargetWindowEvent = 0L
        cancelFlowTimeout()
        handler?.removeCallbacksAndMessages(null)
        handler = null
        servicePollRunnable = null
        retryCount = 0
        Log.d(TAG, "Service destroyed")
    }
}
