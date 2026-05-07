<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Snackbar, Dialog, ActionSheet } from '@varlet/ui';
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
  acronym_search: false
});

const isSaving = ref(false);
const appVersion = '1.0.0';
const showFolderPicker = ref(false);

// 用于存储 matchMedia 监听器引用，以便在组件卸载时移除
let colorSchemeListener: ((e: MediaQueryListEvent) => void) | null = null;
let colorSchemeQuery: MediaQueryList | null = null;

// 记录进入设置页时的原始目录，用于返回时判断是否变化
const originalMemeDir = ref('');

onMounted(async () => {
  await loadConfig();
  // 保存原始目录
  originalMemeDir.value = config.value.meme_dir || '';
});

onUnmounted(() => {
  // 清理 matchMedia 监听器
  if (colorSchemeQuery && colorSchemeListener) {
    colorSchemeQuery.removeEventListener('change', colorSchemeListener);
    colorSchemeQuery = null;
    colorSchemeListener = null;
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
      updateColorMode(result.color_mode);
    }
  } catch (error) {
    console.error('Failed to load config:', error);
  }
}

async function goBack() {
  // 检查目录是否发生变化
  const currentDir = config.value.meme_dir || '';
  if (currentDir && currentDir !== originalMemeDir.value) {
    // 目录发生变化，需要检查新目录是否有效并决定是否刷新
    try {
      // 触发目录变化事件，让主页处理刷新逻辑
      window.dispatchEvent(new CustomEvent('memeDirChanged', { 
        detail: { newDir: currentDir, oldDir: originalMemeDir.value } 
      }));
    } catch (e) {
      console.error('Failed to handle dir change:', e);
    }
  }
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

// 检测是否为移动端
const isMobile = () => {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) 
    || window.innerWidth < 768;
};

// 检测是否为 Android Tauri 环境
const isAndroidTauri = () => {
  return /android/i.test(navigator.userAgent) && typeof (window as any).__TAURI__ !== 'undefined';
};

// 检测是否为桌面端
const isDesktop = () => {
  return !(/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent));
};

// 显示 Android 路径选择 ActionSheet
async function showAndroidPathPicker(): Promise<string | null> {
  const actions = [
    { name: '内部存储', path: '/storage/emulated/0' },
    { name: '下载', path: '/storage/emulated/0/Download' },
    { name: '图片', path: '/storage/emulated/0/Pictures' },
    { name: '文档', path: '/storage/emulated/0/Documents' },
    { name: '自定义路径', path: 'custom' }
  ];
  
  return new Promise((resolve) => {
    ActionSheet({
      title: '选择存储目录',
      actions: actions.map(a => ({ name: a.name })),
      onSelect: (action: any) => {
        const selected = actions.find(a => a.name === action.name);
        if (selected?.path === 'custom') {
          const customPath = window.prompt('请输入路径（如 /storage/emulated/0/MyMemes）：');
          resolve(customPath);
        } else {
          resolve(selected?.path || null);
        }
      },
      onClose: () => {
        resolve(null);
      }
    });
  });
}

async function selectMemeDir() {
  // Android Tauri 环境使用 ActionSheet 选择路径
  if (isAndroidTauri()) {
    try {
      const selectedPath = await showAndroidPathPicker();
      
      if (selectedPath) {
        // 验证路径是否可访问
        try {
          const fs = await import('@tauri-apps/plugin-fs');
          await fs.readDir(selectedPath);
          
          config.value.meme_dir = selectedPath;
          await autoSaveConfig();
          Snackbar.success(`已选择目录: ${selectedPath}`);
        } catch (e) {
          console.error('Cannot access path:', e);
          Snackbar.error('无法访问该目录，请检查权限或路径是否正确');
        }
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
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
      title: '选择表情包存储目录'
    });
    
    if (selected) {
      config.value.meme_dir = selected as string;
      await autoSaveConfig();
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

function resetToDefaults() {
  Dialog({
    title: '确认重置',
    message: '确定要重置为默认设置吗？',
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '确定',
    cancelButtonText: '取消'
  }).then(() => {
    config.value = {
      meme_dir: '',
      color_mode: 'system',
      theme_style: 'modern',
      last_mode: 1,
      last_group: 1,
      share_app: 'wechat',
      grid_size: 4,
      pinyin_search: false,
      acronym_search: false
    };
    autoSaveConfig();
    Snackbar.success('已重置为默认设置');
  });
}

// 处理路径输入框失去焦点事件
async function onPathInputBlur() {
  // 只在 Android Tauri 环境下处理手动输入
  if (!isAndroidTauri() || !config.value.meme_dir) return;
  
  // 验证路径是否可访问
  try {
    const fs = await import('@tauri-apps/plugin-fs');
    await fs.readDir(config.value.meme_dir);
    await autoSaveConfig();
    Snackbar.success(`已保存目录: ${config.value.meme_dir}`);
  } catch (e) {
    console.error('Cannot access path:', e);
    Snackbar.error('无法访问该目录，请检查权限或路径是否正确');
  }
}

async function generateKeywordsFile() {
  try {
    await invoke('generate_keywords_file', { 
      memeDir: config.value.meme_dir,
      generatePinyin: true,
      generateAcronym: true
    });
    Snackbar.success('关键词文件生成成功');
    
    // 触发刷新索引事件，让主页刷新数据
    window.dispatchEvent(new CustomEvent('refreshIndexAfterKeywordsGenerated'));
  } catch (error) {
    console.error('Failed to generate keywords file:', error);
    Snackbar.error('生成关键词文件失败');
  }
}
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
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>本地存储目录</label>
            <p class="setting-desc">本地表情包路径</p>
          </div>
          <div class="setting-control">
            <var-input
              v-model="config.meme_dir"
              :readonly="!isAndroidTauri()"
              class="dir-input"
              @blur="onPathInputBlur"
            />
            <var-button type="primary" @click="selectMemeDir">
              <Icon name="folder-3" :size="18" /> {{ isAndroidTauri() ? '选择' : '浏览' }}
            </var-button>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>颜色模式</label>
          </div>
          <div class="setting-control">
            <var-select v-model="config.color_mode" @change="autoSaveConfig">
              <var-option value="system" label="系统" />
              <var-option value="light" label="浅色" />
              <var-option value="dark" label="深色" />
            </var-select>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>主题</label>
            <p class="setting-desc">界面风格</p>
          </div>
          <div class="setting-control">
            <var-select v-model="config.theme_style" @change="autoSaveConfig">
              <var-option value="default" label="默认" />
              <var-option value="modern" label="现代" />
              <var-option value="minimal" label="极简" />
            </var-select>
          </div>
        </var-cell>
        
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="price-tag-3" :size="24" />
          关键词
        </h2>
        
        <div class="card">
          <div class="card-item">
            <div class="card-item-label">
              <label>生成关键词文件</label>
              <p class="card-item-desc">自动生成 keywords.toml </p>
            </div>
            <div class="card-item-control">
              <var-button type="primary" @click="generateKeywordsFile">
                <Icon name="file-text" :size="18" /> 生成
              </var-button>
            </div>
          </div>
        </div>
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="information" :size="24" />
          关于
        </h2>
        
        <div class="card about-card">
          <div class="about-content">
            <div class="app-logo">meme</div>
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

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  margin-bottom: 8px;
  border-radius: 8px;
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

.setting-control {
  flex: 1;
  display: flex;
  gap: 12px;
  align-items: center;
  max-width: 400px;
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
</style>