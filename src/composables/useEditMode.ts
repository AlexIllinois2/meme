import { ref, computed, Ref } from 'vue';
import { Snackbar, Dialog } from '@varlet/ui';
import type { Mode, Group, Image } from '../types';
import { debug } from '../utils/debug';
import { isAndroidTauri } from '../utils/tauri';

export function useEditMode(
  invoke: (...args: any[]) => any,
  deps: {
    modes: Ref<Mode[]>;
    groups: Ref<Group[]>;
    images: Ref<Image[]>;
    selectedModeId: Ref<number | null>;
    selectedGroupId: Ref<number | null>;
    loadModes: () => Promise<void>;
    loadGroups: (modeId: number) => Promise<void>;
    loadImages: (groupId: number) => Promise<void>;
  },
) {
  const selectedImages = ref<number[]>([]);
  const isGlobalEditMode = ref(false);
  const selectedModeIds = ref<number[]>([]);
  const selectedGroupIds = ref<number[]>([]);
  const showFirstUseMask = ref(false);
  const hasUserInteracted = ref(false);

  const hasSelectedItems = computed(() =>
    selectedModeIds.value.length > 0 ||
    selectedGroupIds.value.length > 0 ||
    selectedImages.value.length > 0,
  );

  function toggleGlobalEditMode() {
    isGlobalEditMode.value = !isGlobalEditMode.value;
    selectedModeIds.value = [];
    selectedGroupIds.value = [];
    selectedImages.value = [];
  }

  function exitGlobalEditMode() {
    isGlobalEditMode.value = false;
    selectedModeIds.value = [];
    selectedGroupIds.value = [];
    selectedImages.value = [];
  }

  function handleUserInteraction(_e: Event) {
    if (!hasUserInteracted.value) {
      hasUserInteracted.value = true;
      showFirstUseMask.value = false;
    }
  }

  function showFirstUsePrompt() {
    if (!hasUserInteracted.value) {
      showFirstUseMask.value = true;
    }
  }

  function handleAppResume() {
    if (document.visibilityState === 'visible') {
      debug.log('[App] App resumed to foreground');
      if (!isAndroidTauri()) {
        window.triggerSearchFocus?.();
      }
    }
  }

  function toggleModeSelection(modeId: number) {
    const index = selectedModeIds.value.indexOf(modeId);
    if (index > -1) {
      selectedModeIds.value.splice(index, 1);
    } else {
      selectedModeIds.value.push(modeId);
    }
  }

  function toggleGroupSelection(groupId: number) {
    const index = selectedGroupIds.value.indexOf(groupId);
    if (index > -1) {
      selectedGroupIds.value.splice(index, 1);
    } else {
      selectedGroupIds.value.push(groupId);
    }
  }

  function toggleImageSelection(imageId: number) {
    const index = selectedImages.value.indexOf(imageId);
    if (index > -1) {
      selectedImages.value.splice(index, 1);
    } else {
      selectedImages.value.push(imageId);
    }
  }

  async function handleBatchDelete() {
    if (!hasSelectedItems.value) return;

    const modeCount = selectedModeIds.value.length;
    const groupCount = selectedGroupIds.value.length;
    const imageCount = selectedImages.value.length;

    let message = '确定要删除选中的内容吗？\n';
    if (modeCount > 0) message += `• ${modeCount} 个模式\n`;
    if (groupCount > 0) message += `• ${groupCount} 个分组\n`;
    if (imageCount > 0) message += `• ${imageCount} 张图片\n`;
    message += '\n注意：删除分组会同时删除其下所有图片！';

    const result = await Dialog({
      title: '确认批量删除',
      message,
      confirmButton: true,
      cancelButton: true,
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    });

    if (result !== 'confirm') return;

    try {
      // 过滤掉已通过分组删除的图片
      const remainingImageIds = selectedImages.value.filter(imgId => {
        const img = deps.images.value.find(i => i.id === imgId);
        return img && !selectedGroupIds.value.includes(img.group_id);
      });

      // 单次 Rust 调用完成所有删除（原子操作）
      await invoke('batch_delete', {
        req: {
          mode_ids: selectedModeIds.value,
          group_ids: selectedGroupIds.value,
          image_ids: remainingImageIds,
        },
      });

      Snackbar.success('批量删除成功');
      exitGlobalEditMode();

      await deps.loadModes();
      if (deps.selectedModeId.value) {
        await deps.loadGroups(deps.selectedModeId.value);
      }
      if (deps.selectedGroupId.value) {
        await deps.loadImages(deps.selectedGroupId.value);
      }
    } catch (error) {
      console.error('Failed to batch delete:', error);
      Snackbar.error('批量删除失败');
    }
  }

  return {
    selectedImages,
    isGlobalEditMode,
    selectedModeIds,
    selectedGroupIds,
    showFirstUseMask,
    hasUserInteracted,
    hasSelectedItems,
    toggleGlobalEditMode,
    exitGlobalEditMode,
    handleUserInteraction,
    showFirstUsePrompt,
    handleAppResume,
    toggleModeSelection,
    toggleGroupSelection,
    toggleImageSelection,
    handleBatchDelete,
  };
}
