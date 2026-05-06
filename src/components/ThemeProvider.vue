<script setup lang="ts">
import { computed, watch } from 'vue';
// StyleProvider 是组件形式使用，不需要导入
// import { StyleProvider } from '@varlet/ui';

const props = defineProps<{
  themeStyle: 'default' | 'modern' | 'minimal';
  colorMode: 'system' | 'light' | 'dark';
}>();

// 根据主题风格生成 CSS 变量
const styleVars = computed(() => {
  const isDark = props.colorMode === 'dark' || 
    (props.colorMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  
  if (props.themeStyle === 'modern') {
    return isDark ? modernDarkVars : modernLightVars;
  } else if (props.themeStyle === 'minimal') {
    return isDark ? minimalDarkVars : minimalLightVars;
  }
  return isDark ? defaultDarkVars : defaultLightVars;
});

// 现代风格 - 浅色
const modernLightVars = {
  // 主色调
  '--color-primary': '#6366f1',
  '--color-primary-light': 'rgba(99, 102, 241, 0.12)',
  '--color-primary-dark': '#4f46e5',
  
  // 背景
  '--color-body': '#f8fafc',
  '--color-surface': '#ffffff',
  '--color-surface-variant': '#f1f5f9',
  
  // 文字
  '--color-text': '#1f2937',
  '--color-text-2': '#6b7280',
  '--color-text-3': '#9ca3af',
  '--color-text-secondary': '#6b7280',
  '--color-text-tertiary': '#9ca3af',
  
  // 边框
  '--color-border': '#e2e8f0',
  '--color-outline': 'rgba(99, 102, 241, 0.2)',
  
  // 圆角
  '--radius-sm': '8px',
  '--radius-md': '12px',
  '--radius-lg': '16px',
  '--radius-xl': '20px',
  '--radius-2xl': '24px',
  
  // 阴影
  '--shadow-sm': '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  '--shadow-md': '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  '--shadow-lg': '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  
  // AppBar
  '--app-bar-color': 'rgba(255, 255, 255, 0.85)',
  '--app-bar-text-color': '#1f2937',
  
  // Cell
  '--cell-background': '#ffffff',
  '--cell-border-color': '#e2e8f0',
  
  // 按钮
  '--button-default-color': '#f1f5f9',
  '--button-default-text-color': '#374151',
  '--button-primary-color': '#6366f1',
};

// 现代风格 - 深色
const modernDarkVars = {
  '--color-primary': '#818cf8',
  '--color-primary-light': 'rgba(129, 140, 248, 0.15)',
  '--color-primary-dark': '#6366f1',
  
  '--color-body': '#0f172a',
  '--color-surface': '#1e293b',
  '--color-surface-variant': '#334155',
  
  '--color-text': '#f1f5f9',
  '--color-text-2': '#94a3b8',
  '--color-text-3': '#64748b',
  '--color-text-secondary': '#94a3b8',
  '--color-text-tertiary': '#64748b',
  
  '--color-border': '#334155',
  '--color-outline': 'rgba(129, 140, 248, 0.2)',
  
  '--radius-sm': '8px',
  '--radius-md': '12px',
  '--radius-lg': '16px',
  '--radius-xl': '20px',
  '--radius-2xl': '24px',
  
  '--shadow-sm': '0 1px 2px 0 rgb(0 0 0 / 0.3)',
  '--shadow-md': '0 4px 6px -1px rgb(0 0 0 / 0.4), 0 2px 4px -2px rgb(0 0 0 / 0.4)',
  '--shadow-lg': '0 10px 15px -3px rgb(0 0 0 / 0.5), 0 4px 6px -4px rgb(0 0 0 / 0.5)',
  
  '--app-bar-color': 'rgba(15, 23, 42, 0.85)',
  '--app-bar-text-color': '#f1f5f9',
  
  '--cell-background': '#1e293b',
  '--cell-border-color': '#334155',
  
  '--button-default-color': '#334155',
  '--button-default-text-color': '#f1f5f9',
  '--button-primary-color': '#6366f1',
};

// 极简风格 - 浅色
const minimalLightVars = {
  '--color-primary': '#18181b',
  '--color-primary-light': 'rgba(24, 24, 27, 0.05)',
  '--color-primary-dark': '#000000',
  
  '--color-body': '#ffffff',
  '--color-surface': '#ffffff',
  '--color-surface-variant': '#fafafa',
  
  '--color-text': '#18181b',
  '--color-text-2': '#71717a',
  '--color-text-3': '#a1a1aa',
  '--color-text-secondary': '#71717a',
  '--color-text-tertiary': '#a1a1aa',
  
  '--color-border': '#e4e4e7',
  '--color-outline': 'rgba(24, 24, 27, 0.1)',
  
  '--radius-sm': '4px',
  '--radius-md': '6px',
  '--radius-lg': '8px',
  '--radius-xl': '12px',
  '--radius-2xl': '16px',
  
  '--shadow-sm': '0 1px 2px 0 rgb(0 0 0 / 0.02)',
  '--shadow-md': '0 1px 3px 0 rgb(0 0 0 / 0.05)',
  '--shadow-lg': '0 4px 6px -1px rgb(0 0 0 / 0.05)',
  
  '--app-bar-color': '#ffffff',
  '--app-bar-text-color': '#18181b',
  
  '--cell-background': '#ffffff',
  '--cell-border-color': '#e4e4e7',
  
  '--button-default-color': '#f4f4f5',
  '--button-default-text-color': '#18181b',
  '--button-primary-color': '#18181b',
};

// 极简风格 - 深色
const minimalDarkVars = {
  '--color-primary': '#fafafa',
  '--color-primary-light': 'rgba(250, 250, 250, 0.1)',
  '--color-primary-dark': '#ffffff',
  
  '--color-body': '#09090b',
  '--color-surface': '#18181b',
  '--color-surface-variant': '#27272a',
  
  '--color-text': '#fafafa',
  '--color-text-2': '#a1a1aa',
  '--color-text-3': '#71717a',
  '--color-text-secondary': '#a1a1aa',
  '--color-text-tertiary': '#71717a',
  
  '--color-border': '#27272a',
  '--color-outline': 'rgba(250, 250, 250, 0.1)',
  
  '--radius-sm': '4px',
  '--radius-md': '6px',
  '--radius-lg': '8px',
  '--radius-xl': '12px',
  '--radius-2xl': '16px',
  
  '--shadow-sm': '0 1px 2px 0 rgb(0 0 0 / 0.2)',
  '--shadow-md': '0 1px 3px 0 rgb(0 0 0 / 0.3)',
  '--shadow-lg': '0 4px 6px -1px rgb(0 0 0 / 0.4)',
  
  '--app-bar-color': '#18181b',
  '--app-bar-text-color': '#fafafa',
  
  '--cell-background': '#18181b',
  '--cell-border-color': '#27272a',
  
  '--button-default-color': '#27272a',
  '--button-default-text-color': '#fafafa',
  '--button-primary-color': '#fafafa',
};

// 默认风格 - 浅色
const defaultLightVars = {
  '--color-primary': '#3b82f6',
  '--color-primary-light': 'rgba(59, 130, 246, 0.1)',
  '--color-primary-dark': '#2563eb',
  
  '--color-body': '#ffffff',
  '--color-surface': '#ffffff',
  '--color-surface-variant': '#f3f4f6',
  
  '--color-text': '#1f2937',
  '--color-text-2': '#6b7280',
  '--color-text-3': '#9ca3af',
  '--color-text-secondary': '#6b7280',
  '--color-text-tertiary': '#9ca3af',
  
  '--color-border': '#e5e7eb',
  '--color-outline': 'rgba(59, 130, 246, 0.15)',
  
  '--radius-sm': '6px',
  '--radius-md': '8px',
  '--radius-lg': '12px',
  '--radius-xl': '16px',
  '--radius-2xl': '20px',
  
  '--shadow-sm': '0 1px 2px rgba(0, 0, 0, 0.05)',
  '--shadow-md': '0 4px 6px rgba(0, 0, 0, 0.1)',
  '--shadow-lg': '0 10px 15px rgba(0, 0, 0, 0.1)',
  
  '--app-bar-color': '#ffffff',
  '--app-bar-text-color': '#1f2937',
  
  '--cell-background': '#ffffff',
  '--cell-border-color': '#e5e7eb',
  
  '--button-default-color': '#f3f4f6',
  '--button-default-text-color': '#374151',
  '--button-primary-color': '#3b82f6',
};

// 默认风格 - 深色
const defaultDarkVars = {
  '--color-primary': '#60a5fa',
  '--color-primary-light': 'rgba(96, 165, 250, 0.15)',
  '--color-primary-dark': '#3b82f6',
  
  '--color-body': '#111827',
  '--color-surface': '#1f2937',
  '--color-surface-variant': '#374151',
  
  '--color-text': '#f3f4f6',
  '--color-text-2': '#9ca3af',
  '--color-text-3': '#6b7280',
  '--color-text-secondary': '#9ca3af',
  '--color-text-tertiary': '#6b7280',
  
  '--color-border': '#374151',
  '--color-outline': 'rgba(96, 165, 250, 0.2)',
  
  '--radius-sm': '6px',
  '--radius-md': '8px',
  '--radius-lg': '12px',
  '--radius-xl': '16px',
  '--radius-2xl': '20px',
  
  '--shadow-sm': '0 1px 2px rgba(0, 0, 0, 0.3)',
  '--shadow-md': '0 4px 6px rgba(0, 0, 0, 0.4)',
  '--shadow-lg': '0 10px 15px rgba(0, 0, 0, 0.5)',
  
  '--app-bar-color': '#1f2937',
  '--app-bar-text-color': '#f3f4f6',
  
  '--cell-background': '#1f2937',
  '--cell-border-color': '#374151',
  
  '--button-default-color': '#374151',
  '--button-default-text-color': '#f3f4f6',
  '--button-primary-color': '#3b82f6',
};

// 监听主题变化应用到 CSS 变量
watch(() => [props.themeStyle, props.colorMode], () => {
  const vars = styleVars.value;
  Object.entries(vars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
}, { immediate: true });
</script>

<template>
  <var-style-provider :style-vars="styleVars">
    <slot />
  </var-style-provider>
</template>