package com.v.meme

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.webkit.WebView
import android.webkit.JavascriptInterface
import android.view.ViewGroup
import android.widget.Toast
import android.util.Log
import androidx.activity.enableEdgeToEdge
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import android.view.MotionEvent
import android.os.SystemClock
import com.v.meme.BuildConfig

class MainActivity : TauriActivity() {
  private var backCallback: OnBackPressedCallback? = null
  private var jsInterfaceInjected = false
  private val TAG = "MainActivity"
  private val ACTION_TRIGGER_SEARCH = "com.v.meme.ACTION_TRIGGER_SEARCH"
  
  private val storagePermissionLauncher = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { permissions ->
    val allGranted = permissions.entries.all { it.value }
    if (allGranted) {
      Toast.makeText(this@MainActivity, "存储权限已授权", Toast.LENGTH_SHORT).show()
    } else {
      Toast.makeText(this@MainActivity, "存储权限被拒绝，部分功能可能无法使用", Toast.LENGTH_LONG).show()
    }
  }
  
  private val manageStorageLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            if (Environment.isExternalStorageManager()) {
                Toast.makeText(this@MainActivity, "完整存储权限已授权", Toast.LENGTH_SHORT).show()
            } else {
                Toast.makeText(this@MainActivity, "完整存储权限被拒绝，部分功能可能无法使用", Toast.LENGTH_LONG).show()
            }
        }
    }

    // 悬浮窗权限申请 launcher
    private val floatingWindowPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            if (Settings.canDrawOverlays(this)) {
                Toast.makeText(this@MainActivity, "悬浮窗权限已授权", Toast.LENGTH_SHORT).show()
                startFloatingWindowService()
            } else {
                Toast.makeText(this@MainActivity, "悬浮窗权限被拒绝", Toast.LENGTH_LONG).show()
            }
        }
    }
  
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    
    Log.d(TAG, "onCreate called")
    
    if (BuildConfig.DEBUG) {
      WebView.setWebContentsDebuggingEnabled(true)
    }
    
    // 设置 WindowInsets 处理，确保 WebView 不会被状态栏/导航栏遮挡
    setupWindowInsets()
    
    setupBackPressHandler()
    requestStoragePermissions()
    
    // 检查启动 Intent
    handleIntent(intent)
  }
  
  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    setIntent(intent)
    Log.d(TAG, "onNewIntent called")
    handleIntent(intent)
  }
  
  private fun handleIntent(intent: Intent) {
    if (intent.action == ACTION_TRIGGER_SEARCH) {
      Log.d(TAG, "Received trigger search intent from floating window")
      // 延迟执行，确保应用已经完全到前台且 JS 接口已注入
      Handler(Looper.getMainLooper()).postDelayed({
        if (jsInterfaceInjected) {
          triggerSearchFocusInWebViewWithRetry(0)
        } else {
          Log.w(TAG, "JS interface not ready, waiting...")
          waitForJsInterfaceAndFocus()
        }
      }, 200)
    }
  }
  
  private fun waitForJsInterfaceAndFocus() {
    var retryCount = 0
    val maxRetries = 10
    
    val checkInterval = object : Runnable {
      override fun run() {
        if (jsInterfaceInjected) {
          Log.d(TAG, "JS interface ready, triggering focus")
          triggerSearchFocusInWebViewWithRetry(0)
        } else if (retryCount < maxRetries) {
          retryCount++
          Log.d(TAG, "Waiting for JS interface... ($retryCount/$maxRetries)")
          Handler(Looper.getMainLooper()).postDelayed(this, 100)
        } else {
          Log.e(TAG, "JS interface not available after $maxRetries retries")
        }
      }
    }
    
    Handler(Looper.getMainLooper()).postDelayed(checkInterval, 100)
  }
  
  private fun setupWindowInsets() {
    val rootView = findViewById<ViewGroup>(android.R.id.content)
    rootView?.let { view ->
      ViewCompat.setOnApplyWindowInsetsListener(view) { v, windowInsets ->
        val insets = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars())
        v.updatePadding(
          top = insets.top,
          bottom = insets.bottom
        )
        windowInsets
      }
    }
  }
  
  override fun onStart() {
    super.onStart()
    Log.d(TAG, "onStart called, jsInterfaceInjected=$jsInterfaceInjected")
    if (!jsInterfaceInjected) {
      injectJavaScriptInterface()
    }
  }
  
  override fun onResume() {
    super.onResume()
    Log.d(TAG, "onResume called, jsInterfaceInjected=$jsInterfaceInjected")
    if (!jsInterfaceInjected) {
      injectJavaScriptInterface()
    }
  }
  
  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    Log.d(TAG, "onWindowFocusChanged: hasFocus=$hasFocus, jsInterfaceInjected=$jsInterfaceInjected")
    if (hasFocus && !jsInterfaceInjected) {
      injectJavaScriptInterface()
    }
  }
  
  private fun injectJavaScriptInterface() {
    Log.d(TAG, "Attempting to inject JavaScript interface")
    val webView = findWebView()
    
    if (webView == null) {
      Log.w(TAG, "WebView not found, retrying in 300ms")
      Handler(Looper.getMainLooper()).postDelayed({
        injectJavaScriptInterface()
      }, 300)
      return
    }
    
    try {
      // 先移除旧的接口（如果存在），避免重复注入
      try {
        webView.removeJavascriptInterface("AndroidNative")
        Log.d(TAG, "Removed existing AndroidNative interface")

        // 获取 WebView 并预激活
        webView?.postDelayed({
            webView?.let {
                it.requestFocus()
                it.isFocusable = true
                it.isFocusableInTouchMode = true
                
                // 模拟一个触摸事件
                val downTime = SystemClock.uptimeMillis()
                val eventTime = SystemClock.uptimeMillis()
                val motionEvent = MotionEvent.obtain(
                    downTime, eventTime,
                    MotionEvent.ACTION_DOWN,
                    10f, 10f, 0
                )
                it.dispatchTouchEvent(motionEvent)
                motionEvent.recycle()
            }
        }, 500)
      } catch (e: Exception) {
        // 忽略异常，可能接口不存在
      }
      
      // 启用 JavaScript
      webView.settings.javaScriptEnabled = true
      
      // 注入新的接口
      webView.addJavascriptInterface(this, "AndroidNative")
      jsInterfaceInjected = true
      Log.i(TAG, "JavaScript interface injected successfully")
      
      // 验证注入是否成功 - 执行一段测试代码
      webView.evaluateJavascript("""
        (function() {
          if (typeof window.AndroidNative !== 'undefined') {
            console.log('[Android] AndroidNative interface verified');
            return true;
          } else {
            console.error('[Android] AndroidNative interface NOT available after injection');
            return false;
          }
        })()
      """.trimIndent(), null)
      
    } catch (e: Exception) {
      Log.e(TAG, "Failed to inject JavaScript interface", e)
      jsInterfaceInjected = false
      // 增加重试次数限制，避免无限循环
      Handler(Looper.getMainLooper()).postDelayed({
        if (!jsInterfaceInjected) {
          Log.w(TAG, "Retrying JavaScript interface injection")
          injectJavaScriptInterface()
        }
      }, 500)
    }
  }
  
  private fun requestStoragePermissions() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      // Android 13+ (API 33+): 请求新的媒体权限 + MANAGE_EXTERNAL_STORAGE
      val permissionsToRequest = mutableListOf<String>()
      
      // 请求图片、视频、音频权限
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_MEDIA_IMAGES) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.READ_MEDIA_IMAGES)
      }
      
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_MEDIA_VIDEO) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.READ_MEDIA_VIDEO)
      }
      
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_MEDIA_AUDIO) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.READ_MEDIA_AUDIO)
      }
      
      // 如果有未授权的媒体权限，先请求它们
      if (permissionsToRequest.isNotEmpty()) {
        storagePermissionLauncher.launch(permissionsToRequest.toTypedArray())
      }
      
      // 同时请求完整存储管理权限（用于访问任意目录）
      if (!Environment.isExternalStorageManager()) {
        try {
          val intent = android.content.Intent(
            android.provider.Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
            android.net.Uri.parse("package:$packageName")
          )
          manageStorageLauncher.launch(intent)
        } catch (e: Exception) {
          val intent = android.content.Intent(
            android.provider.Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION
          )
          manageStorageLauncher.launch(intent)
        }
      }
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      // Android 11-12 (API 30-32): 只请求 MANAGE_EXTERNAL_STORAGE
      if (!Environment.isExternalStorageManager()) {
        try {
          val intent = android.content.Intent(
            android.provider.Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
            android.net.Uri.parse("package:$packageName")
          )
          manageStorageLauncher.launch(intent)
        } catch (e: Exception) {
          val intent = android.content.Intent(
            android.provider.Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION
          )
          manageStorageLauncher.launch(intent)
        }
      }
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      // Android 6.0 - 10 (API 23-29): 请求传统存储权限
      val permissionsToRequest = mutableListOf<String>()
      
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_EXTERNAL_STORAGE) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.READ_EXTERNAL_STORAGE)
      }
      
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.WRITE_EXTERNAL_STORAGE) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
      }
      
      if (permissionsToRequest.isNotEmpty()) {
        storagePermissionLauncher.launch(permissionsToRequest.toTypedArray())
      }
    }
  }
  
  private fun findWebView(): WebView? {
    val decorView = window.decorView as? ViewGroup
    decorView?.let {
      return findWebViewInViewGroup(it)
    }
    return null
  }
  
  private fun findWebViewInViewGroup(group: ViewGroup): WebView? {
    for (i in 0 until group.childCount) {
      val child = group.getChildAt(i)
      if (child is WebView) {
        Log.d(TAG, "WebView found")
        return child
      } else if (child is ViewGroup) {
        val result = findWebViewInViewGroup(child)
        if (result != null) {
          return result
        }
      }
    }
    return null
  }
  
  private fun setupBackPressHandler() {
    backCallback = object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val webView = findWebView()
        webView?.evaluateJavascript("""
          (function() {
            if (window.__TAURI__) {
              window.dispatchEvent(new CustomEvent('tauri-android-back', {
                detail: { timestamp: Date.now() }
              }));
            }
            return true;
          })()
        """.trimIndent(), null)
        
        // 不在这里延迟，让前端决定是否最小化
      }
    }
    
    onBackPressedDispatcher.addCallback(this, backCallback!!)
  }
  
  @JavascriptInterface
  fun minimizeApp() {
    Log.d(TAG, "minimizeApp called - moving task to background")
    runOnUiThread {
      moveTaskToBack(true)
    }
  }
  
  @JavascriptInterface
  fun requestStoragePermission() {
    Log.d(TAG, "requestStoragePermission called from JS")
    runOnUiThread {
      requestStoragePermissions()
    }
  }

  @JavascriptInterface
  fun shareImageToApp(imagePath: String, targetApp: String) {
    Log.d(TAG, "shareImageToApp called: imagePath=$imagePath, targetApp='$targetApp'")
    runOnUiThread {
      try {
        val file = java.io.File(imagePath)
        if (!file.exists()) {
          Log.e(TAG, "Image file not found: $imagePath")
          Toast.makeText(this@MainActivity, "图片文件不存在", Toast.LENGTH_SHORT).show()
          return@runOnUiThread
        }
        
        val uri = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
          androidx.core.content.FileProvider.getUriForFile(
            this@MainActivity,
            "${this@MainActivity.packageName}.fileprovider",
            file
          )
        } else {
          android.net.Uri.fromFile(file)
        }
        
        // 如果 targetApp 为空或未指定，显示系统分享菜单
        val intent = if (targetApp.isEmpty()) {
          Log.d(TAG, "Showing system share menu")
          android.content.Intent(android.content.Intent.ACTION_SEND).apply {
            type = "image/*"
            putExtra(android.content.Intent.EXTRA_STREAM, uri)
            addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
          }
        } else {
          // 分享到特定应用
          val targetPackageName = when (targetApp) {
            "wechat" -> "com.tencent.mm"
            "qq" -> "com.tencent.mobileqq"
            else -> {
              Log.e(TAG, "Unknown target app: $targetApp")
              Toast.makeText(this@MainActivity, "未知的目标应用: $targetApp", Toast.LENGTH_SHORT).show()
              return@runOnUiThread
            }
          }
          
          android.content.Intent(android.content.Intent.ACTION_SEND).apply {
            type = "image/*"
            putExtra(android.content.Intent.EXTRA_STREAM, uri)
            setPackage(targetPackageName)
            addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            addFlags(android.content.Intent.FLAG_ACTIVITY_CLEAR_TOP)
          }
        }
        
        startActivity(intent)
        Log.i(TAG, "Share intent started successfully")
        if (targetApp.isEmpty()) {
          Toast.makeText(this@MainActivity, "请选择分享目标", Toast.LENGTH_SHORT).show()
        } else {
          Toast.makeText(this@MainActivity, "正在分享...", Toast.LENGTH_SHORT).show()
        }
      } catch (e: android.content.ActivityNotFoundException) {
        val appName = if (targetApp == "wechat") "微信" else "QQ"
        Log.e(TAG, "$appName not found", e)
        Toast.makeText(this@MainActivity, "未找到${appName}应用", Toast.LENGTH_LONG).show()
      } catch (e: Exception) {
        Log.e(TAG, "Share failed", e)
        Toast.makeText(this@MainActivity, "分享失败: ${e.message}", Toast.LENGTH_LONG).show()
        e.printStackTrace()
      }
    }
  }
  
  @JavascriptInterface
  fun bringToFrontAndFocusSearch() {
    Log.d(TAG, "bringToFrontAndFocusSearch called from JS")
    runOnUiThread {
      triggerSearchFocusInWebViewWithRetry(0)
    }
  }
  
  private var focusRetryCount = 0
  private val MAX_FOCUS_RETRY = 5
  
  private fun triggerSearchFocusInWebViewWithRetry(retryCount: Int) {
    focusRetryCount = retryCount
    val webView = findWebView()
    if (webView != null) {
      Log.d(TAG, "Triggering search focus in WebView (attempt ${retryCount + 1})")
      // 通过 JavaScript 触发，并等待回调确认
      webView.evaluateJavascript("""
          (function() {
              console.log('[Native] Triggering search focus, attempt ${retryCount + 1}');
              if (window.triggerSearchFocus) {
                  window.triggerSearchFocus();
                  return 'success';
              } else {
                  // 如果全局方法还没准备好，发送自定义事件
                  var event = new CustomEvent('triggerSearchFocus');
                  window.dispatchEvent(event);
                  return 'event_sent';
              }
          })();
      """.trimIndent()) { result ->
        Log.d(TAG, "JavaScript execution result: $result")
        // 如果第一次尝试失败，进行重试
        if (result == "null" && retryCount < MAX_FOCUS_RETRY) {
          Log.w(TAG, "First attempt failed, retrying... (${retryCount + 1}/$MAX_FOCUS_RETRY)")
          Handler(Looper.getMainLooper()).postDelayed({
            triggerSearchFocusInWebViewWithRetry(retryCount + 1)
          }, 200)
        }
      }
    } else {
      Log.w(TAG, "WebView not found, retrying... (${retryCount + 1}/$MAX_FOCUS_RETRY)")
      if (retryCount < MAX_FOCUS_RETRY) {
        Handler(Looper.getMainLooper()).postDelayed({
          triggerSearchFocusInWebViewWithRetry(retryCount + 1)
        }, 200)
      }
    }
  }
  
  private fun triggerSearchFocusInWebView() {
    triggerSearchFocusInWebViewWithRetry(0)
  }

  // ========== 悬浮窗相关方法 ==========

  @JavascriptInterface
  fun isFloatingWindowEnabled(): Boolean {
    return FloatingWindowService.isRunning
  }

  @JavascriptInterface
  fun requestFloatingWindowPermission() {
    Log.d(TAG, "Requesting floating window permission")
    runOnUiThread {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
        if (!Settings.canDrawOverlays(this)) {
          // 跳转到悬浮窗权限设置页面
          val intent = Intent(
            Settings.ACTION_MANAGE_OVERLAY_PERMISSION,
            Uri.parse("package:$packageName")
          )
          floatingWindowPermissionLauncher.launch(intent)
        } else {
          startFloatingWindowService()
        }
      } else {
        // Android 6.0 以下，直接启动
        startFloatingWindowService()
      }
    }
  }

  @JavascriptInterface
  fun startFloatingWindow() {
    Log.d(TAG, "startFloatingWindow called")
    runOnUiThread {
      requestFloatingWindowPermission()
    }
  }

  @JavascriptInterface
  fun stopFloatingWindow() {
    Log.d(TAG, "stopFloatingWindow called")
    runOnUiThread {
      val intent = Intent(this, FloatingWindowService::class.java)
      stopService(intent)
      Toast.makeText(this@MainActivity, "悬浮窗已关闭", Toast.LENGTH_SHORT).show()
    }
  }

  private fun startFloatingWindowService() {
    Log.d(TAG, "Starting floating window service")
    val intent = Intent(this, FloatingWindowService::class.java)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      startForegroundService(intent)
    } else {
      startService(intent)
    }
    Toast.makeText(this@MainActivity, "悬浮窗已开启", Toast.LENGTH_SHORT).show()
  }
}
