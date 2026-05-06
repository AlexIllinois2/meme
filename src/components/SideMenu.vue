<script setup lang="ts">
import Icon from "./Icon.vue";

const props = defineProps<{
  isOpen: boolean;
  activeMenu: string;
  isSidebar?: boolean;
}>();

const emit = defineEmits(['update:isOpen', 'menu-change']);

function toggleMenu() {
  emit('update:isOpen', !props.isOpen);
}

function setActiveMenu(menu: string) {
  emit('menu-change', menu);
  emit('update:isOpen', false);
}
</script>

<template>
  <div class="side-menu-container">
    <!-- PC端侧边栏模式 -->
    <aside v-if="isSidebar" class="sidebar" :class="{ 'sidebar--open': isOpen }">
      <div class="drawer-content">
        <div class="menu-header">
          <h2>Meme Manager</h2>
          <button class="btn-icon" @click="toggleMenu">
            <Icon name="close" :size="24" />
          </button>
        </div>
        <div class="menu-items">
          <var-list class="modern-list">
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'home'"
              @click="setActiveMenu('home')"
            >
              <template #icon>
                <Icon name="home-3" :size="20" />
              </template>
              主页
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'mode'"
              @click="setActiveMenu('mode')"
            >
              <template #icon>
                <Icon name="folders" :size="20" />
              </template>
              模式管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'group'"
              @click="setActiveMenu('group')"
            >
              <template #icon>
                <Icon name="folder-3" :size="20" />
              </template>
              分组管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'keyword'"
              @click="setActiveMenu('keyword')"
            >
              <template #icon>
                <Icon name="price-tag-3" :size="20" />
              </template>
              关键词管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'settings'"
              @click="setActiveMenu('settings')"
            >
              <template #icon>
                <Icon name="settings-3" :size="20" />
              </template>
              设置
            </var-list-item>
          </var-list>
        </div>
      </div>
    </aside>
    
    <!-- 移动端抽屉模式 -->
    <var-popup
        v-else
        :show="isOpen"
        position="left"
        :overlay="true"
        class="modern-drawer"
        @update:show="(value: boolean) => emit('update:isOpen', value)"
      >
      <div class="drawer-content mobile-drawer-content">
        <div class="menu-header">
          <h2>Meme Manager</h2>
          <button class="btn-icon" @click="toggleMenu">
            <Icon name="close" :size="24" />
          </button>
        </div>
        <div class="menu-items">
          <var-list class="modern-list">
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'home'"
              @click="setActiveMenu('home')"
            >
              <template #icon>
                <Icon name="home-3" :size="20" />
              </template>
              主页
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'mode'"
              @click="setActiveMenu('mode')"
            >
              <template #icon>
                <Icon name="folders" :size="20" />
              </template>
              模式管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'group'"
              @click="setActiveMenu('group')"
            >
              <template #icon>
                <Icon name="folder-3" :size="20" />
              </template>
              分组管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'keyword'"
              @click="setActiveMenu('keyword')"
            >
              <template #icon>
                <Icon name="price-tag-3" :size="20" />
              </template>
              关键词管理
            </var-list-item>
            <var-list-item
              class="menu-item"
              :active="activeMenu === 'settings'"
              @click="setActiveMenu('settings')"
            >
              <template #icon>
                <Icon name="settings-3" :size="20" />
              </template>
              设置
            </var-list-item>
          </var-list>
        </div>
      </div>
    </var-popup>
  </div>
</template>

<style scoped>
.side-menu-container {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 1000;
  height: 100vh;
}

/* PC端侧边栏 */
.sidebar {
  position: fixed;
  top: 0;
  left: 0;
  width: 280px;
  height: 100vh;
  background-color: var(--color-bg);
  transform: translateX(-100%);
  transition: transform 0.3s ease;
  box-shadow: 2px 0 20px rgba(0, 0, 0, 0.1);
  z-index: 1001;
}

.sidebar--open {
  transform: translateX(0);
}

.sidebar .drawer-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding-bottom: 20px;
}

.sidebar .menu-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 20px;
  padding-top: max(24px, env(safe-area-inset-top));
  border-bottom: 1px solid var(--color-border);
}

.sidebar .menu-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-primary);
  letter-spacing: 0.5px;
}

.sidebar .menu-items {
  padding: 16px 0;
  overflow-y: auto;
  flex: 1;
}

.sidebar .modern-list :deep(.var-list-item) {
  border-radius: 12px;
  margin: 4px 16px;
  transition: all 0.2s ease;
}

.sidebar .modern-list :deep(.var-list-item--active) {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.sidebar .modern-list :deep(.var-list-item:hover:not(.var-list-item--active)) {
  background-color: var(--color-bg-2);
}

.sidebar .menu-item {
  font-size: 15px;
  padding: 12px 16px;
}

.sidebar .menu-item :deep(.var-list-item__icon) {
  margin-right: 12px;
  color: var(--color-text-2);
}

.sidebar .menu-item :deep(.var-list-item--active .var-list-item__icon) {
  color: var(--color-primary);
}

.sidebar .close-btn {
  margin-right: -8px;
  color: var(--color-text-2);
  transition: color 0.2s ease;
}

.sidebar .close-btn:hover {
  color: var(--color-text);
}

/* 移动端抽屉模式 */
.modern-drawer :deep(.var-popup__content) {
  border-radius: 0 24px 24px 0;
  box-shadow: 2px 0 20px rgba(0, 0, 0, 0.1);
  background-color: var(--color-bg);
  width: 280px;
}

.mobile-drawer-content {
  width: 280px;
}

.drawer-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding-bottom: 20px;
}

.menu-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 20px;
  padding-top: max(24px, env(safe-area-inset-top));
  border-bottom: 1px solid var(--color-border);
}

.menu-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-primary);
  letter-spacing: 0.5px;
}

.close-btn {
  margin-right: -8px;
  color: var(--color-text-2);
  transition: color 0.2s ease;
}

.close-btn:hover {
  color: var(--color-text);
}

.menu-items {
  padding: 16px 0;
  overflow-y: auto;
  flex: 1;
}

.modern-list :deep(.var-list-item) {
  border-radius: 12px;
  margin: 4px 16px;
  transition: all 0.2s ease;
}

.modern-list :deep(.var-list-item--active) {
  background-color: var(--color-primary-light);
  color: var(--color-primary);
}

.modern-list :deep(.var-list-item:hover:not(.var-list-item--active)) {
  background-color: var(--color-bg-2);
}

.menu-item {
  font-size: 15px;
  padding: 12px 16px;
}

.menu-item :deep(.var-list-item__icon) {
  margin-right: 12px;
  color: var(--color-text-2);
}

.menu-item :deep(.var-list-item--active .var-list-item__icon) {
  color: var(--color-primary);
}

/* 滚动条样式 */
.menu-items::-webkit-scrollbar {
  width: 6px;
}

.menu-items::-webkit-scrollbar-track {
  background: var(--color-bg-2);
  border-radius: 3px;
  margin: 10px 0;
}

.menu-items::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.menu-items::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
  
  .modern-drawer :deep(.var-popup__content) {
    width: 260px;
    border-radius: 0 20px 20px 0;
  }
  
  .menu-header {
    padding: 20px 20px 16px;
    padding-top: max(20px, env(safe-area-inset-top));
  }
  
  .menu-header h2 {
    font-size: 18px;
  }
  
  .menu-item {
    font-size: 14px;
    padding: 10px 14px;
  }
  
  .modern-list :deep(.var-list-item) {
    margin: 3px 12px;
  }
}
</style>
