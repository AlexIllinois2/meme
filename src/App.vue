<script setup lang="ts">
import { ref, onMounted, computed, nextTick, onUnmounted, defineAsyncComponent } from "vue";
import { Snackbar, Dialog } from '@varlet/ui';
import { isTauri, isAndroidTauri, toAssetPath, invoke } from './utils/tauri';
import { useConfig } from './composables/useConfig';
import { useMemeData } from './composables/useMemeData';
import ThemeProvider from "./components/ThemeProvider.vue";
import FloatingSearchButton from "./components/FloatingSearchButton.vue";
import Icon from "./components/Icon.vue";

// 延迟加载的非首屏组件
const Settings = defineAsyncComponent(() => import("./views/Settings.vue"));
const Support = defineAsyncComponent(() => import("./views/Support.vue"));
const UserAgreement = defineAsyncComponent(() => import("./views/UserAgreement.vue"));
const PrivacyPolicy = defineAsyncComponent(() => import("./views/PrivacyPolicy.vue"));
const ContextMenu = defineAsyncComponent(() => import("./components/ContextMenu.vue"));
const KeywordManager = defineAsyncComponent(() => import("./components/KeywordManager.vue"));
import type { Mode, Group, Image } from "./types";
import { debug } from './utils/debug';
import { useVisibility } from './composables/useVisibility';

// 窗口非活动时隐藏图片区域（停止 CSS 动画 + GIF 解码）
const { isActive } = useVisibility();

// 窗口焦点回调（供 onMounted/onUnmounted 共用）
const handleWindowFocus = () => {
  if (!isAndroidTauri()) {
    handleAppResume();
  }
};

const { config, currentColorMode, gridColumns, pinyinSearchEnabled, acronymSearchEnabled, globalFloatingWindowEnabled, themeKey, safeUpdateConfig, loadConfig, applyTheme, syncSystemTheme, handleResize, handleWheel, handleTouchStart, handleTouchMove, isMobile } = useConfig(invoke);

const { modes, groups, images, selectedModeId, selectedGroupId, loadModes, loadGroups, loadImages, switchMode, switchGroup, fullRefresh } = useMemeData(config, safeUpdateConfig);

import { useSearch } from './composables/useSearch';

const { searchKeyword, searchInputRef, searchImages, handleFloatingSearchClick } = useSearch(invoke, {
  selectedModeId,
  pinyinSearchEnabled,
  acronymSearchEnabled,
  groups,
  images,
  selectedGroupId,
  loadImages,
});

import { useEditMode } from './composables/useEditMode';

const { selectedImages, isGlobalEditMode, selectedModeIds, selectedGroupIds, showFirstUseMask, hasSelectedItems, toggleGlobalEditMode, exitGlobalEditMode, handleUserInteraction, showFirstUsePrompt, handleAppResume, toggleModeSelection, toggleGroupSelection, toggleImageSelection, handleBatchDelete } = useEditMode(invoke, {
  modes,
  groups,
  images,
  selectedModeId,
  selectedGroupId,
  loadModes,
  loadGroups,
  loadImages,
});

// 全局编辑模式状态
// 全局悬浮窗状态

// 首次使用提示弹窗
// 协议确认弹窗
const showAgreementConfirm = ref(false);
const hasAcceptedAgreement = ref(false);

// 协议确认处理
const handleAgreementAccept = () => {
  hasAcceptedAgreement.value = true;
  showAgreementConfirm.value = false;
  localStorage.setItem('meme_agreement_accepted', 'true');
};

const handleAgreementReject = () => {
  showAgreementConfirm.value = false;
  // 如果用户拒绝协议，退出应用
  if (isTauri) {
    invoke('exit_app').catch((e: any) => console.error('[App] Failed to exit app:', e));
  } else {
    window.close();
  }
};

const openUserAgreementFromPopup = () => {
  showAgreementConfirm.value = false;
  activeMenu.value = 'user-agreement';
};

const openPrivacyPolicyFromPopup = () => {
  showAgreementConfirm.value = false;
  activeMenu.value = 'privacy-policy';
};

// 监听用户交互

// 显示首次使用提示
// selectedImages 已存在，复用它

// 是否有选中项

// 切换全局编辑模式

// 监听应用恢复前台事件

// 退出全局编辑模式

// Android 返回键处理
function handleAndroidBack(event: any) {
  // 0. 若自动发送流程进行中，按返回键即终止
  if (isAndroidTauri()) {
    const native = window.AndroidNative;
    if (native && typeof native.deactivateSendFlow === 'function') {
      native.deactivateSendFlow();
    }
  }

  // 1. 关闭所有弹窗（优先级从高到低）
  if (showCustomAppsPopup.value) {
    showCustomAppsPopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showKeywordManager.value) {
    showKeywordManager.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showGroupEditPopup.value) {
    showGroupEditPopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showModeEditPopup.value) {
    showModeEditPopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showAddGroupPopup.value) {
    showAddGroupPopup.value = false;
    event.preventDefault?.();
    return true;
  }
  if (showAddModePopup.value) {
    showAddModePopup.value = false;
    event.preventDefault?.();
    return true;
  }
  
  // 2. 关闭分组操作菜单
  if (showGroupActionMenu.value) {
    showGroupActionMenu.value = false;
    event.preventDefault?.();
    return true;
  }
  
  // 3. 关闭菜单弹窗
  if (isMenuPopupOpen.value) {
    isMenuPopupOpen.value = false;
    event.preventDefault?.();
    return true;
  }
  
  // 4. 退出编辑模式
  if (isGlobalEditMode.value) {
    exitGlobalEditMode();
    event.preventDefault?.();
    return true;
  }
  
  // 6. 取消搜索框聚焦
  if (searchInputRef.value) {
    const inputEl = searchInputRef.value.$el?.querySelector?.('input') || searchInputRef.value.$el;
    if (inputEl && document.activeElement === inputEl) {
      inputEl.blur();
      searchKeyword.value = '';
      event.preventDefault?.();
      return true;
    }
  }
  
  // 7. 从设置子页面返回设置页，从其他页面返回首页
  if (activeMenu.value !== 'home') {
    const settingsSubPages = ['user-agreement', 'privacy-policy', 'support'];
    if (settingsSubPages.includes(activeMenu.value)) {
      activeMenu.value = 'settings';
    } else {
      activeMenu.value = 'home';
    }
    event.preventDefault?.();
    return true;
  }
  
  // 8. 如果在首页且没有其他状态，将应用后台到桌面
  if (isAndroidTauri() && typeof window.AndroidNative?.minimizeApp === 'function') {
    window.AndroidNative.minimizeApp();
    event.preventDefault?.();
    return true;
  }
  
  // 非 Android 平台，不阻止默认行为
  return false;
}

// 切换模式选中

// 切换分组选中

// 批量删除

// 新增模式弹窗状态
const showAddModePopup = ref(false);
const newModeName = ref('');
const newModeSortOrder = ref(1);

// 显示新增模式弹窗
function showAddModeDialog() {
  newModeName.value = '';
  newModeSortOrder.value = Math.max(...modes.value.map(m => m.sort_order), 0) + 1;
  showAddModePopup.value = true;
}

// 提交新增模式
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

const shareApp = ref<string>('all');  // 支持自定义包名
const activeMenu = ref('home');
const isMenuPopupOpen = ref(false);
const menuAnchor = ref<HTMLElement | { $el: HTMLElement } | null>(null);
const showGroupActionMenu = ref(false);
const currentEditingGroup = ref<Group | null>(null);
// 预览功能已移除，相关变量保留以备后续需要
// const isImagePreviewOpen = ref(false);
// const previewImageIndex = ref(0);

import { useSwipe } from './composables/useSwipe';

const { swipeOffset, isSwiping, handleSwipeStart, handleSwipeMove, handleSwipeEnd } = useSwipe({
  groups,
  selectedGroupId,
  switchGroup,
  isGlobalEditMode,
});

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

// 全局分享应用选择回调
if (typeof window !== 'undefined') {
  // 注入全局回调供 Android 原生调用 — 在 android.d.ts 中声明了类型
  window.onCustomAppSelected = async (packageName: string, appName: string) => {
    try {
      if ((!appName || appName === packageName) && isAndroidTauri() && typeof window.AndroidNative?.getApplicationName === 'function') {
        appName = window.AndroidNative.getApplicationName(packageName);
      }
      await invoke('add_custom_share_app', { packageName, appName });
      Snackbar.success(`已保存应用: ${appName || packageName}`);
      await loadCustomApps();
      shareApp.value = packageName;
      handleShareAppChange(packageName);
    } catch (error) {
      console.error('Failed to save custom app:', error);
      Snackbar.error('保存应用失败');
    }
  };
}


/**
 * 创建类型安全的配置对象
 * 确保 Config 中所有字段类型正确，防止 UI 组件返回非预期类型
 */

/**
 * 安全更新配置到后端
 */

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
      debug.log('[Restore] State restored:', state);
    } catch (e) {
      console.error('[Restore] Failed to parse saved state:', e);
    }
  }

  // 检查是否已接受用户协议
  const agreementAccepted = localStorage.getItem('meme_agreement_accepted');
  hasAcceptedAgreement.value = agreementAccepted === 'true';
  
  if (!hasAcceptedAgreement.value) {
    showAgreementConfirm.value = true;
  }

  // 并行加载配置和模式列表（两者互不依赖）
  await Promise.all([loadConfig(), loadModes()]);
  
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
    try {
      debug.log('[Theme] 系统深色模式切换');
      await syncSystemTheme(); // 直接调用抽出来的函数
    } catch (e) {
      console.error('[Theme] Failed to sync system theme on change:', e);
    }
  });
  
  window.addEventListener('modeImported', (async (event: Event) => {
    try {
      const { modeId } = (event as CustomEvent<{ modeId: number }>).detail;
      await loadModes();
      if (modeId && modes.value.find(m => m.id === modeId)) {
        await switchMode(modeId);
      }
    } catch (e) {
      console.error('[Mode] Failed to handle modeImported event:', e);
    }
  }) as EventListener);
  
  window.addEventListener('modeCreated', (async (event: Event) => {
    try {
      const { modeId } = (event as CustomEvent<{ modeId: number }>).detail;
      await loadModes();
      if (modeId && modes.value.find(m => m.id === modeId)) {
        await switchMode(modeId);
        activeMenu.value = 'home';
        Snackbar.success('已切换到新模式');
      }
    } catch (e) {
      console.error('[Mode] Failed to handle modeCreated event:', e);
    }
  }) as EventListener);
  
  window.addEventListener('navigateHome', () => {
    activeMenu.value = 'home';
  });
  
  window.addEventListener('navigateToMenu', (event: any) => {
    const detail = event.detail;
    if (typeof detail === 'string') {
      activeMenu.value = detail;
    } else if (detail && detail.menu) {
      activeMenu.value = detail.menu;
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
    try {
      if (document.visibilityState === 'visible') {
        // 应用恢复到前台
        debug.log('[Theme] App resumed to foreground');
        await syncSystemTheme(); 
        handleAppResume();
      }
    } catch (e) {
      console.error('[Theme] Failed to handle visibility change:', e);
    }
  });

  // 桌面端窗口重新获得焦点时同样触发搜索框聚焦
  window.addEventListener('focus', handleWindowFocus);

  // 添加悬浮窗触发搜索的全局方法
  window.triggerSearchFocus = (foregroundApp?: { packageName: string; appName: string } | null) => {
    debug.log('[Floating] Triggering search focus', foregroundApp);

    // 如果从悬浮窗触发且检测到前台应用（非本应用），自动切换分享目标
    if (foregroundApp && foregroundApp.packageName) {
      const pkg = foregroundApp.packageName;
      const mappedApp = pkg === 'com.tencent.mobileqq' ? 'qq'
                      : pkg === 'com.tencent.mm' ? 'wechat'
                      : pkg;
      debug.log('[Floating] Auto-switching share target to:', mappedApp, '(pkg:', pkg, ')');
      shareApp.value = mappedApp;
      handleShareAppChange(mappedApp);
    }

    // 显示首次使用提示
    showFirstUsePrompt();
    
    // 清空搜索词
    searchKeyword.value = '';
    
    // 立即聚焦搜索框
    if (searchInputRef.value) {
      debug.log('[Floating] Using Vue ref to focus');
      
      // Varlet Input 组件的聚焦方法
      if (typeof searchInputRef.value.focus === 'function') {
        searchInputRef.value.focus();
        debug.log('[Floating] Called ref.focus()');
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
        debug.log('[Floating] Found native input element');
        
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
  window.addEventListener('triggerSearchFocus', ((e: CustomEvent) => {
    debug.log('[Floating] Received triggerSearchFocus event', e.detail);
    window.triggerSearchFocus(e.detail);
    showFirstUsePrompt();
  }) as EventListener);

  // 监听用户交互
document.addEventListener('touchstart', handleUserInteraction, { passive: false });
document.addEventListener('click', handleUserInteraction);


  
  // Android 返回键监听
  if (isAndroidTauri()) {
    window.addEventListener('tauri-android-back', handleAndroidBack);
  }
  
  // 桌面端 ESC 键监听
  if (!isAndroidTauri()) {
    window.addEventListener('keydown', handleEscKey);
  }
});

// 桌面端 ESC 键处理
function handleEscKey(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    // 使用与 Android 返回键相同的处理逻辑
    const handled = handleAndroidBack(event);
    if (handled) {
      event.preventDefault();
    }
  }
}

onUnmounted(() => {
  // 移除 Android 返回键监听
  if (isAndroidTauri()) {
    window.removeEventListener('tauri-android-back', handleAndroidBack);
  }
  
  // 移除桌面端 ESC 键监听
  if (!isAndroidTauri()) {
    window.removeEventListener('keydown', handleEscKey);
  }
  
  // 清理事件监听
  document.removeEventListener('touchstart', handleUserInteraction);
  document.removeEventListener('click', handleUserInteraction);
  document.removeEventListener('visibilitychange', handleAppResume);
  window.removeEventListener('focus', handleWindowFocus);
});

// 监听全局悬浮窗状态变化
window.addEventListener('globalFloatingWindowChanged', ((e: CustomEvent) => {
  globalFloatingWindowEnabled.value = e.detail;
  debug.log('[Floating] Global floating window state changed:', e.detail);
}) as EventListener);


// 检查 AndroidNative 接口是否可用
if (isAndroidTauri()) {
  setTimeout(() => {
    if (typeof window.AndroidNative === 'undefined') {
      console.warn('[Android] AndroidNative interface not ready, waiting...');
      // 再等待一下
      setTimeout(() => {
        if (typeof window.AndroidNative === 'undefined') {
          console.error('[Android] AndroidNative interface still not available after waiting');
        } else {
          debug.log('[Android] AndroidNative interface is now available');
        }
      }, 2000);
    } else {
      debug.log('[Android] AndroidNative interface is available');
    }
  }, 1000);
}



async function loadCustomApps() {
  try {
    customShareApps.value = await invoke<any[]>("get_custom_share_apps") || [];
    
    // 从 SharedPreferences 同步到数据库（处理分享后未返回应用的情况）
    if (isAndroidTauri() && typeof window.AndroidNative?.getCustomAppsFromPrefs === 'function') {
      const prefsJson = window.AndroidNative.getCustomAppsFromPrefs();
      if (prefsJson) {
        const prefsPackages: string[] = JSON.parse(prefsJson);
        for (const pkg of prefsPackages) {
          if (pkg === 'com.tencent.mm' || pkg === 'com.tencent.mobileqq') continue;
          const exists = customShareApps.value.some(app => app.package_name === pkg);
          if (!exists) {
            const appName = window.AndroidNative.getApplicationName(pkg);
            await invoke('add_custom_share_app', { packageName: pkg, appName: appName || pkg });
            debug.log('Synced from SharedPreferences to DB:', pkg, appName);
          }
        }
        customShareApps.value = await invoke<any[]>("get_custom_share_apps") || [];
      }
    }
    
    // 补全缺失的应用名（app_name 为空或等于包名时，从原生端获取真实应用名）
    if (isAndroidTauri() && typeof window.AndroidNative?.getApplicationName === 'function') {
      let needsUpdate = false;
      for (const app of customShareApps.value) {
        if (!app.app_name || app.app_name === app.package_name) {
          const realName = window.AndroidNative.getApplicationName(app.package_name);
          if (realName && realName !== app.package_name) {
            app.app_name = realName;
            needsUpdate = true;
            await invoke('add_custom_share_app', { packageName: app.package_name, appName: realName });
          }
        }
      }
      if (needsUpdate) {
        debug.log('Updated missing app names for custom apps');
      }
    }
    
    // 同步到 SharedPreferences（确保悬浮窗服务能读取到最新数据）
    if (isAndroidTauri() && typeof window.AndroidNative?.syncCustomAppsToPrefs === 'function') {
      const packages = customShareApps.value.map(app => app.package_name);
      window.AndroidNative.syncCustomAppsToPrefs(JSON.stringify(packages));
      debug.log(`Synced ${packages.length} apps to SharedPreferences`);
    }
  } catch (error) {
    console.error("Failed to load custom apps:", error);
  }
}

async function removeCustomApp(id: number) {
  try {
    await invoke("remove_custom_share_app", { id });
    Snackbar.success("已移除应用");
    
    // 先同步到 SharedPreferences（必须在 loadCustomApps 之前，否则会被重新加回来）
    if (isAndroidTauri() && typeof window.AndroidNative?.syncCustomAppsToPrefs === 'function') {
      const packages = customShareApps.value
        .filter(app => app.id !== id)
        .map(app => app.package_name);
      window.AndroidNative.syncCustomAppsToPrefs(JSON.stringify(packages));
    }
    
    await loadCustomApps();
    
    // 如果历史应用被删完了，自动切回 "all"
    if (customShareApps.value.length === 0 && shareApp.value !== 'all') {
      shareApp.value = 'all';
      handleShareAppChange('all');
    }
  } catch (error) {
    console.error("Failed to remove custom app:", error);
    Snackbar.error("移除失败");
  }
}



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
      Snackbar.success('添加成功');
    }
  } catch (error) {
    console.error("Failed to upload images:", error);
    Snackbar.warning("添加图片失败: " + error);
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
	debug.log('[Android] uploadImagesAndroid 开始');
	debug.log('[Android] config:', config.value);
	debug.log('[Android] selectedGroupId:', selectedGroupId.value);
	debug.log('[Android] selectedModeId:', selectedModeId.value);
	
	if (!config.value || !selectedGroupId.value || !selectedModeId.value) {
		Snackbar.warning('请先选择分组');
		return;
	}
	
	try {
		// 步骤1：打开系统图片选择器，获取 content:// URI
		debug.log('[Android] 打开图片选择器...');
		const { open } = await import("@tauri-apps/plugin-dialog");
		
		const selected = await open({
			multiple: true,
			filters: [{
				name: "Images",
				extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"]
			}]
		});
		
		debug.log('[Android] 选择器返回结果:', selected);
		
		if (!selected || (Array.isArray(selected) && selected.length === 0)) {
			debug.log('[Android] 未选择文件');
			return;
		}
		
		// 统一转为数组
		const uris: string[] = Array.isArray(selected) ? selected : [selected];
		debug.log('[Android] 待上传 URI 列表:', uris);
		
		// 步骤2：读取文件内容并转为 Base64
		debug.log('[Android] 读取文件内容...');
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
				debug.log('[Android] 读取文件成功:', name, '大小:', fileData.length);
			} catch (err) {
				console.error('[Android] 读取文件失败:', uri, err);
			}
		}
		
		if (imagesData.length === 0) {
			Snackbar.warning('没有成功读取任何文件');
			return;
		}
		
		// 步骤3：调用后端上传接口
		debug.log('[Android] 调用后端 upload_images_android...');
		const successCount = await invoke<number>("upload_images_android", {
			imagesData: imagesData,
			groupId: selectedGroupId.value,
			modeId: selectedModeId.value
		});
		
		debug.log('[Android] 上传完成，成功数量:', successCount);
		
		// 步骤4：刷新图片列表
		await loadImages(selectedGroupId.value);
		
		Snackbar.success(`成功添加 ${successCount} 张图片`);
		debug.log('[Android] uploadImagesAndroid 完成');
	} catch (error) {
		console.error("[Android] 上传失败:", error);
		Snackbar.error('添加失败: ' + error);
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

function handleShareAppChange(app: string) {
  shareApp.value = app;
  if (config.value) {
    // 保存到配置时，'all' 转换为空字符串，自定义包名直接保存
    const shareAppValue = app === 'all' ? '' : app;
    config.value.share_app = shareAppValue;
    invoke("update_config", { config: config.value });
  }
}






function handleKeyDown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key === 'v') {
    handlePasteImage();
    return;
  }

  // 桌面端主页面：Ctrl+= 放大图片（列数-1），Ctrl+- 缩小图片（列数+1）
  if (event.ctrlKey && !isAndroidTauri() && activeMenu.value === 'home') {
    if (event.key === '=' || event.key === '+' || event.code === 'NumpadAdd') {
      event.preventDefault();
      adjustGridColumns(-1);
    } else if (event.key === '-' || event.key === '_' || event.code === 'NumpadSubtract') {
      event.preventDefault();
      adjustGridColumns(1);
    }
  }
}

// 调整列数（范围 2~8），并持久化到配置
function adjustGridColumns(delta: number) {
  const newColumns = Math.max(2, Math.min(8, gridColumns.value + delta));
  if (newColumns === gridColumns.value) return;
  gridColumns.value = newColumns;
  if (config.value) {
    config.value.grid_size = gridColumns.value;
    safeUpdateConfig(config.value);
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
    case 'editGroup':
      openGroupEditDialog();
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
      editingModeSortOrder.value = mode.sort_order;
      showModeEditPopup.value = true;
      break;
    case 'delete': {
      const result = await Dialog({
        title: '确认删除',
        message: `确定要删除模式 "${mode.name}" 吗？`,
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
        if (selectedModeId.value) {
          await loadGroups(selectedModeId.value);
        } else {
          groups.value = [];
          images.value = [];
          return;
        }
        if (selectedGroupId.value) {
          await loadImages(selectedGroupId.value);
        } else {
          images.value = [];
          return;
        }
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
  
  // 校验排序号：必须为正整数
  const sortOrder = Math.floor(editingModeSortOrder.value);
  if (!Number.isFinite(sortOrder) || sortOrder < 1) {
    Snackbar.warning('排序序号必须为正整数');
    return;
  }
  editingModeSortOrder.value = sortOrder;
  
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
        message: `确定要删除分组 "${group.name}" 吗？`,
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
    debug.log(`[Android] 分享图片到 ${app}:`, imagePath);
    
    // 如果选择的是"所有应用"或未指定特定应用，传递空字符串以显示系统分享菜单
    const targetApp = (app === 'all' || !app) ? '' : app;

    if (isAndroidTauri()) {
      // 检查 AndroidNative 接口是否可用，带重试机制
      let androidNative = window.AndroidNative;
      
      if (!androidNative || typeof androidNative.shareImageToApp !== 'function') {
        console.warn('[Android] AndroidNative interface not immediately available, waiting...');
        
        // 等待最多 2 秒，每 100ms 检查一次
        for (let i = 0; i < 20; i++) {
          await new Promise(resolve => setTimeout(resolve, 100));
          androidNative = window.AndroidNative;
          
          if (androidNative && typeof androidNative.shareImageToApp === 'function') {
            debug.log(`[Android] AndroidNative interface became available after ${i + 1} attempts`);
            break;
          }
        }
      }
      
      if (androidNative && typeof androidNative.shareImageToApp === 'function') {
        debug.log(`[Android] 调用原生分享接口:`, imagePath, targetApp);
        // 激活自动发送流程：用户已点击图片，准备打开系统分享
        if (typeof androidNative.activateSendFlow === 'function') {
          androidNative.activateSendFlow();
        }
        androidNative.shareImageToApp(imagePath, targetApp);
        await invoke("share_image", { imageId: img.id });
        
        if (!targetApp) {
          Snackbar.success('已打开系统分享菜单');
        } else {
          Snackbar.success(`正在分享到 ${app}`);
        }
      } else {
        console.error('[Android] AndroidNative interface still not available after retries');
        debug.log('[Android] Window keys:', Object.keys(window).filter(k => k.includes('Android') || k.includes('android')));
        
        // 尝试使用 Tauri Share 插件作为备选方案
        try {
          debug.log('[Android] Falling back to tauri-plugin-share');
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
    <div class="app-container">
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
            <button class="btn-icon" @click="uploadImages" title="添加图片">
              <Icon name="image" :size="24" />
            </button>
            <!-- 全局编辑模式按钮 -->
            <button 
              v-if="!isGlobalEditMode"
              class="btn-icon"
              @click="toggleGlobalEditMode" 
              title="编辑"
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
                <var-icon name="window-close" size="20" color="var(--color-text)" />
              </var-button>
              </div>
              <div class="group-action-list">
                <div class="group-action-item" @click="handleRenameGroup">
                  <var-icon name="pencil" size="20" />
                  <span>重命名分组</span>
                </div>
                <div class="group-action-item" @click="selectedGroupForMenu = currentEditingGroup; showKeywordManager = true; closeGroupActionMenu()">
                  <var-icon name="label" size="20" />
                  <span>关键词</span>
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
          v-show="isActive || isAndroidTauri()"
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
            @click="uploadImages"
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

        <!-- 窗口非活动时的暂停占位（仅桌面端） -->
        <div v-if="!isActive && !isAndroidTauri()" class="inactive-overlay">
          <div class="inactive-overlay-content">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="6" y="4" width="4" height="16"/>
              <rect x="14" y="4" width="4" height="16"/>
            </svg>
            <span>窗口非活跃，已暂停渲染</span>
          </div>
        </div>

        <!-- 模式编辑弹窗 -->
        <var-popup  class="edit-mode-popup" :show="showModeEditPopup" @click-overlay="showModeEditPopup = false">
          <div class="edit-popup">
            <div class="edit-popup-header">
              <h3>编辑模式</h3>
              <var-button text round @click="showModeEditPopup = false">
        <var-icon name="window-close" size="20" color="var(--color-text)" />
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
        <var-icon name="window-close" size="20" color="var(--color-text)" />
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
        <var-icon name="window-close" size="20" color="var(--color-text)" />
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
              <var-input
                :model-value="String(newModeSortOrder)"
                label="排序序号"
                type="number"
                placeholder="请输入排序序号"
                class="edit-input"
                @update:model-value="(val: string) => newModeSortOrder = Number(val)"
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
        <var-icon name="window-close" size="20" color="var(--color-text)" />
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

        <!-- 关键词管理弹窗 -->
        <KeywordManager
          v-model:show="showKeywordManager"
          :group-name="selectedGroupForMenu?.name || ''"
        />
        
        <!-- 自定义分享应用管理弹窗 -->
        <var-popup class="custom-apps-popup" :show="showCustomAppsPopup" @click-overlay="showCustomAppsPopup = false">
          <div class="keyword-manager">
            <div class="keyword-manager-header">
              <h3>历史应用</h3>
              <button class="btn-icon" @click="showCustomAppsPopup = false">
                <Icon name="close" :size="24" color="var(--color-text-secondary)" />
              </button>
            </div>
            
            <div class="keyword-manager-body">
              <!-- 应用列表 -->
              <div class="keywords-list">
                <div v-if="customShareApps.length === 0" class="empty-state">
                  <Icon name="apps-2" :size="32" />
                  <p>暂无应用</p>
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
      </div>

      <Settings v-else-if="activeMenu === 'settings'" />
      <Support v-else-if="activeMenu === 'support'" />
      <UserAgreement v-else-if="activeMenu === 'user-agreement'" @open-privacy-policy="activeMenu = 'privacy-policy'" />
      <PrivacyPolicy v-else-if="activeMenu === 'privacy-policy'" />
      </main>
      
      <!-- 浮动搜索按钮 -->
      <FloatingSearchButton
        :visible="activeMenu === 'home' && !isGlobalEditMode && (!globalFloatingWindowEnabled || !isAndroidTauri())" 
        @click="handleFloatingSearchClick"
      />
    </div>

    <!-- 首次使用提示弹窗 -->
    <div v-if="showFirstUseMask" class="first-use-popup" @click.stop="handleUserInteraction">
      <div class="first-use-popup-arrow"></div>
      <div class="first-use-popup-card">
        <span>Σ(っ °Д °;)っ!!! 你不要过来啊!!!</span>
      </div>
    </div>
    
    <!-- 用户协议确认弹窗 -->
    <var-popup v-model:show="showAgreementConfirm" class="agreement-confirm-popup" teleport="body">
      <var-card class="agreement-confirm-card">
        <div class="agreement-header">
          <h2>用户协议和隐私政策</h2>
        </div>
        <div class="agreement-content">
          <p>欢迎使用咪萌！请您务必审慎阅读、充分理解以下条款内容。</p>
          <p>1. 本应用仅供个人非商业用途使用，禁止反编译、修改或用于商业目的。</p>
          <p>2. 本应用仅在本地存储您的数据，不会收集或传输您的个人信息。</p>
          <p>3. 请您认真阅读并理解<a @click="openUserAgreementFromPopup" class="link">《用户服务协议》</a>和<a @click="openPrivacyPolicyFromPopup" class="link">《隐私政策》</a>的全部内容。</p>
        </div>
        <div class="agreement-actions">
          <var-button type="default" block @click="handleAgreementReject">拒绝</var-button>
          <var-button type="primary" block @click="handleAgreementAccept" style="margin-top: 12px;">同意并继续</var-button>
        </div>
      </var-card>
    </var-popup>
  </ThemeProvider>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background-color: var(--color-body);
  position: relative;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
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

/* 修复深色模式下搜索框文字颜色 */
:global(.var-dark .search-input .var-input__input) {
  color: var(--color-text) !important;
}

:global(.var-dark .search-input .var-input__input::placeholder) {
  color: var(--color-text-secondary) !important;
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


/* 图片选择复选框 */






/* 图片选择复选框 */






/* 图片选择复选框 */






/* 图片选择复选框 */






/* 图片选择复选框 */






/* 图片选择复选框 */





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
  color: var(--color-text-tertiary);
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
/* :global(body .var-popup) {
  background-color: var(--color-surface) !important;
} */

:global(body .var-popup--center) {
  background-color: var(--color-surface) !important;
}

:global(body .var-popup__content) {
  background-color: var(--color-surface) !important;
}

/* 取消所有弹窗的蒙板显示，但保留点击事件 */
:global(.var-popup__overlay),
:global(body .var-popup__overlay),
:global(.var-overlay),
:global(body .var-overlay) {
  background-color: transparent !important;
  opacity: 1 !important;
  pointer-events: auto !important;
}

/* 确保深色模式下也没有蒙板 */
:global(.var-dark .var-popup__overlay),
:global(.var-dark .var-overlay) {
  background-color: transparent !important;
  opacity: 1 !important;
  pointer-events: auto !important;
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

/* 首次使用提示弹窗 */
.first-use-popup {
  position: fixed;
  top: 72px;
  left: 16px;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  animation: popupFadeIn 0.3s ease-out;
  pointer-events: auto;
}

.first-use-popup-card {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #4A90E2;
  color: #fff;
  padding: 12px 24px;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(74, 144, 226, 0.4);
  white-space: nowrap;
  cursor: pointer;
}

.first-use-popup-arrow {
  width: 0;
  height: 0;
  margin-left: 24px;
  border-left: 10px solid transparent;
  border-right: 10px solid transparent;
  border-bottom: 10px solid #4A90E2;
}

@keyframes popupFadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 协议确认弹窗样式 */
:global(.agreement-confirm-popup .var-popup__content) {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
}

.agreement-confirm-card {
  width: 90%;
  min-width: 320px;
  max-width: 500px;
  padding: 24px;
  margin: 0 auto;
}

.agreement-header h2 {
  text-align: center;
  font-size: 20px;
  margin-bottom: 20px;
  color: var(--text-color);
}

.agreement-content p {
  font-size: 14px;
  line-height: 1.6;
  margin-bottom: 12px;
  color: var(--text-secondary);
}

.agreement-content .link {
  color: var(--color-primary);
  cursor: pointer;
}

.agreement-content .link:hover {
  text-decoration: underline;
}

.agreement-actions {
  margin-top: 24px;
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


/* 自定义分享应用弹窗样式 */
.custom-apps-popup :deep(.var-popup) {
  border-radius: 16px !important;
  overflow: hidden !important;
}

/* 强制设置背景色，优先级最高 */
/* .custom-apps-popup :deep(.var-popup), */
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
  /* 确保滚动流畅 */
  -webkit-overflow-scrolling: touch;
  scroll-behavior: smooth;
}

/* 关键词芯片样式 */
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

/* 应用列表容器 - 使用 flexbox 自动换行 */
.custom-apps-popup .keywords-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-content: flex-start;
}

.app-chip {
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  /* 优化大量应用时的显示 */
  margin: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
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
  padding-left: 28px !important; /* 为对勾留出空间 */
}

.app-chip-selected::before {
  content: '✓';
  position: absolute;
  left: 6px;
  top: 50%;
  transform: translateY(-50%);
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
