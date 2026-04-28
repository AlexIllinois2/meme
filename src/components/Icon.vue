<template>
  <i 
    :class="iconClass" 
    :style="iconStyle"
    aria-hidden="true"
  ></i>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  /** 图标名称，对应 Remix Icon 的类名（不含 ri- 前缀） */
  name: string;
  /** 图标大小，默认 24px */
  size?: number | string;
  /** 图标颜色，默认继承父元素 */
  color?: string;
  /** 是否填充风格，默认镂空（line） */
  fill?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 24,
  color: undefined,
  fill: false,
});

const iconClass = computed(() => {
  const suffix = props.fill ? 'fill' : 'line';
  return `ri-${props.name}-${suffix}`;
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
/* 确保图标垂直居中 */
i {
  line-height: 1;
}
</style>