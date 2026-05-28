import { ref, Ref } from 'vue';
import type { Group, Image } from '../types';

export function useSearch(
  invoke: (...args: any[]) => any,
  deps: {
    selectedModeId: Ref<number | null>;
    pinyinSearchEnabled: Ref<boolean>;
    acronymSearchEnabled: Ref<boolean>;
    groups: Ref<Group[]>;
    images: Ref<Image[]>;
    selectedGroupId: Ref<number | null>;
    loadImages: (groupId: number) => Promise<void>;
  },
) {
  const searchKeyword = ref('');
  const searchInputRef = ref<any>(null);

  async function searchImages() {
    if (!searchKeyword.value.trim()) {
      // 搜索清空时重新从后端加载全部分组（groups 已被搜索覆盖）
      try {
        const allGroups = await invoke('get_groups_by_mode', {
          modeId: deps.selectedModeId.value,
        }) as Group[];
        if (allGroups?.length) {
          deps.groups.value = allGroups;
          deps.selectedGroupId.value = allGroups[0].id;
          await deps.loadImages(allGroups[0].id);
        } else {
          deps.groups.value = [];
          deps.images.value = [];
        }
      } catch (error) {
        console.error('Failed to load groups:', error);
      }
      return;
    }
    try {
      const matchedGroups = await invoke('search_groups', {
        keyword: searchKeyword.value,
        modeId: deps.selectedModeId.value,
        pinyinSearch: deps.pinyinSearchEnabled.value,
        acronymSearch: deps.acronymSearchEnabled.value,
      }) as Group[];
      if (matchedGroups?.length) {
        deps.groups.value = matchedGroups;
        deps.selectedGroupId.value = matchedGroups[0].id;
        await deps.loadImages(matchedGroups[0].id);
      } else {
        deps.groups.value = [];
        deps.images.value = [];
      }
    } catch (error) {
      console.error('Failed to search:', error);
      try {
        const result = await invoke('search_images', {
          keyword: searchKeyword.value,
          pinyin: deps.pinyinSearchEnabled.value,
          acronym: deps.acronymSearchEnabled.value,
        }) as Image[];
        deps.images.value = result || [];
      } catch (err) {
        console.error('Failed to search images:', err);
      }
    }
  }

  function handleFloatingSearchClick() {
    searchKeyword.value = '';
    const searchInput = document.querySelector('.search-input input') as HTMLInputElement;
    searchInput?.focus();
  }

  return {
    searchKeyword,
    searchInputRef,
    searchImages,
    handleFloatingSearchClick,
  };
}
