<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Snackbar, Dialog } from '@varlet/ui';
import Icon from "../components/Icon.vue";
import type { Group, Mode } from "../types";

const groups = ref<Group[]>([]);
const modes = ref<Mode[]>([]);
const selectedGroups = ref<number[]>([]);
const selectedModeId = ref<number | null>(null);
const showAddPopup = ref(false);
const newGroupName = ref('');

// 编辑相关
const editingGroup = ref<Group | null>(null);
const editGroupName = ref('');

onMounted(() => {
  loadModes();
  
  // Android 返回键监听
  if (isAndroidTauri()) {
    window.addEventListener('tauri-android-back', handleAndroidBack);
  }
});

onUnmounted(() => {
  // 移除 Android 返回键监听
  if (isAndroidTauri()) {
    window.removeEventListener('tauri-android-back', handleAndroidBack);
  }
});

async function loadModes() {
  try {
    const result = await invoke<Mode[]>('get_modes');
    if (result) {
      modes.value = result;
      if (modes.value.length > 0 && !selectedModeId.value) {
        selectedModeId.value = modes.value[0].id;
        await loadGroups(selectedModeId.value);
      }
    }
  } catch (error) {
    console.error('Failed to load modes:', error);
  }
}

async function loadGroups(modeId: number) {
  try {
    const result = await invoke<Group[]>('get_groups_by_mode', { modeId });
    if (result) {
      groups.value = result;
    }
  } catch (error) {
    console.error('Failed to load groups:', error);
    Snackbar.error('加载分组失败');
  }
}

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}

// 检测是否为 Android 环境
function isAndroidTauri() {
  return typeof window !== 'undefined' && /Android/i.test(navigator.userAgent);
}

// Android 返回键处理
function handleAndroidBack(event: any) {
  // 关闭弹窗
  if (showAddPopup.value) {
    closeAddPopup();
    event.preventDefault?.();
    return true;
  }
  // 取消编辑
  if (editingGroup.value) {
    cancelEditGroup();
    event.preventDefault?.();
    return true;
  }
  // 取消选择
  if (selectedGroups.value.length > 0) {
    selectedGroups.value = [];
    event.preventDefault?.();
    return true;
  }
  // 返回首页
  goBack();
  event.preventDefault?.();
  return true;
}

async function handleModeChange(modeId: number) {
  selectedModeId.value = modeId;
  selectedGroups.value = [];
  await loadGroups(modeId);
}

function startAddGroup() {
  newGroupName.value = '';
  showAddPopup.value = true;
}

function closeAddPopup() {
  showAddPopup.value = false;
  newGroupName.value = '';
}

async function submitAddGroup() {
  const trimmedName = newGroupName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('分组名称不能为空');
    return;
  }
  
  if (!selectedModeId.value) {
    Snackbar.warning('请先选择一个模式');
    return;
  }
  
  // 检查该模式下是否已存在同名分组
  const existingGroup = groups.value.find(g => g.name === trimmedName);
  if (existingGroup) {
    Snackbar.error('该模式下已存在同名分组');
    return;
  }
  
  try {
    await invoke('add_group', {
      name: trimmedName,
      modeId: selectedModeId.value
    });
    Snackbar.success('分组创建成功');
    closeAddPopup();
    await loadGroups(selectedModeId.value);
  } catch (error) {
    console.error('Failed to add group:', error);
    Snackbar.error('创建分组失败');
  }
}

function startEditGroup(group: Group) {
  editingGroup.value = group;
  editGroupName.value = group.name;
}

function cancelEditGroup() {
  editingGroup.value = null;
  editGroupName.value = '';
}

async function submitEditGroup() {
  if (!editingGroup.value || !selectedModeId.value) return;
  
  const trimmedName = editGroupName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('分组名称不能为空');
    return;
  }
  
  // 检查是否与其他分组重名
  const existingGroup = groups.value.find(g => 
    g.name === trimmedName && g.id !== editingGroup.value!.id
  );
  if (existingGroup) {
    Snackbar.error('该模式下已存在同名分组');
    return;
  }
  
  try {
    await invoke('update_group', {
      id: editingGroup.value.id,
      name: trimmedName,
      modeId: selectedModeId.value
    });
    Snackbar.success('分组更新成功');
    cancelEditGroup();
    await loadGroups(selectedModeId.value);
  } catch (error) {
    console.error('Failed to update group:', error);
    Snackbar.error('更新分组失败');
  }
}

async function deleteSingleGroup(groupId: number) {
  Dialog({
    title: '确认删除',
    message: '确定要删除此分组吗？相关文件将移动到回收站。',
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      await invoke('delete_group', { groupId });
      Snackbar.success('分组已删除');
      if (selectedModeId.value) {
        await loadGroups(selectedModeId.value);
      }
      const index = selectedGroups.value.indexOf(groupId);
      if (index > -1) {
        selectedGroups.value.splice(index, 1);
      }
    } catch (error) {
      console.error('Failed to delete group:', error);
      Snackbar.error('删除分组失败');
    }
  });
}

function toggleGroupSelection(groupId: number) {
  const index = selectedGroups.value.indexOf(groupId);
  if (index > -1) {
    selectedGroups.value.splice(index, 1);
  } else {
    selectedGroups.value.push(groupId);
  }
}

async function deleteSelectedGroups() {
  if (selectedGroups.value.length === 0) return;
  
  Dialog({
    title: '确认批量删除',
    message: `确定要删除选中的 ${selectedGroups.value.length} 个分组吗？相关文件将移动到回收站。`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      for (const groupId of selectedGroups.value) {
        await invoke('delete_group', { groupId });
      }
      Snackbar.success('批量删除成功');
      selectedGroups.value = [];
      if (selectedModeId.value) {
        await loadGroups(selectedModeId.value);
      }
    } catch (error) {
      console.error('Failed to delete groups:', error);
      Snackbar.error('批量删除失败');
    }
  });
}
</script>

<template>
  <div class="group-management">
    <var-app-bar title="分组管理">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
      <template #right>
        <var-button type="primary" @click="startAddGroup" :disabled="!selectedModeId">
          <Icon name="add" :size="18" /> 新增分组
        </var-button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div class="mode-selector">
        <label>选择模式：</label>
        <var-select v-model="selectedModeId" @change="handleModeChange">
          <var-option 
            v-for="mode in modes" 
            :key="mode.id" 
            :value="mode.id" 
            :label="mode.name" 
          />
        </var-select>
      </div>
      
      <div v-if="selectedGroups.length > 0" class="batch-actions">
        <var-button type="danger" @click="deleteSelectedGroups">
          <Icon name="delete-bin" :size="18" /> 删除选中 ({{ selectedGroups.length }})
        </var-button>
      </div>
      
      <var-list class="group-list">
        <var-cell
          v-for="group in groups"
          :key="group.id"
          class="group-item"
        >
          <template #icon>
            <var-checkbox
              :model-value="selectedGroups.includes(group.id)"
              @update:model-value="toggleGroupSelection(group.id)"
            />
          </template>
          
          <div class="group-info">
            <div v-if="editingGroup?.id === group.id">
              <var-input
                v-model="editGroupName"
                placeholder="分组名称"
                size="small"
                @keydown.enter="submitEditGroup"
              />
            </div>
            <div v-else>
              <span class="group-name">{{ group.name }}</span>
            </div>
          </div>
          
          <template #extra>
            <div v-if="editingGroup?.id === group.id" class="action-buttons">
              <var-button text round size="small" @click="submitEditGroup">
                <Icon name="check" :size="18" />
              </var-button>
              <var-button text round size="small" @click="cancelEditGroup">
                <Icon name="close" :size="18" />
              </var-button>
            </div>
            <div v-else class="action-buttons">
              <var-button text round @click="startEditGroup(group)">
                <Icon name="edit-2" :size="18" />
              </var-button>
              <var-button text round type="danger" @click="deleteSingleGroup(group.id)">
                <Icon name="delete-bin" :size="18" />
              </var-button>
            </div>
          </template>
        </var-cell>
      </var-list>
      
      <var-popup :show="showAddPopup" @click-overlay="closeAddPopup">
        <div class="add-popup">
          <div class="add-popup-header">
            <h3>新增分组</h3>
            <button class="btn-icon" @click="closeAddPopup">
              <Icon name="close" :size="24" color="var(--color-text-secondary)" />
            </button>
          </div>
          <div class="add-popup-body">
            <div class="current-mode" v-if="selectedModeId">
              <Icon name="folders" :size="16" />
              <span>当前模式: {{ modes.find(m => m.id === selectedModeId)?.name }}</span>
            </div>
            <var-input
              v-model="newGroupName"
              placeholder="分组名称"
              class="add-input"
              @keydown.enter="submitAddGroup"
            />
            <div class="add-popup-actions">
              <var-button type="primary" block @click="submitAddGroup">创建</var-button>
              <var-button type="default" block @click="closeAddPopup">取消</var-button>
            </div>
          </div>
        </div>
      </var-popup>
    </div>
  </div>
</template>

<style scoped>
.group-management {
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

.mode-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--color-bg-2);
  border-radius: 8px;
}

.mode-selector label {
  font-size: 14px;
  color: var(--color-text-2);
  white-space: nowrap;
}

.batch-actions {
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--color-bg-3);
  border-radius: 8px;
}

.group-list {
  margin-bottom: 16px;
}

.group-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
}

.group-info {
  flex: 1;
}

.group-name {
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

.current-mode {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background-color: var(--color-bg-2);
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 14px;
  color: var(--color-text-2);
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