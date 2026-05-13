package com.v.meme

import android.Manifest
import android.content.Intent
import android.content.Context
import android.content.pm.PackageManager
import android.content.pm.ResolveInfo
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.provider.MediaStore
import android.content.ContentValues
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.IntentFilter
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
import com.v.meme.BuildConfig

class MainActivity : TauriActivity() {
  private var backCallback: OnBackPressedCallback? = null
  private var jsInterfaceInjected = false
  private val TAG = "MainActivity"
  private val ACTION_TRIGGER_SEARCH = "com.v.meme.ACTION_TRIGGER_SEARCH"
  private val ACTION_SHARE_RESULT = "com.v.meme.ACTION_SHARE_RESULT"
  
  private var permissionFlowInProgress = false
  private var pendingSharePackage: String? = null
  
  // BroadcastReceiver 用于接收分享结果
  private val shareResultReceiver = object : BroadcastReceiver() {
    override fun onReceive(context: Context?, intent: Intent?) {
      if (intent?.action == ACTION_SHARE_RESULT) {
        val chosenComponent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
          intent.getParcelableExtra(Intent.EXTRA_CHOSEN_COMPONENT, android.content.ComponentName::class.java)
        } else {
          @Suppress("DEPRECATION")
          intent.getParcelableExtra(android.content.Intent.EXTRA_CHOSEN_COMPONENT)
        }
        
        if (chosenComponent != null) {
          val packageName = chosenComponent.packageName
          Log.d(TAG, "User selected app via chooser: $packageName")
          
          if (packageName != "com.tencent.mm" && packageName != "com.tencent.mobileqq") {
            // 1. 保存到 SharedPreferences（供悬浮窗服务使用）
            val prefs = getSharedPreferences("custom_share_apps_prefs", Context.MODE_PRIVATE)
            val savedApps = prefs.getStringSet("apps", emptySet())?.toMutableSet() ?: mutableSetOf()
            if (!savedApps.contains(packageName)) {
              savedApps.add(packageName)
              prefs.edit().putStringSet("apps", savedApps).apply()
              Log.d(TAG, "Saved to SharedPreferences: $packageName")
              
              if (FloatingWindowService.isRunning) {
                Log.d(TAG, "Restarting floating window service to reload config")
                val intent = Intent(context, FloatingWindowService::class.java)
                context?.stopService(intent)
                Thread.sleep(200)
                startFloatingWindowService()
              }
            }
            
            // 2. 标记为待处理，并立即尝试通知前端
            pendingSharePackage = packageName
            Log.d(TAG, "Marked as pending share: $packageName")
            android.os.Handler(android.os.Looper.getMainLooper()).post {
              processPendingShare()
            }
          }
        }
      }
    }
  }
  
  private val storagePermissionLauncher = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
  ) { permissions ->
    val allGranted = permissions.entries.all { it.value }
    if (allGranted) {
      Toast.makeText(this@MainActivity, "存储权限已授权", Toast.LENGTH_SHORT).show()
    } else {
      Toast.makeText(this@MainActivity, "存储权限被拒绝，部分功能可能无法使用", Toast.LENGTH_LONG).show()
    }
    onRuntimePermissionsDone()
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
        onManageStorageDone()
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
        onFloatingWindowDone()
    }

    // 应用选择器 launcher
    private val appPickerLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK) {
            val data = result.data
            val componentName = data?.resolveActivity(packageManager)
            if (componentName != null) {
                val packageName = componentName.packageName
                val appName = getApplicationName(packageName)
                Log.d(TAG, "Selected app: $packageName ($appName)")
                
                // 通知前端
                val webView = findWebView()
                webView?.evaluateJavascript("""
                    (function() {
                        if (window.onCustomAppSelected) {
                            window.onCustomAppSelected('$packageName', '$appName');
                        }
                    })()
                """.trimIndent(), null)
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
    requestAllPermissions() // 请求所有需要的权限
    
    // 注册 ShareResultReceiver（在 onCreate 中注册，不在 onStop 中注销，
    // 确保 chooser 弹出后 Activity 进入后台时仍能接收分享结果）
    val filter = IntentFilter(ACTION_SHARE_RESULT)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      registerReceiver(shareResultReceiver, filter, Context.RECEIVER_EXPORTED)
    } else {
      @Suppress("DEPRECATION")
      registerReceiver(shareResultReceiver, filter)
    }
    
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
      Log.d(TAG, "Received trigger search intent")
      val foregroundApp = intent.getStringExtra("foreground_app")
      bringToFrontAndFocusSearchInternal(foregroundApp)
    }
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
    
    logPermissionStatus()
    
    // 处理待保存的分享应用（在 onResume 中处理，确保 WebView 已就绪）
    processPendingShare()
  }

  private fun processPendingShare() {
    val packageName = pendingSharePackage ?: return
    pendingSharePackage = null
    
    Log.d(TAG, "Processing pending share: $packageName")
    
    val appName = try {
      val ai = packageManager.getApplicationInfo(packageName, 0)
      packageManager.getApplicationLabel(ai).toString()
    } catch (e: Exception) {
      Log.e(TAG, "Failed to get app name for $packageName", e)
      packageName
    }
    
    val webView = findWebView()
    if (webView != null) {
      webView.evaluateJavascript("""
        (function() {
          if (window.onCustomAppSelected) {
            window.onCustomAppSelected('$packageName', '$appName');
          }
        })()
      """.trimIndent(), null)
      Toast.makeText(this, "已自动保存应用: $appName", Toast.LENGTH_SHORT).show()
    } else {
      Log.w(TAG, "WebView not found, cannot process pending share")
      pendingSharePackage = packageName
    }
  }
  
  private fun logPermissionStatus() {
    Log.d(TAG, "=== Permission Status Check ===")
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      Log.d(TAG, "READ_MEDIA_IMAGES: ${hasPermission(Manifest.permission.READ_MEDIA_IMAGES)}")
      Log.d(TAG, "READ_MEDIA_VIDEO: ${hasPermission(Manifest.permission.READ_MEDIA_VIDEO)}")
      Log.d(TAG, "READ_MEDIA_AUDIO: ${hasPermission(Manifest.permission.READ_MEDIA_AUDIO)}")
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      Log.d(TAG, "READ_EXTERNAL_STORAGE: ${hasPermission(Manifest.permission.READ_EXTERNAL_STORAGE)}")
      Log.d(TAG, "WRITE_EXTERNAL_STORAGE: ${hasPermission(Manifest.permission.WRITE_EXTERNAL_STORAGE)}")
    }
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      Log.d(TAG, "MANAGE_EXTERNAL_STORAGE: ${Environment.isExternalStorageManager()}")
    }
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      Log.d(TAG, "SYSTEM_ALERT_WINDOW (Floating Window): ${Settings.canDrawOverlays(this)}")
    }
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
      val appOps = getSystemService(android.app.AppOpsManager::class.java)
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
      Log.d(TAG, "USAGE_STATS: ${mode == android.app.AppOpsManager.MODE_ALLOWED}")
    }
    
    Log.d(TAG, "=== Permission Status Check Complete ===")
  }
  
  private fun hasPermission(permission: String): Boolean {
    return ContextCompat.checkSelfPermission(this, permission) == PackageManager.PERMISSION_GRANTED
  }
  
  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    Log.d(TAG, "onWindowFocusChanged: hasFocus=$hasFocus, jsInterfaceInjected=$jsInterfaceInjected")
    if (hasFocus && !jsInterfaceInjected) {
      injectJavaScriptInterface()
    }
  }
  
  override fun onStop() {
    super.onStop()
  }

  override fun onDestroy() {
    super.onDestroy()
    try {
      unregisterReceiver(shareResultReceiver)
    } catch (e: Exception) {
      Log.e(TAG, "Failed to unregister receiver", e)
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
  
  // 幂等式请求所有需要的权限
  private fun requestAllPermissions() {
    Log.d(TAG, "requestAllPermissions called - starting sequential flow")
    if (permissionFlowInProgress) {
      Log.d(TAG, "Permission flow already in progress, skipping")
      return
    }
    permissionFlowInProgress = true
    requestRuntimePermissions()
  }
  
  private fun requestRuntimePermissions() {
    val permissionsToRequest = mutableListOf<String>()
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      // Android 13+ (API 33+): 请求新的媒体权限
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
    } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      // Android 6.0 - 12 (API 23-32): 请求传统存储权限
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_EXTERNAL_STORAGE) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.READ_EXTERNAL_STORAGE)
      }
      
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.WRITE_EXTERNAL_STORAGE) 
          != PackageManager.PERMISSION_GRANTED) {
        permissionsToRequest.add(Manifest.permission.WRITE_EXTERNAL_STORAGE)
      }
    }
    
    if (permissionsToRequest.isNotEmpty()) {
      Log.d(TAG, "Requesting runtime permissions: ${permissionsToRequest.joinToString()}")
      storagePermissionLauncher.launch(permissionsToRequest.toTypedArray())
    } else {
      Log.d(TAG, "All runtime permissions already granted")
      onRuntimePermissionsDone()
    }
  }
  
  private fun requestManageStoragePermission() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      if (!Environment.isExternalStorageManager()) {
        Log.d(TAG, "Requesting MANAGE_EXTERNAL_STORAGE permission")
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
      } else {
        Log.d(TAG, "MANAGE_EXTERNAL_STORAGE already granted")
        onManageStorageDone()
      }
    } else {
      Log.d(TAG, "Android < 11, skipping MANAGE_EXTERNAL_STORAGE")
      onManageStorageDone()
    }
  }
  
  private fun requestFloatingWindowPermissionIfNeeded() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      if (!Settings.canDrawOverlays(this)) {
        Log.d(TAG, "Floating window permission not granted, showing dialog")
        showFloatingWindowPermissionDialog()
      } else {
        Log.d(TAG, "Floating window permission already granted")
        onFloatingWindowDone()
      }
    } else {
      Log.d(TAG, "Android < 6.0, skipping floating window permission")
      onFloatingWindowDone()
    }
  }
  
  private fun showFloatingWindowPermissionDialog() {
    runOnUiThread {
      try {
        // 直接使用 androidx AlertDialog.Builder
        val builder = androidx.appcompat.app.AlertDialog.Builder(this)
        builder.setTitle("需要悬浮窗权限")
        builder.setMessage("为了使用悬浮窗快捷搜索功能，需要授予悬浮窗权限。\n\n请点击\"确定\"前往设置页面授权。")
        builder.setPositiveButton("确定") { dialog, _ ->
          dialog.dismiss()
          requestFloatingWindowPermission()
        }
        builder.setNegativeButton("取消") { dialog, _ ->
          dialog.dismiss()
          Log.d(TAG, "User cancelled floating window permission request")
          onFloatingWindowDone()
        }
        
        val dialog = builder.create()
        dialog.show()
        Log.d(TAG, "Floating window permission dialog shown")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to show floating window permission dialog", e)
        onFloatingWindowDone()
      }
    }
  }
  
  private fun requestUsageStatsPermissionIfNeeded() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
      val appOps = getSystemService(android.app.AppOpsManager::class.java)
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
        Log.d(TAG, "Usage stats permission not granted, showing dialog")
        showUsageStatsPermissionDialog()
      } else {
        Log.d(TAG, "Usage stats permission already granted")
        onAllPermissionsDone()
      }
    } else {
      Log.d(TAG, "Android < 5.0, skipping usage stats permission")
      onAllPermissionsDone()
    }
  }
  
  private fun onRuntimePermissionsDone() {
    Log.d(TAG, "Runtime permissions step done, proceeding to manage storage")
    requestManageStoragePermission()
  }
  
  private fun onManageStorageDone() {
    Log.d(TAG, "Manage storage step done, proceeding to floating window")
    requestFloatingWindowPermissionIfNeeded()
  }
  
  private fun onFloatingWindowDone() {
    Log.d(TAG, "Floating window step done, proceeding to usage stats")
    requestUsageStatsPermissionIfNeeded()
  }
  
  private fun onAllPermissionsDone() {
    Log.d(TAG, "All permissions flow completed")
    permissionFlowInProgress = false
  }
  
  private fun showUsageStatsPermissionDialog() {
    runOnUiThread {
      try {
        // 直接使用 androidx AlertDialog.Builder
        val builder = androidx.appcompat.app.AlertDialog.Builder(this)
        builder.setTitle("需要使用情况访问权限")
        builder.setMessage("为了让悬浮窗能够智能检测前台应用，需要授予使用情况访问权限。\n\n请点击\"确定\"前往设置页面授权。")
        builder.setPositiveButton("确定") { dialog, _ ->
          dialog.dismiss()
          requestUsageStatsPermission()
        }
        builder.setNegativeButton("取消") { dialog, _ ->
          dialog.dismiss()
          Log.d(TAG, "User cancelled usage stats permission request")
          onAllPermissionsDone()
        }
        
        val dialog = builder.create()
        dialog.show()
        Log.d(TAG, "Usage stats permission dialog shown")
      } catch (e: Exception) {
        Log.e(TAG, "Failed to show usage stats permission dialog", e)
        onAllPermissionsDone()
      }
    }
  }
  
  // 保留原有的 requestStoragePermissions 函数，供前端调用
  private fun requestStoragePermissions() {
    requestAllPermissions()
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
        if (targetApp.isEmpty()) {
          Log.d(TAG, "Showing system share menu with BroadcastReceiver")
          val sendIntent = android.content.Intent(android.content.Intent.ACTION_SEND).apply {
            type = "image/*"
            putExtra(android.content.Intent.EXTRA_STREAM, uri)
            addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
          }
          
          // 创建 PendingIntent 用于接收用户选择的结果
          val pendingIntent = PendingIntent.getBroadcast(
            this@MainActivity,
            0,
            Intent(ACTION_SHARE_RESULT).setPackage(packageName),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
          )
          
          // 使用 createChooser 并传入 PendingIntent
          val chooserIntent = Intent.createChooser(sendIntent, "分享图片", pendingIntent.intentSender)
          chooserIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
          startActivity(chooserIntent)
          
          Toast.makeText(this@MainActivity, "请选择分享目标", Toast.LENGTH_SHORT).show()
        } else {
          // 分享到特定应用
          val targetPackageName = when (targetApp) {
            "wechat" -> "com.tencent.mm"
            "qq" -> "com.tencent.mobileqq"
            else -> {
              // 如果 targetApp 看起来像包名（包含点号），直接使用
              if (targetApp.contains(".")) {
                Log.d(TAG, "Using custom app package: $targetApp")
                targetApp
              } else {
                Log.e(TAG, "Unknown target app: $targetApp")
                Toast.makeText(this@MainActivity, "未知的目标应用: $targetApp", Toast.LENGTH_SHORT).show()
                return@runOnUiThread
              }
            }
          }
          
          val intent = android.content.Intent(android.content.Intent.ACTION_SEND).apply {
            type = "image/*"
            putExtra(android.content.Intent.EXTRA_STREAM, uri)
            setPackage(targetPackageName)
            addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            addFlags(android.content.Intent.FLAG_ACTIVITY_CLEAR_TOP)
          }
          
          startActivity(intent)
          Log.i(TAG, "Share intent started successfully to $targetPackageName")
          Toast.makeText(this@MainActivity, "正在分享...", Toast.LENGTH_SHORT).show()
        }
      } catch (e: android.content.ActivityNotFoundException) {
        val appName = when (targetApp) {
          "wechat" -> "微信"
          "qq" -> "QQ"
          else -> getApplicationName(targetApp)
        }
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
      bringToFrontAndFocusSearchInternal()
    }
  }
  
  private fun bringToFrontAndFocusSearchInternal(foregroundApp: String? = null) {
    Log.d(TAG, "Bringing app to front and focusing search, foregroundApp=$foregroundApp")
    triggerSearchFocusInWebView(foregroundApp)
  }
  
  private fun triggerSearchFocusInWebView(foregroundApp: String? = null) {
    val webView = findWebView()
    if (webView != null) {
      Log.d(TAG, "Triggering search focus in WebView")
      val appJson = if (foregroundApp != null) {
        val appName = getApplicationName(foregroundApp)
        """{"packageName":"$foregroundApp","appName":"$appName"}"""
      } else {
        "null"
      }
      webView.evaluateJavascript("""
          (function() {
              var app = $appJson;
              if (window.triggerSearchFocus) {
                  window.triggerSearchFocus(app);
              } else {
                  var event = new CustomEvent('triggerSearchFocus', { detail: app });
                  window.dispatchEvent(event);
              }
          })();
      """.trimIndent(), null)
    } else {
      Log.w(TAG, "WebView not found")
    }
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

  @JavascriptInterface
  fun pickShareApp() {
    Log.d(TAG, "pickShareApp called")
    runOnUiThread {
      val mainIntent = Intent(Intent.ACTION_MAIN, null)
      mainIntent.addCategory(Intent.CATEGORY_LAUNCHER)
      val chooserIntent = Intent.createChooser(mainIntent, "选择分享应用")
      appPickerLauncher.launch(chooserIntent)
    }
  }

  @JavascriptInterface
  fun syncCustomAppsToPrefs(packagesJson: String) {
    Log.d(TAG, "syncCustomAppsToPrefs called with: $packagesJson")
    try {
      // 解析 JSON 数组
      val jsonArray = org.json.JSONArray(packagesJson)
      val packages = mutableSetOf<String>()
      for (i in 0 until jsonArray.length()) {
        packages.add(jsonArray.getString(i))
      }
      
      // 保存到 SharedPreferences
      val prefs = getSharedPreferences("custom_share_apps_prefs", Context.MODE_PRIVATE)
      prefs.edit().putStringSet("apps", packages).apply()
      Log.d(TAG, "Synced ${packages.size} apps to SharedPreferences")
      
      // 如果悬浮窗服务正在运行，通知它更新
      if (FloatingWindowService.isRunning) {
        // 重启悬浮窗服务以重新加载配置
        val intent = Intent(this, FloatingWindowService::class.java)
        stopService(intent)
        Thread.sleep(100) // 等待服务停止
        startFloatingWindowService()
      }
    } catch (e: Exception) {
      Log.e(TAG, "Failed to sync custom apps", e)
    }
  }

  @JavascriptInterface
  fun getCustomAppsFromPrefs(): String {
    val prefs = getSharedPreferences("custom_share_apps_prefs", Context.MODE_PRIVATE)
    val savedApps = prefs.getStringSet("apps", emptySet()) ?: emptySet()
    val jsonArray = org.json.JSONArray()
    for (app in savedApps) {
      jsonArray.put(app)
    }
    return jsonArray.toString()
  }

  @JavascriptInterface
  fun requestUsageStatsPermission() {
    Log.d(TAG, "requestUsageStatsPermission called")
    runOnUiThread {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
        val intent = Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS)
        startActivity(intent)
        Toast.makeText(this, "请授予\"使用情况访问权限\"以启用悬浮窗智能显示", Toast.LENGTH_LONG).show()
      } else {
        Toast.makeText(this, "您的 Android 版本不支持此功能", Toast.LENGTH_SHORT).show()
      }
      onAllPermissionsDone()
    }
  }
  
  @JavascriptInterface
  fun resetPermissionDialogFlag() {
    Log.d(TAG, "resetPermissionDialogFlag called")
    permissionFlowInProgress = false
  }

  @JavascriptInterface
  fun getApplicationName(packageName: String): String {
    Log.d(TAG, "getApplicationName called for: $packageName")
    return try {
      val pm = packageManager
      val ai = pm.getApplicationInfo(packageName, 0)
      pm.getApplicationLabel(ai).toString()
    } catch (e: Exception) {
      Log.e(TAG, "Failed to get application name for $packageName", e)
      packageName
    }
  }
  
  @JavascriptInterface
  fun requestAllPermissionsFromJS() {
    Log.d(TAG, "requestAllPermissionsFromJS called")
    runOnUiThread {
      permissionFlowInProgress = false
      requestAllPermissions()
    }
  }

  @JavascriptInterface
  fun saveImageToGallery(json: String) {
    Log.d(TAG, "saveImageToGallery called: $json")
    try {
      val obj = org.json.JSONObject(json)
      val path = obj.getString("path")
      val displayName = obj.getString("displayName")

      val file = java.io.File(path)
      if (!file.exists()) {
        Log.e(TAG, "File not found: $path")
        return
      }

      val mimeType = if (displayName.endsWith(".jpg") || displayName.endsWith(".jpeg")) {
        "image/jpeg"
      } else if (displayName.endsWith(".gif")) {
        "image/gif"
      } else {
        "image/png"
      }

      val contentValues = ContentValues().apply {
        put(MediaStore.Images.Media.DISPLAY_NAME, displayName)
        put(MediaStore.Images.Media.MIME_TYPE, mimeType)
        put(MediaStore.Images.Media.IS_PENDING, 1)
      }

      val uri = contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)
      if (uri != null) {
        contentResolver.openOutputStream(uri)?.use { outputStream ->
          file.inputStream().use { inputStream ->
            inputStream.copyTo(outputStream)
          }
        }
        contentValues.clear()
        contentValues.put(MediaStore.Images.Media.IS_PENDING, 0)
        contentResolver.update(uri, contentValues, null, null)
        Log.d(TAG, "Image saved to MediaStore: $uri")
      } else {
        Log.e(TAG, "Failed to create MediaStore entry")
      }

      file.delete()
    } catch (e: Exception) {
      Log.e(TAG, "Failed to save image to gallery", e)
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
