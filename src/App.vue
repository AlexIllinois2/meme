<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from "vue";
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

// 检测是否为 Android Tauri 环境
function isAndroidTauri() {
  return isTauri && /android/i.test(navigator.userAgent);
}

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
          await safeUpdateConfig(config.value);
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
const searchInputRef = ref<any>(null);  // 搜索框组件引用
const isEditMode = ref(false);
const selectedImages = ref<number[]>([]);

// 全局编辑模式状态
const isGlobalEditMode = ref(false);
// 全局悬浮窗状态
const globalFloatingWindowEnabled = ref(false);
const selectedModeIds = ref<number[]>([]);
const selectedGroupIds = ref<number[]>([]);

// 首次使用提示蒙版
const showFirstUseMask = ref(false);
const hasUserInteracted = ref(false);

// 监听用户交互
function handleUserInteraction(e: Event) {
  if (!hasUserInteracted.value) {
    e.preventDefault();
    e.stopPropagation();
    hasUserInteracted.value = true;
    showFirstUseMask.value = false;
    // 自动聚焦输入框
    setTimeout(() => {
      (window as any).triggerSearchFocus();
    }, 100);
  }
}

// 显示首次使用提示
function showFirstUsePrompt() {
  console.log('[Floating] showFirstUsePrompt');
  if (!hasUserInteracted.value) {
    showFirstUseMask.value = true;
  }
}
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
  
  const result = await Dialog({
    title: '确认批量删除',
    message,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  });
  
  if (result !== 'confirm') return;
  
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
  if (!selectedModeId.value) {
    Snackbar.warning('请先选择一个模式');
    return;
  }
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
async function showAddImageDialog() {
  if (!config.value || !selectedGroupId.value || !selectedModeId.value) {
    Snackbar.warning('请先选择分组');
    return;
  }
  // Android 端直接调用系统图片选择器
  if (isAndroidTauri()) {
    await uploadImagesAndroid();
    return;
  }
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
const shareApp = ref<string>('all');  // 支持自定义包名
const gridColumns = ref<number>(5);
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

const swipeStartX = ref(0);
const swipeStartY = ref(0);
const swipeOffset = ref(0);
const isSwiping = ref(false);

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
const showCustomAppsPopup = ref(false);
const customShareApps = ref<{ id: number; package_name: string; app_name: string | null }[]>([]);

// 用于强制 ThemeProvider 重新渲染的 key
const themeKey = ref(0);

// 全局分享应用选择回调
if (typeof window !== 'undefined') {
  (window as any).onCustomAppSelected = async (packageName: string, appName: string) => {
    try {
      await invoke('add_custom_share_app', { packageName, appName });
      Snackbar.success(`已保存应用: ${appName || packageName}`);
      await loadCustomApps();
      // 自动切换到该应用
      shareApp.value = packageName;
      handleShareAppChange(packageName);
    } catch (error) {
      console.error('Failed to save custom app:', error);
      Snackbar.error('保存应用失败');
    }
  };
}

const isMobile = computed(() => window.innerWidth < 768);
const isSidebarMode = computed(() => !isMobile.value);

/**
 * 创建类型安全的配置对象
 * 确保 Config 中所有字段类型正确，防止 UI 组件返回非预期类型
 */
function createSafeConfig(cfg: Config): Config {
  return {
    meme_dir: String(cfg.meme_dir || ''),
    color_mode: typeof cfg.color_mode === 'string' ? cfg.color_mode : 'system',
    theme_style: typeof cfg.theme_style === 'string' ? cfg.theme_style : 'modern',
    last_mode: Number(cfg.last_mode) || 1,
    last_group: Number(cfg.last_group) || 1,
    share_app: typeof cfg.share_app === 'string' ? cfg.share_app : '',
    grid_size: Number(cfg.grid_size) || 4,
    pinyin_search: Boolean(cfg.pinyin_search),
    acronym_search: Boolean(cfg.acronym_search),
    global_floating_window: Boolean(cfg.global_floating_window),
  };
}

/**
 * 安全更新配置到后端
 */
async function safeUpdateConfig(cfg: Config | null) {
  if (!cfg) return;
  const safeConfig = createSafeConfig(cfg);
  await invoke("update_config", { config: safeConfig });
}

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
  // 检查是否有待恢复的状态（目录变更后重启）
  const savedState = localStorage.getItem('meme_restore_state');
  if (savedState) {
    try {
      const state = JSON.parse(savedState);
      localStorage.removeItem('meme_restore_state');
      if (state.page) activeMenu.value = state.page;
      if (state.modeId) selectedModeId.value = state.modeId;
      if (state.groupId) selectedGroupId.value = state.groupId;
      console.log('[Restore] State restored:', state);
    } catch (e) {
      console.error('[Restore] Failed to parse saved state:', e);
    }
  }

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
  
  // 初始化完成后，滚动到选中的模式和分组
  await nextTick();
  
  // 使用 setTimeout 确保DOM完全渲染后再执行滚动
  setTimeout(() => {
    // 滚动到选中的模式
    const modeContainer = document.querySelector('.modern-tabs-container:not(.secondary) .modern-tabs-scroll') as HTMLElement;
    const modeTabs = document.querySelectorAll('.modern-tabs-container:not(.secondary) .modern-tab');
    
    if (modeContainer && modeTabs.length > 0) {
      let activeIndex = -1;
      modeTabs.forEach((tab, index) => {
        if (tab.classList.contains('active')) {
          activeIndex = index;
        }
      });
      
      if (activeIndex >= 0) {
        const activeTab = modeTabs[activeIndex] as HTMLElement;
        const tabLeft = activeTab.offsetLeft;
        const tabWidth = activeTab.offsetWidth;
        const containerWidth = modeContainer.offsetWidth;
        
        // 计算滚动位置，使选中的tab居中
        const targetScroll = tabLeft - (containerWidth - tabWidth) / 2;
        modeContainer.scrollLeft = Math.max(0, targetScroll);
      }
    }
    
    // 滚动到选中的分组
    const groupContainer = document.querySelector('.modern-tabs-container.secondary .modern-tabs-scroll') as HTMLElement;
    const groupTabs = document.querySelectorAll('.modern-tabs-container.secondary .modern-tab');
    
    if (groupContainer && groupTabs.length > 0) {
      let activeIndex = -1;
      groupTabs.forEach((tab, index) => {
        if (tab.classList.contains('active')) {
          activeIndex = index;
        }
      });
      
      if (activeIndex >= 0) {
        const activeTab = groupTabs[activeIndex] as HTMLElement;
        const tabLeft = activeTab.offsetLeft;
        const tabWidth = activeTab.offsetWidth;
        const containerWidth = groupContainer.offsetWidth;
        
        // 计算滚动位置，使选中的tab居中
        const targetScroll = tabLeft - (containerWidth - tabWidth) / 2;
        groupContainer.scrollLeft = Math.max(0, targetScroll);
      }
    }
  }, 300);
  
  // 添加事件监听
  window.addEventListener('resize', handleResize);
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('wheel', handleWheel, { passive: false });
  window.addEventListener('touchstart', handleTouchStart, { passive: false });
  window.addEventListener('touchmove', handleTouchMove, { passive: false });
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', async () => {
    console.log('[Theme] 系统深色模式切换');
    await syncSystemTheme(); // 直接调用抽出来的函数
  });
  
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

  // 监听颜色模式变化（从 Settings 页面触发）
  window.addEventListener('colorModeChanged', (event: any) => {
    const mode = event.detail;
    if (mode) {
      currentColorMode.value = mode;
      applyTheme();
    }
  });

  // 监听应用恢复前台事件，自动同步系统颜色模式
  document.addEventListener('visibilitychange', async () => {
    if (document.visibilityState === 'visible') {
      // 应用恢复到前台
      console.log('[Theme] App resumed to foreground');
      await syncSystemTheme(); 
      handleAppResume();
    }
  });

  // 添加悬浮窗触发搜索的全局方法
  (window as any).triggerSearchFocus = () => {
    console.log('[Floating] Triggering search focus');

    // 显示首次使用提示
    showFirstUsePrompt();
    
    // 清空搜索词
    searchKeyword.value = '';
    
    // 立即聚焦搜索框
    if (searchInputRef.value) {
      console.log('[Floating] Using Vue ref to focus');
      
      // Varlet Input 组件的聚焦方法
      if (typeof searchInputRef.value.focus === 'function') {
        searchInputRef.value.focus();
        console.log('[Floating] Called ref.focus()');
      }
      
      // 尝试获取内部 input 元素
      const component = searchInputRef.value;
      let nativeInput: HTMLInputElement | null = null;
      
      // 尝试多种方式获取原生 input
      if (component && component.$el) {
        nativeInput = component.$el.querySelector('input');
      }
      if (!nativeInput && component && component.el) {
        nativeInput = component.el.querySelector('input');
      }
      if (!nativeInput) {
        nativeInput = document.querySelector('.search-input input, .search-input [role="textbox"]');
      }
      
      if (nativeInput) {
        console.log('[Floating] Found native input element');
        
        // 强制聚焦
        nativeInput.focus({ preventScroll: false });
      } else {
        console.warn('[Floating] Native input not found via ref');
      }
    } else {
      console.warn('[Floating] searchInputRef is null');
    }
  };

  // 监听原生端触发的搜索聚焦事件
  window.addEventListener('triggerSearchFocus', () => {
    console.log('[Floating] Received triggerSearchFocus event');
    (window as any).triggerSearchFocus();
    // 显示首次使用提示
    showFirstUsePrompt();
  });

  // 监听应用恢复前台事件
  function handleAppResume() {
    if (document.visibilityState === 'visible') {
      console.log('[App] App resumed to foreground');
      // 桌面端恢复前台后调用triggerSearchFocus
      if (!isAndroidTauri()) {
        (window as any).triggerSearchFocus();
      }
    }
  }

  // 监听用户交互
  document.addEventListener('touchstart', handleUserInteraction, { passive: false });
  document.addEventListener('click', handleUserInteraction);

  // 监听全局悬浮窗状态变化
  window.addEventListener('globalFloatingWindowChanged', ((e: CustomEvent) => {
    globalFloatingWindowEnabled.value = e.detail;
    console.log('[Floating] Global floating window state changed:', e.detail);
  }) as EventListener);

  // 清理事件监听
  onUnmounted(() => {
    document.removeEventListener('touchstart', handleUserInteraction);
    document.removeEventListener('click', handleUserInteraction);
    document.removeEventListener('visibilitychange', handleAppResume);
  });

  async function syncSystemTheme() {
    // 只有当前是【跟随系统模式】才执行同步
    if (currentColorMode.value !== 'system' || !config.value) return;

    // 判断系统当前是否深色
    const isSystemDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
    // 判断当前页面是否深色
    const isCurrentlyDark = document.documentElement.classList.contains('var-dark');

    // 不一致 → 重新同步
    if (isSystemDark !== isCurrentlyDark) {
      console.log('[Theme] 系统主题变化，正在同步...');
      
      applyTheme();          // 应用主题到 DOM
      themeKey.value++;      // 强制 ThemeProvider 更新
      console.log('[Theme] 主题同步完成');
    }
  }

  // Android 返回键处理
  if (isAndroidTauri()) {
    window.addEventListener('tauri-android-back', (event: Event) => {
      // 如果在编辑模式，先退出编辑模式
      if (isGlobalEditMode.value) {
        exitGlobalEditMode();
        event.preventDefault?.();
        return;
      }
      // 如果在搜索框，先退出搜索框
      if (searchInputRef.value) {
        searchInputRef.value.blur();
        event.preventDefault?.();
        return;
      }

      // 如果在子页面，返回主页
      if (activeMenu.value !== 'home') {
        activeMenu.value = 'home';
        // 阻止默认退出行为
        event.preventDefault?.();
      } else {
        // 在主页时，最小化应用到后台（而不是退出）
        if (typeof (window as any).AndroidNative !== 'undefined' && (window as any).AndroidNative.minimizeApp) {
          console.log('[Android] Minimizing app to background');
          (window as any).AndroidNative.minimizeApp();
        } else {
          console.warn('[Android] minimizeApp not available, will exit app');
        }
        // 阻止默认退出行为
        event.preventDefault?.();
      }
    });

    // 检查 AndroidNative 接口是否可用
    setTimeout(() => {
      if (typeof (window as any).AndroidNative === 'undefined') {
        console.warn('[Android] AndroidNative interface not ready, waiting...');
        // 再等待一下
        setTimeout(() => {
          if (typeof (window as any).AndroidNative === 'undefined') {
            console.error('[Android] AndroidNative interface still not available after waiting');
          } else {
            console.log('[Android] AndroidNative interface is now available');
          }
        }, 2000);
      } else {
        console.log('[Android] AndroidNative interface is available');
      }
    }, 1000);
  }
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
      
      // 分享目标app：只在配置为空字符串（首次启动）时默认所有应用，其他情况使用用户的选择
      // null/undefined 视为首次启动，空字符串也视为首次启动
      const savedShareApp = config.value.share_app;
      if (!savedShareApp) {
        // 首次启动，默认为"所有应用"(显示系统分享菜单)并保存
        shareApp.value = 'all';
        config.value.share_app = '';  // 保存到配置时为空字符串
        await safeUpdateConfig(config.value);
      } else {
        // 使用用户之前的选择（可能是 wechat、qq 或自定义包名）
        shareApp.value = savedShareApp;
      }
      
      gridColumns.value = config.value.grid_size || 4;
      currentColorMode.value = config.value.color_mode as 'system' | 'light' | 'dark';
      pinyinSearchEnabled.value = config.value.pinyin_search || false;
      acronymSearchEnabled.value = config.value.acronym_search || false;
      
      // Android 平台：从原生服务同步悬浮窗实际状态，并在需要时自动启动服务
      if (/Android/i.test(navigator.userAgent)) {
        try {
          const nativeEnabled = (window as any).AndroidNative?.isFloatingWindowEnabled?.();
          const shouldBeEnabled = config.value.global_floating_window === true;
          
          if (shouldBeEnabled && nativeEnabled !== true) {
            // 配置中开启了但服务未运行，自动启动
            console.log('[Floating] Config says enabled but service not running, starting...');
            (window as any).AndroidNative?.startFloatingWindow?.();
            globalFloatingWindowEnabled.value = true;
          } else {
            globalFloatingWindowEnabled.value = nativeEnabled === true;
          }
          console.log('[Floating] Synced from native service:', nativeEnabled, 'config:', shouldBeEnabled);
        } catch (e) {
          // 如果无法获取原生状态，使用配置中的值
          globalFloatingWindowEnabled.value = config.value.global_floating_window || false;
          console.warn('[Floating] Failed to get native status, using config value:', e);
        }
      } else {
        globalFloatingWindowEnabled.value = config.value.global_floating_window || false;
      }
      
      // 加载自定义分享应用
      await loadCustomApps();
      
      applyTheme();
      applyTheme();
    } else {
      // 首次启动，需要选择目录
      await setupInitialConfig();
      // 注意：setupInitialConfig 已经完成了自动刷新和数据加载
      // 所以这里直接返回，不再执行后续的 loadModes 等
      return;
    }
  } catch (error) {
    console.error("Failed to load config:", error);
    await setupInitialConfig();
    // 注意：setupInitialConfig 已经完成了自动刷新和数据加载
    // 所以这里直接返回，不再执行后续的 loadModes 等
    return;
  }
}

async function loadCustomApps() {
  try {
    customShareApps.value = await invoke<any[]>("get_custom_share_apps") || [];
    
    // 同步到 SharedPreferences（确保悬浮窗服务能读取到最新数据）
    if (isAndroidTauri() && typeof (window as any).AndroidNative?.syncCustomAppsToPrefs === 'function') {
      const packages = customShareApps.value.map(app => app.package_name);
      (window as any).AndroidNative.syncCustomAppsToPrefs(JSON.stringify(packages));
      console.log(`Synced ${packages.length} apps to SharedPreferences`);
    }
  } catch (error) {
    console.error("Failed to load custom apps:", error);
  }
}

async function removeCustomApp(id: number) {
  try {
    await invoke("remove_custom_share_app", { id });
    Snackbar.success("已移除应用");
    await loadCustomApps();
    
    // 同步到 SharedPreferences
    if (isAndroidTauri() && typeof (window as any).AndroidNative?.syncCustomAppsToPrefs === 'function') {
      const packages = customShareApps.value.map(app => app.package_name);
      (window as any).AndroidNative.syncCustomAppsToPrefs(JSON.stringify(packages));
    }
  } catch (error) {
    console.error("Failed to remove custom app:", error);
    Snackbar.error("移除失败");
  }
}

function handlePickShareApp() {
  if (isAndroidTauri() && typeof (window as any).AndroidNative?.pickShareApp === 'function') {
    (window as any).AndroidNative.pickShareApp();
  } else {
    Snackbar.warning("仅支持 Android 平台");
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
      share_app: "",
      grid_size: 4,
      pinyin_search: false,
      acronym_search: false,
      global_floating_window: false
    };
    config.value = newConfig;
    await safeUpdateConfig(config.value);
    applyTheme();
    
    // 首次启动，自动刷新数据到数据库并加载
    await autoRefreshAfterDirChange(selectedDir);
  } else {
    // 用户取消了选择，使用默认目录
    const defaultMemeDir = isAndroidTauri()
      ? '/storage/emulated/0/meme'
      : '/home/' + (navigator.userAgent.includes('Linux') ? 'user' : '') + '/meme';
    const defaultConfig: Config = {
      meme_dir: defaultMemeDir,
      color_mode: "system",
      theme_style: "modern",
      last_mode: 1,
      last_group: 1,
      share_app: "",
      grid_size: 4,
      pinyin_search: false,
      acronym_search: false,
      global_floating_window: false
    };
    config.value = defaultConfig;
    await safeUpdateConfig(config.value);
    applyTheme();
    
    // 使用默认目录时，也尝试刷新数据并加载
    await autoRefreshAfterDirChange(defaultMemeDir);
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
  
  // Android 端使用原生方式选择文件
  if (isAndroidTauri()) {
    await uploadImagesAndroid();
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

/**
 * Android 端上传图片
 * 
 * 标准流程：
 * 1. open() → 拿到 content:// URI
 * 2. 前端读取文件内容并转为 Base64
 * 3. invoke 传给 Rust 后端
 * 4. 后端解码 Base64 并保存
 * 5. 前端接收结果 → 更新 UI
 */
async function uploadImagesAndroid() {
	console.log('[Android] uploadImagesAndroid 开始');
	console.log('[Android] config:', config.value);
	console.log('[Android] selectedGroupId:', selectedGroupId.value);
	console.log('[Android] selectedModeId:', selectedModeId.value);
	
	if (!config.value || !selectedGroupId.value || !selectedModeId.value) {
		Snackbar.warning('请先选择分组');
		return;
	}
	
	try {
		// 步骤1：打开系统图片选择器，获取 content:// URI
		console.log('[Android] 打开图片选择器...');
		const { open } = await import("@tauri-apps/plugin-dialog");
		
		const selected = await open({
			multiple: true,
			filters: [{
				name: "Images",
				extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"]
			}]
		});
		
		console.log('[Android] 选择器返回结果:', selected);
		
		if (!selected || (Array.isArray(selected) && selected.length === 0)) {
			console.log('[Android] 未选择文件');
			return;
		}
		
		// 统一转为数组
		const uris: string[] = Array.isArray(selected) ? selected : [selected];
		console.log('[Android] 待上传 URI 列表:', uris);
		
		// 步骤2：读取文件内容并转为 Base64
		console.log('[Android] 读取文件内容...');
		const { readFile } = await import("@tauri-apps/plugin-fs");
		
		const imagesData: Array<{ name: string; data: string }> = [];
		
		for (const uri of uris) {
			try {
				// 读取文件内容
				const fileData = await readFile(uri);
				
				// 转换为 Base64
				const base64 = arrayBufferToBase64(fileData);
				
				// 提取文件名
				const name = uri.split('/').pop() || 'image.png';
				
				imagesData.push({ name, data: base64 });
				console.log('[Android] 读取文件成功:', name, '大小:', fileData.length);
			} catch (err) {
				console.error('[Android] 读取文件失败:', uri, err);
			}
		}
		
		if (imagesData.length === 0) {
			Snackbar.warning('没有成功读取任何文件');
			return;
		}
		
		// 步骤3：调用后端上传接口
		console.log('[Android] 调用后端 upload_images_android...');
		const successCount = await invoke<number>("upload_images_android", {
			imagesData: imagesData,
			groupId: selectedGroupId.value,
			modeId: selectedModeId.value
		});
		
		console.log('[Android] 上传完成，成功数量:', successCount);
		
		// 步骤4：刷新图片列表
		await loadImages(selectedGroupId.value);
		
		Snackbar.success(`成功上传 ${successCount} 张图片`);
		console.log('[Android] uploadImagesAndroid 完成');
	} catch (error) {
		console.error("[Android] 上传失败:", error);
		Snackbar.error('上传失败: ' + error);
	}
}

/**
 * 将 ArrayBuffer 转换为 Base64 字符串
 */
function arrayBufferToBase64(buffer: Uint8Array): string {
	let binary = '';
	const len = buffer.byteLength;
	for (let i = 0; i < len; i++) {
		binary += String.fromCharCode(buffer[i]);
	}
	return btoa(binary);
}

async function deleteSelectedImages() {
  if (selectedImages.value.length === 0) return;
  
  const result = await Dialog({
    title: '确认删除',
    message: `确定要删除选中的 ${selectedImages.value.length} 张表情包吗？`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '确定',
    cancelButtonText: '取消'
  });
  
  if (result !== 'confirm') return;
  
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
  shareApp.value = app;
  if (config.value) {
    // 保存到配置时，'all' 转换为空字符串，自定义包名直接保存
    const shareAppValue = app === 'all' ? '' : app;
    config.value.share_app = shareAppValue;
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
      safeUpdateConfig(config.value);
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
      safeUpdateConfig(config.value);
    }
  }
}

function handleSwipeStart(event: TouchEvent) {
  if (isGlobalEditMode.value || event.touches.length !== 1) return;
  swipeStartX.value = event.touches[0].clientX;
  swipeStartY.value = event.touches[0].clientY;
  swipeOffset.value = 0;
  isSwiping.value = true;
}

function handleSwipeMove(event: TouchEvent) {
  if (!isSwiping.value || isGlobalEditMode.value) return;
  if (event.touches.length !== 1) {
    isSwiping.value = false;
    swipeOffset.value = 0;
    return;
  }

  const deltaX = event.touches[0].clientX - swipeStartX.value;
  const deltaY = event.touches[0].clientY - swipeStartY.value;

  if (Math.abs(deltaY) > Math.abs(deltaX) && Math.abs(deltaY) > 10) {
    isSwiping.value = false;
    swipeOffset.value = 0;
    return;
  }

  if (Math.abs(deltaX) > 10) {
    event.preventDefault();
    swipeOffset.value = deltaX;
  }
}

function handleSwipeEnd() {
  if (!isSwiping.value) return;

  const threshold = 80;
  if (Math.abs(swipeOffset.value) < threshold) {
    isSwiping.value = false;
    swipeOffset.value = 0;
    return;
  }

  const currentIndex = groups.value.findIndex(g => g.id === selectedGroupId.value);
  if (currentIndex === -1) {
    isSwiping.value = false;
    swipeOffset.value = 0;
    return;
  }

  if (swipeOffset.value < -threshold && currentIndex < groups.value.length - 1) {
    const nextGroup = groups.value[currentIndex + 1];
    performSwipeTransition('left', () => switchGroup(nextGroup.id));
  } else if (swipeOffset.value > threshold && currentIndex > 0) {
    const prevGroup = groups.value[currentIndex - 1];
    performSwipeTransition('right', () => switchGroup(prevGroup.id));
  } else {
    isSwiping.value = false;
    swipeOffset.value = 0;
  }
}

async function performSwipeTransition(_direction: 'left' | 'right', onComplete: () => Promise<void> | void) {
  await onComplete();
  swipeOffset.value = 0;
  await nextTick();
  await new Promise<void>(resolve => requestAnimationFrame(() => resolve()));
  isSwiping.value = false;
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
    console.log('Full refresh result:', result);
    await reloadPageState();
    Snackbar.success('数据刷新完成');
  } catch (error) {
    console.error("Failed to refresh:", error);
    try { await reloadPageState(); } catch (e) { /* ignore */ }
    Snackbar.error('数据刷新失败: ' + error);
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

/**
 * 目录变更后自动刷新数据到数据库
 * @param newDir 新的表情包目录路径
 */
async function autoRefreshAfterDirChange(newDir: string) {
  try {
    console.log('[Auto Refresh] Checking directory:', newDir);
    
    // 检查存储是否可访问
    const accessible = await invoke<boolean>('check_storage_accessible', { memeDir: newDir });
    if (!accessible) {
      console.log('[Auto Refresh] Storage not accessible, skipping');
      return;
    }
    
    // 执行全量刷新，将文件系统数据同步到数据库
    Snackbar.info('正在初始化数据...');
    const result = await invoke<string>("full_refresh", { memeDir: newDir });
    console.log('[Auto Refresh] Result:', result);
    
    // 刷新完成后重新加载页面状态
    await reloadPageState();
    Snackbar.success('数据初始化完成');
  } catch (error) {
    console.error('[Auto Refresh] Failed:', error);
    // 即使刷新失败，也尝试加载已有数据
    try {
      await reloadPageState();
    } catch (e) {
      console.error('[Auto Refresh] Reload also failed:', e);
    }
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
      fullRefresh();
      break;
    case 'reload':
      location.reload();
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
  const result = await Dialog({
    title: '确认删除',
    message: `确定要删除分组 "${group.name}" 吗？此操作不可恢复！`,
    confirmButton: true,
    cancelButton: true,
    confirmButtonText: '删除',
    cancelButtonText: '取消'
  });
  
  if (result !== 'confirm') return;
  
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
    safeUpdateConfig(config.value);
  }
}

function toggleAcronymSearch() {
  acronymSearchEnabled.value = !acronymSearchEnabled.value;
  if (config.value) {
    config.value.acronym_search = acronymSearchEnabled.value;
    safeUpdateConfig(config.value);
  }
}

// ========== 长按/右键菜单相关函数 ==========

// 模式菜单配置
const modeMenuItems = [
  { label: '编辑', value: 'rename', icon: 'pencil' },
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
    case 'delete': {
      const result = await Dialog({
        title: '确认删除',
        message: `确定要删除模式 "${mode.name}" 吗？相关文件将移动到回收站。`,
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      });
      
      if (result !== 'confirm') return;
      
      try {
        await invoke('delete_mode', { modeId: mode.id });
        Snackbar.success('模式已删除');
        await loadModes();
      } catch (error) {
        console.error('Failed to delete mode:', error);
        Snackbar.error('删除模式失败');
      }
      break;
    }
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
    case 'delete': {
      const result = await Dialog({
        title: '确认删除',
        message: `确定要删除分组 "${group.name}" 吗？相关文件将移动到回收站。`,
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      });
      
      if (result !== 'confirm') return;
      
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
      break;
    }
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

// 处理图片点击：Android 端分享，桌面端复制
async function handleImageClick(img: Image) {
  if (isAndroidTauri()) {
    // Android 端：直接分享到当前选中的 app
    await shareImageToApp(img);
  } else {
    // 桌面端：复制到剪贴板
    await copySingleImage(img);
  }
}

// 复制单张图片（桌面端）
async function copySingleImage(img: Image) {
  try {
    await invoke("copy_images", { imageIds: [img.id] });
    Snackbar.success('已复制到剪贴板');
  } catch (error) {
    console.error("Failed to copy image:", error);
    Snackbar.error('复制失败');
  }
}

// 分享图片到 App（Android 端）
async function shareImageToApp(img: Image) {
  try {
    const imagePath = img.image_path;
    const app = shareApp.value;
    console.log(`[Android] 分享图片到 ${app}:`, imagePath);
    
    // 如果选择的是"所有应用"或未指定特定应用，传递空字符串以显示系统分享菜单
    const targetApp = (app === 'all' || !app) ? '' : app;

    if (isAndroidTauri()) {
      // 检查 AndroidNative 接口是否可用，带重试机制
      let androidNative = (window as any).AndroidNative;
      
      if (!androidNative || !androidNative.shareImageToApp) {
        console.warn('[Android] AndroidNative interface not immediately available, waiting...');
        
        // 等待最多 2 秒，每 100ms 检查一次
        for (let i = 0; i < 20; i++) {
          await new Promise(resolve => setTimeout(resolve, 100));
          androidNative = (window as any).AndroidNative;
          
          if (androidNative && androidNative.shareImageToApp) {
            console.log(`[Android] AndroidNative interface became available after ${i + 1} attempts`);
            break;
          }
        }
      }
      
      if (androidNative && androidNative.shareImageToApp) {
        console.log(`[Android] 调用原生分享接口:`, imagePath, targetApp);
        androidNative.shareImageToApp(imagePath, targetApp);
        await invoke("share_image", { imageId: img.id });
        
        if (!targetApp) {
          Snackbar.success('已打开系统分享菜单');
        } else {
          Snackbar.success(`正在分享到 ${app}`);
        }
      } else {
        console.error('[Android] AndroidNative interface still not available after retries');
        console.log('[Android] Window keys:', Object.keys(window).filter(k => k.includes('Android') || k.includes('android')));
        
        // 尝试使用 Tauri Share 插件作为备选方案
        try {
          console.log('[Android] Falling back to tauri-plugin-share');
          const { shareFile } = await import('tauri-plugin-share');
          const ext = imagePath.split('.').pop()?.toLowerCase() || 'png';
          const mimeMap: Record<string, string> = {
            png: 'image/png',
            jpg: 'image/jpeg',
            jpeg: 'image/jpeg',
            gif: 'image/gif',
            webp: 'image/webp',
            bmp: 'image/bmp',
          };
          const mime = mimeMap[ext] || 'image/png';
          await shareFile(imagePath, mime);
          await invoke("share_image", { imageId: img.id });
          Snackbar.success('已打开分享菜单');
        } catch (fallbackError) {
          console.error('[Android] Fallback also failed:', fallbackError);
          Snackbar.error('Android原生接口不可用，请重启应用');
        }
      }
    } else {
      // 桌面端使用 Tauri Share 插件
      const { shareFile } = await import('tauri-plugin-share');
      const ext = imagePath.split('.').pop()?.toLowerCase() || 'png';
      const mimeMap: Record<string, string> = {
        png: 'image/png',
        jpg: 'image/jpeg',
        jpeg: 'image/jpeg',
        gif: 'image/gif',
        webp: 'image/webp',
        bmp: 'image/bmp',
      };
      const mime = mimeMap[ext] || 'image/png';
      await shareFile(imagePath, mime);
      await invoke("share_image", { imageId: img.id });
      Snackbar.success('已打开分享菜单');
    }
  } catch (error) {
    console.error("Failed to share image:", error);
    Snackbar.error('分享失败: ' + error);
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
    case 'delete': {
      const result = await Dialog({
        title: '确认删除',
        message: '确定要删除这张表情包吗？',
        confirmButton: true,
        cancelButton: true,
        confirmButtonText: '删除',
        cancelButtonText: '取消'
      });
      
      if (result !== 'confirm') return;
      
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
      break;
    }
  }
}
</script>

<template>
  <ThemeProvider 
    :key="themeKey"
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
              ref="searchInputRef"
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
              <Icon name="image" :size="24" />
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
        </div>

        <teleport to="body">
          <div v-if="isMenuPopupOpen" class="menu-overlay" @click="isMenuPopupOpen = false">
            <div 
              class="menu-popup"
              :style="menuPopupStyle"
              @click.stop
            >
              <div class="menu-popup-content">
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
                  <span>刷新数据</span>
                </div>
                <div class="menu-item" @click="handleMenuAction('reload')">
                  <var-icon name="replay" size="18" />
                  <span>刷新页面</span>
                </div>
                <div class="menu-divider"></div>
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
              <!-- 添加模式按钮（编辑模式下或没有模式时显示） -->
              <div
                v-if="isGlobalEditMode || modes.length === 0"
                class="modern-tab add-tab"
                @click="showAddModeDialog"
              >
                <div class="add-tab-content">
                  <var-icon name="plus" size="18" />
                  <span class="add-tab-text">添加模式</span>
                </div>
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
                  <var-icon v-else name="checkbox-blank-outline" size="18" color="var(--color-text)" />
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
                    @click="isGlobalEditMode ? toggleModeSelection(mode.id) : switchMode(mode.id)"
                  >
                    {{ mode.name }}
                  </div>
                </ContextMenu>
              </div>
            </div>
          </div>
          <div v-if="isMobile && !isGlobalEditMode" class="share-app-icons">
            <div
              class="share-app-icon" :class="{ active: shareApp === 'qq' }"
              @click="shareApp = 'qq'; handleShareAppChange('qq')">
              <Icon name="qq" :size="shareApp === 'qq' ? 28 : 22" :fill="true"
                :color="shareApp === 'qq' ? 'var(--color-primary)' : 'var(--color-text-2)'" />
              <span class="indicator" v-if="shareApp === 'qq'" />
            </div>
            <div
              class="share-app-icon" :class="{ active: shareApp === 'wechat' }"
              @click="shareApp = 'wechat'; handleShareAppChange('wechat')">
              <Icon name="wechat" :size="shareApp === 'wechat' ? 28 : 22" :fill="true"
                :color="shareApp === 'wechat' ? 'var(--color-primary)' : 'var(--color-text-2)'" />
              <span class="indicator" v-if="shareApp === 'wechat'" />
            </div>
            <!-- 所有应用选项 -->
            <div
              class="share-app-icon" :class="{ active: shareApp === 'all' }"
              @click="shareApp = 'all'; handleShareAppChange('all')">
              <Icon name="apps-2" :size="shareApp === 'all' ? 28 : 22" :fill="false"
                :color="shareApp === 'all' ? 'var(--color-primary)' : 'var(--color-text-2)'" />
              <span class="indicator" v-if="shareApp === 'all'" />
            </div>
            <!-- 更多应用（九个点） -->
            <div
              class="share-app-icon" :class="{ active: customShareApps.some(a => a.package_name === shareApp) }"
              @click="showCustomAppsPopup = true">
              <Icon name="more" :size="shareApp !== 'wechat' && shareApp !== 'qq' && shareApp !== 'all' ? 28 : 22" :fill="false"
                :color="shareApp !== 'wechat' && shareApp !== 'qq' && shareApp !== 'all' ? 'var(--color-primary)' : 'var(--color-text-2)'" />
              <span class="indicator" v-if="customShareApps.some(a => a.package_name === shareApp)" />
            </div>
          </div>
        </div>

        <!-- 分组选择栏 -->
        <div class="modern-tabs-container secondary" :class="{ 'edit-mode': isGlobalEditMode }">
          <div class="modern-tabs-scroll">
            <div class="modern-tabs">
              <!-- 添加分组按钮（编辑模式下或没有分组时显示） -->
              <div
                v-if="isGlobalEditMode || groups.length === 0"
                class="modern-tab add-tab"
                @click="showAddGroupDialog"
              >
                <div class="add-tab-content">
                  <var-icon name="plus" size="18" />
                  <span class="add-tab-text">添加分组</span>
                </div>
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
                  <var-icon v-else name="checkbox-blank-outline" size="18" color="var(--color-text)" />
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
                    @click="isGlobalEditMode ? toggleGroupSelection(group.id) : switchGroup(group.id)"
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
          :style="{ 
            gridTemplateColumns: `repeat(${gridColumns}, 1fr)`,
            transform: `translateX(${swipeOffset}px)`,
            transition: isSwiping ? 'none' : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
          }"
          :class="{ 'edit-mode': isGlobalEditMode }"
          @touchstart.passive="handleSwipeStart"
          @touchmove="handleSwipeMove"
          @touchend="handleSwipeEnd"
        >
          <!-- 添加图片按钮（编辑模式下或没有图片时显示） -->
          <div
            v-if="isGlobalEditMode || images.length === 0"
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
          @click="isGlobalEditMode && toggleImageSelection(img.id)"
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
                color="white" 
              />
            </div>
            <ContextMenu
              v-if="!isGlobalEditMode"
              :items="imageMenuItems"
              @select="(action) => handleImageMenuSelect(img, action)"
            >
              <var-image
                :src="toAssetPath(img.thumbnail_path || img.image_path)"
                fit="cover"
                class="image-content"
                @click="handleImageClick(img)"
              />
            </ContextMenu>
            <var-image
              v-else
              :src="toAssetPath(img.thumbnail_path || img.image_path)"
              fit="cover"
              class="image-content"
            />
            <div v-if="selectedImages.includes(img.id) && !isGlobalEditMode" class="check-overlay">
              <var-icon name="check-circle" size="24" color="#fff" />
            </div>
          </div>
        </div>

        <!-- 模式编辑弹窗 -->
        <var-popup class="edit-mode-popup" :show="showModeEditPopup" @click-overlay="showModeEditPopup = false">
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
        <var-popup class="edit-mode-popup" :show="showGroupEditPopup" @click-overlay="showGroupEditPopup = false">
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
        <var-popup class="edit-mode-popup" :show="showAddModePopup" @click-overlay="showAddModePopup = false">
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
        <var-popup class="edit-mode-popup" :show="showAddGroupPopup" @click-overlay="showAddGroupPopup = false">
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
        <var-popup class="edit-mode-popup" :show="showAddImagePopup" @click-overlay="showAddImagePopup = false">
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
        
        <!-- 自定义分享应用管理弹窗 -->
        <var-popup class="custom-apps-popup" :show="showCustomAppsPopup" @click-overlay="showCustomAppsPopup = false">
          <div class="keyword-manager">
            <div class="keyword-manager-header">
              <h3>管理分享应用</h3>
              <button class="btn-icon" @click="showCustomAppsPopup = false">
                <Icon name="close" :size="24" />
              </button>
            </div>
            
            <div class="keyword-manager-body">
              <!-- 添加应用按钮 -->
              <!-- <div class="add-app-section">
                <var-button type="primary" block @click="handlePickShareApp">
                  <Icon name="add" :size="18" /> 选择新应用
                </var-button>
              </div> -->
              
              <!-- 应用列表 -->
              <div class="keywords-list">
                <div v-if="customShareApps.length === 0" class="empty-state">
                  <Icon name="apps-2" :size="32" />
                  <p>暂无自定义应用</p>
                </div>
                
                <div v-else class="keywords-chips">
                  <div
                    v-for="app in customShareApps"
                    :key="app.id"
                    :class="['keyword-chip', 'app-chip', { 'app-chip-selected': shareApp === app.package_name }]"
                    @click="shareApp = app.package_name; handleShareAppChange(app.package_name); showCustomAppsPopup = false"
                  >
                    <span>{{ app.app_name || app.package_name }}</span>
                    <var-button
                      text
                      round
                      size="mini"
                      class="remove-btn"
                      @click.stop="removeCustomApp(app.id)"
                    >
                      <Icon name="close" :size="14" />
                    </var-button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </var-popup>
        
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
        :visible="activeMenu === 'home' && !isGlobalEditMode && !globalFloatingWindowEnabled"
        @click="handleFloatingSearchClick"
      />
    </div>

    <!-- 首次使用提示蒙版 -->
    <div v-if="showFirstUseMask" class="first-use-mask" @click="handleUserInteraction" @touchstart="handleUserInteraction">
      <div class="first-use-card">
        <div class="first-use-icon">
          <Icon name="keyboard" color="#4A90E2" :size="48" />
        </div>
        <h2 class="first-use-title">首次使用提示</h2>
        <p class="first-use-description">
          (由于WebView限制)<br/>
          App启动后首次使用需要<br/>
          轻触屏幕以解锁自动能力<br/>
        </p>
        <div class="first-use-arrow">
          <Icon name="arrow-down" color="#999" :size="32" />
        </div>
      </div>
    </div>
  </ThemeProvider>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background-color: var(--color-body);
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
  padding-top: max(12px, env(safe-area-inset-top));
  background-color: var(--color-surface);
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
  background-color: var(--color-surface-variant);
}

.search-actions {
  display: flex;
  gap: 8px;
}

/* 现代化 Tabs */
.modern-tabs-container {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface);
  padding: 8px 12px;
  /* 禁用长按菜单和文本选择 */
  -webkit-touch-callout: none;
  user-select: none;
}

.modern-tabs-container.secondary {
  background-color: var(--color-surface);
}

.modern-tabs-scroll {
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  white-space: nowrap;
  scrollbar-width: none;
  -ms-overflow-style: none;
  -webkit-overflow-scrolling: touch;
  /* 优化滚动体验 */
  touch-action: pan-x;
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
  color: var(--color-text);
  background-color: transparent;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  white-space: nowrap;
}

.modern-tab:hover {
  background-color: var(--color-surface-variant);
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

/* 添加按钮内容布局 */
.add-tab-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
}

.add-tab-text {
  font-size: 10px;
  font-weight: 500;
  line-height: 1.2;
  white-space: nowrap;
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
  background-color: var(--color-surface-variant);
}

.select-checkbox.is-checked {
  background-color: var(--color-primary-light);
}

/* 分享目标 app 图标 */
.share-app-icons {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  padding-left: 4px;
  border-left: 1px solid var(--color-border);
}

.share-app-icon {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s, transform 0.2s;
}

.share-app-icon:hover {
  background: var(--color-surface-variant);
}

.share-app-icon.active {
  transform: scale(1.15);
}

.share-app-icon .indicator {
  position: absolute;
  bottom: -1px;
  left: 50%;
  transform: translateX(-50%);
  width: 12px;
  height: 3px;
  border-radius: 2px;
  background: var(--color-primary);
}

/* 当前模式/分组显示 */
.current-mode-display,
.current-group-display {
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
.current-mode-display,
.current-group-display {
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
  background-color: var(--color-body);
  display: grid;
  gap: 12px;
  align-content: start;
  -webkit-touch-callout: none;
  user-select: none;
  touch-action: pan-y pan-x;
  will-change: transform;
}

.image-grid.is-swiping {
  touch-action: none;
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
  /* 禁用默认的长按行为 */
  -webkit-touch-callout: none;
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
  background-color: var(--color-surface-variant);
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
  background-color: var(--color-surface-variant);
  border-radius: 16px;
  margin: 20px 0;
}
.empty-state {
  grid-column: 1 / -1;
  text-align: center;
  padding: 60px 20px;
  background-color: var(--color-surface-variant);
  border-radius: 16px;
  margin: 20px 0;
}

.edit-actions {
  padding: 12px;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-surface);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.08);
}
.edit-actions {
  padding: 12px;
  border-top: 1px solid var(--color-border);
  background-color: var(--color-surface);
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
    padding-top: max(8px, env(safe-area-inset-top));
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
  background: var(--color-surface-variant);
  border-radius: 3px;
}
.image-grid::-webkit-scrollbar-track {
  background: var(--color-surface-variant);
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
  background-color: var(--color-surface);
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
  background-color: var(--color-surface-variant);
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
  background-color: var(--color-surface);
  border-radius: 20px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  animation: menuAppear 0.2s ease;
}
.group-action-menu {
  position: relative;
  z-index: 10000;
  width: 280px;
  max-width: 90vw;
  background-color: var(--color-surface);
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
  background-color: var(--color-surface-variant);
}
.group-action-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface-variant);
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
  background-color: var(--color-surface-variant);
}
.group-action-item:hover {
  background-color: var(--color-surface-variant);
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
  background-color: var(--color-surface);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
}

.edit-popup-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface-variant);
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

/* 覆盖 var-popup 默认样式 - 针对编辑弹窗 */
.edit-mode-popup :deep(.var-popup__overlay) {
  background-color: rgba(0, 0, 0, 0.5) !important;
}

.edit-mode-popup :deep(.var-popup__content) {
  border-radius: 16px !important;
  box-shadow: none !important;
  overflow: hidden !important;
}

.edit-mode-popup :deep(.var-popup) {
  border-radius: 16px !important;
  overflow: hidden !important;
}

/* 全局覆盖所有 var-popup 使用主题背景色 */
:global(body .var-popup) {
  background-color: var(--color-surface) !important;
}

:global(body .var-popup--center) {
  background-color: var(--color-surface) !important;
}

:global(body .var-popup__content) {
  background-color: var(--color-surface) !important;
}

/* 确保 overlay 有正确的蒙版 */
:global(.var-popup__overlay) {
  background-color: rgba(0, 0, 0, 0.5) !important;
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

/* 修复移动端按钮点击后高亮状态不自动取消的问题 */

/* 首次使用提示蒙版 */
.first-use-mask {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(8px);
  animation: fadeIn 0.3s ease-in-out;
  pointer-events: auto;
}

.first-use-card {
  background-color: #fff;
  border-radius: 24px;
  padding: 48px 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  text-align: center;
  max-width: 360px;
  animation: slideUp 0.5s ease-out;
}

.first-use-icon {
  margin-bottom: 24px;
  padding: 16px;
  background-color: rgba(74, 144, 226, 0.1);
  border-radius: 50%;
  display: inline-block;
}

.first-use-title {
  font-size: 24px;
  font-weight: 600;
  color: #333;
  margin-bottom: 16px;
}

.first-use-description {
  font-size: 16px;
  color: #666;
  line-height: 1.6;
  margin-bottom: 32px;
}

.first-use-arrow {
  animation: bounce 2s infinite;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    transform: translateY(50px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes bounce {
  0%, 20%, 50%, 80%, 100% {
    transform: translateY(0);
  }
  40% {
    transform: translateY(-10px);
  }
  60% {
    transform: translateY(-5px);
  }
}

/* 深色模式适配 */
:global(.dark-mode) .first-use-card {
  background-color: #2c2c2c;
}

:global(.dark-mode) .first-use-title {
  color: #fff;
}

:global(.dark-mode) .first-use-description {
  color: #aaa;
}

:global(.dark-mode) .first-use-arrow {
  color: #666;
}
@media (hover: none) {
  /* 搜索栏按钮 */
  .btn-icon {
    -webkit-tap-highlight-color: transparent;
    transition: transform 0.1s ease, background-color 0.1s ease;
  }
  
  .btn-icon:active {
    transform: scale(0.92);
    background-color: var(--color-primary-light);
  }
  
  /* 确保点击后不会保持高亮 */
  .btn-icon:not(:active) {
    background-color: transparent;
  }
  
  /* 添加图片按钮 */
  .image-item.add-image-item {
    -webkit-tap-highlight-color: transparent;
    transition: all 0.2s ease;
  }
  
  .image-item.add-image-item:active {
    transform: scale(0.95);
    background-color: var(--color-primary);
  }
  
  .image-item.add-image-item:not(:active) {
    background-color: var(--color-primary-light);
  }
  
  /* 添加模式/分组按钮 */
  .modern-tab.add-tab {
    -webkit-tap-highlight-color: transparent;
    transition: all 0.2s ease;
  }
  
  .modern-tab.add-tab:active {
    transform: scale(0.95);
    background-color: var(--color-primary);
    color: white;
  }
}

/* 桌面端保持原有hover效果 */
@media (hover: hover) {
  .btn-icon:hover {
    color: var(--color-primary);
    background-color: var(--color-primary-light);
  }
}

/* 移除所有按钮的focus outline */
.btn-icon:focus,
.btn-icon:focus-visible {
  outline: none;
  -webkit-tap-highlight-color: transparent;
}

.image-item.add-image-item:hover .add-image-content span {
  color: white;
}

/* 自定义分享应用弹窗样式 */
.custom-apps-popup :deep(.var-popup) {
  border-radius: 16px !important;
  overflow: hidden !important;
}

/* 强制设置背景色，优先级最高 */
.custom-apps-popup :deep(.var-popup),
.custom-apps-popup :deep(.var-popup__content),
.custom-apps-popup .keyword-manager {
  background-color: var(--color-surface) !important;
}

/* 给内层容器也添加背景色 */
.custom-apps-popup .keyword-manager {
  background-color: var(--color-surface);
  min-width: 320px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.custom-apps-popup .keyword-manager-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--color-surface);
}

.custom-apps-popup .keyword-manager-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.custom-apps-popup .keyword-manager-body {
  padding: 20px;
  overflow-y: auto;
  background-color: var(--color-surface);
}

.app-chip {
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.app-chip:hover {
  background-color: var(--color-primary);
  color: white;
  transform: translateY(-1px);
}

/* 选中的应用高亮显示 */
.app-chip-selected {
  background-color: var(--color-primary) !important;
  color: white !important;
  box-shadow: 0 2px 8px rgba(var(--color-primary-rgb), 0.3);
  border: 2px solid var(--color-primary);
}

.app-chip-selected::before {
  content: '✓';
  position: absolute;
  top: -4px;
  right: -4px;
  width: 18px;
  height: 18px;
  background-color: var(--color-success);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: bold;
}

.add-app-section {
  margin-bottom: 16px;
}
</style>
