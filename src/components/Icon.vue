<template>
  <Icon 
    :icon="iconName" 
    :style="iconStyle"
    aria-hidden="true"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Icon } from '@iconify/vue';

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
</style>
