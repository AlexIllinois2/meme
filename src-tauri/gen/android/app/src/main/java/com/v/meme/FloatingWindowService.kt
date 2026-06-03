package com.v.meme

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.app.usage.UsageStats
import android.app.usage.UsageStatsManager
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.graphics.PixelFormat
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.util.Log
import android.view.Display
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.widget.ImageView
import android.widget.Toast
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat

class FloatingWindowService : Service() {

    private lateinit var windowManager: WindowManager
    private lateinit var floatingView: View
    private lateinit var prefs: SharedPreferences
    private var initialX = 0
    private var initialY = 0
    private var initialTouchX = 0f
    private var initialTouchY = 0f
    private var isMoving = false
    private val TAG = "FloatingWindowService"
    private val CHANNEL_ID = "floating_window_channel"
    private val NOTIFICATION_ID = 1
    private val PREFS_NAME = "floating_window_prefs"
    private val KEY_POS_X = "pos_x"
    private val KEY_POS_Y = "pos_y"
    private val DEFAULT_POS_X = 100
    private val DEFAULT_POS_Y = 200
    
    // 允许显示悬浮窗的应用包名列表（包括微信、QQ、本应用和自定义应用）
    private val allowedPackages = mutableSetOf<String>()

    companion object {
        var isRunning = false
            private set
    }

    override fun onCreate() {
        super.onCreate()
        isRunning = true
        
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification().build())
        
        prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        
        allowedPackages.add("com.tencent.mm")
        allowedPackages.add("com.tencent.mobileqq")
        allowedPackages.add(packageName)
        Log.d(TAG, "Initialized with base packages: $allowedPackages")
        
        loadAllowedPackages()
        
        createFloatingView()
        startVisibilityChecker()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
        stopVisibilityChecker()
        if (::floatingView.isInitialized) {
            windowManager.removeView(floatingView)
        }
    }

    private var visibilityHandler: Handler? = null
    private var visibilityRunnable: Runnable? = null

    private fun startVisibilityChecker() {
        visibilityHandler = Handler(Looper.getMainLooper())
        visibilityRunnable = object : Runnable {
            override fun run() {
                checkAndToggleVisibility()
                visibilityHandler?.postDelayed(this, 1000) // 每秒检查一次
            }
        }
        visibilityHandler?.post(visibilityRunnable!!)
    }

    private fun stopVisibilityChecker() {
        visibilityHandler?.removeCallbacksAndMessages(null)
        visibilityHandler = null
        visibilityRunnable = null
    }

    private fun checkAndToggleVisibility() {
        val foregroundApp = getForegroundApp()
        val shouldShow = foregroundApp != null && allowedPackages.contains(foregroundApp)
        
        // 添加详细日志
        if (foregroundApp != null) {
            Log.d(TAG, "Foreground app: $foregroundApp, shouldShow: $shouldShow, allowedPackages: $allowedPackages")
        }
        
        if (shouldShow && floatingView.visibility != View.VISIBLE) {
            Log.d(TAG, "Showing floating window for app: $foregroundApp")
            floatingView.visibility = View.VISIBLE
        } else if (!shouldShow && floatingView.visibility == View.VISIBLE) {
            Log.d(TAG, "Hiding floating window, current app: $foregroundApp")
            floatingView.visibility = View.GONE
        }
    }

    private fun getForegroundApp(): String? {
        // 检查是否有 USAGE_STATS 权限
        val appOps = getSystemService(Context.APP_OPS_SERVICE) as android.app.AppOpsManager
        val mode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            appOps.unsafeCheckOpNoThrow(
                android.app.AppOpsManager.OPSTR_GET_USAGE_STATS,
                android.os.Process.myUid(),
                packageName
            )
        } else {
            @Suppress("DEPRECATION")
            appOps.checkOpNoThrow(
                android.app.AppOpsManager.OPSTR_GET_USAGE_STATS,
                android.os.Process.myUid(),
                packageName
            )
        }
        
        if (mode != android.app.AppOpsManager.MODE_ALLOWED) {
            Log.w(TAG, "No USAGE_STATS permission, cannot detect foreground app")
            return null
        }
        
        val usageStatsManager = getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val endTime = System.currentTimeMillis()
        val startTime = endTime - 1000 * 60 // 查看过去一分钟的统计
        
        val usageStats = usageStatsManager.queryUsageStats(
            UsageStatsManager.INTERVAL_DAILY,
            startTime,
            endTime
        )
        
        if (usageStats.isNullOrEmpty()) return null
        
        var recentStats: UsageStats? = null
        for (stats in usageStats) {
            if (recentStats == null || stats.lastTimeUsed > recentStats!!.lastTimeUsed) {
                recentStats = stats
            }
        }
        
        return recentStats?.packageName
    }

    private fun loadAllowedPackages() {
        // 从 SharedPreferences 加载自定义保存的应用
        val prefs = getSharedPreferences("custom_share_apps_prefs", Context.MODE_PRIVATE)
        val customApps = prefs.getStringSet("apps", emptySet()) ?: emptySet()
        
        Log.d(TAG, "=== DEBUG: Loading allowed packages ===")
        Log.d(TAG, "SharedPreferences name: custom_share_apps_prefs")
        Log.d(TAG, "Raw data from SharedPreferences: $customApps")
        Log.d(TAG, "Size: ${customApps.size}")
        
        allowedPackages.addAll(customApps)
        
        Log.d(TAG, "Loaded ${customApps.size} custom apps: $customApps")
        Log.d(TAG, "Final allowed packages: $allowedPackages")
        Log.d(TAG, "=== END DEBUG ===")
    }

    fun updateAllowedPackages(packages: Set<String>) {
        allowedPackages.clear()
        allowedPackages.add("com.tencent.mm")
        allowedPackages.add("com.tencent.mobileqq")
        allowedPackages.add(packageName)  // 本应用
        allowedPackages.addAll(packages)
        val prefs = getSharedPreferences("custom_share_apps_prefs", Context.MODE_PRIVATE)
        prefs.edit().putStringSet("apps", packages).apply()
        Log.d(TAG, "Updated allowed packages: $allowedPackages")
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "悬浮窗服务",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "用于显示全局悬浮窗"
            }
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }

    private var notificationPendingIntent: PendingIntent? = null

    private fun createNotification(): NotificationCompat.Builder {
        if (notificationPendingIntent == null) {
            val intent = Intent(this, MainActivity::class.java).apply {
                action = "com.v.meme.ACTION_TRIGGER_SEARCH"
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            notificationPendingIntent = PendingIntent.getActivity(
                this, 0, intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )
        }
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("咪萌悬浮窗")
            .setContentText("悬浮窗服务运行中")
            .setSmallIcon(android.R.drawable.ic_menu_search)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .setContentIntent(notificationPendingIntent)
    }

    private fun createFloatingView() {
        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager

        // 获取屏幕尺寸用于边界检测
        val display = windowManager.defaultDisplay
        val screenWidth = display.width
        val screenHeight = display.height
        val buttonSize = 140
        
        // 默认位置：屏幕右边贴边，垂直居中
        val defaultX = screenWidth - buttonSize
        val defaultY = screenHeight * 2 / 3 - buttonSize / 2
        
        // 加载保存的位置（首次使用默认右边居中）
        val savedX = prefs.getInt(KEY_POS_X, defaultX)
        val savedY = prefs.getInt(KEY_POS_Y, defaultY)
        
        // 计算带边界检测的初始位置
        val initialPosX = clampPosition(savedX, 0, screenWidth - buttonSize)
        val initialPosY = clampPosition(savedY, 0, screenHeight - buttonSize)

        // 创建悬浮窗 View - 圆形半透明按钮
        val context = this
        
        // 主题色 - 浅蓝色 (与主页主题一致)
        val themeColor = 0x997DD3FC.toInt() // 半透明天蓝色 (与主题色一致)
        
        floatingView = android.widget.FrameLayout(context).apply {
            // 设置圆形背景 - 无白色边框
            val shape = android.graphics.drawable.GradientDrawable()
            shape.shape = android.graphics.drawable.GradientDrawable.OVAL
            shape.setColor(themeColor)
            
            background = shape
            elevation = 12f
            
            // 添加内部图标 - 放大镜图标
            val iconView = ImageView(context).apply {
                setImageResource(android.R.drawable.ic_menu_search)
                setColorFilter(0xFFFFFFFF.toInt()) // 白色图标
                setPadding(20, 20, 20, 20)
                scaleType = ImageView.ScaleType.CENTER_INSIDE
            }
            
            addView(iconView, android.widget.FrameLayout.LayoutParams(
                android.widget.FrameLayout.LayoutParams.MATCH_PARENT,
                android.widget.FrameLayout.LayoutParams.MATCH_PARENT
            ).apply {
                setMargins(4, 4, 4, 4)
            })
        }

        // 设置 LayoutParams
        val layoutFlag = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
        } else {
            @Suppress("DEPRECATION")
            WindowManager.LayoutParams.TYPE_PHONE
        }

        val params = WindowManager.LayoutParams(
            buttonSize,
            buttonSize,
            layoutFlag,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or 
            WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            //or WindowManager.LayoutParams.FLAG_WATCH_OUTSIDE_TOUCH,
            PixelFormat.TRANSLUCENT
        )

        params.gravity = Gravity.TOP or Gravity.START
        params.x = initialPosX
        params.y = initialPosY
        
        Log.d(TAG, "悬浮窗初始位置: ($initialPosX, $initialPosY), 屏幕尺寸: ${screenWidth}x${screenHeight}")

        // 添加触摸监听器
        floatingView.setOnTouchListener(object : View.OnTouchListener {
            override fun onTouch(v: View?, event: MotionEvent?): Boolean {
                when (event?.action) {
                    MotionEvent.ACTION_DOWN -> {
                        isMoving = false
                        initialX = params.x
                        initialY = params.y
                        initialTouchX = event.rawX
                        initialTouchY = event.rawY
                        // 按下效果 - 稍微缩小并增加透明度
                        v?.animate()?.scaleX(0.9f)?.scaleY(0.9f)?.setDuration(100)?.start()
                        (v as? android.widget.FrameLayout)?.background?.alpha = 200
                        return true
                    }
                    MotionEvent.ACTION_MOVE -> {
                        val deltaX = (event.rawX - initialTouchX).toInt()
                        val deltaY = (event.rawY - initialTouchY).toInt()
                        
                        if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) {
                            isMoving = true
                        }
                        
                        // 应用边界检测
                        val newX = clampPosition(initialX + deltaX, 0, screenWidth - buttonSize)
                        val newY = clampPosition(initialY + deltaY, 0, screenHeight - buttonSize)
                        
                        params.x = newX
                        params.y = newY
                        windowManager.updateViewLayout(floatingView, params)
                        return true
                    }
                    MotionEvent.ACTION_UP -> {
                        // 释放效果 - 恢复原状
                        v?.animate()?.scaleX(1f)?.scaleY(1f)?.setDuration(100)?.start()
                        (v as? android.widget.FrameLayout)?.background?.alpha = 153
                        
                        // 保存最终位置
                        savePosition(params.x, params.y)
                        Log.d(TAG, "悬浮窗位置已保存: (${params.x}, ${params.y})")
                        
                        if (!isMoving) {
                            // 点击事件 - 触发搜索
                            triggerSearch()
                        }
                        return true
                    }
                    MotionEvent.ACTION_CANCEL -> {
                        // 取消时恢复原状
                        v?.animate()?.scaleX(1f)?.scaleY(1f)?.setDuration(100)?.start()
                        (v as? android.widget.FrameLayout)?.background?.alpha = 153
                        return true
                    }
                }
                return false
            }
        })

        // 点击事件
        floatingView.setOnClickListener {
            if (!isMoving) {
                triggerSearch()
            }
        }

        // 添加到 WindowManager
        windowManager.addView(floatingView, params)
    }

    /**
     * 保存悬浮窗位置到 SharedPreferences
     */
    private fun savePosition(x: Int, y: Int) {
        prefs.edit()
            .putInt(KEY_POS_X, x)
            .putInt(KEY_POS_Y, y)
            .apply()
    }

    /**
     * 边界检测：确保位置在有效范围内
     * @param position 当前位置
     * @param minPos 最小允许位置
     * @param maxPos 最大允许位置
     * @return 限制后的位置
     */
    private fun clampPosition(position: Int, minPos: Int, maxPos: Int): Int {
        return position.coerceIn(minPos, maxPos)
    }

    private fun triggerSearch() {
        val foregroundApp = getForegroundApp()

        // 构建最新 Intent（每次构建确保 foreground_app 是最新的）
        val intent = Intent(this, MainActivity::class.java).apply {
            action = "com.v.meme.ACTION_TRIGGER_SEARCH"
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            if (foregroundApp != null && foregroundApp != packageName) {
                putExtra("foreground_app", foregroundApp)
            }
        }

        // 通过 notification PendingIntent 启动 Activity
        // Android 14+ 对 foreground service notification 来源的 PendingIntent
        // 豁免后台 Activity 启动限制，比直接 startActivity() 更可靠
        val pi = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        notificationPendingIntent = pi

        try {
            pi.send()
            Log.d(TAG, "已通过 PendingIntent 触发搜索, foregroundApp=$foregroundApp")
        } catch (e: Exception) {
            Log.w(TAG, "PendingIntent.send() 失败，尝试 startActivity 兜底", e)
            try {
                startActivity(intent)
                Log.d(TAG, "通过 startActivity 兜底成功, foregroundApp=$foregroundApp")
            } catch (e2: Exception) {
                Log.e(TAG, "后台启动 Activity 均被系统拦截", e2)
                // 最终手段：通过更新 notification 让用户手动点击通知来触发
                val notification = createNotification().build()
                val nm = getSystemService(NotificationManager::class.java)
                nm.notify(NOTIFICATION_ID, notification)
                Toast.makeText(
                    this,
                    "系统限制了后台弹出，请在设置中开启「后台弹出界面」权限，或点击通知进入",
                    Toast.LENGTH_LONG
                ).show()
            }
        }
    }
}
