<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { Dialog, Snackbar } from '@varlet/ui';
import Icon from './Icon.vue';

interface FolderItem {
  name: string;
  path: string;
  isDirectory: boolean;
}

const props = defineProps<{
  modelValue: boolean;
  defaultPath?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'select', path: string): void;
  (e: 'cancel'): void;
}>();

const currentPath = ref('');
const folders = ref<FolderItem[]>([]);
const isLoading = ref(false);
const error = ref('');
const history = ref<string[]>([]);

// 本地存储的 key
const STORAGE_KEY = 'folder_picker_last_path';

// 获取默认路径
function getDefaultPath(): string {
  // 如果有传入默认路径，使用它
  if (props.defaultPath) {
    return props.defaultPath;
  }
  // 尝试从 localStorage 读取上次路径
  const savedPath = localStorage.getItem(STORAGE_KEY);
  if (savedPath) {
    return savedPath;
  }
  // 首次使用默认路径
  return '/storage/emulated/0';
}

// Android 端使用系统文件选择器
async function openAndroidFolderPicker() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择图片文件夹'
    });
    
    if (selected) {
      // 用户选择了文件夹，直接使用该路径
      const folderPath = Array.isArray(selected) ? selected[0] : selected;
      emit('select', folderPath);
      emit('update:modelValue', false);
      Dialog.close();
    } else {
      // 用户取消
      cancel();
    }
  } catch (e: any) {
    console.error('Failed to open folder picker:', e);
    Snackbar.error('打开文件夹选择器失败: ' + (e?.message || String(e)));
    cancel();
  }
}

// 加载目录内容
async function loadDirectory(path: string) {
  isLoading.value = true;
  error.value = '';
  
  try {
    // 动态导入 tauri fs 插件
    const fs = await import('@tauri-apps/plugin-fs');
    
    // 读取目录
    const entries = await fs.readDir(path);
    
    // 过滤出文件夹并按名称排序
    const dirs: FolderItem[] = [];
    
    for (const entry of entries) {
      // entry 可能是 name + children (目录) 或 name 单独 (文件)
      const entryName = entry.name;
      if (!entryName) continue;
      
      // 尝试判断是否为目录（有 children 属性的是目录）
      const isDir = 'children' in entry;
      
      if (isDir) {
        dirs.push({
          name: entryName,
          path: `${path}/${entryName}`.replace(/\/+/g, '/'),
          isDirectory: true
        });
      }
    }
    
    dirs.sort((a, b) => a.name.localeCompare(b.name));
    folders.value = dirs;
    currentPath.value = path;
    
    // 保存当前路径到 localStorage
    localStorage.setItem(STORAGE_KEY, path);
  } catch (e: any) {
    console.error('Failed to read directory:', e);
    const errorMsg = e?.message || String(e);
    
    // 提供更具体的错误信息
    if (errorMsg.includes('denied') || errorMsg.includes('permission')) {
      error.value = '权限被拒绝，请在设置中授予存储权限';
      Snackbar.error('存储权限被拒绝，请在系统设置中授权');
    } else if (errorMsg.includes('not found') || errorMsg.includes('exist')) {
      error.value = '目录不存在';
      Snackbar.error('目录不存在');
    } else {
      error.value = '无法读取目录: ' + errorMsg;
      Snackbar.error('读取失败: ' + errorMsg);
    }
  } finally {
    isLoading.value = false;
  }
}

// 进入文件夹
function enterFolder(folder: FolderItem) {
  history.value.push(currentPath.value);
  loadDirectory(folder.path);
}

// 返回上级
function goBack() {
  if (history.value.length > 0) {
    const parentPath = history.value.pop()!;
    loadDirectory(parentPath);
  } else {
    // 如果没有历史，尝试从路径推断
    const parent = currentPath.value.substring(0, currentPath.value.lastIndexOf('/'));
    if (parent && parent !== currentPath.value) {
      loadDirectory(parent || '/');
    }
  }
}

// 选择当前目录
function selectCurrentFolder() {
  emit('select', currentPath.value);
  emit('update:modelValue', false);
  Dialog.close();
}

// 取消
function cancel() {
  emit('cancel');
  emit('update:modelValue', false);
  Dialog.close();
}

// 监听显示状态
watch(() => props.modelValue, (visible) => {
  if (visible) {
    const defaultPath = getDefaultPath();
    loadDirectory(defaultPath);
  }
});

onMounted(() => {
  if (props.modelValue) {
    const defaultPath = getDefaultPath();
    loadDirectory(defaultPath);
  }
});
</script>

<template>
  <teleport to="body">
    <div v-if="modelValue" class="folder-picker-overlay" @click.self="cancel">
      <div class="folder-picker">
        <!-- 头部 -->
        <div class="picker-header">
          <button class="btn-icon" @click="cancel">
            <Icon name="close" :size="24" />
          </button>
          <h3 class="picker-title">选择文件夹</h3>
          <button class="btn-icon confirm-btn" @click="selectCurrentFolder">
            <Icon name="check" :size="24" />
          </button>
        </div>
        
        <!-- 路径栏 -->
        <div class="path-bar">
          <button class="btn-icon back-btn" @click="goBack" :disabled="currentPath === '/storage/emulated/0/' && history.length === 0">
            <Icon name="arrow-up" :size="20" />
          </button>
          <div class="current-path">{{ currentPath || '/' }}</div>
        </div>
        
        <!-- 文件夹列表 -->
        <div class="folder-list">
          <div v-if="isLoading" class="loading">
            <Icon name="loader-4" :size="32" class="spin" />
            <span>加载中...</span>
          </div>
          
          <div v-else-if="error" class="error">
            <Icon name="error-warning" :size="32" />
            <span>{{ error }}</span>
          </div>
          
          <div v-else-if="folders.length === 0" class="empty">
            <Icon name="folder-5" :size="48" />
            <span>空文件夹</span>
          </div>
          
          <div v-else class="folder-items">
            <div 
              v-for="folder in folders" 
              :key="folder.path"
              class="folder-item"
              @click="enterFolder(folder)"
            >
              <Icon name="folder-3" :size="24" color="var(--color-primary)" />
              <span class="folder-name">{{ folder.name }}</span>
              <Icon name="arrow-right-s" :size="20" color="var(--color-text-tertiary)" />
            </div>
          </div>
        </div>
        
        <!-- 底部按钮 -->
        <div class="picker-footer">
          <var-button type="default" @click="cancel">取消</var-button>
          <var-button type="primary" @click="selectCurrentFolder">
            选择此文件夹
          </var-button>
        </div>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.folder-picker-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 1000;
}

.folder-picker {
  background: var(--color-surface);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.picker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.picker-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.btn-icon {
  background: none;
  border: none;
  padding: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  transition: background-color 0.2s;
}

.btn-icon:active {
  background-color: var(--color-surface-variant);
}

.btn-icon:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.confirm-btn {
  color: var(--color-primary);
}

.path-bar {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-surface-variant);
  gap: 8px;
}

.back-btn {
  flex-shrink: 0;
}

.current-path {
  flex: 1;
  font-size: 14px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  min-height: 200px;
}

.loading,
.error,
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  gap: 12px;
  color: var(--color-text-secondary);
}

.error {
  color: var(--color-danger);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.folder-items {
  padding: 0 16px;
}

.folder-item {
  display: flex;
  align-items: center;
  padding: 12px 0;
  gap: 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border);
}

.folder-item:last-child {
  border-bottom: none;
}

.folder-item:active {
  background-color: var(--color-surface-variant);
  margin: 0 -16px;
  padding-left: 16px;
  padding-right: 16px;
}

.folder-name {
  flex: 1;
  font-size: 16px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-footer {
  display: flex;
  gap: 12px;
  padding: 16px;
  border-top: 1px solid var(--color-border);
  justify-content: flex-end;
}
</style>