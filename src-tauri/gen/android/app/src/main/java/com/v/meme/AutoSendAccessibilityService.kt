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

    private val handler = Handler(Looper.getMainLooper())
    private var lastProcessedTime = 0L
    private val debounceMs = 3000L
    private var retryCount = 0
    private val maxRetries = 5

    companion object {
        private const val TAG = "AutoSendA11y"
        private const val PREFS_NAME = "auto_send_prefs"
        private const val KEY_ENABLED = "auto_send_enabled"

        private const val WECHAT_PACKAGE = "com.tencent.mm"
        private const val QQ_PACKAGE = "com.tencent.mobileqq"

        var isRunning = false
            private set

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
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        isRunning = true
        Log.d(TAG, "Service connected")

        val info = AccessibilityServiceInfo().apply {
            eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED or
                    AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            flags = AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS or
                    AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS
            notificationTimeout = 100
        }
        serviceInfo = info
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event == null) return

        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        if (!prefs.getBoolean(KEY_ENABLED, false)) return

        val packageName = event.packageName?.toString() ?: return
        if (packageName != WECHAT_PACKAGE && packageName != QQ_PACKAGE) return

        val now = System.currentTimeMillis()
        if (now - lastProcessedTime < debounceMs) return

        when (event.eventType) {
            AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                Log.d(TAG, "Window state changed: ${event.className}, package: $packageName")
                startSendButtonSearch()
            }
            AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED -> {
                val className = event.className?.toString() ?: ""
                if (className.contains("Dialog") || className.contains("BottomSheet") ||
                    className.contains("Popup") || className.contains("Send")) {
                    Log.d(TAG, "Content changed in dialog-like view: $className")
                    startSendButtonSearch()
                }
            }
        }
    }

    private fun startSendButtonSearch() {
        retryCount = 0
        handler.removeCallbacksAndMessages(null)
        attemptFindSendButton()
    }

    private fun attemptFindSendButton() {
        if (findAndClickSendButton()) {
            retryCount = 0
            return
        }

        if (retryCount < maxRetries) {
            retryCount++
            val delay = 300L * retryCount
            Log.d(TAG, "Send button not found, retry $retryCount/$maxRetries in ${delay}ms")
            handler.postDelayed({ attemptFindSendButton() }, delay)
        } else {
            Log.d(TAG, "Max retries reached, giving up")
            retryCount = 0
        }
    }

    private fun findAndClickSendButton(): Boolean {
        val root = rootInActiveWindow ?: return false

        try {
            Log.d(TAG, "Searching for send button in window...")
            dumpNodeTree(root, 0)

            val sendButton = findSendButtonFlexible(root)
            if (sendButton != null) {
                lastProcessedTime = System.currentTimeMillis()
                Log.d(TAG, "Found send button, performing click")
                sendButton.performAction(AccessibilityNodeInfo.ACTION_CLICK)
                sendButton.recycle()
                return true
            }

            Log.d(TAG, "Send button not found in this attempt")
            return false
        } finally {
            root.recycle()
        }
    }

    private fun findSendButtonFlexible(node: AccessibilityNodeInfo): AccessibilityNodeInfo? {
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
            val result = findSendButtonFlexible(child)
            if (result != null) return result
            child.recycle()
        }

        return null
    }

    private fun dumpNodeTree(node: AccessibilityNodeInfo, depth: Int) {
        if (depth > 4) return
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
            dumpNodeTree(child, depth + 1)
            child.recycle()
        }
    }

    override fun onInterrupt() {
        Log.d(TAG, "Service interrupted")
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
        handler.removeCallbacksAndMessages(null)
        retryCount = 0
        Log.d(TAG, "Service destroyed")
    }
}