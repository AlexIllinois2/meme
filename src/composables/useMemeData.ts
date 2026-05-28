import { ref, type Ref, nextTick } from "vue";
import * as tauri from "@tauri-apps/api/core";
import { Snackbar } from '@varlet/ui';
import type { Mode, Group, Image, Config } from "../types";
import { debug } from '../utils/debug';

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI__;
const invoke = isTauri ? tauri.invoke : async () => {
  console.warn("Tauri is not available, running in browser mode");
  return null;
};

export function useMemeData(
  config: Ref<Config | null>,
  safeUpdateConfig: (cfg: Config | null) => Promise<void>,
  searchKeyword?: Ref<string>,
  searchImages?: () => Promise<void>
) {
  const modes = ref<Mode[]>([]);
  const groups = ref<Group[]>([]);
  const images = ref<Image[]>([]);
  const selectedModeId = ref<number | null>(null);
  const selectedGroupId = ref<number | null>(null);

  async function loadModes() {
    try {
      const result = await invoke<Mode[]>("get_modes");
      if (result) {
        modes.value = result;

        if (selectedModeId.value && !modes.value.find(m => m.id === selectedModeId.value)) {
          selectedModeId.value = null;
        }

        if (modes.value.length > 0 && !selectedModeId.value) {
          selectedModeId.value = modes.value[0].id;
        }
      } else {
        modes.value = [{
          id: 1,
          name: "默认模式",
          sort_order: 1,
          folder_path: "/tmp/memes/default"
        }];
        selectedModeId.value = 1;
      }
    } catch (error) {
      console.error("Failed to load modes:", error);
      modes.value = [{
        id: 1,
        name: "默认模式",
        sort_order: 1,
        folder_path: "/tmp/memes/default"
      }];
      selectedModeId.value = 1;
    }
  }

  async function loadGroups(modeId: number) {
    try {
      const result = await invoke<Group[]>("get_groups_by_mode", { modeId });
      if (result) {
        groups.value = result;

        if (selectedGroupId.value && !groups.value.find(g => g.id === selectedGroupId.value)) {
          selectedGroupId.value = null;
        }

        if (groups.value.length > 0 && !selectedGroupId.value) {
          selectedGroupId.value = groups.value[0].id;
          await loadImages(selectedGroupId.value);
        } else if (groups.value.length === 0) {
          images.value = [];
          selectedGroupId.value = null;
        }
      } else {
        groups.value = [{
          id: 1,
          name: "默认分组",
          folder_path: "/tmp/memes/default/group1",
          share_count: 0,
          mode_id: modeId
        }];
        selectedGroupId.value = 1;
        await loadImages(1);
      }
    } catch (error) {
      console.error("Failed to load groups:", error);
      groups.value = [{
        id: 1,
        name: "默认分组",
        folder_path: "/tmp/memes/default/group1",
        share_count: 0,
        mode_id: modeId
      }];
      selectedGroupId.value = 1;
      await loadImages(1);
    }
  }

  async function loadImages(groupId: number) {
    try {
      const result = await invoke<Image[]>("get_images_by_group", { groupId });
      if (result) {
        images.value = result;
      } else {
        images.value = [];
      }
    } catch (error) {
      console.error("Failed to load images:", error);
      images.value = [];
    }
  }

  async function switchMode(active: string | number) {
    const modeId = Number(active);
    selectedModeId.value = modeId;
    await loadGroups(modeId);
    if (config.value) {
      config.value.last_mode = modeId;
      await safeUpdateConfig(config.value);
    }

    await nextTick();

    const activeTab = document.querySelector('.modern-tabs-container:not(.secondary) .modern-tab.active');
    const scrollContainer = document.querySelector('.modern-tabs-container:not(.secondary) .modern-tabs-scroll');

    if (activeTab && scrollContainer) {
      activeTab.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
        inline: 'center'
      });
    }

    // 切换模式后，如果搜索栏有内容，自动进行搜索
    if (searchKeyword && searchKeyword.value.trim() && searchImages) {
      await searchImages();
    }
  }

  async function switchGroup(active: string | number) {
    const groupId = Number(active);
    selectedGroupId.value = groupId;
    await loadImages(groupId);
    if (config.value) {
      config.value.last_group = groupId;
      await safeUpdateConfig(config.value);
    }

    await nextTick();

    const activeTab = document.querySelector('.modern-tabs-container.secondary .modern-tab.active');
    const scrollContainer = document.querySelector('.modern-tabs-container.secondary .modern-tabs-scroll');

    if (activeTab && scrollContainer) {
      activeTab.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
        inline: 'center'
      });
    }
  }

  async function reloadPageState() {
    await loadModes();
    if (selectedModeId.value) {
      await loadGroups(selectedModeId.value);
    }
    if (selectedGroupId.value) {
      await loadImages(selectedGroupId.value);
    }
  }

  async function fullRefresh() {
    try {
      const memeDir = config.value?.meme_dir;
      if (!memeDir) {
        Snackbar.warning('请先在设置中配置表情包目录');
        return;
      }

      const accessible = await invoke<boolean>('check_storage_accessible', { memeDir });
      if (!accessible) {
        if (typeof (window as any).AndroidNative?.requestStoragePermission === 'function') {
          (window as any).AndroidNative.requestStoragePermission();
        }
        Snackbar.warning('请授予存储权限后重试');
        return;
      }

      Snackbar.info('正在刷新数据...');
      const result = await invoke<string>("full_refresh", { memeDir });
      debug.log('Full refresh result:', result);
      await reloadPageState();
      Snackbar.success('数据刷新完成');
    } catch (error) {
      console.error("Failed to refresh:", error);
      try { await reloadPageState(); } catch (e) { /* ignore */ }
      Snackbar.error('数据刷新失败: ' + error);
    }
  }

  async function autoRefreshAfterDirChange(newDir: string) {
    try {
      debug.log('[Auto Refresh] Checking directory:', newDir);

      const accessible = await invoke<boolean>('check_storage_accessible', { memeDir: newDir });
      if (!accessible) {
        debug.log('[Auto Refresh] Storage not accessible, skipping');
        return;
      }

      Snackbar.info('正在初始化数据...');
      const result = await invoke<string>("full_refresh", { memeDir: newDir });
      debug.log('[Auto Refresh] Result:', result);

      await reloadPageState();
      Snackbar.success('数据初始化完成');
    } catch (error) {
      console.error('[Auto Refresh] Failed:', error);
      try {
        await reloadPageState();
      } catch (e) {
        console.error('[Auto Refresh] Reload also failed:', e);
      }
    }
  }

  return {
    modes,
    groups,
    images,
    selectedModeId,
    selectedGroupId,
    loadModes,
    loadGroups,
    loadImages,
    switchMode,
    switchGroup,
    reloadPageState,
    fullRefresh,
    autoRefreshAfterDirChange,
  };
}
