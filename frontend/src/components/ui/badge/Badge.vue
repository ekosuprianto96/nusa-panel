<script setup lang="ts">
/**
 * Badge Component
 * 
 * Status indicator/label dengan variant system
 * 
 * @example
 * <Badge>Default</Badge>
 * <Badge variant="success">Active</Badge>
 * <Badge variant="destructive">Error</Badge>
 * <Badge variant="outline">Beta</Badge>
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'
import { badgeVariants, type BadgeVariants } from './badge.variants'

// ============================================
// PROPS
// ============================================

interface BadgeProps {
  /** Visual variant */
  variant?: BadgeVariants['variant']
  /** Size variant */
  size?: BadgeVariants['size']
  /** Additional classes */
  class?: string
}

const props = withDefaults(defineProps<BadgeProps>(), {
  variant: 'default',
  size: 'default'
})

// ============================================
// COMPUTED
// ============================================

const badgeClasses = computed(() => {
  return cn(
    badgeVariants({ variant: props.variant, size: props.size }),
    props.class
  )
})
</script>

<template>
  <div :class="badgeClasses">
    <slot />
  </div>
</template>
