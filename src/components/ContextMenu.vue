<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import Icon from "./Icon.vue";

interface MenuItem {
  label: string;
  value: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
}

const props = defineProps<{
  items: MenuItem[];
}>();

const emit = defineEmits<{
  (e: 'select', value: string): void;
}>();

const show = ref(false);
const position = ref({ x: 0, y: 0 });
const longPressTimer = ref<number | null>(null);
const isLongPress = ref(false);
const menuRef = ref<HTMLElement | null>(null);
const touchStartPos = ref({ x: 0, y: 0 });
const hasMoved = ref(false);

// 处理长按开始
function handleTouchStart(event: TouchEvent) {
  isLongPress.value = false;
  hasMoved.value = false;
  const touch = event.touches[0];
  touchStartPos.value = { x: touch.clientX, y: touch.clientY };
  
  longPressTimer.value = window.setTimeout(() => {
    // 只有在没有明显移动时才触发长按
    if (!hasMoved.value) {
      isLongPress.value = true;
      openMenu(touch.clientX, touch.clientY);
    }
  }, 500); // 500ms 长按触发
}

// 处理触摸移动 - 检测是否滑动
function handleTouchMove(event: TouchEvent) {
  if (longPressTimer.value && !isLongPress.value) {
    const touch = event.touches[0];
    const dx = Math.abs(touch.clientX - touchStartPos.value.x);
    const dy = Math.abs(touch.clientY - touchStartPos.value.y);
    
    // 如果移动距离超过 10px,认为是滑动而非长按
    if (dx > 10 || dy > 10) {
      hasMoved.value = true;
      clearTimeout(longPressTimer.value!);
      longPressTimer.value = null;
    }
  }
}

// 处理长按结束
function handleTouchEnd(event: TouchEvent) {
  if (longPressTimer.value) {
    clearTimeout(longPressTimer.value);
    longPressTimer.value = null;
  }
  // 如果是长按触发，阻止默认行为
  if (isLongPress.value) {
    event.preventDefault();
  }
}

// 处理右键点击
function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  openMenu(event.clientX, event.clientY);
}

// 打开菜单
function openMenu(x: number, y: number) {
  // 确保菜单不会超出屏幕边界
  const menuWidth = 160;
  const menuHeight = props.items.length * 44 + 16;
  
  let finalX = x;
  let finalY = y;
  
  if (x + menuWidth > window.innerWidth) {
    finalX = x - menuWidth;
  }
  if (y + menuHeight > window.innerHeight) {
    finalY = y - menuHeight;
  }
  
  position.value = { x: finalX, y: finalY };
  show.value = true;
}

// 关闭菜单
function closeMenu() {
  show.value = false;
}

// 选择菜单项
function selectItem(value: string) {
  if (!show.value) return;
  closeMenu();
  emit('select', value);
}

// 计算菜单位置样式
const menuStyle = computed(() => ({
  position: 'fixed' as const,
  left: `${position.value.x}px`,
  top: `${position.value.y}px`,
  zIndex: 9999,
}));

defineExpose({
  open: openMenu,
  close: closeMenu,
});

// Android 返回键/桌面ESC键处理 - 如果菜单打开，关闭它
function handleBackButton(event: Event) {
  if (show.value) {
    closeMenu();
    event.preventDefault?.();
  }
}

function handleEscKey(event: KeyboardEvent) {
  if (event.key === 'Escape' && show.value) {
    closeMenu();
    event.preventDefault();
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('tauri-android-back', handleBackButton);
    // 桌面端添加ESC键监听
    if (!/Android/i.test(navigator.userAgent)) {
      window.addEventListener('keydown', handleEscKey);
    }
  }
});

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('tauri-android-back', handleBackButton);
    // 桌面端移除ESC键监听
    if (!/Android/i.test(navigator.userAgent)) {
      window.removeEventListener('keydown', handleEscKey);
    }
  }
});
</script>

<template>
  <div
    class="context-menu-trigger"
    @touchstart.passive="handleTouchStart"
    @touchmove.passive="handleTouchMove"
    @touchend="handleTouchEnd"
    @touchcancel="handleTouchEnd"
    @contextmenu.prevent="handleContextMenu"
  >
    <slot />
  </div>
  
  <!-- 菜单 -->
  <teleport to="body">
    <div v-if="show" class="context-menu-overlay" @click="closeMenu">
      <div
        ref="menuRef"
        class="context-menu"
        :style="menuStyle"
        @click.stop
      >
        <div class="context-menu-list">
          <div
            v-for="item in items"
            :key="item.value"
            class="context-menu-item"
            :class="{ danger: item.danger, disabled: item.disabled }"
            @click="!item.disabled && selectItem(item.value)"
          >
            <Icon v-if="item.icon" :name="item.icon" :size="20" />
            <span>{{ item.label }}</span>
          </div>
        </div>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.context-menu-trigger {
  display: contents;
  /* 禁用默认的长按行为 */
  -webkit-touch-callout: none;
  user-select: none;
}

.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9998;
  background: transparent;
}

.context-menu {
  min-width: 160px;
  max-width: 220px;
  background-color: var(--color-surface);
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  overflow: hidden;
  animation: menuAppear 0.15s ease;
}

@keyframes menuAppear {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-list {
  padding: 8px;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  color: var(--color-text);
  user-select: none;
}

.context-menu-item:hover {
  background-color: var(--color-surface-variant);
}

.context-menu-item:active {
  background-color: var(--color-primary-light);
  transform: scale(0.98);
}

.context-menu-item.danger {
  color: var(--color-danger, #f44336);
}

.context-menu-item.danger:hover {
  background-color: var(--color-danger-light, rgba(244, 67, 54, 0.1));
}

.context-menu-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.context-menu-item.disabled:hover {
  background-color: transparent;
}
</style>