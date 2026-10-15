<template>
  <Icon 
    :icon="iconName" 
    :style="iconStyle"
    aria-hidden="true"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
// 使用 offline 版本：图标数据来自本地注册的 remix-icons.json 子集，
// 避免运行时请求 Iconify 在线 API（离线/弱网时图标会缺失）
import { Icon } from '@iconify/vue/offline';

interface Props {
  name: string;
  size?: number | string;
  color?: string;
  fill?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 24,
  color: undefined,
  fill: false,
});

const iconName = computed(() => {
  const suffix = props.fill ? 'fill' : 'line';
  return `ri:${props.name}-${suffix}`;
});

const iconStyle = computed(() => {
  const style: Record<string, string> = {
    fontSize: typeof props.size === 'number' ? `${props.size}px` : props.size,
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    verticalAlign: 'middle',
  };
  
  if (props.color) {
    style.color = props.color;
  }
  
  return style;
});
</script>

<style scoped>
/* 加个空格或注释就修复了 */
</style>
