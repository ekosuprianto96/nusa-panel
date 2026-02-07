<script setup lang="ts">
/**
 * CardTitle Component
 * 
 * Title element untuk Card
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardTitleProps {
  class?: string
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6'
}

const props = withDefaults(defineProps<CardTitleProps>(), {
  as: 'h3'
})

const titleClasses = computed(() => {
  return cn('text-lg font-semibold leading-none tracking-tight', props.class)
})
</script>

<template>
  <component :is="as" :class="titleClasses">
    <slot />
  </component>
</template>
