<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import * as tauri from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Snackbar, Dialog } from '@varlet/ui';
import SideMenu from "./components/SideMenu.vue";
import ModeManagement from "./views/ModeManagement.vue";
import GroupManagement from "./views/GroupManagement.vue";
import KeywordManagement from "./views/KeywordManagement.vue";
import Settings from "./views/Settings.vue";
import ContextMenu from "./components/ContextMenu.vue";
import KeywordManager from "./components/KeywordManager.vue";
import ThemeProvider from "./components/ThemeProvider.vue";
import FloatingSearchButton from "./components/FloatingSearchButton.vue";
import Icon from "./components/Icon.vue";
import type { Mode, Group, Image, Config } from "./types";

const isTauri = typeof window !== 'undefined' && window.__TAURI__;
const invoke = isTauri ? tauri.invoke : async () => {
  console.warn("Tauri is not available, running in browser mode");
  return null;
};

function toAssetPath(filePath: string | null | undefined): string {
  if (!filePath) return '';
  if (!isTauri) return filePath;
  try {
    return convertFileSrc(filePath);
  } catch (e) {
    console.error('Failed to convert file path:', filePath, e);
    return filePath;
  }
}

async function selectDirectory() {
  if (isTauri) {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择表情包存储目录"
      });
      if (selected) {
        const dirPath = Array.isArray(selected) ? selected[0] : selected;
        if (config.value) {
          config.value.meme_dir = dirPath;
          await invoke("update_config", { config: config.value });
        }
        return dirPath;
      }
    } catch (error) {
      console.error("选择目录失败:", error);
    }
  }
  return null;
};

const modes = ref<Mode[]>([]);
const groups = ref<Group[]>([]);
const images = ref<Image[]>([]);
const config = ref<Config | null>(null);
const selectedModeId = ref<number | null>(null);
const selectedGroupId = ref<number | null>(null);
const searchKeyword = ref("");
const isEditMode = ref(false);
const selectedImages = ref<number[]>([]);

// 全局编辑模式状态
const isGlobalEditMode = ref(false);
const selectedModeIds = ref<number[]>([]);
const selectedGroupIds = ref<number[]>([]);
// selectedImages 已存在，复用它

// 是否有选中项
const hasSelectedItems = computed(() => {
  return selectedModeIds.value.length > 0 || 
         selectedGroupIds.value.length > 0 || 
         selectedImages.value.length > 0;
});

// 切换全局编辑模式
function toggleGlobalEditMode() {
  isGlobalEditMode.value = !isGlobalEditMode.value;
  // 清空所有选中
  selectedModeIds.value = [];
  selectedGroupIds.value = [];
  selectedImages.value = [];
}

// 退出全局编辑模式
function exitGlobalEditMode() {
  isGlobalEditMode.value = false;
  selectedModeIds.value = [];
  selectedGroupIds.value = [];
  selectedImages.value = [];
}

// 切换模式选中
function toggleModeSelection(modeId: number) {
  const index = selectedModeIds.value.indexOf(modeId);
  if (index > -1) {
    selectedModeIds.value.splice(index, 1);
  } else {
    selectedModeIds.value.push(modeId);
  }
}

// 切换分组选中
function toggleGroupSelection(groupId: number) {
  const index = selectedGroupIds.value.indexOf(groupId);
  if (index > -1) {
    selectedGroupIds.value.splice(index, 1);
  } else {
    selectedGroupIds.value.push(groupId);
  }
}

// 批量删除
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
  
  Dialog({
    title: '确认批量删除',
    message,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      // 1. 删除选中的模式
      if (selectedModeIds.value.length > 0) {
        await invoke('delete_modes', { modeIds: selectedModeIds.value });
      }
      
      // 2. 删除选中的分组
      if (selectedGroupIds.value.length > 0) {
        await invoke('delete_groups', { groupIds: selectedGroupIds.value });
      }
      
      // 3. 删除选中的图片（只删除未被分组删除覆盖的）
      // 过滤掉属于已删除分组的图片
      const remainingImageIds = selectedImages.value.filter(imgId => {
        const img = images.value.find(i => i.id === imgId);
        // 如果图片所在的分组没有被删除，才删除这张图片
        return img && !selectedGroupIds.value.includes(img.group_id);
      });
      
      if (remainingImageIds.length > 0) {
        await invoke('delete_images', { imageIds: remainingImageIds });
      }
      
      Snackbar.success('批量删除成功');
      exitGlobalEditMode();
      
      // 刷新数据
      await loadModes();
      if (selectedModeId.value) {
        await loadGroups(selectedModeId.value);
      }
      if (selectedGroupId.value) {
        await loadImages(selectedGroupId.value);
      }
    } catch (error) {
      console.error('Failed to batch delete:', error);
      Snackbar.error('批量删除失败');
    }
  });
}

// 新增模式弹窗状态
const showAddModePopup = ref(false);
const newModeName = ref('');
const newModeSortOrder = ref(1);

// 显示新增模式弹窗
function showAddModeDialog() {
  newModeName.value = '';
  newModeSortOrder.value = modes.value.length + 1;
  showAddModePopup.value = true;
}

// 提交新增模式
async function submitAddMode() {
  const trimmedName = newModeName.value.trim();
  if (!trimmedName) {
    Snackbar.warning('模式名称不能为空');
    return;
  }
  
  // 检查是否已存在同名模式
  const existingMode = modes.value.find(m => m.name === trimmedName);
  if (existingMode) {
    Snackbar.error('已存在同名模式');
    return;
  }
  
  try {
    await invoke('add_mode', {
      name: trimmedName,
      sortOrder: newModeSortOrder.value
    });
    Snackbar.success('模式创建成功');
    showAddModePopup.value = false;
    await loadModes();
  } catch (error) {
    console.error('Failed to add mode:', error);
    Snackbar.error('创建模式失败');
  }
}

// 新增分组弹窗状态
const showAddGroupPopup = ref(false);
const newGroupName = ref('');
const showAddImagePopup = ref(false);

// 显示新增分组弹窗
function showAddGroupDialog() {
  newGroupName.value = '';
  showAddGroupPopup.value = true;
}

// 提交新增分组
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
  const existingGroup = groups.value.find(g => 
    g.mode_id === selectedModeId.value && g.name === trimmedName
  );
  if (existingGroup) {
    Snackbar.error('该模式下已存在同名分组');
    return;
  }
  
  try {
    const newGroupId = await invoke<number>('add_group', {
      name: trimmedName,
      modeId: selectedModeId.value
    });
    Snackbar.success('分组创建成功');
    showAddGroupPopup.value = false;
    await loadGroups(selectedModeId.value);
    
    // 自动选中新创建的分组
    if (newGroupId) {
      selectedGroupId.value = newGroupId;
      await loadImages(newGroupId);
    }
  } catch (error) {
    console.error('Failed to add group:', error);
    Snackbar.error('创建分组失败');
  }
}

// 显示新增图片弹窗
function showAddImageDialog() {
  showAddImagePopup.value = true;
}

// 处理图片上传（编辑模式版本）
async function handleEditModeUpload() {
  if (!config.value || !selectedGroupId.value || !selectedModeId.value) {
    Snackbar.warning('请先选择分组');
    return;
  }
  
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: true,
      filters: [{
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"]
      }]
    });
    
    if (!selected || (Array.isArray(selected) && selected.length === 0)) {
      return;
    }
    
    const filePaths = Array.isArray(selected) ? selected : [selected];
    
    await invoke("upload_images", {
      filePaths: filePaths,
      groupId: selectedGroupId.value,
      modeId: selectedModeId.value,
      memeDir: config.value.meme_dir
    });
    
    Snackbar.success(`成功上传 ${filePaths.length} 张图片`);
    showAddImagePopup.value = false;
    
    // 刷新图片列表
    await loadImages(selectedGroupId.value);
  } catch (error) {
    console.error("Failed to upload images:", error);
    Snackbar.warning("上传图片失败: " + error);
  }
}

const currentColorMode = ref<'system' | 'light' | 'dark'>('system');
const shareApp = ref<'wechat' | 'qq' | ''>('wechat');
const gridColumns = ref<number>(4);
const activeMenu = ref('home');
const isSideMenuOpen = ref(false);
const isMenuPopupOpen = ref(false);
const menuAnchor = ref<HTMLElement | { $el: HTMLElement } | null>(null);
const showGroupActionMenu = ref(false);
const currentEditingGroup = ref<Group | null>(null);
const pinyinSearchEnabled = ref(false);
const acronymSearchEnabled = ref(false);
// 预览功能已移除，相关变量保留以备后续需要
// const isImagePreviewOpen = ref(false);
// const previewImageIndex = ref(0);
const initialPinchScale = ref(1);
const lastPinchDistance = ref(0);

// 长按/右键菜单相关状态
const selectedModeForMenu = ref<Mode | null>(null);
const selectedGroupForMenu = ref<Group | null>(null);
const selectedImageForMenu = ref<Image | null>(null);

// 编辑弹窗状态
const showModeEditPopup = ref(false);
const showGroupEditPopup = ref(false);
const editingModeName = ref('');
const editingModeSortOrder = ref(0);
const editingGroupName = ref('');
const showKeywordManager = ref(false);

const isMobile = computed(() => window.innerWidth < 768);
const isSidebarMode = computed(() => !isMobile.value);

const menuPopupStyle = computed(() => {
  if (menuAnchor.value) {
    // 获取 DOM 元素（组件实例通过 $el 访问）
    const element = ('$el' in menuAnchor.value) ? menuAnchor.value.$el : menuAnchor.value;
    const rect = element.getBoundingClientRect();
    return {
      position: 'fixed' as const,
      top: `${rect.bottom + 8}px`,
      right: `${window.innerWidth - rect.right}px`,
    };
  }
  return {
    position: 'fixed' as const,
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
  };
});

onMounted(async () => {
  // 先加载配置，但不设置响应式监听
  await loadConfig();
  
  // 然后加载模式和数据
  await loadModes();
  
  if (selectedModeId.value) {
    await loadGroups(selectedModeId.value);
  }
  
  // 确保自动选择第一个分组（如果没有已选分组或已选分组不存在）
  if (!selectedGroupId.value && groups.value.length > 0) {
    selectedGroupId.value = groups.value[0].id;
    await loadImages(groups.value[0].id);
  } else if (selectedGroupId.value) {
    await loadImages(selectedGroupId.value);
  }
  
  // 添加事件监听
  window.addEventListener('resize', handleResize);
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('wheel', handleWheel, { passive: false });
  window.addEventListener('touchstart', handleTouchStart, { passive: false });
  window.addEventListener('touchmove', handleTouchMove, { passive: false });
  
  // 只在用户明确改变窗口大小时才调整列数
  let resizeTimeout: number;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = window.setTimeout(() => {
      // 如果用户没有手动调整过列数，才自动调整
      // 这里暂时不做自动调整，保留用户设置
    }, 300);
  });
  
  window.addEventListener('modeImported', async (event: any) => {
    const { modeId } = event.detail;
    await loadModes();
    if (modeId && modes.value.find(m => m.id === modeId)) {
      await switchMode(modeId);
    }
  });
  
  window.addEventListener('modeCreated', async (event: any) => {
    const { modeId } = event.detail;
    await loadModes();
    if (modeId && modes.value.find(m => m.id === modeId)) {
      await switchMode(modeId);
      activeMenu.value = 'home';
      Snackbar.success('已切换到新模式');
    }
  });
  
  window.addEventListener('navigateHome', () => {
    activeMenu.value = 'home';
  });
  
  window.addEventListener('navigateToMenu', (event: any) => {
    const detail = event.detail;
    if (typeof detail === 'string') {
      activeMenu.value = detail;
    } else if (detail && detail.menu) {
      activeMenu.value = detail.menu;
      // 如果有 groupId，存储在全局变量中
      if (detail.groupId) {
        (window as any).currentKeywordGroupId = detail.groupId;
      }
    }
  });
});

function handleResize() {
  if (window.innerWidth < 768) {
    gridColumns.value = 3;
  } else if (window.innerWidth < 1024) {
    gridColumns.value = 4;
  } else {
    gridColumns.value = 6;
  }
}

async function loadConfig() {
  try {
    const result = await invoke<Config>("get_config");
    if (result && result.meme_dir) {
      config.value = result;
      selectedModeId.value = config.value.last_mode || null;
      selectedGroupId.value = config.value.last_group || null;
      shareApp.value = config.value.share_app || 'wechat';
      gridColumns.value = config.value.grid_size || 4;
      currentColorMode.value = config.value.color_mode as 'system' | 'light' | 'dark';
      pinyinSearchEnabled.value = config.value.pinyin_search || false;
      acronymSearchEnabled.value = config.value.acronym_search || false;
      applyTheme();
      
      // 检查并加载 keywords.toml
      await checkAndLoadKeywords();
    } else {
      // 首次启动，需要选择目录
      await setupInitialConfig();
    }
  } catch (error) {
    console.error("Failed to load config:", error);
    await setupInitialConfig();
  }
}

async function setupInitialConfig() {
  const selectedDir = await selectDirectory();
  if (selectedDir) {
    const newConfig: Config = {
      meme_dir: selectedDir,
      color_mode: "system",
      theme_style: "modern",
      last_mode: 1,
      last_group: 1,
      share_app: "wechat",
      grid_size: 4,
      pinyin_search: false,
      acronym_search: false
    };
    config.value = newConfig;
    await invoke("update_config", { config: config.value });
    applyTheme();
    
    // 首次初始化后，检查是否需要生成 keywords.toml
    await checkAndLoadKeywords();
  } else {
    // 使用默认目录
    const defaultConfig: Config = {
      meme_dir: "/tmp/memes",
      color_mode: "system",
      theme_style: "modern",
      last_mode: 1,
      last_group: 1,
      share_app: "wechat",
      grid_size: 4,
      pinyin_search: false,
      acronym_search: false
    };
    config.value = defaultConfig;
    await invoke("update_config", { config: config.value });
    applyTheme();
  }
}

async function checkAndLoadKeywords() {
  if (!config.value?.meme_dir) return;
  
  try {
    // 直接生成关键词文件（覆盖现有文件）
    await invoke("generate_keywords_file", { 
      memeDir: config.value.meme_dir,
      generatePinyin: true,
      generateAcronym: true
    });
    console.log("Keywords file generated");
  } catch (error) {
    console.error("Failed to generate keywords file:", error);
  }
}

function applyTheme() {
  if (currentColorMode.value === 'dark') {
    document.documentElement.classList.add('var-dark');
  } else if (currentColorMode.value === 'light') {
    document.documentElement.classList.remove('var-dark');
  } else {
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.classList.add('var-dark');
    } else {
      document.documentElement.classList.remove('var-dark');
    }
  }
}

async function loadModes() {
  try {
    const result = await invoke<Mode[]>("get_modes");
    if (result) {
      modes.value = result.sort((a, b) => a.sort_order - b.sort_order);
      
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
      groups.value = result.sort((a, b) => b.share_count - a.share_count);
      
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
      images.value = result.sort((a, b) => b.share_count - a.share_count);
    } else {
      images.value = [];
    }
  } catch (error) {
    console.error("Failed to load images:", error);
    images.value = [];
  }
}

async function searchImages() {
  if (!searchKeyword.value.trim()) {
    // 搜索框为空，获取当前模式下所有分组
    try {
      const allGroups = await invoke<Group[]>("get_groups_by_mode", { 
        modeId: selectedModeId.value
      });
      
      if (allGroups && allGroups.length > 0) {
        groups.value = allGroups;
        // 自动选择第一个分组并加载图片
        selectedGroupId.value = allGroups[0].id;
        await loadImages(allGroups[0].id);
      } else {
        groups.value = [];
        images.value = [];
      }
    } catch (error) {
      console.error("Failed to load groups:", error);
    }
    return;
  }
  
  try {
    // 首先搜索匹配的分组，带上拼音搜索设置
    const matchedGroups = await invoke<Group[]>("search_groups", { 
      keyword: searchKeyword.value,
      modeId: selectedModeId.value,
      pinyinSearch: pinyinSearchEnabled.value,
      acronymSearch: acronymSearchEnabled.value
    });
    
    if (matchedGroups && matchedGroups.length > 0) {
      // 找到匹配的分组，更新分组列表
      groups.value = matchedGroups;
      
      // 自动选择第一个匹配的分组
      selectedGroupId.value = matchedGroups[0].id;
      await loadImages(matchedGroups[0].id);
    } else {
      // 没有找到匹配的分组，清空图片
      groups.value = [];
      images.value = [];
    }
  } catch (error) {
    console.error("Failed to search:", error);
    // 如果搜索分组失败，尝试原来的搜索图片方式
    try {
      const result = await invoke<Image[]>("search_images", { 
        keyword: searchKeyword.value,
        pinyin: pinyinSearchEnabled.value,
        acronym: acronymSearchEnabled.value
      });
      images.value = result || [];
    } catch (err) {
      console.error("Failed to search images:", err);
    }
  }
}

async function switchMode(active: string | number) {
  const modeId = Number(active);
  selectedModeId.value = modeId;
  await loadGroups(modeId);
  if (config.value) {
    config.value.last_mode = modeId;
    await invoke("update_config", { config: config.value });
  }
}

async function switchGroup(active: string | number) {
  const groupId = Number(active);
  selectedGroupId.value = groupId;
  await loadImages(groupId);
  if (config.value) {
    config.value.last_group = groupId;
    await invoke("update_config", { config: config.value });
  }
}

function toggleEditMode() {
  isEditMode.value = !isEditMode.value;
  selectedImages.value = [];
}

function toggleImageSelection(imageId: number) {
  const index = selectedImages.value.indexOf(imageId);
  if (index > -1) {
    selectedImages.value.splice(index, 1);
  } else {
    selectedImages.value.push(imageId);
  }
}

// 预览功能已移除，点击图片直接复制
// function openImagePreview(_index: number) {
//   console.log('Preview disabled, click to copy instead');
// }

async function uploadImages() {
  if (!config.value) return;
  
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: true,
      filters: [{
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"]
      }]
    });
    
    if (!selected || (Array.isArray(selected) && selected.length === 0)) {
      return;
    }
    
    const filePaths = Array.isArray(selected) ? selected : [selected];
    
    if (selectedGroupId.value && selectedModeId.value) {
      await invoke("upload_images", {
        filePaths: filePaths,
        groupId: selectedGroupId.value,
        modeId: selectedModeId.value,
        memeDir: config.value.meme_dir
      });
      
      await loadImages(selectedGroupId.value);
      Snackbar.success('上传成功');
    }
  } catch (error) {
    console.error("Failed to upload images:", error);
    Snackbar.warning("上传图片失败: " + error);
  }
}

async function deleteSelectedImages() {
  if (selectedImages.value.length === 0) return;
  
  Dialog({
    title: '确认删除',
    message: `确定要删除选中的 ${selectedImages.value.length} 张表情包吗？`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '确定',
    cancelButtonText: '取消'
  }).then(async () => {
    try {
      await invoke("delete_images", { imageIds: selectedImages.value });
      if (selectedGroupId.value) {
        await loadImages(selectedGroupId.value);
      }
      selectedImages.value = [];
      isEditMode.value = false;
      Snackbar.success('删除成功');
    } catch (error) {
      console.error("Failed to delete images:", error);
      Snackbar.error('删除失败');
    }
  });
}

async function copySelectedImages() {
  if (selectedImages.value.length === 0) return;
  
  try {
    await invoke("copy_images", { imageIds: selectedImages.value });
    Snackbar.success('已复制到剪贴板');
  } catch (error) {
    console.error("Failed to copy images:", error);
    Snackbar.error('复制失败');
  }
}

async function moveSelectedImages() {
  if (selectedImages.value.length === 0) return;
  
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择目标分组文件夹"
    });
    
    if (selected && selectedGroupId.value) {
      const targetPath = Array.isArray(selected) ? selected[0] : selected;
      await invoke("move_images", { 
        imageIds: selectedImages.value, 
        targetGroupId: selectedGroupId.value,
        targetPath 
      });
      await loadImages(selectedGroupId.value);
      selectedImages.value = [];
      isEditMode.value = false;
      Snackbar.success('移动成功');
    }
  } catch (error) {
    console.error("Failed to move images:", error);
    Snackbar.error('移动失败');
  }
}

async function addNewGroup() {
  if (!config.value) return;
  
  const mode = modes.value.find(m => m.id === selectedModeId.value);
  if (!mode) return;
  
  // 先检查该模式下是否已有分组，用于提示默认名称
  const existingGroups = groups.value.filter(g => g.mode_id === selectedModeId.value);
  const defaultName = existingGroups.length > 0 ? `新分组${existingGroups.length + 1}` : '新分组';
  
  // 使用浏览器原生 prompt 获取分组名
  const groupName = window.prompt('请输入分组名称', defaultName)?.trim();
  
  // 用户点击取消
  if (groupName === undefined) return;
  
  if (!groupName) {
    Snackbar.warning('分组名称不能为空');
    return;
  }
  
  // 检查该模式下是否已存在同名分组
  const existingGroup = groups.value.find(g => 
    g.mode_id === selectedModeId.value && g.name === groupName
  );
  if (existingGroup) {
    Snackbar.error(`该模式下已存在名为 "${groupName}" 的分组`);
    return;
  }
  
  try {
    const newGroupId = await invoke<number>("add_group", { 
      name: groupName,
      modeId: selectedModeId.value
    });
    
    // 重新加载分组列表
    if (selectedModeId.value) {
      await loadGroups(selectedModeId.value);
    }
    
    // 自动选中新创建的分组
    if (newGroupId) {
      selectedGroupId.value = newGroupId;
      await loadImages(newGroupId);
      Snackbar.success('分组创建成功');
    }
  } catch (error) {
    console.error("Failed to add group:", error);
    Snackbar.error('创建分组失败');
  }
}

function handleShareAppChange(app: string) {
  shareApp.value = app as 'wechat' | 'qq' | '';
  if (config.value) {
    config.value.share_app = shareApp.value;
    invoke("update_config", { config: config.value });
  }
}

function handleMenuChange(menu: string) {
  activeMenu.value = menu;
  isSideMenuOpen.value = false;
}

function handleWheel(event: WheelEvent) {
  if (!isMobile.value && event.ctrlKey) {
    event.preventDefault();
    if (event.deltaY < 0 && gridColumns.value < 8) {
      gridColumns.value++;
    } else if (event.deltaY > 0 && gridColumns.value > 2) {
      gridColumns.value--;
    }
    if (config.value) {
      config.value.grid_size = gridColumns.value;
      invoke("update_config", { config: config.value });
    }
  }
}

function handleTouchStart(event: TouchEvent) {
  if (event.touches.length === 2) {
    const dx = event.touches[0].clientX - event.touches[1].clientX;
    const dy = event.touches[0].clientY - event.touches[1].clientY;
    lastPinchDistance.value = Math.sqrt(dx * dx + dy * dy);
    initialPinchScale.value = gridColumns.value;
  }
}

function handleTouchMove(event: TouchEvent) {
  if (event.touches.length === 2) {
    event.preventDefault();
    const dx = event.touches[0].clientX - event.touches[1].clientX;
    const dy = event.touches[0].clientY - event.touches[1].clientY;
    const distance = Math.sqrt(dx * dx + dy * dy);
    const scale = distance / lastPinchDistance.value;
    const newColumns = Math.round(initialPinchScale.value / scale);
    gridColumns.value = Math.max(2, Math.min(8, newColumns));
    if (config.value) {
      config.value.grid_size = gridColumns.value;
      invoke("update_config", { config: config.value });
    }
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key === 'v') {
    handlePasteImage();
  }
}

async function handlePasteImage() {
  if (!isTauri) {
    Snackbar.warning('剪贴板功能仅在桌面端可用');
    return;
  }
  
  try {
    // 使用 Tauri 后端读取剪贴板图片并保存到临时文件
    const { tempDir } = await import('@tauri-apps/api/path');
    const tempDirPath = await tempDir();
    
    const tempFilePath = await invoke<string>("paste_image_from_clipboard_raw", {
      tempDir: tempDirPath
    });
    
    if (!tempFilePath) {
      Snackbar.info('剪贴板中没有图片');
      return;
    }
    
    // 上传到当前分组
    if (selectedGroupId.value && selectedModeId.value && config.value) {
      await invoke("upload_images", {
        filePaths: [tempFilePath],
        groupId: selectedGroupId.value,
        modeId: selectedModeId.value,
        memeDir: config.value.meme_dir
      });
      await loadImages(selectedGroupId.value);
      Snackbar.success('图片已粘贴到当前分组');
    }
  } catch (error) {
    console.error("Failed to paste image:", error);
    Snackbar.warning('粘贴图片失败: ' + error);
  }
}

async function refreshIndex() {
  try {
    Snackbar.info('正在刷新索引...');
    
    // 重建所有分组的关键词关联
    const count = await invoke("rebuild_all_group_keywords");
    console.log(`Rebuilt keywords for ${count} groups`);
    
    await invoke("refresh_index", { memeDir: config.value?.meme_dir });
    await loadModes();
    if (selectedModeId.value) {
      await loadGroups(selectedModeId.value);
    }
    if (selectedGroupId.value) {
      await loadImages(selectedGroupId.value);
    }
    Snackbar.success(`索引刷新完成，已重建 ${count} 个分组的搜索关联`);
  } catch (error) {
    console.error("Failed to refresh index:", error);
    Snackbar.error('索引刷新失败');
  }
}

function openMenu(event: Event) {
  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  if (target) {
    menuAnchor.value = target;
    isMenuPopupOpen.value = true;
  }
}

function handleMenuAction(action: string) {
  isMenuPopupOpen.value = false;
  
  switch (action) {
    case 'edit':
      toggleEditMode();
      break;
    case 'editGroup':
      openGroupEditDialog();
      break;
    case 'addMode':
      activeMenu.value = 'mode';
      break;
    case 'addGroup':
      addNewGroup();
      break;
    case 'refresh':
      refreshIndex();
      break;
    case 'settings':
      activeMenu.value = 'settings';
      break;
  }
}

function openGroupEditDialog() {
  if (!selectedGroupId.value) {
    Snackbar.warning('请先选择一个分组');
    return;
  }
  
  const group = groups.value.find(g => g.id === selectedGroupId.value);
  if (!group) {
    Snackbar.error('当前分组不存在');
    return;
  }
  
  // 打开居中操作菜单
  currentEditingGroup.value = group;
  showGroupActionMenu.value = true;
}

function closeGroupActionMenu() {
  showGroupActionMenu.value = false;
  currentEditingGroup.value = null;
}

async function handleRenameGroup() {
  const group = currentEditingGroup.value;
  if (!group) return;
  
  closeGroupActionMenu();
  
  const newName = window.prompt('请输入新分组名称:', group.name);
  if (newName && newName.trim() && newName.trim() !== group.name) {
    const trimmedName = newName.trim();
    // 检查重名
    const existingGroup = groups.value.find(g => 
      g.mode_id === group.mode_id && g.name === trimmedName && g.id !== group.id
    );
    if (existingGroup) {
      Snackbar.error('该模式下已存在同名分组');
      return;
    }
    
    try {
      await invoke('update_group', {
        id: group.id,
        name: trimmedName,
        modeId: group.mode_id
      });
      Snackbar.success('分组已重命名');
      if (selectedModeId.value) {
        await loadGroups(selectedModeId.value);
      }
    } catch (error) {
      console.error('Failed to rename group:', error);
      Snackbar.error('重命名失败');
    }
  }
}

async function handleDeleteGroup() {
  const group = currentEditingGroup.value;
  if (!group) return;
  
  closeGroupActionMenu();
  
  // 使用 Dialog 确认删除
  Dialog({
    title: '确认删除',
    message: `确定要删除分组 "${group.name}" 吗？此操作不可恢复！`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  }).then(async (result) => {
    if (result === 'confirm') {
      try {
        await invoke('delete_group', { groupId: group.id });
        Snackbar.success('分组已删除');
        if (selectedModeId.value) {
          await loadGroups(selectedModeId.value);
        }
      } catch (error) {
        console.error('Failed to delete group:', error);
        Snackbar.error('删除分组失败');
      }
    }
  });
}

function handleManageKeywords() {
  const group = currentEditingGroup.value;
  if (!group) return;
  
  closeGroupActionMenu();
  
  // 跳转到关键词管理页面
  window.dispatchEvent(new CustomEvent('navigateToMenu', { 
    detail: { menu: 'keyword', groupId: group.id }
  }));
}

function togglePinyinSearch() {
  pinyinSearchEnabled.value = !pinyinSearchEnabled.value;
  if (config.value) {
    config.value.pinyin_search = pinyinSearchEnabled.value;
    invoke("update_config", { config: config.value });
  }
}

function toggleAcronymSearch() {
  acronymSearchEnabled.value = !acronymSearchEnabled.value;
  if (config.value) {
    config.value.acronym_search = acronymSearchEnabled.value;
    invoke("update_config", { config: config.value });
  }
}

// ========== 长按/右键菜单相关函数 ==========

// 模式菜单配置
const modeMenuItems = [
  { label: '重命名', value: 'rename', icon: 'pencil' },
  { label: '修改序号', value: 'sort', icon: 'sort-numeric-ascending' },
  { label: '删除', value: 'delete', icon: 'delete', danger: true },
];

// 分组菜单配置
const groupMenuItems = [
  { label: '重命名', value: 'rename', icon: 'pencil' },
  { label: '关键词', value: 'keywords', icon: 'label' },
  { label: '删除', value: 'delete', icon: 'delete', danger: true },
];

// 图片菜单配置
const imageMenuItems = [
  { label: '删除', value: 'delete', icon: 'delete', danger: true },
];

// 处理模式菜单选择
async function handleModeMenuSelect(mode: Mode, action: string) {
  selectedModeForMenu.value = mode;
  
  switch (action) {
    case 'rename':
      editingModeName.value = mode.name;
      showModeEditPopup.value = true;
      break;
    case 'sort':
      editingModeSortOrder.value = mode.sort_order;
      showModeEditPopup.value = true;
      break;
    case 'delete':
      Dialog({
        title: '确认删除',
        message: `确定要删除模式 "${mode.name}" 吗？相关文件将移动到回收站。`,
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      }).then(async () => {
        try {
          await invoke('delete_mode', { modeId: mode.id });
          Snackbar.success('模式已删除');
          await loadModes();
        } catch (error) {
          console.error('Failed to delete mode:', error);
          Snackbar.error('删除模式失败');
        }
      });
      break;
  }
}

// 提交模式编辑
async function submitModeEdit() {
  if (!selectedModeForMenu.value) return;
  
  const mode = selectedModeForMenu.value;
  const trimmedName = editingModeName.value.trim();
  
  if (!trimmedName) {
    Snackbar.warning('模式名称不能为空');
    return;
  }
  
  try {
    await invoke('update_mode', {
      id: mode.id,
      name: trimmedName,
      sortOrder: editingModeSortOrder.value
    });
    Snackbar.success('模式已更新');
    showModeEditPopup.value = false;
    await loadModes();
  } catch (error) {
    console.error('Failed to update mode:', error);
    Snackbar.error('更新模式失败');
  }
}

// 处理分组菜单选择
async function handleGroupMenuSelect(group: Group, action: string) {
  selectedGroupForMenu.value = group;
  
  switch (action) {
    case 'rename':
      editingGroupName.value = group.name;
      showGroupEditPopup.value = true;
      break;
    case 'keywords':
      showKeywordManager.value = true;
      break;
    case 'delete':
      Dialog({
        title: '确认删除',
        message: `确定要删除分组 "${group.name}" 吗？相关文件将移动到回收站。`,
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      }).then(async () => {
        try {
          await invoke('delete_group', { groupId: group.id });
          Snackbar.success('分组已删除');
          if (selectedModeId.value) {
            await loadGroups(selectedModeId.value);
          }
        } catch (error) {
          console.error('Failed to delete group:', error);
          Snackbar.error('删除分组失败');
        }
      });
      break;
  }
}

// 提交分组编辑
async function submitGroupEdit() {
  if (!selectedGroupForMenu.value) return;
  
  const group = selectedGroupForMenu.value;
  const trimmedName = editingGroupName.value.trim();
  
  if (!trimmedName) {
    Snackbar.warning('分组名称不能为空');
    return;
  }
  
  // 检查该模式下是否已存在同名分组
  const existingGroup = groups.value.find(g => 
    g.mode_id === group.mode_id && g.name === trimmedName && g.id !== group.id
  );
  if (existingGroup) {
    Snackbar.error('该模式下已存在同名分组');
    return;
  }
  
  try {
    await invoke('update_group', {
      id: group.id,
      name: trimmedName,
      modeId: group.mode_id
    });
    Snackbar.success('分组已更新');
    showGroupEditPopup.value = false;
    if (selectedModeId.value) {
      await loadGroups(selectedModeId.value);
    }
  } catch (error) {
    console.error('Failed to update group:', error);
    Snackbar.error('更新分组失败');
  }
}

// 复制单张图片
async function copySingleImage(img: Image) {
  try {
    await invoke("copy_images", { imageIds: [img.id] });
    Snackbar.success('已复制到剪贴板');
  } catch (error) {
    console.error("Failed to copy image:", error);
    Snackbar.error('复制失败');
  }
}

// 处理浮动搜索按钮点击
function handleFloatingSearchClick() {
  searchKeyword.value = '';
  // 聚焦搜索框
  const searchInput = document.querySelector('.search-input input') as HTMLInputElement;
  if (searchInput) {
    searchInput.focus();
  }
}

// 处理图片菜单选择
async function handleImageMenuSelect(img: Image, action: string) {
  selectedImageForMenu.value = img;
  
  switch (action) {
    case 'delete':
      Dialog({
        title: '确认删除',
        message: '确定要删除这张表情包吗？',
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      }).then(async () => {
        try {
          await invoke('delete_images', { imageIds: [img.id] });
          Snackbar.success('图片已删除');
          if (selectedGroupId.value) {
            await loadImages(selectedGroupId.value);
          }
        } catch (error) {
          console.error('Failed to delete image:', error);
          Snackbar.error('删除图片失败');
        }
      });
      break;
  }
}
</script>

<template>
  <ThemeProvider 
  :theme-style="config?.theme_style || 'modern'"
    :color-mode="currentColorMode"
  >
    <div class="app-container" :class="{ 'sidebar-open': isSidebarMode && isSideMenuOpen }">
      <SideMenu 
      :is-open="isSideMenuOpen" 
      :active-menu="activeMenu"
      :is-sidebar="isSidebarMode"
      @update:is-open="isSideMenuOpen = $event"
      @menu-change="handleMenuChange"
    />
    
    <main class="main-content">
      <div v-if="activeMenu === 'home'" class="view-home">
        <div class="top-search-bar">
          <div class="search-container">
            <var-input
              v-model="searchKeyword" 
              placeholder="搜索关键词..." 
              @keydown.enter="searchImages"
              clearable
              class="search-input"
            >
              <template #prepend-icon>
                <var-icon name="magnify" />
              </template>
            </var-input>
          <div class="search-actions">
            <button class="btn-icon" @click="uploadImages" title="上传图片">
              <Icon name="camera" :size="24" />
            </button>
            <!-- 全局编辑模式按钮 -->
            <button 
              v-if="!isGlobalEditMode"
              class="btn-icon"
              @click="toggleGlobalEditMode" 
              title="进入编辑模式"
            >
              <Icon name="edit-2" :size="24" />
            </button>
            <button 
              v-else-if="hasSelectedItems"
              class="btn-icon btn-danger"
              @click="handleBatchDelete" 
              title="批量删除"
            >
              <Icon name="delete-bin" :size="24" />
            </button>
            <button 
              v-if="isGlobalEditMode"
              class="btn-icon"
              @click="exitGlobalEditMode" 
              title="取消编辑"
            >
              <Icon name="close" :size="24" />
            </button>
            <button class="btn-icon" @click="openMenu" title="菜单" ref="menuAnchor">
              <Icon name="menu-3" :size="24" />
            </button>
          </div>
          </div>
          
          <div v-if="isMobile" class="share-app-selector">
            <var-radio-group v-model="shareApp" direction="horizontal" size="small" @change="handleShareAppChange">
              <var-radio name="wechat">微信</var-radio>
              <var-radio name="qq">QQ</var-radio>
            </var-radio-group>
          </div>
        </div>

        <teleport to="body">
          <div v-if="isMenuPopupOpen" class="menu-overlay" @click="isMenuPopupOpen = false">
            <div 
              class="menu-popup"
              :style="menuPopupStyle"
              @click.stop
            >
              <div class="menu-popup-content">
                <div class="menu-item" @click="handleMenuAction('edit')">
                  <var-icon name="pencil" size="18" />
                  <span>{{ isEditMode ? '退出管理' : '管理图片' }}</span>
                </div>
                <div class="menu-item" @click="handleMenuAction('editGroup')">
                  <var-icon name="folder-edit" size="18" />
                  <span>编辑分组</span>
                </div>
                <div class="menu-divider"></div>
                <div class="menu-item" @click="handleMenuAction('addMode')">
                  <var-icon name="folder-plus" size="18" />
                  <span>添加模式</span>
                </div>
                <div class="menu-item" @click="handleMenuAction('addGroup')">
                  <var-icon name="folder-outline" size="18" />
                  <span>添加分组</span>
                </div>
                <div class="menu-divider"></div>
                <div class="menu-item" @click="togglePinyinSearch">
                  <var-icon :name="pinyinSearchEnabled ? 'checkbox-marked' : 'checkbox-blank-outline'" size="18" />
                  <span>拼音搜索</span>
                </div>
                <div class="menu-item" @click="toggleAcronymSearch">
                  <var-icon :name="acronymSearchEnabled ? 'checkbox-marked' : 'checkbox-blank-outline'" size="18" />
                  <span>首字母搜索</span>
                </div>
                <div class="menu-divider"></div>
                <div class="menu-item" @click="handleMenuAction('refresh')">
                  <var-icon name="refresh" size="18" />
                  <span>刷新索引</span>
                </div>
                <div class="menu-item" @click="handleMenuAction('settings')">
                  <var-icon name="cog" size="18" />
                  <span>设置</span>
                </div>
              </div>
            </div>
          </div>
        </teleport>

        <!-- 编辑分组操作菜单 - 居中纵向菜单 -->
        <teleport to="body">
          <div v-if="showGroupActionMenu" class="group-action-overlay" @click="closeGroupActionMenu">
            <div class="group-action-menu" @click.stop>
              <div class="group-action-header">
                <h3>{{ currentEditingGroup?.name }}</h3>
                <var-button text round @click="closeGroupActionMenu">
                  <var-icon name="window-close" size="20" />
                </var-button>
              </div>
              <div class="group-action-list">
                <div class="group-action-item" @click="handleRenameGroup">
                  <var-icon name="pencil" size="20" />
                  <span>重命名分组</span>
                </div>
                <div class="group-action-item" @click="handleManageKeywords">
                  <var-icon name="label" size="20" />
                  <span>管理关键词</span>
                </div>
                <div class="group-action-divider"></div>
                <div class="group-action-item danger" @click="handleDeleteGroup">
                  <var-icon name="delete" size="20" />
                  <span>删除分组</span>
                </div>
              </div>
            </div>
          </div>
        </teleport>

        <!-- 模式选择栏 -->
        <div class="modern-tabs-container" :class="{ 'edit-mode': isGlobalEditMode }">
          <div class="modern-tabs-scroll">
            <div class="modern-tabs">
              <!-- 添加模式按钮（编辑模式下显示） -->
              <div
                v-if="isGlobalEditMode"
                class="modern-tab add-tab"
                @click="showAddModeDialog"
              >
                <var-icon name="plus" size="18" />
              </div>
              <!-- 模式列表 -->
              <div
                v-for="mode in modes"
                :key="mode.id"
                class="modern-tab-wrapper"
                :class="{ 'is-selectable': isGlobalEditMode }"
              >
                <!-- 多选复选框 -->
                <div
                  v-if="isGlobalEditMode"
                  class="select-checkbox"
                  :class="{ 'is-checked': selectedModeIds.includes(mode.id) }"
                  @click.stop="toggleModeSelection(mode.id)"
                >
                  <var-icon v-if="selectedModeIds.includes(mode.id)" name="checkbox-marked" size="18" color="var(--color-primary)" />
                  <var-icon v-else name="checkbox-blank-outline" size="18" color="var(--color-text-3)" />
                </div>
                <ContextMenu
                  :items="modeMenuItems"
                  @select="(action) => handleModeMenuSelect(mode, action)"
                >
                  <div
                    class="modern-tab"
                    :class="{ 
                      active: selectedModeId === mode.id,
                      'is-selected': selectedModeIds.includes(mode.id)
                    }"
                    @click="!isGlobalEditMode && switchMode(mode.id)"
                  >
                    {{ mode.name }}
                  </div>
                </ContextMenu>
              </div>
            </div>
          </div>
        </div>

        <!-- 分组选择栏 -->
        <div class="modern-tabs-container secondary" :class="{ 'edit-mode': isGlobalEditMode }">
          <div class="modern-tabs-scroll">
            <div class="modern-tabs">
              <!-- 添加分组按钮（编辑模式下显示） -->
              <div
                v-if="isGlobalEditMode"
                class="modern-tab add-tab"
                @click="showAddGroupDialog"
              >
                <var-icon name="plus" size="18" />
              </div>
              <!-- 分组列表 -->
              <div
                v-for="group in groups"
                :key="group.id"
                class="modern-tab-wrapper"
                :class="{ 'is-selectable': isGlobalEditMode }"
              >
                <!-- 多选复选框 -->
                <div
                  v-if="isGlobalEditMode"
                  class="select-checkbox"
                  :class="{ 'is-checked': selectedGroupIds.includes(group.id) }"
                  @click.stop="toggleGroupSelection(group.id)"
                >
                  <var-icon v-if="selectedGroupIds.includes(group.id)" name="checkbox-marked" size="18" color="var(--color-primary)" />
                  <var-icon v-else name="checkbox-blank-outline" size="18" color="var(--color-text-3)" />
                </div>
                <ContextMenu
                  :items="groupMenuItems"
                  @select="(action) => handleGroupMenuSelect(group, action)"
                >
                  <div
                    class="modern-tab"
                    :class="{ 
                      active: selectedGroupId === group.id,
                      'is-selected': selectedGroupIds.includes(group.id)
                    }"
                    @click="!isGlobalEditMode && switchGroup(group.id)"
                  >
                    {{ group.name }}
                  </div>
                </ContextMenu>
              </div>
            </div>
          </div>
        </div>

        <div
          class="image-grid"
          :style="{ gridTemplateColumns: `repeat(${gridColumns}, 1fr)` }"
          :class="{ 'edit-mode': isGlobalEditMode }"
        >
          <!-- 添加图片按钮（编辑模式下显示） -->
          <div
            v-if="isGlobalEditMode"
            class="image-item add-image-item"
            @click="showAddImageDialog"
          >
            <div class="add-image-content">
              <var-icon name="plus" size="32" color="var(--color-primary)" />
              <span>添加图片</span>
            </div>
          </div>
          <!-- 图片列表 -->
        <div
          v-for="img in images" 
          :key="img.id"
          class="image-item"
          :class="{ 
            selected: selectedImages.includes(img.id),
            'is-selectable': isGlobalEditMode 
          }"
        >
            <!-- 多选复选框 -->
            <div
              v-if="isGlobalEditMode"
              class="image-select-checkbox"
              :class="{ 'is-checked': selectedImages.includes(img.id) }"
              @click.stop="toggleImageSelection(img.id)"
            >
              <var-icon 
                v-if="selectedImages.includes(img.id)" 
                name="checkbox-marked" 
                size="20" 
                color="var(--color-primary)" 
              />
              <var-icon 
                v-else 
                name="checkbox-blank-outline" 
                size="20" 
                color="var(--color-text-3)" 
              />
            </div>
            <ContextMenu
              :items="imageMenuItems"
              @select="(action) => handleImageMenuSelect(img, action)"
            >
              <var-image
                :src="toAssetPath(img.thumbnail_path || img.image_path)"
                fit="cover"
                class="image-content"
                @click="!isGlobalEditMode && copySingleImage(img)"
              />
            </ContextMenu>
            <div v-if="selectedImages.includes(img.id) && !isGlobalEditMode" class="check-overlay">
              <var-icon name="check-circle" size="24" color="#fff" />
            </div>
          </div>
        </div>

        <!-- 模式编辑弹窗 -->
        <var-popup :show="showModeEditPopup" @click-overlay="showModeEditPopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>编辑模式</h3>
              <var-button text round @click="showModeEditPopup = false">
                <var-icon name="window-close" size="20" />
              </var-button>
            </div>
            <div class="edit-popup-body">
              <var-input
                v-model="editingModeName"
                label="模式名称"
                placeholder="请输入模式名称"
                class="edit-input"
              />
              <var-input
                :model-value="String(editingModeSortOrder)"
                label="排序序号"
                type="number"
                placeholder="请输入排序序号"
                class="edit-input"
                @update:model-value="(val: string) => editingModeSortOrder = Number(val)"
              />
              <div class="edit-popup-actions">
                <var-button type="primary" block @click="submitModeEdit">保存</var-button>
                <var-button type="default" block @click="showModeEditPopup = false">取消</var-button>
              </div>
            </div>
          </div>
        </var-popup>

        <!-- 分组编辑弹窗 -->
        <var-popup :show="showGroupEditPopup" @click-overlay="showGroupEditPopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>重命名分组</h3>
              <var-button text round @click="showGroupEditPopup = false">
                <var-icon name="window-close" size="20" />
              </var-button>
            </div>
            <div class="edit-popup-body">
              <var-input
                v-model="editingGroupName"
                label="分组名称"
                placeholder="请输入分组名称"
                class="edit-input"
                @keydown.enter="submitGroupEdit"
              />
              <div class="edit-popup-actions">
                <var-button type="primary" block @click="submitGroupEdit">保存</var-button>
                <var-button type="default" block @click="showGroupEditPopup = false">取消</var-button>
              </div>
            </div>
          </div>
        </var-popup>

        <!-- 新增模式弹窗 -->
        <var-popup :show="showAddModePopup" @click-overlay="showAddModePopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>新增模式</h3>
              <var-button text round @click="showAddModePopup = false">
                <var-icon name="window-close" size="20" />
              </var-button>
            </div>
            <div class="edit-popup-body">
              <var-input
                v-model="newModeName"
                label="模式名称"
                placeholder="请输入模式名称"
                class="edit-input"
                @keydown.enter="submitAddMode"
              />
              <div class="edit-popup-actions">
                <var-button type="primary" block @click="submitAddMode">创建</var-button>
                <var-button type="default" block @click="showAddModePopup = false">取消</var-button>
              </div>
            </div>
          </div>
        </var-popup>

        <!-- 新增分组弹窗 -->
        <var-popup :show="showAddGroupPopup" @click-overlay="showAddGroupPopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>新增分组</h3>
              <var-button text round @click="showAddGroupPopup = false">
                <var-icon name="window-close" size="20" />
              </var-button>
            </div>
            <div class="edit-popup-body">
              <div class="current-mode-display" v-if="selectedModeId">
                <var-icon name="folder" size="16" />
                <span>当前模式: {{ modes.find(m => m.id === selectedModeId)?.name }}</span>
              </div>
              <var-input
                v-model="newGroupName"
                label="分组名称"
                placeholder="请输入分组名称"
                class="edit-input"
                @keydown.enter="submitAddGroup"
              />
              <div class="edit-popup-actions">
                <var-button type="primary" block @click="submitAddGroup">创建</var-button>
                <var-button type="default" block @click="showAddGroupPopup = false">取消</var-button>
              </div>
            </div>
          </div>
        </var-popup>

        <!-- 新增图片弹窗 -->
        <var-popup :show="showAddImagePopup" @click-overlay="showAddImagePopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>添加图片</h3>
              <var-button text round @click="showAddImagePopup = false">
                <var-icon name="window-close" size="20" />
              </var-button>
            </div>
            <div class="edit-popup-body">
              <div class="current-group-display" v-if="selectedGroupId">
                <var-icon name="image" size="16" />
                <span>添加到: {{ groups.find(g => g.id === selectedGroupId)?.name }}</span>
              </div>
              <div class="upload-hint">
                <var-icon name="cloud-upload" size="48" color="var(--color-primary)" />
                <p>点击选择图片上传</p>
                <p class="upload-subtext">支持多选，支持 png、jpg、jpeg、gif、webp、bmp</p>
              </div>
              <div class="edit-popup-actions">
                <var-button type="primary" block @click="handleEditModeUpload">选择图片</var-button>
                <var-button type="default" block @click="showAddImagePopup = false">取消</var-button>
              </div>
            </div>
          </div>
        </var-popup>

        <!-- 关键词管理弹窗 -->
        <KeywordManager
          v-model:show="showKeywordManager"
          :group-name="selectedGroupForMenu?.name || ''"
        />
        
        <var-result
          v-if="images.length === 0" 
          type="empty"
          title="暂无表情包"
          class="empty-state"
        />

        <var-card v-if="isEditMode" class="edit-actions">
          <div class="edit-buttons">
            <var-button type="danger" @click="deleteSelectedImages" block>
              <var-icon name="delete" /> 删除 ({{ selectedImages.length }})
            </var-button>
            <var-button type="primary" @click="copySelectedImages" block>
              <var-icon name="content-copy" /> 复制
            </var-button>
            <var-button type="warning" @click="moveSelectedImages" block>
              <var-icon name="arrow-right-bold" /> 移动
            </var-button>
            <var-button type="default" @click="toggleEditMode" block>
              <var-icon name="close" /> 取消
            </var-button>
          </div>
        </var-card>
      </div>

      <ModeManagement v-else-if="activeMenu === 'mode'" />
      <GroupManagement v-else-if="activeMenu === 'group'" />
      <KeywordManagement v-else-if="activeMenu === 'keyword'" />
      <Settings v-else-if="activeMenu === 'settings'" />
      </main>
      
      <!-- 浮动搜索按钮 -->
      <FloatingSearchButton
        :visible="activeMenu === 'home' && !isGlobalEditMode"
        @click="handleFloatingSearchClick"
      />
    </div>
  </ThemeProvider>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background-color: var(--color-bg);
  position: relative;
}

.app-container.sidebar-open .main-content {
  margin-left: 280px;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: margin-left 0.3s ease;
}

.view-home {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.top-search-bar {
  padding: 12px;
  background-color: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  z-index: 10;
}

.search-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-input {
  flex: 1;
  border-radius: 16px;
  background-color: var(--color-bg-2);
}

.search-actions {
  display: flex;
  gap: 8px;
}

.share-app-selector {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border);
}

/* 现代化 Tabs */
.modern-tabs-container {
  position: relative;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-bg);
  padding: 8px 12px;
}

.modern-tabs-container.secondary {
  background-color: var(--color-bg-2);
}

.modern-tabs-scroll {
  overflow-x: auto;
  white-space: nowrap;
  scrollbar-width: none;
  -ms-overflow-style: none;
  -webkit-overflow-scrolling: touch;
}

.modern-tabs-scroll::-webkit-scrollbar {
  display: none;
}

.modern-tabs {
  display: inline-flex;
  gap: 8px;
  padding: 4px;
}

.modern-tab {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-2);
  background-color: transparent;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  white-space: nowrap;
}

.modern-tab:hover {
  background-color: var(--color-bg-2);
  color: var(--color-text);
}

.modern-tab.active {
  background-color: var(--color-primary);
  color: white;
  box-shadow: 0 2px 8px rgba(var(--color-primary-rgb, 59, 130, 246), 0.4);
  transform: translateY(-1px);
}

.modern-tab:active {
  transform: scale(0.95);
}

/* 添加按钮样式 */
.modern-tab.add-tab {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
  border: 2px dashed var(--color-primary);
  min-width: 60px;
}

.modern-tab.add-tab:hover {
  background-color: var(--color-primary);
  color: white;
  border-style: solid;
}

/* Tab 包装器和选择模式 */
.modern-tab-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.modern-tab-wrapper.is-selectable .modern-tab {
  padding-left: 8px;
}

.modern-tab.is-selected {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
  border: 2px solid var(--color-primary);
}

/* 选择复选框 */
.select-checkbox {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.select-checkbox:hover {
  background-color: var(--color-bg-2);
}

.select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

/* 当前模式/分组显示 */
.current-mode-display,
.current-group-display {
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

.current-mode-display span,
.current-group-display span {
  font-weight: 500;
  color: var(--color-text);
}

/* 上传提示 */
.upload-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 20px;
  gap: 8px;
  text-align: center;
}

.upload-hint p {
  margin: 0;
  font-size: 16px;
  color: var(--color-text);
  font-weight: 500;
}

.upload-hint .upload-subtext {
  font-size: 13px;
  color: var(--color-text-3);
  font-weight: normal;
}

/* 旧样式保留兼容 */
.tab-container {
  position: relative;
  border-bottom: 1px solid var(--color-border);
}

.tab-scroll {
  overflow-x: auto;
  white-space: nowrap;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.tab-scroll::-webkit-scrollbar {
  display: none;
}

.image-grid {
  flex: 1;
  min-height: 300px;
  padding: 12px;
  overflow-y: auto;
  width: 100%;
  height: calc(100vh - 250px);
  box-sizing: border-box;
  background-color: var(--color-bg);
  display: grid;
  gap: 12px;
  align-content: start;
}

.image-item {
  position: relative;
  height: 0;
  padding-bottom: 100%;
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border-radius: 16px;
}

.image-content :deep(img) {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 16px;
}

.image-item:active {
  transform: scale(0.95);
}

.image-item.selected {
  outline: 3px solid var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-light);
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

/* 图片网格编辑模式 */
.image-grid.edit-mode {
  padding-top: 8px;
}

/* 添加图片按钮 */
.image-item.add-image-item {
  background-color: var(--color-primary-light);
  border: 2px dashed var(--color-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-item.add-image-item:hover {
  background-color: var(--color-primary);
  border-style: solid;
}

.add-image-content {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.add-image-content span {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-primary);
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

.image-item.add-image-item:hover :deep(.var-icon) {
  color: white !important;
}

/* 图片选择复选框 */
.image-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.image-select-checkbox:hover {
  transform: scale(1.1);
  background-color: white;
}

.image-select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

.image-item.is-selectable {
  cursor: default;
}

.image-item.is-selectable .image-content {
  cursor: pointer;
}

.image-error {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-2);
  border-radius: 16px;
}

.check-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background-color: rgba(0, 0, 0, 0.6);
  border-radius: 50%;
}

.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
  background-color: var(--color-bg-2);
  border-radius: 16px;
  margin: 20px 0;
}

.edit-actions {
  padding: 12px;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-bg);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.08);
}

.edit-buttons {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

@media (max-width: 768px) {
  .app-container.sidebar-open .main-content {
    margin-left: 0;
  }
  
  .top-search-bar {
    padding: 8px 12px;
  }
  
  .image-grid {
    padding: 8px;
  }
  
  .image-item {
    border-radius: 12px;
  }
  
  .image-content {
    border-radius: 12px;
  }
  
  .edit-buttons {
    grid-template-columns: 1fr;
  }
}

.image-grid::-webkit-scrollbar {
  width: 6px;
}

.image-grid::-webkit-scrollbar-track {
  background: var(--color-bg-2);
  border-radius: 3px;
}

.image-grid::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.image-grid::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-3);
}

.menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2000;
  background: transparent;
}

.menu-popup {
  padding: 8px 0;
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  background-color: var(--color-bg);
  backdrop-filter: blur(10px);
  min-width: 180px;
}

.menu-popup-content {
  min-width: 180px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  font-size: 14px;
  color: var(--color-text);
  border-radius: 12px;
  margin: 2px 8px;
}

.menu-item:hover {
  background-color: var(--color-bg-2);
}

.menu-item:active {
  background-color: var(--color-primary-light);
}

.menu-item span {
  flex: 1;
}

.menu-divider {
  height: 1px;
  background-color: var(--color-border);
  margin: 8px 16px;
}

/* 编辑分组操作菜单 - 居中纵向菜单 */
.group-action-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
  background-color: rgba(0, 0, 0, 0);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
}

.group-action-menu {
  position: relative;
  z-index: 10000;
  width: 280px;
  max-width: 90vw;
  background-color: var(--color-bg);
  border-radius: 20px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  animation: menuAppear 0.2s ease;
}

@keyframes menuAppear {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.group-action-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-bg-2);
}

.group-action-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.group-action-list {
  padding: 12px;
}

.group-action-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 15px;
  color: var(--color-text);
  border-radius: 12px;
  margin: 4px 0;
}

.group-action-item:hover {
  background-color: var(--color-bg-2);
}

.group-action-item:active {
  background-color: var(--color-primary-light);
  transform: scale(0.98);
}

.group-action-item.danger {
  color: var(--color-danger, #f44336);
}

.group-action-item.danger:hover {
  background-color: var(--color-danger-light, rgba(244, 67, 54, 0.1));
}

.group-action-divider {
  height: 1px;
  background-color: var(--color-border);
  margin: 12px 0;
}

@media (max-width: 768px) {
  .group-action-menu {
    width: 260px;
    border-radius: 16px;
  }
  
  .group-action-header {
    padding: 14px 16px;
  }
  
  .group-action-header h3 {
    font-size: 15px;
  }
  
  .group-action-item {
    padding: 12px 14px;
    font-size: 14px;
  }
  
  .group-action-list {
    padding: 8px;
  }
}

/* 编辑弹窗样式 */
.edit-popup {
  width: 320px;
  max-width: 90vw;
  background-color: var(--color-bg);
  border-radius: 16px;
  overflow: hidden;
}

.edit-popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-bg-2);
}

.edit-popup-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.edit-popup-body {
  padding: 20px;
}

.edit-input {
  margin-bottom: 16px;
}

.edit-popup-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.edit-popup-actions .var-button {
  flex: 1;
}

@media (max-width: 768px) {
  .edit-popup {
    width: 300px;
    border-radius: 14px;
  }
  
  .edit-popup-header {
    padding: 14px 16px;
  }
  
  .edit-popup-header h3 {
    font-size: 15px;
  }
  
  .edit-popup-body {
    padding: 16px;
  }
}
</style>
