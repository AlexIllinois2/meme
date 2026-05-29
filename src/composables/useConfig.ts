import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { Config } from '../types';
import { debug } from '../utils/debug';
import { isTauri, isAndroidTauri } from '../utils/tauri';

/**
 * useConfig - 配置/主题/缩放手势 composable
 *
 * 从 App.vue 提取，负责：
 * - config 加载/更新
 * - 主题同步 (applyTheme, syncSystemTheme)
 * - 缩放手势处理 (handleWheel, handleTouchStart, handleTouchMove)
 * - 格子列数自适应 (handleResize)
 */
export function useConfig(invoke: (...args: any[]) => any) {
  // ==============================
  // Refs
  // ==============================
  const config = ref<Config | null>(null);
  const currentColorMode = ref<'system' | 'light' | 'dark'>('system');
  const gridColumns = ref<number>(5);
  const pinyinSearchEnabled = ref(true);
  const acronymSearchEnabled = ref(true);
  const globalFloatingWindowEnabled = ref(true);
  const themeKey = ref(0);
  const initialPinchScale = ref(1);
  const lastPinchDistance = ref(0);
  const isMobile = computed(() => window.innerWidth < 768);

  // ==============================
  // selectDirectory
  // ==============================
  async function selectDirectory(): Promise<string | null> {
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
  }

  // ==============================
  // Config helpers
  // ==============================
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

  // ==============================
  // Config loading
  // ==============================
  /**
   * 加载配置
   * @param context.onConfigLoaded - 配置加载成功后回调 (处理 shareApp/selectedModeId/loadCustomApps)
   * @param context.onSetupInitial - 首次启动/出错时，调用 setupInitialConfig 的回调
   */
  async function loadConfig(context?: {
    onConfigLoaded?: (cfg: Config) => Promise<void>;
    onSetupInitial?: () => Promise<void>;
  }): Promise<void> {
    try {
      const result = await invoke("get_config") as Config;
      if (result && result.meme_dir) {
        config.value = result;

        // 不再后台执行 full_refresh（它会持锁阻塞 update_config 等其他命令）
        // full_refresh 仅在用户手动刷新或切换目录时执行

        // 通知 App.vue 设置其自身的状态 (selectedModeId, selectedGroupId, shareApp, loadCustomApps)
        if (context?.onConfigLoaded) {
          await context.onConfigLoaded(result);
        }

        gridColumns.value = config.value.grid_size || 4;
        currentColorMode.value = config.value.color_mode as 'system' | 'light' | 'dark';
        pinyinSearchEnabled.value = config.value.pinyin_search || true;
        acronymSearchEnabled.value = config.value.acronym_search || true;

        // Android 平台：从原生服务同步悬浮窗实际状态，并在需要时自动启动服务
        if (/Android/i.test(navigator.userAgent)) {
          try {
            const nativeEnabled = (window as any).AndroidNative?.isFloatingWindowEnabled?.();
            const shouldBeEnabled = config.value.global_floating_window === true;

            if (shouldBeEnabled && nativeEnabled !== true) {
              debug.log('[Floating] Config says enabled but service not running, starting...');
              (window as any).AndroidNative?.startFloatingWindow?.();
              globalFloatingWindowEnabled.value = true;
            } else {
              globalFloatingWindowEnabled.value = nativeEnabled === true;
            }
            debug.log('[Floating] Synced from native service:', nativeEnabled, 'config:', shouldBeEnabled);
          } catch (e) {
            globalFloatingWindowEnabled.value = config.value.global_floating_window || true;
            console.warn('[Floating] Failed to get native status, using config value:', e);
          }
        } else {
          globalFloatingWindowEnabled.value = config.value.global_floating_window || true;
        }

        applyTheme();
      } else {
        // 首次启动，需要选择目录
        if (context?.onSetupInitial) {
          await context.onSetupInitial();
        }
        return;
      }
    } catch (error) {
      console.error("Failed to load config:", error);
      if (context?.onSetupInitial) {
        await context.onSetupInitial();
      }
      return;
    }
  }

  // ==============================
  // Initial config setup
  // ==============================
  /**
   * 首次启动设置
   * @param context.autoRefreshAfterDirChange - 目录变更后的自动刷新回调
   */
  async function setupInitialConfig(context?: {
    autoRefreshAfterDirChange?: (dir: string) => Promise<void>;
  }): Promise<void> {
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
        pinyin_search: true,
        acronym_search: true,
        global_floating_window: true
      };
      config.value = newConfig;
      await safeUpdateConfig(config.value);
      applyTheme();

      // 首次启动，自动刷新数据到数据库并加载
      if (context?.autoRefreshAfterDirChange) {
        await context.autoRefreshAfterDirChange(selectedDir);
      }
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
        pinyin_search: true,
        acronym_search: true,
        global_floating_window: true
      };
      config.value = defaultConfig;
      await safeUpdateConfig(config.value);
      applyTheme();

      // 使用默认目录时，也尝试刷新数据并加载
      if (context?.autoRefreshAfterDirChange) {
        await context.autoRefreshAfterDirChange(defaultMemeDir);
      }
    }
  }

  // ==============================
  // Theme
  // ==============================
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

  async function syncSystemTheme() {
    // 只有当前是【跟随系统模式】才执行同步
    if (currentColorMode.value !== 'system' || !config.value) return;

    // 判断系统当前是否深色
    const isSystemDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? false;
    // 判断当前页面是否深色
    const isCurrentlyDark = document.documentElement.classList.contains('var-dark');

    // 不一致 → 重新同步
    if (isSystemDark !== isCurrentlyDark) {
      debug.log('[Theme] 系统主题变化，正在同步...');

      applyTheme();          // 应用主题到 DOM
      themeKey.value++;      // 强制 ThemeProvider 更新
      debug.log('[Theme] 主题同步完成');
    }
  }

  // ==============================
  // Event handlers
  // ==============================
  function handleResize() {
    if (window.innerWidth < 768) {
      gridColumns.value = 3;
    } else if (window.innerWidth < 1024) {
      gridColumns.value = 4;
    } else {
      gridColumns.value = 6;
    }
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

  // ==============================
  // Event listener lifecycle
  // ==============================
  let darkModeMediaQuery: MediaQueryList | null = null;

  const onSystemThemeChange = async () => {
    try {
      debug.log('[Theme] 系统深色模式切换');
      await syncSystemTheme();
    } catch (e) {
      console.error('[Theme] Failed to sync system theme on change:', e);
    }
  };

  const onColorModeChanged = (event: any) => {
    const mode = event.detail;
    if (mode) {
      currentColorMode.value = mode;
      applyTheme();
    }
  };

  const onGlobalFloatingWindowChanged = ((e: CustomEvent) => {
    globalFloatingWindowEnabled.value = e.detail;
    debug.log('[Floating] Global floating window state changed:', e.detail);
  }) as EventListener;

  onMounted(() => {
    window.addEventListener('resize', handleResize);
    window.addEventListener('wheel', handleWheel, { passive: false });
    window.addEventListener('touchstart', handleTouchStart, { passive: false });
    window.addEventListener('touchmove', handleTouchMove, { passive: false });

    darkModeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    darkModeMediaQuery.addEventListener('change', onSystemThemeChange);

    window.addEventListener('colorModeChanged', onColorModeChanged);
    window.addEventListener('globalFloatingWindowChanged', onGlobalFloatingWindowChanged);
  });

  onUnmounted(() => {
    window.removeEventListener('resize', handleResize);
    window.removeEventListener('wheel', handleWheel);
    window.removeEventListener('touchstart', handleTouchStart);
    window.removeEventListener('touchmove', handleTouchMove);

    if (darkModeMediaQuery) {
      darkModeMediaQuery.removeEventListener('change', onSystemThemeChange);
    }

    window.removeEventListener('colorModeChanged', onColorModeChanged);
    window.removeEventListener('globalFloatingWindowChanged', onGlobalFloatingWindowChanged);
  });

  return {
    config,
    currentColorMode,
    gridColumns,
    pinyinSearchEnabled,
    acronymSearchEnabled,
    globalFloatingWindowEnabled,
    themeKey,
    initialPinchScale,
    lastPinchDistance,
    isMobile,
    selectDirectory,
    createSafeConfig,
    safeUpdateConfig,
    loadConfig,
    setupInitialConfig,
    applyTheme,
    syncSystemTheme,
    handleResize,
    handleWheel,
    handleTouchStart,
    handleTouchMove,
  };
}
