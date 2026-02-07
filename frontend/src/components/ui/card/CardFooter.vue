<script setup lang="ts">
/**
 * CardFooter Component
 * 
 * Footer section untuk Card - biasanya untuk actions
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardFooterProps {
  class?: string
}

const props = defineProps<CardFooterProps>()

const footerClasses = computed(() => {
  return cn('flex items-center p-6 pt-0', props.class)
})
</script>

<template>
  <div :class="footerClasses">
    <slot />
  </div>
</template>
