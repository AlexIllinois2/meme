<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Icon from './Icon.vue';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'click'): void;
}>();

const buttonRef = ref<HTMLElement | null>(null);
const position = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const buttonStart = ref({ x: 0, y: 0 });

const STORAGE_KEY = 'floating_search_btn_pos';

// 加载保存的位置
onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      const pos = JSON.parse(saved);
      position.value = pos;
    } catch {
      resetPosition();
    }
  } else {
    resetPosition();
  }
});

function resetPosition() {
  // 默认位置：右下角，距离边缘 24px（考虑安全区域）
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  
  // 获取底部安全区域
  const rootStyle = getComputedStyle(document.documentElement);
  const safeAreaBottom = parseInt(rootStyle.getPropertyValue('--safe-area-bottom')) || 0;
  
  position.value = {
    x: vw - 80,
    y: vh - 120 - safeAreaBottom
  };
}

function handleMouseDown(e: MouseEvent) {
  if (!buttonRef.value) return;
  isDragging.value = false;
  dragStart.value = { x: e.clientX, y: e.clientY };
  buttonStart.value = { ...position.value };
  
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
}

function handleTouchStart(e: TouchEvent) {
  if (!buttonRef.value) return;
  isDragging.value = false;
  const touch = e.touches[0];
  dragStart.value = { x: touch.clientX, y: touch.clientY };
  buttonStart.value = { ...position.value };
  
  document.addEventListener('touchmove', handleTouchMove, { passive: false });
  document.addEventListener('touchend', handleTouchEnd);
}

function handleMouseMove(e: MouseEvent) {
  const dx = e.clientX - dragStart.value.x;
  const dy = e.clientY - dragStart.value.y;
  
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    isDragging.value = true;
  }
  
  updatePosition(buttonStart.value.x + dx, buttonStart.value.y + dy);
}

function handleTouchMove(e: TouchEvent) {
  e.preventDefault();
  const touch = e.touches[0];
  const dx = touch.clientX - dragStart.value.x;
  const dy = touch.clientY - dragStart.value.y;
  
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    isDragging.value = true;
  }
  
  updatePosition(buttonStart.value.x + dx, buttonStart.value.y + dy);
}

function updatePosition(x: number, y: number) {
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const btnSize = 42; // 按钮大小（与 .floating-search-btn 的 CSS 尺寸保持一致）
  
  // 获取安全区域大小（从 CSS 变量读取，单位是 px）
  const rootStyle = getComputedStyle(document.documentElement);
  const safeAreaTop = parseInt(rootStyle.getPropertyValue('--safe-area-top')) || 0;
  const safeAreaBottom = parseInt(rootStyle.getPropertyValue('--safe-area-bottom')) || 0;
  
  // 边界检测（考虑安全区域）
  const minY = Math.max(8, safeAreaTop);
  const maxY = vh - btnSize - Math.max(8, safeAreaBottom);
  
  x = Math.max(8, Math.min(vw - btnSize - 8, x));
  y = Math.max(minY, Math.min(maxY, y));
  
  position.value = { x, y };
}

function handleMouseUp() {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  savePosition();
}

function handleTouchEnd() {
  document.removeEventListener('touchmove', handleTouchMove);
  document.removeEventListener('touchend', handleTouchEnd);
  savePosition();
}

function savePosition() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(position.value));
}

function handleClick() {
  if (!isDragging.value) {
    emit('click');
  }
}

onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  document.removeEventListener('touchmove', handleTouchMove);
  document.removeEventListener('touchend', handleTouchEnd);
});
</script>

<template>
  <Transition name="fade">
    <div
      v-if="visible"
      ref="buttonRef"
      class="floating-search-btn"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
      @mousedown="handleMouseDown"
      @touchstart="handleTouchStart"
      @click="handleClick"
    >
      <Icon name="search" :size="22" color="var(--color-on-primary)" />
    </div>
  </Transition>
</template>

<style scoped>
/* 颜色全部走主题变量：--color-primary-rgb / --color-on-primary 由 ThemeProvider
   按主题风格（modern/minimal/default）与明暗模式注入，避免硬编码导致换主题后失调 */
.floating-search-btn {
  position: fixed;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: rgba(var(--color-primary-rgb), 0.55);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow:
    0 4px 12px rgba(var(--color-primary-rgb), 0.35),
    0 2px 4px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  user-select: none;
  touch-action: none;
  transition:
    transform 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;
}

.floating-search-btn:hover {
  background: rgba(var(--color-primary-rgb), 0.75);
  transform: scale(1.05);
  box-shadow:
    0 6px 20px rgba(var(--color-primary-rgb), 0.45),
    0 2px 8px rgba(0, 0, 0, 0.15);
}

.floating-search-btn:active {
  transform: scale(0.95);
  background: rgba(var(--color-primary-rgb), 0.75);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: scale(0.8);
}
</style>