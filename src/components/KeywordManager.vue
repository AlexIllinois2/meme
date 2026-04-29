<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import * as tauri from "@tauri-apps/api/core";
import { Snackbar } from '@varlet/ui';
import Icon from "./Icon.vue";

const invoke = tauri.invoke;

const props = defineProps<{
  show: boolean;
  groupName: string;
}>();

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void;
  (e: 'updated'): void;
}>();

const keywords = ref<string[]>([]);
const loading = ref(false);
const newKeyword = ref('');
const adding = ref(false);

// 加载关键词列表
async function loadKeywords() {
  if (!props.groupName) return;
  
  loading.value = true;
  try {
    const result = await invoke<string[]>("get_keywords_by_group_name", { 
      groupName: props.groupName 
    });
    keywords.value = result || [];
  } catch (error) {
    console.error("Failed to load keywords:", error);
    Snackbar.error("加载关键词失败");
  } finally {
    loading.value = false;
  }
}

// 移除关键词
async function removeKeyword(keyword: string) {
  try {
    await invoke("remove_keyword_from_group_name", {
      groupName: props.groupName,
      keyword: keyword
    });
    Snackbar.success("关键词已移除");
    await loadKeywords();
    emit("updated");
  } catch (error) {
    console.error("Failed to remove keyword:", error);
    Snackbar.error("移除关键词失败");
  }
}

// 添加关键词
async function addKeyword() {
  const trimmed = newKeyword.value.trim();
  if (!trimmed) {
    Snackbar.warning("请输入关键词");
    return;
  }
  
  if (keywords.value.includes(trimmed)) {
    Snackbar.warning("该关键词已存在");
    return;
  }
  
  adding.value = true;
  try {
    await invoke("add_keyword_to_group_name", {
      groupName: props.groupName,
      keyword: trimmed
    });
    Snackbar.success("关键词已添加");
    newKeyword.value = "";
    await loadKeywords();
    emit("updated");
  } catch (error) {
    console.error("Failed to add keyword:", error);
    Snackbar.error("添加关键词失败");
  } finally {
    adding.value = false;
  }
}

// 关闭弹窗
function close() {
  emit('update:show', false);
}

// 监听显示状态
watch(() => props.show, (newVal) => {
  if (newVal) {
    loadKeywords();
    newKeyword.value = '';
  }
});

onMounted(() => {
  if (props.show) {
    loadKeywords();
  }
});
</script>

<template>
  <var-popup class="keyword-manager-popup" :show="show" @click-overlay="close">
    <div class="keyword-manager">
      <div class="keyword-manager-header">
        <h3>管理关键词</h3>
          <button class="btn-icon" @click="close">
            <Icon name="close" :size="24" />
          </button>
      </div>
      
      <div class="keyword-manager-body">
        <div class="group-name-display">
          <Icon name="folder-3" :size="20" />
          <span>{{ groupName }}</span>
        </div>
        
        <!-- 关键词列表 -->
        <div class="keywords-list">
          <div v-if="loading" class="loading-state">
            <var-loading type="circle" size="small" />
            <span>加载中...</span>
          </div>
          
          <div v-else-if="keywords.length === 0" class="empty-state">
            <Icon name="price-tag-3" :size="32" />
            <p>暂无关联关键词</p>
          </div>
          
          <div v-else class="keywords-chips">
            <div
              v-for="(keyword, index) in keywords"
              :key="index"
              class="keyword-chip"
            >
              <span>{{ keyword }}</span>
              <var-button
                text
                round
                size="mini"
                class="remove-btn"
                @click="removeKeyword(keyword)"
              >
                <Icon name="close" :size="14" />
              </var-button>
            </div>
          </div>
        </div>
        
        <!-- 添加新关键词 -->
        <div class="add-keyword-section">
          <div class="add-keyword-row">
            <var-input
              v-model="newKeyword"
              placeholder="输入新关键词..."
              :disabled="adding"
              @keydown.enter="addKeyword"
              class="keyword-input"
            />
            <var-button
              type="primary"
              size="small"
              :loading="adding"
              :disabled="!newKeyword.trim()"
              @click="addKeyword"
            >
              <Icon name="add" :size="20" />
            </var-button>
          </div>
        </div>
      </div>
    </div>
  </var-popup>
</template>

<style scoped>
.keyword-manager {
  width: 320px;
  max-width: 90vw;
  background-color: var(--color-surface);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
}

/* 覆盖 var-popup 默认样式 */
.keyword-manager-popup :deep(.var-popup__overlay) {
  background-color: rgba(0, 0, 0, 0.5) !important;
}

.keyword-manager-popup :deep(.var-popup__content) {
  background-color: transparent !important;
  border-radius: 16px !important;
  box-shadow: none !important;
  overflow: hidden !important;
}

.keyword-manager-popup :deep(.var-popup) {
  border-radius: 16px !important;
  overflow: hidden !important;
  background-color: transparent !important;
}

/* 全局覆盖 */
:global(.var-popup) {
  background-color: transparent !important;
}

:global(.var-popup--center) {
  background-color: transparent !important;
}

:global(.var-popup__content) {
  background-color: transparent !important;
}

.keyword-manager-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface-variant);
}

.keyword-manager-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.keyword-manager-body {
  padding: 16px 20px;
}

.group-name-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background-color: var(--color-surface-variant);
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 14px;
  color: var(--color-text-secondary);
}

.group-name-display span {
  font-weight: 500;
  color: var(--color-text);
}

.keywords-list {
  min-height: 80px;
  max-height: 240px;
  overflow-y: auto;
  margin-bottom: 16px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  gap: 8px;
  color: var(--color-text-tertiary);
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.keywords-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 4px;
}

.keyword-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background-color: var(--color-primary-light);
  color: var(--color-primary);
  border-radius: 16px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.keyword-chip:hover {
  background-color: var(--color-primary);
  color: white;
}

.keyword-chip .remove-btn {
  opacity: 0.7;
  transition: opacity 0.2s;
}

.keyword-chip:hover .remove-btn {
  opacity: 1;
}

.keyword-chip :deep(.var-button) {
  color: inherit;
}

.add-keyword-section {
  border-top: 1px solid var(--color-border);
  padding-top: 16px;
}

.add-keyword-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.keyword-input {
  flex: 1;
}
</style>