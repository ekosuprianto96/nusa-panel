<script setup lang="ts">
/**
 * CardDescription Component
 * 
 * Description text untuk Card
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardDescriptionProps {
  class?: string
}

const props = defineProps<CardDescriptionProps>()

const descriptionClasses = computed(() => {
  return cn('text-sm text-muted-foreground', props.class)
})
</script>

<template>
  <p :class="descriptionClasses">
    <slot />
  </p>
</template>
