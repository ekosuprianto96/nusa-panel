<script setup lang="ts">
/**
 * CardContent Component
 * 
 * Main content area untuk Card
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardContentProps {
  class?: string
}

const props = defineProps<CardContentProps>()

const contentClasses = computed(() => {
  return cn('p-6 pt-0', props.class)
})
</script>

<template>
  <div :class="contentClasses">
    <slot />
  </div>
</template>
