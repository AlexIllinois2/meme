package com.v.meme

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.view.Display
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.widget.ImageView
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

    companion object {
        var isRunning = false
            private set
    }

    override fun onCreate() {
        super.onCreate()
        isRunning = true
        prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification().build())
        createFloatingView()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        isRunning = false
        if (::floatingView.isInitialized) {
            windowManager.removeView(floatingView)
        }
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

    private fun createNotification(): NotificationCompat.Builder {
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("咪萌悬浮窗")
            .setContentText("悬浮窗服务运行中")
            .setSmallIcon(android.R.drawable.ic_menu_search)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
    }

    private fun createFloatingView() {
        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager

        // 加载保存的位置
        val savedPosition = loadPosition()
        
        // 获取屏幕尺寸用于边界检测
        val display = windowManager.defaultDisplay
        val screenWidth = display.width
        val screenHeight = display.height
        val buttonSize = 140
        
        // 计算带边界检测的初始位置
        val initialPosX = clampPosition(savedPosition.first, 0, screenWidth - buttonSize)
        val initialPosY = clampPosition(savedPosition.second, 0, screenHeight - buttonSize)

        // 创建悬浮窗 View - 圆形半透明按钮
        val context = this
        
        // 主题色 - 浅蓝色 (与主页主题一致)
        val themeColor = 0x994A90E2.toInt() // 半透明浅蓝色 (60% 不透明度)
        
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
            // WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            WindowManager.LayoutParams.FOCUSABLE_TOUCH_MODE or WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL or
    WindowManager.LayoutParams.FLAG_WATCH_OUTSIDE_TOUCH,
            PixelFormat.TRANSLUCENT
        ).apply {
    // 软键盘模式：悬浮窗专属，必须设置
    softInputMode = WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_VISIBLE or
                     WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
                     }

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
     * 从 SharedPreferences 加载保存的位置
     * @return Pair<x, y> 坐标
     */
    private fun loadPosition(): Pair<Int, Int> {
        val x = prefs.getInt(KEY_POS_X, DEFAULT_POS_X)
        val y = prefs.getInt(KEY_POS_Y, DEFAULT_POS_Y)
        Log.d(TAG, "加载保存的位置: ($x, $y)")
        return Pair(x, y)
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
        Log.d(TAG, "===== Floating window clicked by user =====")
        Log.d(TAG, "Service running status: $isRunning")
        
        // 检查悬浮窗服务是否仍在运行
        if (!isRunning) {
            Log.w(TAG, "Floating window service is not running, stopping button")
            stopSelf()
            return
        }
        
        // 启动 MainActivity 并传递触发搜索的标记
        val intent = Intent(this, MainActivity::class.java).apply {
            action = "com.v.meme.ACTION_TRIGGER_SEARCH"
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
            addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP)
        }
        
        Log.d(TAG, "Launching MainActivity with ACTION_TRIGGER_SEARCH")
        
        try {
            startActivity(intent)
            Log.d(TAG, "✓ App launched successfully")
            Log.d(TAG, "Focus will be triggered via JS interface when app is ready")
        } catch (e: Exception) {
            Log.e(TAG, "✗ Failed to launch app", e)
            e.printStackTrace()
            Toast.makeText(this@FloatingWindowService, "启动应用失败: ${e.message}", Toast.LENGTH_LONG).show()
        }
    }
}
