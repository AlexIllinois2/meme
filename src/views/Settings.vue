<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/plugin-dialog';
import { Snackbar } from '@varlet/ui';
import Icon from "../components/Icon.vue";
import FolderPicker from "../components/FolderPicker.vue";
import type { Config } from "../types";

const config = ref<Config>({
  meme_dir: '',
  color_mode: 'system',
  theme_style: 'modern',
  last_mode: 1,
  last_group: 1,
  share_app: 'wechat',
  grid_size: 4,
  pinyin_search: false,
  acronym_search: false,
  global_floating_window: false
});

const isSaving = ref(false);
const appVersion = ref('0.0.0'); // ✅ 改为响应式变量
const showFolderPicker = ref(false);
const showColorModePopup = ref(false);
const showThemePopup = ref(false);

const colorModeOptions: { value: 'system' | 'light' | 'dark'; label: string }[] = [
  { value: 'system', label: '跟随系统' },
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' }
];

const colorModeLabel = computed(() => {
  const option = colorModeOptions.find(o => o.value === config.value.color_mode);
  return option ? option.label : '跟随系统';
});

function selectColorMode(value: 'system' | 'light' | 'dark') {
  config.value.color_mode = value;
  updateColorMode(value);
  autoSaveConfig();
  showColorModePopup.value = false;
}

const themeStyleOptions: { value: 'default' | 'modern' | 'minimal'; label: string }[] = [
  { value: 'default', label: '默认' },
  { value: 'modern', label: '现代' },
  { value: 'minimal', label: '极简' }
];

function selectThemeStyle(value: 'default' | 'modern' | 'minimal') {
  config.value.theme_style = value;
  autoSaveConfig();
  showThemePopup.value = false;
}

// 用于存储 matchMedia 监听器引用，以便在组件卸载时移除
let colorSchemeListener: ((e: MediaQueryListEvent) => void) | null = null;
let colorSchemeQuery: MediaQueryList | null = null;

// 记录进入设置页时的原始目录，用于返回时判断是否变化
const originalMemeDir = ref('');

onMounted(async () => {
  await loadConfig();
  // 保存原始目录
  originalMemeDir.value = config.value.meme_dir || '';
  
  // ✅ 异步获取版本号
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.error('Failed to get version:', error);
  }
  
  // Android 返回键监听
  if (isAndroidTauri()) {
    window.addEventListener('tauri-android-back', handleAndroidBack);
  }
});

onUnmounted(() => {
  // 清理 matchMedia 监听器
  if (colorSchemeQuery && colorSchemeListener) {
    colorSchemeQuery.removeEventListener('change', colorSchemeListener);
    colorSchemeQuery = null;
    colorSchemeListener = null;
  }
  
  // 移除 Android 返回键监听
  if (isAndroidTauri()) {
    window.removeEventListener('tauri-android-back', handleAndroidBack);
  }
});

async function loadConfig() {
  try {
    const result = await invoke<Config>('get_config');
    if (result) {
      // 逐个字段赋值，确保响应式更新
      config.value.meme_dir = result.meme_dir || '';
      config.value.color_mode = result.color_mode || 'system';
      config.value.theme_style = result.theme_style || 'modern';
      config.value.last_mode = result.last_mode || 1;
      config.value.last_group = result.last_group || 1;
      config.value.share_app = result.share_app || '';
      config.value.grid_size = result.grid_size || 4;
      config.value.pinyin_search = result.pinyin_search || false;
      config.value.acronym_search = result.acronym_search || false;
      config.value.global_floating_window = result.global_floating_window || false;
      // 只更新配置，但不会重置颜色模式 - 让App.vue的currentColorMode保持不变
      updateColorMode(result.color_mode);
      
      // Android 平台：从原生服务同步悬浮窗实际状态
      if (isAndroidTauri()) {
        try {
          const nativeEnabled = (window as any).AndroidNative?.isFloatingWindowEnabled?.();
          if (nativeEnabled !== undefined) {
            config.value.global_floating_window = nativeEnabled === true;
          }
        } catch (e) {
          console.warn('Failed to get floating window status:', e);
        }
      }
    }
  } catch (error) {
    console.error('Failed to load config:', error);
  }
}

async function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}

function updateColorMode(mode: string) {
  const htmlElement = document.documentElement;
  
  // 清理旧的监听器
  if (colorSchemeQuery && colorSchemeListener) {
    colorSchemeQuery.removeEventListener('change', colorSchemeListener);
    colorSchemeQuery = null;
    colorSchemeListener = null;
  }
  
  if (mode === 'system') {
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      htmlElement.classList.add('var-dark');
    } else {
      htmlElement.classList.remove('var-dark');
    }
    
    // 设置新的监听器
    colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
    colorSchemeListener = (e: MediaQueryListEvent) => {
      if (config.value.color_mode === 'system') {
        if (e.matches) {
          htmlElement.classList.add('var-dark');
        } else {
          htmlElement.classList.remove('var-dark');
        }
      }
    };
    colorSchemeQuery.addEventListener('change', colorSchemeListener);
  } else if (mode === 'dark') {
    htmlElement.classList.add('var-dark');
  } else {
    htmlElement.classList.remove('var-dark');
  }
  
  window.dispatchEvent(new CustomEvent('colorModeChanged', { detail: mode }));
}

async function toggleGlobalFloatingWindow(enabled: boolean) {
  config.value.global_floating_window = enabled;
  await autoSaveConfig();
  
  // 通知其他组件悬浮窗状态变化
  window.dispatchEvent(new CustomEvent('globalFloatingWindowChanged', { detail: enabled }));
  
  if (isAndroidTauri()) {
    if (enabled) {
      if (typeof (window as any).AndroidNative?.startFloatingWindow === 'function') {
        (window as any).AndroidNative.startFloatingWindow();
      }
    } else {
      if (typeof (window as any).AndroidNative?.stopFloatingWindow === 'function') {
        (window as any).AndroidNative.stopFloatingWindow();
      }
    }
  }
}



// 检测是否为移动端 - 预留功能
// const isMobile = () => {
//   return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) 
//     || window.innerWidth < 768;
// };

// 检测是否为 Android Tauri 环境
const isAndroidTauri = () => {
  return /Android/i.test(navigator.userAgent);
};

// Android 返回键处理
function handleAndroidBack(event: any) {
  // 关闭所有弹窗
  if (showColorModePopup.value) {
    showColorModePopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showThemePopup.value) {
    showThemePopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showFolderPicker.value) {
    showFolderPicker.value = false;
    event.preventDefault?.();
    return true;
  }
  // 返回首页
  goBack();
  event.preventDefault?.();
  return true;
}

// 检测是否为桌面端
const isDesktop = () => {
  return !(/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent));
};

async function selectMemeDir() {
  // Android Tauri 环境使用 tauri-plugin-android-fs 插件的目录选择器
  if (isAndroidTauri()) {
    try {
      // 调用 Rust 后端的 Android 目录选择器
      const selectedPath = await invoke<string>('select_directory_android');
      
      if (selectedPath) {
        // 与当前目录相同，无需更改
        if (selectedPath === config.value.meme_dir) {
          Snackbar.info('未更改');
          return;
        }

        // 验证路径是否可访问
        try {
          const fs = await import('@tauri-apps/plugin-fs');
          await fs.readDir(selectedPath);
          
          config.value.meme_dir = selectedPath;
          await autoSaveConfig();
          Snackbar.info('正在初始化数据...');
          
          // 执行全量刷新
          try {
            const result = await invoke<string>("full_refresh", { memeDir: selectedPath });
            console.log('[Settings] Auto refresh result:', result);
            
            // 保存当前状态用于重启后恢复
            const savedPage = localStorage.getItem('meme_active_page') || 'home';
            localStorage.setItem('meme_restore_state', JSON.stringify({
              page: savedPage,
            }));
            
            Snackbar.success('数据初始化完成，应用将重启...');
            // 延迟重启，让 snackbar 显示一下
            setTimeout(() => location.reload(), 800);
          } catch (refreshError) {
            console.error('[Settings] Auto refresh failed:', refreshError);
            // 即使刷新失败也重启，让用户看到新目录
            localStorage.setItem('meme_restore_state', JSON.stringify({
              page: 'settings'
            }));
            Snackbar.warning('数据初始化失败，请手动刷新');
            setTimeout(() => location.reload(), 1500);
          }
        } catch (e) {
          console.error('Cannot access path:', e);
          if (typeof (window as any).AndroidNative?.requestStoragePermission === 'function') {
            (window as any).AndroidNative.requestStoragePermission();
          }
          Snackbar.warning('无法访问目录，请授予存储权限后重试');
        }
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
      Snackbar.error('选择文件夹失败: ' + error);
    }
    return;
  }
  
  // 其他移动端使用内嵌文件夹浏览器
  if (!isDesktop()) {
    showFolderPicker.value = true;
    return;
  }
  
  // 桌面端使用系统文件选择器
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择表情包文件夹'
    });
    
    if (selected) {
      const dirPath = selected as string;
      if (dirPath === config.value.meme_dir) {
        Snackbar.info('未更改');
        return;
      }
      config.value.meme_dir = dirPath;
      await autoSaveConfig();
      Snackbar.info('正在初始化数据...');
      
      // 执行全量刷新
      try {
        const result = await invoke<string>("full_refresh", { memeDir: dirPath });
        console.log('[Settings] Auto refresh result:', result);
        
        localStorage.setItem('meme_restore_state', JSON.stringify({ page: 'settings' }));
        Snackbar.success('数据初始化完成，应用将重启...');
        setTimeout(() => location.reload(), 800);
      } catch (refreshError) {
        console.error('[Settings] Auto refresh failed:', refreshError);
        localStorage.setItem('meme_restore_state', JSON.stringify({ page: 'settings' }));
        Snackbar.warning('数据初始化失败，请手动刷新');
        setTimeout(() => location.reload(), 1500);
      }
    }
  } catch (error) {
    console.error('Failed to select directory:', error);
    Snackbar.error('选择目录失败');
  }
}

function onFolderSelected(path: string) {
  config.value.meme_dir = path;
  autoSaveConfig();
  Snackbar.success(`已选择目录: ${path}`);
}

async function autoSaveConfig() {
  try {
    isSaving.value = true;
    
    // 确保所有字段类型正确，防止 Varlet UI 组件返回非预期类型
    const safeConfig: Config = {
      meme_dir: String(config.value.meme_dir || ''),
      color_mode: typeof config.value.color_mode === 'string' 
        ? config.value.color_mode 
        : 'system',
      theme_style: typeof config.value.theme_style === 'string' 
        ? config.value.theme_style 
        : 'modern',
      last_mode: Number(config.value.last_mode) || 1,
      last_group: Number(config.value.last_group) || 1,
      share_app: typeof config.value.share_app === 'string' 
        ? config.value.share_app 
        : '',
      grid_size: Number(config.value.grid_size) || 4,
      pinyin_search: Boolean(config.value.pinyin_search),
      acronym_search: Boolean(config.value.acronym_search),
      global_floating_window: Boolean(config.value.global_floating_window),
    };
    
    await invoke('update_config', { config: safeConfig });
    updateColorMode(safeConfig.color_mode);
  } catch (error) {
    console.error('Failed to save config:', error);
    Snackbar.error('配置保存失败');
  } finally {
    isSaving.value = false;
  }
}

// 重置为默认设置 - 预留功能
// function resetToDefaults() {
//   Dialog({
//     title: '确认重置',
//     message: '确定要重置为默认设置吗？',
//     confirmButton: true,


</script>

<template>
  <div class="settings">
    <var-app-bar class="floating-app-bar" title="设置">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div class="settings-section">
        <h2>
          <Icon name="settings-3" :size="24" />
          基本设置
        </h2>
        
        <div class="setting-row" @click="selectMemeDir">
          <Icon name="folder-3" :size="22" class="setting-icon" />
          <div class="setting-label">
            <label>表情包文件夹</label>
            <p class="setting-path">{{ config.meme_dir || '点击选择文件夹' }}</p>
          </div>
          <Icon name="chevron-right" :size="20" class="arrow-icon" />
        </div>
        
        <div class="setting-row" @click="showColorModePopup = true">
          <div class="setting-label">
            <label>主题</label>
          </div>
          <div class="setting-control">
            <span class="selected-value">{{ colorModeLabel }}</span>
            <Icon name="chevron-right" :size="20" class="arrow-icon" />
          </div>
        </div>
        
        <!-- 全局悬浮窗开关 - 仅 Android 显示 -->
        <div class="setting-row" v-if="isAndroidTauri()">
          <div class="setting-label">
            <label>全局悬浮窗</label>
            <p class="setting-desc">在其他应用中快速打开搜索</p>
          </div>
          <div class="setting-control">
            <var-switch v-model="config.global_floating_window" @change="toggleGlobalFloatingWindow" />
          </div>
        </div>
        
        <!-- 使用情况访问权限授权 - 仅 Android 显示 -->
        <!-- <div class="setting-row" v-if="isAndroidTauri()" @click="requestUsageStatsPermission">
          <div class="setting-label">
            <label>授权使用情况访问</label>
            <p class="setting-desc">允许悬浮窗根据当前应用自动显示/隐藏</p>
            <p class="setting-hint">点击后将跳转到系统设置页面，请找到“咪萌”并开启权限</p>
          </div>
          <div class="setting-control">
            <Icon name="chevron-right" :size="20" class="arrow-icon" />
          </div>
        </div> -->
        
        <!-- <div class="setting-row" @click="showThemePopup = true">
          <div class="setting-label">
            <label>主题</label>
            <p class="setting-desc">界面风格</p>
          </div>
          <div class="setting-control">
            <span class="selected-value">{{ themeStyleLabel }}</span>
            <Icon name="chevron-right" :size="20" class="arrow-icon" />
          </div>
        </div> -->
        
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="information" :size="24" />
          关于
        </h2>
        
        <div class="card about-card">
          <div class="about-content">
            <div class="app-logo">≧▽≦</div>
            <p class="version">v{{ appVersion }}</p>
            <p class="description">本地表情包管理和分享工具</p>
            
            <div class="platform-info">
              <p>支持平台：Linux (x86_64) / Android (aarch64)</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
  
  <FolderPicker 
    v-model="showFolderPicker" 
    @select="onFolderSelected"
    :default-path="config.meme_dir || undefined"
  />
  
  <var-popup :show="showColorModePopup" @click-overlay="showColorModePopup = false">
    <div class="settings-popup-content">
      <div class="settings-popup-header">
        <h3>主题</h3>
        <var-button text round @click="showColorModePopup = false">
          <Icon name="x" :size="20" />
        </var-button>
      </div>
      <div class="settings-popup-body">
        <div 
          v-for="option in colorModeOptions" 
          :key="option.value" 
          class="popup-option"
          :class="{ active: config.color_mode === option.value }"
          @click="selectColorMode(option.value)"
        >
          <div class="radio-wrapper">
            <div class="radio" :class="{ checked: config.color_mode === option.value }"></div>
          </div>
          <span class="option-label">{{ option.label }}</span>
        </div>
      </div>
      <div class="settings-popup-footer">
        <var-button type="default" block @click="showColorModePopup = false">取消</var-button>
      </div>
    </div>
  </var-popup>
  
  <var-popup :show="showThemePopup" @click-overlay="showThemePopup = false">
    <div class="settings-popup-content">
      <div class="settings-popup-header">
        <h3>主题</h3>
        <var-button text round @click="showThemePopup = false">
          <Icon name="x" :size="20" />
        </var-button>
      </div>
      <div class="settings-popup-body">
        <div 
          v-for="option in themeStyleOptions" 
          :key="option.value" 
          class="popup-option"
          :class="{ active: config.theme_style === option.value }"
          @click="selectThemeStyle(option.value)"
        >
          <div class="radio-wrapper">
            <div class="radio" :class="{ checked: config.theme_style === option.value }"></div>
          </div>
          <span class="option-label">{{ option.label }}</span>
        </div>
      </div>
      <div class="settings-popup-footer">
        <var-button type="default" block @click="showThemePopup = false">取消</var-button>
      </div>
    </div>
  </var-popup>
</template>

<style scoped>
.settings {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section h2 {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 16px 0;
  font-size: 18px;
  color: var(--text-color);
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  margin-bottom: 8px;
  border-radius: 12px;
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: background-color 0.2s ease, box-shadow 0.2s ease;
}

.setting-row:hover {
  background: var(--color-surface-variant);
  box-shadow: var(--shadow-md);
}

.setting-icon {
  margin-right: 12px;
  color: var(--color-text-secondary);
}

.setting-label {
  flex: 1;
  min-width: 200px;
}

.setting-label label {
  display: block;
  font-weight: 600;
  font-size: 14px;
  color: var(--text-color);
  margin-bottom: 4px;
}

.setting-desc {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.setting-hint {
  margin: 4px 0 0 0;
  font-size: 11px;
  color: var(--color-primary);
  font-style: italic;
}

.setting-control {
  flex: 1;
  display: flex;
  gap: 12px;
  align-items: center;
  max-width: 400px;
  justify-content: flex-end;
}

.selected-value {
  font-size: 14px;
  color: var(--text-secondary);
}

.arrow-icon {
  color: var(--text-secondary);
}

.dir-input {
  flex: 1;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.grid-slider {
  flex: 1;
}

.slider-value {
  min-width: 60px;
  text-align: right;
  font-weight: 600;
  color: var(--primary-color);
  font-size: 14px;
}

.card {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
}

.card-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.card-item:hover {
  background-color: var(--color-surface-variant);
}

.card-item-label {
  flex: 1;
}

.card-item-label label {
  display: block;
  font-weight: 600;
  font-size: 15px;
  color: var(--color-text);
  margin-bottom: 4px;
}

.card-item-desc {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.card-item-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-divider {
  height: 1px;
  background-color: var(--color-border);
  margin: 0 20px;
}

.about-card {
  padding: 32px 24px;
  text-align: center;
}

.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.app-logo {
  font-size: 56px;
  font-weight: 700;
  background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-dark) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 8px;
}

.version {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.description {
  margin: 0 0 20px 0;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.features {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 8px;
  margin-bottom: 24px;
}

.platform-info {
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
  width: 100%;
}

.platform-info p {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.floating-app-bar {
  margin: 16px;
  margin-top: max(16px, env(safe-area-inset-top));
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}

.setting-item {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  margin-bottom: 12px;
  box-shadow: var(--shadow-sm);
}

.setting-label label {
  color: var(--color-text);
}

.setting-desc {
  color: var(--color-text-secondary);
}

.slider-value {
  color: var(--color-primary);
}

.settings-section h2 {
  color: var(--color-text);
  font-weight: 600;
}

.settings-popup-content {
  width: 320px;
  max-width: 90vw;
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.settings-popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border);
}

.settings-popup-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.settings-popup-body {
  padding: 12px 24px;
}

.settings-popup-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--color-border);
}

.popup-option {
  display: flex;
  align-items: center;
  padding: 16px 0;
  cursor: pointer;
  transition: background-color 0.2s ease;
  border-radius: 12px;
  margin: 4px 0;
}

.popup-option:hover {
  background-color: var(--color-surface-variant);
}

.popup-option.active {
  background-color: var(--color-surface-variant);
}

.radio-wrapper {
  margin-right: 16px;
}

.radio {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--color-border);
  position: relative;
  transition: all 0.2s ease;
}

.radio.checked {
  border-color: var(--color-primary);
}

.radio.checked::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--color-primary);
}

.option-label {
  font-size: 16px;
  color: var(--color-text);
  flex: 1;
}

.selected-value {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.setting-path {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
}
</style>