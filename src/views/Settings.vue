<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Snackbar, Dialog } from '@varlet/ui';
import Icon from "../components/Icon.vue";
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

onMounted(async () => {
  await loadConfig();
});

async function loadConfig() {
  try {
    const result = await invoke<Config>('get_config');
    if (result) {
      config.value = result;
      updateColorMode(result.color_mode);
    }
  } catch (error) {
    console.error('Failed to load config:', error);
  }
}

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}

function updateColorMode(mode: string) {
  const htmlElement = document.documentElement;
  
  if (mode === 'system') {
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      htmlElement.classList.add('var-dark');
    } else {
      htmlElement.classList.remove('var-dark');
    }
    
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (config.value.color_mode === 'system') {
        if (e.matches) {
          htmlElement.classList.add('var-dark');
        } else {
          htmlElement.classList.remove('var-dark');
        }
      }
    });
  } else if (mode === 'dark') {
    htmlElement.classList.add('var-dark');
  } else {
    htmlElement.classList.remove('var-dark');
  }
  
  window.dispatchEvent(new CustomEvent('colorModeChanged', { detail: mode }));
}

async function selectMemeDir() {
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

async function autoSaveConfig() {
  try {
    isSaving.value = true;
    await invoke('update_config', { config: config.value });
    updateColorMode(config.value.color_mode);
  } catch (error) {
    console.error('Failed to save config:', error);
    Snackbar.error('配置保存失败');
  } finally {
    isSaving.value = false;
  }
}

function goToModeManagement() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'mode' }));
}

function goToGroupManagement() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'group' }));
}

function goToKeywordManagement() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'keyword' }));
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

async function generateKeywordsFile() {
  try {
    await invoke('generate_keywords_file', { 
      memeDir: config.value.meme_dir,
      generatePinyin: true,
      generateAcronym: true
    });
    Snackbar.success('关键词文件生成成功');
  } catch (error) {
    console.error('Failed to generate keywords file:', error);
    Snackbar.error('生成关键词文件失败');
  }
}
</script>

<template>
  <div class="settings">
    <!-- 圆角卡片式app bar -->
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
            <label>表情包存储目录</label>
            <p class="setting-desc">选择本地表情包的存储位置</p>
          </div>
          <div class="setting-control">
            <var-input
              v-model="config.meme_dir"
              readonly
              placeholder="未选择目录"
              class="dir-input"
            />
            <var-button type="primary" @click="selectMemeDir">
              <Icon name="folder-3" :size="18" /> 浏览
            </var-button>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>颜色模式</label>
            <p class="setting-desc">选择应用的主题颜色</p>
          </div>
          <div class="setting-control">
            <var-select v-model="config.color_mode" @change="autoSaveConfig">
              <var-option value="system" label="跟随系统" />
              <var-option value="light" label="浅色模式" />
              <var-option value="dark" label="深色模式" />
            </var-select>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>主题风格</label>
            <p class="setting-desc">选择应用的界面风格</p>
          </div>
          <div class="setting-control">
            <var-select v-model="config.theme_style" @change="autoSaveConfig">
              <var-option value="modern" label="现代风格" />
              <var-option value="minimal" label="极简风格" />
              <var-option value="default" label="默认风格" />
            </var-select>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>默认分享应用</label>
            <p class="setting-desc">移动端分享时的默认目标应用</p>
          </div>
          <div class="setting-control">
            <var-select v-model="config.share_app" @change="autoSaveConfig">
              <var-option value="wechat" label="微信" />
              <var-option value="qq" label="QQ" />
              <var-option value="" label="每次询问" />
            </var-select>
          </div>
        </var-cell>
        
        <var-cell class="setting-item">
          <div class="setting-label">
            <label>网格大小</label>
            <p class="setting-desc">主页表情包网格的列数（2-8）</p>
          </div>
          <div class="setting-control">
            <div class="slider-container">
              <var-slider
                v-model="config.grid_size"
                :min="2"
                :max="8"
                :step="1"
                @change="autoSaveConfig"
                class="grid-slider"
              />
              <span class="slider-value">{{ config.grid_size }} 列</span>
            </div>
          </div>
        </var-cell>
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="price-tag-3" :size="24" />
          关键词文件管理
        </h2>
        
        <div class="card">
          <div class="card-item">
            <div class="card-item-label">
              <label>生成关键词文件</label>
              <p class="card-item-desc">自动生成 keywords.toml 文件</p>
            </div>
            <div class="card-item-control">
              <var-button type="primary" @click="generateKeywordsFile">
                <Icon name="file-text" :size="18" /> 生成文件
              </var-button>
            </div>
          </div>
        </div>
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="tools" :size="24" />
          高级设置
        </h2>
        
        <div class="card">
          <div class="card-item">
            <div class="card-item-label">
              <label>重置设置</label>
              <p class="card-item-desc">将所有设置恢复为默认值</p>
            </div>
            <div class="card-item-control">
              <var-button type="danger" @click="resetToDefaults">
                <Icon name="refresh" :size="18" /> 重置
              </var-button>
            </div>
          </div>
        </div>
      </div>
      
      <div class="settings-section">
        <h2>
          <Icon name="database-2" :size="24" />
          数据管理
        </h2>
        
        <div class="card">
          <div class="card-item" @click="goToModeManagement">
            <div class="card-item-label">
              <label>模式管理</label>
              <p class="card-item-desc">管理表情包模式</p>
            </div>
            <div class="card-item-control">
              <Icon name="arrow-right-s" :size="24" color="var(--color-text-tertiary)" />
            </div>
          </div>
          
          <div class="card-divider"></div>
          
          <div class="card-item" @click="goToGroupManagement">
            <div class="card-item-label">
              <label>分组管理</label>
              <p class="card-item-desc">管理表情包分组</p>
            </div>
            <div class="card-item-control">
              <Icon name="arrow-right-s" :size="24" color="var(--color-text-tertiary)" />
            </div>
          </div>
          
          <div class="card-divider"></div>
          
          <div class="card-item" @click="goToKeywordManagement">
            <div class="card-item-label">
              <label>关键词管理</label>
              <p class="card-item-desc">管理搜索关键词</p>
            </div>
            <div class="card-item-control">
              <Icon name="arrow-right-s" :size="24" color="var(--color-text-tertiary)" />
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
            <p class="description">本地表情包分享和管理工具</p>
            
            <div class="features">
              <var-chip type="primary" size="small">模式管理</var-chip>
              <var-chip type="primary" size="small">分组管理</var-chip>
              <var-chip type="primary" size="small">智能搜索</var-chip>
              <var-chip type="primary" size="small">关键词管理</var-chip>
            </div>
            
            <div class="platform-info">
              <p>支持平台：Linux (x86_64) / Android (aarch64)</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
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

/* 卡片样式 */
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

/* 关于卡片 */
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

/* 悬浮 AppBar */
.floating-app-bar {
  margin: 16px;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}

/* 基本设置项样式更新 */
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

/* 章节标题 */
.settings-section h2 {
  color: var(--color-text);
  font-weight: 600;
}
</style>