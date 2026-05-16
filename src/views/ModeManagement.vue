<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Snackbar, Dialog } from '@varlet/ui';
import Icon from "../components/Icon.vue";
import type { Mode } from "../types";

const modes = ref<Mode[]>([]);
const selectedModes = ref<number[]>([]);
const showAddPopup = ref(false);
const newModeName = ref('');
const newModeSortOrder = ref(1);

// 编辑相关
const editingMode = ref<Mode | null>(null);
const editModeName = ref('');
const editModeSortOrder = ref(1);

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
      modes.value = result.sort((a, b) => a.sort_order - b.sort_order);
    }
  } catch (error) {
    console.error('Failed to load modes:', error);
    Snackbar.error('加载模式失败');
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
  if (editingMode.value) {
    cancelEditMode();
    event.preventDefault?.();
    return true;
  }
  // 取消选择
  if (selectedModes.value.length > 0) {
    selectedModes.value = [];
    event.preventDefault?.();
    return true;
  }
  // 返回首页
  goBack();
  event.preventDefault?.();
  return true;
}

function startAddMode() {
  newModeName.value = '';
  newModeSortOrder.value = Math.max(...modes.value.map(m => m.sort_order), 0) + 1;
  showAddPopup.value = true;
}

function closeAddPopup() {
  showAddPopup.value = false;
  newModeName.value = '';
}

async function submitAddMode() {
  const trimmedName = newModeName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('模式名称不能为空');
    return;
  }
  
  // 校验排序号：必须为正整数
  const sortOrder = Math.floor(newModeSortOrder.value);
  if (!Number.isFinite(sortOrder) || sortOrder < 1) {
    Snackbar.warning('排序序号必须为正整数');
    return;
  }
  newModeSortOrder.value = sortOrder;
  
  // 检查是否已存在同名模式
  const existingMode = modes.value.find(m => m.name === trimmedName);
  if (existingMode) {
    Snackbar.error('已存在同名模式');
    return;
  }
  
  // 检查是否与其他模式排序号冲突（仅警告，不阻止）
  const conflictingMode = modes.value.find(m => m.sort_order === sortOrder && m.name !== 'phantom');
  if (conflictingMode) {
    Snackbar.warning(`排序序号 ${sortOrder} 已存在（${conflictingMode.name}），多个模式可使用相同序号`);
  }
  
  try {
    await invoke('add_mode', {
      name: trimmedName,
      sortOrder: newModeSortOrder.value
    });
    Snackbar.success('模式创建成功');
    closeAddPopup();
    await loadModes();
  } catch (error) {
    console.error('Failed to add mode:', error);
    Snackbar.error('创建模式失败');
  }
}

function startEditMode(mode: Mode) {
  editingMode.value = mode;
  editModeName.value = mode.name;
  editModeSortOrder.value = mode.sort_order;
}

function cancelEditMode() {
  editingMode.value = null;
  editModeName.value = '';
  editModeSortOrder.value = 1;
}

async function submitEditMode() {
  if (!editingMode.value) return;
  
  const trimmedName = editModeName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('模式名称不能为空');
    return;
  }
  
  // 校验排序号：必须为正整数
  const sortOrder = Math.floor(editModeSortOrder.value);
  if (!Number.isFinite(sortOrder) || sortOrder < 1) {
    Snackbar.warning('排序序号必须为正整数');
    return;
  }
  editModeSortOrder.value = sortOrder;
  
  // 检查是否与其他模式重名
  const existingMode = modes.value.find(m => 
    m.name === trimmedName && m.id !== editingMode.value!.id
  );
  if (existingMode) {
    Snackbar.error('已存在同名模式');
    return;
  }
  
  try {
    await invoke('update_mode', {
      id: editingMode.value.id,
      name: trimmedName,
      sortOrder: editModeSortOrder.value
    });
    Snackbar.success('模式更新成功');
    cancelEditMode();
    await loadModes();
  } catch (error) {
    console.error('Failed to update mode:', error);
    Snackbar.error('更新模式失败');
  }
}

async function deleteSingleMode(modeId: number) {
  Dialog({
    title: '确认删除',
    message: '确定要删除此模式吗？相关文件将移动到回收站。',
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      await invoke('delete_mode', { modeId });
      Snackbar.success('模式已删除');
      await loadModes();
      // 从选中列表中移除
      const index = selectedModes.value.indexOf(modeId);
      if (index > -1) {
        selectedModes.value.splice(index, 1);
      }
    } catch (error) {
      console.error('Failed to delete mode:', error);
      Snackbar.error('删除模式失败');
    }
  });
}

function toggleModeSelection(modeId: number) {
  const index = selectedModes.value.indexOf(modeId);
  if (index > -1) {
    selectedModes.value.splice(index, 1);
  } else {
    selectedModes.value.push(modeId);
  }
}

async function deleteSelectedModes() {
  if (selectedModes.value.length === 0) return;
  
  Dialog({
    title: '确认批量删除',
    message: `确定要删除选中的 ${selectedModes.value.length} 个模式吗？相关文件将移动到回收站。`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      for (const modeId of selectedModes.value) {
        await invoke('delete_mode', { modeId });
      }
      Snackbar.success('批量删除成功');
      selectedModes.value = [];
      await loadModes();
    } catch (error) {
      console.error('Failed to delete modes:', error);
      Snackbar.error('批量删除失败');
    }
  });
}

async function moveModeUp(modeId: number) {
  const index = modes.value.findIndex(m => m.id === modeId);
  if (index <= 0) return;
  
  const currentMode = modes.value[index];
  const prevMode = modes.value[index - 1];
  
  // 交换排序序号
  const tempOrder = currentMode.sort_order;
  currentMode.sort_order = prevMode.sort_order;
  prevMode.sort_order = tempOrder;
  
  try {
    await invoke('update_mode', {
      id: currentMode.id,
      name: currentMode.name,
      sortOrder: currentMode.sort_order
    });
    await invoke('update_mode', {
      id: prevMode.id,
      name: prevMode.name,
      sortOrder: prevMode.sort_order
    });
    await loadModes();
  } catch (error) {
    console.error('Failed to reorder modes:', error);
    Snackbar.error('调整顺序失败');
  }
}

async function moveModeDown(modeId: number) {
  const index = modes.value.findIndex(m => m.id === modeId);
  if (index >= modes.value.length - 1) return;
  
  const currentMode = modes.value[index];
  const nextMode = modes.value[index + 1];
  
  // 交换排序序号
  const tempOrder = currentMode.sort_order;
  currentMode.sort_order = nextMode.sort_order;
  nextMode.sort_order = tempOrder;
  
  try {
    await invoke('update_mode', {
      id: currentMode.id,
      name: currentMode.name,
      sortOrder: currentMode.sort_order
    });
    await invoke('update_mode', {
      id: nextMode.id,
      name: nextMode.name,
      sortOrder: nextMode.sort_order
    });
    await loadModes();
  } catch (error) {
    console.error('Failed to reorder modes:', error);
    Snackbar.error('调整顺序失败');
  }
}
</script>

<template>
  <div class="mode-management">
    <var-app-bar title="模式管理">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
      <template #right>
        <var-button type="primary" @click="startAddMode">
          <Icon name="add" :size="18" /> 新增模式
        </var-button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div v-if="selectedModes.length > 0" class="batch-actions">
        <var-button type="danger" @click="deleteSelectedModes">
          <Icon name="delete-bin" :size="18" /> 删除选中 ({{ selectedModes.length }})
        </var-button>
      </div>
      
      <var-list class="mode-list">
        <var-cell
          v-for="mode in modes"
          :key="mode.id"
          class="mode-item"
        >
          <template #icon>
            <var-checkbox
              :model-value="selectedModes.includes(mode.id)"
              @update:model-value="toggleModeSelection(mode.id)"
            />
          </template>
          
          <div class="mode-info">
            <div v-if="editingMode?.id === mode.id" class="edit-form">
              <var-input
                v-model="editModeName"
                placeholder="模式名称"
                size="small"
              />
              <var-input
                :model-value="String(editModeSortOrder)"
                type="number"
                placeholder="排序"
                size="small"
                class="sort-input"
                @update:model-value="(val: string) => editModeSortOrder = Number(val)"
              />
            </div>
            <div v-else>
              <span class="mode-name">{{ mode.name }}</span>
              <span class="mode-sort">序号: {{ mode.sort_order }}</span>
            </div>
          </div>
          
          <template #extra>
            <div v-if="editingMode?.id === mode.id" class="action-buttons">
              <var-button text round size="small" @click="submitEditMode">
                <Icon name="check" :size="18" />
              </var-button>
              <var-button text round size="small" @click="cancelEditMode">
                <Icon name="close" :size="18" />
              </var-button>
            </div>
            <div v-else class="action-buttons">
              <var-button text round @click="moveModeUp(mode.id)" :disabled="modes.indexOf(mode) === 0">
                <Icon name="arrow-up" :size="18" />
              </var-button>
              <var-button text round @click="moveModeDown(mode.id)" :disabled="modes.indexOf(mode) === modes.length - 1">
                <Icon name="arrow-down" :size="18" />
              </var-button>
              <var-button text round @click="startEditMode(mode)">
                <Icon name="edit-2" :size="18" />
              </var-button>
              <var-button text round type="danger" @click="deleteSingleMode(mode.id)">
                <Icon name="delete-bin" :size="18" />
              </var-button>
            </div>
          </template>
        </var-cell>
      </var-list>
      
      <var-popup :show="showAddPopup" @click-overlay="closeAddPopup">
        <div class="add-popup">
          <div class="add-popup-header">
            <h3>新增模式</h3>
            <button class="btn-icon" @click="closeAddPopup">
              <Icon name="close" :size="24" color="var(--color-text-secondary)" />
            </button>
          </div>
          <div class="add-popup-body">
            <var-input
              v-model="newModeName"
              placeholder="模式名称"
              class="add-input"
            />
            <var-input
              :model-value="String(newModeSortOrder)"
              type="number"
              placeholder="排序序号"
              class="add-input"
              @update:model-value="(val: string) => newModeSortOrder = Number(val)"
            />
            <div class="add-popup-actions">
              <var-button type="primary" block @click="submitAddMode">创建</var-button>
              <var-button type="default" block @click="closeAddPopup">取消</var-button>
            </div>
          </div>
        </div>
      </var-popup>
    </div>
  </div>
</template>

<style scoped>
.mode-management {
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

.mode-list {
  margin-bottom: 16px;
}

.mode-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
}

.mode-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mode-name {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text);
}

.mode-sort {
  font-size: 12px;
  color: var(--color-text-3);
}

.edit-form {
  display: flex;
  gap: 8px;
  align-items: center;
}

.sort-input {
  width: 80px;
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