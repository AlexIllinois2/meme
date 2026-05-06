package com.v.meme

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Handler
import android.os.Looper
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
      Log.w(TAG, "WebView not found, retrying in 500ms")
      Handler(Looper.getMainLooper()).postDelayed({
        injectJavaScriptInterface()
      }, 500)
      return
    }
    
    try {
      webView.addJavascriptInterface(this, "AndroidNative")
      jsInterfaceInjected = true
      Log.i(TAG, "JavaScript interface injected successfully")
      Toast.makeText(this@MainActivity, "Native interface ready", Toast.LENGTH_SHORT).show()
    } catch (e: Exception) {
      Log.e(TAG, "Failed to inject JavaScript interface", e)
      Handler(Looper.getMainLooper()).postDelayed({
        injectJavaScriptInterface()
      }, 500)
    }
  }
  
  private fun requestStoragePermissions() {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
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
        
        webView?.postDelayed({
        }, 300)
      }
    }
    
    onBackPressedDispatcher.addCallback(this, backCallback!!)
  }
  
  fun setBackPressEnabled(enabled: Boolean) {
    backCallback?.isEnabled = enabled
  }
  
  @JavascriptInterface
  fun shareImageToApp(imagePath: String, targetApp: String) {
    Log.d(TAG, "shareImageToApp called: imagePath=$imagePath, targetApp=$targetApp")
    runOnUiThread {
      try {
        val file = java.io.File(imagePath)
        if (!file.exists()) {
          Log.e(TAG, "Image file not found: $imagePath")
          Toast.makeText(this@MainActivity, "图片文件不存在", Toast.LENGTH_SHORT).show()
          return@runOnUiThread
        }
        
        val targetPackageName = when (targetApp) {
          "wechat" -> "com.tencent.mm"
          "qq" -> "com.tencent.mobileqq"
          else -> {
            Log.e(TAG, "Unknown target app: $targetApp")
            Toast.makeText(this@MainActivity, "未知的目标应用: $targetApp", Toast.LENGTH_SHORT).show()
            return@runOnUiThread
          }
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
        
        val intent = android.content.Intent(android.content.Intent.ACTION_SEND).apply {
          type = "image/*"
          putExtra(android.content.Intent.EXTRA_STREAM, uri)
          setPackage(targetPackageName)
          addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
          addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
          addFlags(android.content.Intent.FLAG_ACTIVITY_CLEAR_TOP)
        }
        
        startActivity(intent)
        Log.i(TAG, "Share intent started successfully")
        Toast.makeText(this@MainActivity, "正在分享...", Toast.LENGTH_SHORT).show()
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
}
