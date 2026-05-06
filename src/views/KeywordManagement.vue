<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Snackbar, Dialog } from '@varlet/ui';
import Icon from "../components/Icon.vue";
import type { Keyword } from "../types";

const keywords = ref<Keyword[]>([]);
const selectedKeywords = ref<number[]>([]);
const showAddPopup = ref(false);
const newKeywordName = ref('');

// 编辑相关
const editingKeyword = ref<Keyword | null>(null);
const editKeywordName = ref('');

onMounted(() => {
  loadKeywords();
});

async function loadKeywords() {
  try {
    const result = await invoke<Keyword[]>('get_keywords');
    if (result) {
      keywords.value = result;
    }
  } catch (error) {
    console.error('Failed to load keywords:', error);
    Snackbar.error('加载关键词失败');
  }
}

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}

function startAddKeyword() {
  newKeywordName.value = '';
  showAddPopup.value = true;
}

function closeAddPopup() {
  showAddPopup.value = false;
  newKeywordName.value = '';
}

async function submitAddKeyword() {
  const trimmedName = newKeywordName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('关键词不能为空');
    return;
  }
  
  // 检查是否已存在同名关键词
  const existingKeyword = keywords.value.find(k => k.keyword === trimmedName);
  if (existingKeyword) {
    Snackbar.error('已存在同名关键词');
    return;
  }
  
  try {
    await invoke('add_keyword', { keyword: trimmedName });
    Snackbar.success('关键词创建成功');
    closeAddPopup();
    await loadKeywords();
  } catch (error) {
    console.error('Failed to add keyword:', error);
    Snackbar.error('创建关键词失败');
  }
}

function startEditKeyword(keyword: Keyword) {
  editingKeyword.value = keyword;
  editKeywordName.value = keyword.keyword;
}

function cancelEditKeyword() {
  editingKeyword.value = null;
  editKeywordName.value = '';
}

async function submitEditKeyword() {
  if (!editingKeyword.value) return;
  
  const trimmedName = editKeywordName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('关键词不能为空');
    return;
  }
  
  // 检查是否与其他关键词重名
  const existingKeyword = keywords.value.find(k => 
    k.keyword === trimmedName && k.id !== editingKeyword.value!.id
  );
  if (existingKeyword) {
    Snackbar.error('已存在同名关键词');
    return;
  }
  
  try {
    await invoke('update_keyword', {
      id: editingKeyword.value.id,
      keyword: trimmedName
    });
    Snackbar.success('关键词更新成功');
    cancelEditKeyword();
    await loadKeywords();
  } catch (error) {
    console.error('Failed to update keyword:', error);
    Snackbar.error('更新关键词失败');
  }
}

async function deleteSingleKeyword(keywordId: number) {
  Dialog({
    title: '确认删除',
    message: '确定要删除此关键词吗？',
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      await invoke('delete_keyword', { keywordId });
      Snackbar.success('关键词已删除');
      await loadKeywords();
      const index = selectedKeywords.value.indexOf(keywordId);
      if (index > -1) {
        selectedKeywords.value.splice(index, 1);
      }
    } catch (error) {
      console.error('Failed to delete keyword:', error);
      Snackbar.error('删除关键词失败');
    }
  });
}

function toggleKeywordSelection(keywordId: number) {
  const index = selectedKeywords.value.indexOf(keywordId);
  if (index > -1) {
    selectedKeywords.value.splice(index, 1);
  } else {
    selectedKeywords.value.push(keywordId);
  }
}

async function deleteSelectedKeywords() {
  if (selectedKeywords.value.length === 0) return;
  
  Dialog({
    title: '确认批量删除',
    message: `确定要删除选中的 ${selectedKeywords.value.length} 个关键词吗？`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      for (const keywordId of selectedKeywords.value) {
        await invoke('delete_keyword', { keywordId });
      }
      Snackbar.success('批量删除成功');
      selectedKeywords.value = [];
      await loadKeywords();
    } catch (error) {
      console.error('Failed to delete keywords:', error);
      Snackbar.error('批量删除失败');
    }
  });
}
</script>

<template>
  <div class="keyword-management">
    <var-app-bar title="关键词管理">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
      <template #right>
        <var-button type="primary" @click="startAddKeyword">
          <Icon name="add" :size="18" /> 新增关键词
        </var-button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div v-if="selectedKeywords.length > 0" class="batch-actions">
        <var-button type="danger" @click="deleteSelectedKeywords">
          <Icon name="delete-bin" :size="18" /> 删除 ({{ selectedKeywords.length }})
        </var-button>
      </div>
      
      <div class="form-hint">
        <Icon name="information" :size="14" />
        <span>中文关键词将自动在后端生成拼音和缩写</span>
      </div>
      
      <var-list class="keyword-list">
        <var-cell
          v-for="keyword in keywords"
          :key="keyword.id"
          class="keyword-item"
        >
          <template #icon>
            <var-checkbox
              :model-value="selectedKeywords.includes(keyword.id)"
              @update:model-value="toggleKeywordSelection(keyword.id)"
            />
          </template>
          
          <div class="keyword-info">
            <div v-if="editingKeyword?.id === keyword.id">
              <var-input
                v-model="editKeywordName"
                placeholder="关键词"
                size="small"
                @keydown.enter="submitEditKeyword"
              />
            </div>
            <div v-else>
              <span class="keyword-name">{{ keyword.keyword }}</span>
            </div>
          </div>
          
          <template #extra>
            <div v-if="editingKeyword?.id === keyword.id" class="action-buttons">
              <var-button text round size="small" @click="submitEditKeyword">
                <Icon name="check" :size="18" />
              </var-button>
              <var-button text round size="small" @click="cancelEditKeyword">
                <Icon name="close" :size="18" />
              </var-button>
            </div>
            <div v-else class="action-buttons">
              <var-button text round @click="startEditKeyword(keyword)">
                <Icon name="edit-2" :size="18" />
              </var-button>
              <var-button text round type="danger" @click="deleteSingleKeyword(keyword.id)">
                <Icon name="delete-bin" :size="18" />
              </var-button>
            </div>
          </template>
        </var-cell>
      </var-list>
      
      <var-popup :show="showAddPopup" @click-overlay="closeAddPopup">
        <div class="add-popup">
          <div class="add-popup-header">
            <h3>新增关键词</h3>
            <button class="btn-icon" @click="closeAddPopup">
              <Icon name="close" :size="24" />
            </button>
          </div>
          <div class="add-popup-body">
            <var-input
              v-model="newKeywordName"
              placeholder="关键词"
              class="add-input"
              @keydown.enter="submitAddKeyword"
            />
            <div class="add-popup-actions">
              <var-button type="primary" block @click="submitAddKeyword">创建</var-button>
              <var-button type="default" block @click="closeAddPopup">取消</var-button>
            </div>
          </div>
        </div>
      </var-popup>
    </div>
  </div>
</template>

<style scoped>
.keyword-management {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding-top: env(safe-area-inset-top);
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.batch-actions {
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--color-bg-2);
  border-radius: 8px;
}

.form-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background-color: var(--color-primary-light);
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 13px;
  color: var(--color-primary);
}

.keyword-list {
  margin-bottom: 16px;
}

.keyword-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
}

.keyword-info {
  flex: 1;
}

.keyword-name {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text);
}

.action-buttons {
  display: flex;
  gap: 4px;
}

.add-popup {
  width: 320px;
  max-width: 90vw;
  background-color: var(--color-bg);
  border-radius: 16px;
  overflow: hidden;
}

.add-popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-bg-2);
}

.add-popup-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.add-popup-body {
  padding: 20px;
}

.add-input {
  margin-bottom: 16px;
}

.add-popup-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.add-popup-actions .var-button {
  flex: 1;
}
</style>