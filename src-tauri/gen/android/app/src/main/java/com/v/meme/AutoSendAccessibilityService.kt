package com.v.meme

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Build
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
    private val maxNodesPerTraversal = 500

    companion object {
        private const val TAG = "AutoSendA11y"
        private const val PREFS_NAME = "auto_send_prefs"
        private const val KEY_ENABLED = "auto_send_enabled"
        private const val SEND_FLOW_TIMEOUT_MS = 30000L
        private const val POLL_INTERVAL_MS = 800L
        
        // 通知相关常量
        private const val NOTIFICATION_CHANNEL_ID = "auto_send_service_channel"
        private const val NOTIFICATION_CHANNEL_NAME = "自动发送辅助服务"
        private const val NOTIFICATION_ID = 1001

        private const val WECHAT_PACKAGE = "com.tencent.mm"
        private const val QQ_PACKAGE = "com.tencent.mobileqq"

        // Intent actions for cross-process communication
        const val ACTION_ACTIVATE_SEND_FLOW = "com.v.meme.ACTION_ACTIVATE_SEND_FLOW"
        const val ACTION_DEACTIVATE_SEND_FLOW = "com.v.meme.ACTION_DEACTIVATE_SEND_FLOW"

        // SharedPreferences keys for cross-process state
        private const val KEY_SERVICE_IS_RUNNING = "service_is_running"
        private const val KEY_SEND_FLOW_ACTIVE = "send_flow_active"
        private const val KEY_IN_DIALOG_CONTEXT = "in_dialog_context"
        private const val KEY_LAST_TARGET_WINDOW_EVENT = "last_target_window_event"
        private const val STALE_EVENT_TIMEOUT_MS = 1500L

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

        // Cross-process state accessors (SharedPreferences-backed)
        fun getServiceRunning(context: Context): Boolean {
            return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .getBoolean(KEY_SERVICE_IS_RUNNING, false)
        }

        fun setServiceRunning(context: Context, running: Boolean) {
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putBoolean(KEY_SERVICE_IS_RUNNING, running)
                .apply()
        }

        fun getSendFlowActive(context: Context): Boolean {
            return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .getBoolean(KEY_SEND_FLOW_ACTIVE, false)
        }

        fun setSendFlowActive(context: Context, active: Boolean) {
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putBoolean(KEY_SEND_FLOW_ACTIVE, active)
                .apply()
        }

        fun getDialogContext(context: Context): Boolean {
            return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .getBoolean(KEY_IN_DIALOG_CONTEXT, false)
        }

        fun setDialogContext(context: Context, inDialog: Boolean) {
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putBoolean(KEY_IN_DIALOG_CONTEXT, inDialog)
                .apply()
        }

        fun getLastWindowEvent(context: Context): Long {
            return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .getLong(KEY_LAST_TARGET_WINDOW_EVENT, 0L)
        }

        fun setLastWindowEvent(context: Context, timestamp: Long) {
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putLong(KEY_LAST_TARGET_WINDOW_EVENT, timestamp)
                .apply()
        }

        fun activateSendFlow(context: Context) {
            Log.d(TAG, "activateSendFlow called, setting sendFlowActive = true")
            setSendFlowActive(context, true)
        }

        fun deactivateSendFlow(context: Context) {
            setSendFlowActive(context, false)
            setDialogContext(context, false)
            setLastWindowEvent(context, 0L)
            Log.d(TAG, "deactivateSendFlow called, sendFlowActive = false")
        }
    }

    override fun onServiceConnected() {
        try {
            super.onServiceConnected()
            setServiceRunning(this, true)
            Log.d(TAG, "Service connected, starting continuous polling")

            val info = AccessibilityServiceInfo().apply {
                eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED or
                        AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED
                feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
                flags = AccessibilityServiceInfo.FLAG_RETRIEVE_INTERACTIVE_WINDOWS or
                        AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS or
                        AccessibilityServiceInfo.FLAG_INCLUDE_NOT_IMPORTANT_VIEWS
                notificationTimeout = 100
            }
            serviceInfo = info

            // 延迟启动前台服务，确保服务完全初始化
            Handler(Looper.getMainLooper()).postDelayed({
                // 启动前台服务，显示常驻通知
                startForegroundService()
            }, 200)
            
            startContinuousPolling()
        } catch (e: Exception) {
            Log.e(TAG, "Error in onServiceConnected: ${e.message}", e)
            setServiceRunning(this, false)
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent != null) {
            when (intent.action) {
                ACTION_ACTIVATE_SEND_FLOW -> {
                    Log.d(TAG, "onStartCommand: received ACTIVATE_SEND_FLOW")
                    setSendFlowActive(this, true)
                    retryCount = 0
                    scheduleFlowTimeout()
                    resumePollingIfNeeded()
                }
                ACTION_DEACTIVATE_SEND_FLOW -> {
                    Log.d(TAG, "onStartCommand: received DEACTIVATE_SEND_FLOW")
                    setSendFlowActive(this, false)
                    setDialogContext(this, false)
                    setLastWindowEvent(this, 0L)
                    retryCount = 0
                    cancelFlowTimeout()
                    stopPolling()
                }
            }
        }
        return START_STICKY
    }
    
    private fun startForegroundService() {
        try {
            createNotificationChannel()
            
            val appName = try {
                getString(R.string.app_name)
            } catch (e: Exception) {
                "咪萌"
            }
            
            val notification = Notification.Builder(this, NOTIFICATION_CHANNEL_ID)
                .setContentTitle(appName)
                .setContentText("辅助自动发送服务当前可用")
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .setOngoing(true) // 常驻通知，用户无法手动清除
                .setPriority(Notification.PRIORITY_LOW)
                .build()
            
            startForeground(NOTIFICATION_ID, notification)
            Log.d(TAG, "Foreground service started with notification")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start foreground service: ${e.message}", e)
            // 如果前台服务启动失败，至少保证服务能继续运行
        }
    }
    
    private fun createNotificationChannel() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val channel = NotificationChannel(
                    NOTIFICATION_CHANNEL_ID,
                    NOTIFICATION_CHANNEL_NAME,
                    NotificationManager.IMPORTANCE_LOW
                ).apply {
                    description = "咪萌无障碍服务"
                    setShowBadge(false)
                }
                
                val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
                notificationManager.createNotificationChannel(channel)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create notification channel: ${e.message}", e)
        }
    }

    private fun startContinuousPolling() {
        handler = Handler(Looper.getMainLooper())
    }

    private fun stopPolling() {
        handler?.removeCallbacksAndMessages(null)
    }

    /** 轮询单次 tick，返回 true 表示需要继续调度下一 tick */
    private fun pollTick(): Boolean {
        try {
            if (!getSendFlowActive(this)) {
                Log.d(TAG, "Poll idle, no active flow")
                retryCount = 0
                return false
            }
            Log.d(TAG, "Poll tick: sendFlowActive=true, retryCount=$retryCount")
            val now = System.currentTimeMillis()
            if (now - lastProcessedTime < debounceMs) return true

            if (findAndClickSendButton()) {
                Log.d(TAG, "Send button clicked via polling, send flow complete")
                deactivateSendFlowInternal()
                lastProcessedTime = now
                retryCount = 0
                return false
            }
            retryCount++
            if (retryCount >= maxRetries) {
                Log.d(TAG, "Max retries ($maxRetries) reached, deactivating send flow")
                deactivateSendFlowInternal()
                retryCount = 0
                return false
            }
            return true
        } catch (e: Exception) {
            Log.e(TAG, "Poll tick exception: ${e.message}", e)
            retryCount = 0
            return false
        }
    }

    /** 如果发送流程激活中且轮询未运行，启动轮询 */
    private fun resumePollingIfNeeded() {
        if (!getSendFlowActive(this)) return
        handler?.removeCallbacksAndMessages(null)
        handler?.post(object : Runnable {
            override fun run() {
                if (pollTick()) {
                    handler?.postDelayed(this, POLL_INTERVAL_MS)
                }
            }
        })
    }

    private fun activateSendFlowInternal() {
        setSendFlowActive(this, true)
        retryCount = 0
        scheduleFlowTimeout()
        resumePollingIfNeeded()
    }

    private fun deactivateSendFlowInternal() {
        setSendFlowActive(this, false)
        setDialogContext(this, false)
        setLastWindowEvent(this, 0L)
        retryCount = 0
        cancelFlowTimeout()
        stopPolling()
        Log.d(TAG, "deactivateSendFlowInternal called, sendFlowActive = false")
    }

    /** 30 秒超时：发送流程激活后如果未完成自动关闭 */
    private var flowTimeoutHandler: Handler? = null
    private var flowTimeoutRunnable: Runnable? = null

    private fun scheduleFlowTimeout() {
        cancelFlowTimeout()
        if (flowTimeoutHandler == null) {
            flowTimeoutHandler = Handler(Looper.getMainLooper())
        }
        flowTimeoutRunnable = Runnable {
            Log.d(TAG, "Send flow timeout (30s), deactivating")
            deactivateSendFlowInternal()
        }
        flowTimeoutHandler?.postDelayed(flowTimeoutRunnable!!, SEND_FLOW_TIMEOUT_MS)
    }

    private fun cancelFlowTimeout() {
        flowTimeoutRunnable?.let { flowTimeoutHandler?.removeCallbacks(it) }
        flowTimeoutRunnable = null
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        try {
            if (event == null) return

            val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            if (!prefs.getBoolean(KEY_ENABLED, false)) return

            if (!getSendFlowActive(this)) return

            val packageName = event.packageName?.toString() ?: return
            if (packageName != WECHAT_PACKAGE && packageName != QQ_PACKAGE) return

            val now = System.currentTimeMillis()
            if (now - lastProcessedTime < debounceMs) return

            when (event.eventType) {
                AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                    val eventClassName = event.className?.toString() ?: ""
                    Log.d(TAG, "Event window state changed: $eventClassName, package: $packageName")

                    setLastWindowEvent(this, now)

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
                        if (!getDialogContext(this)) {
                            setDialogContext(this, true)
                            Log.d(TAG, "inDialogContext set to true")
                        }
                    } else if (isWechatMainChat) {
                        if (getDialogContext(this)) {
                            setDialogContext(this, false)
                            Log.d(TAG, "inDialogContext set to false (main chat)")
                        }
                    } else if (packageName == QQ_PACKAGE && !isDialogEvent) {
                        if (getDialogContext(this)) {
                            setDialogContext(this, false)
                            Log.d(TAG, "inDialogContext set to false")
                        }
                    }
                }
                AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED -> {
                    Log.d(TAG, "Event content changed: ${event.className}")
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error in onAccessibilityEvent: ${e.message}", e)
        }
    }

    private fun findAndClickSendButton(): Boolean {
        if (!getDialogContext(this)) {
            if (!checkDialogFromEventFallback()) {
                Log.d(TAG, "Not in dialog context, skipping send button search")
                return false
            }
        }

        if (System.currentTimeMillis() - getLastWindowEvent(this) > STALE_EVENT_TIMEOUT_MS) {
            Log.d(TAG, "No window events from target app for ${STALE_EVENT_TIMEOUT_MS}ms, treating context as stale, clearing inDialogContext")
            setDialogContext(this, false)
            return false
        }

        val root = rootInActiveWindow
        if (root == null) {
            Log.d(TAG, "rootInActiveWindow is null, cannot search")
            return false
        }

        try {
            visitedNodes = 0
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
                setDialogContext(this, true)
                return true
            }

            val hasDialogLike = hasDialogContainerInTree(root)
            if (hasDialogLike) {
                Log.d(TAG, "Fallback: tree contains dialog-like container, setting inDialogContext=true")
                setDialogContext(this, true)
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
        if (visitedNodes >= maxNodesPerTraversal) return false
        visitedNodes++
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
        if (visitedNodes >= maxNodesPerTraversal) return false
        visitedNodes++
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

    private var visitedNodes = 0

    private fun findSendButtonInTree(node: AccessibilityNodeInfo): AccessibilityNodeInfo? {
        if (visitedNodes >= maxNodesPerTraversal) return null
        visitedNodes++
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

    override fun onUnbind(intent: android.content.Intent?): Boolean {
        Log.d(TAG, "Service onUnbind, requesting rebind on next event")
        // 返回 true 表示当有新的 accessibility 事件时，系统应重新绑定服务
        return true
    }

    override fun onDestroy() {
        try {
            super.onDestroy()
            setServiceRunning(this, false)
            setSendFlowActive(this, false)
            setDialogContext(this, false)
            setLastWindowEvent(this, 0L)
            handler?.removeCallbacksAndMessages(null)
            handler = null
            retryCount = 0
            cancelFlowTimeout()
            flowTimeoutRunnable = null
            flowTimeoutHandler = null
            
            // 停止前台服务
            try {
                stopForeground(true)
            } catch (e: Exception) {
                Log.e(TAG, "Error stopping foreground: ${e.message}", e)
            }
            Log.d(TAG, "Service destroyed")
        } catch (e: Exception) {
            Log.e(TAG, "Error in onDestroy: ${e.message}", e)
        }
    }
}
