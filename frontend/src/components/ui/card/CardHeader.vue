<script setup lang="ts">
/**
 * CardHeader Component
 * 
 * Header section untuk Card
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardHeaderProps {
  class?: string
}

const props = defineProps<CardHeaderProps>()

const headerClasses = computed(() => {
  return cn('flex flex-col space-y-1.5 p-6', props.class)
})
</script>

<template>
  <div :class="headerClasses">
    <slot />
  </div>
</template>
