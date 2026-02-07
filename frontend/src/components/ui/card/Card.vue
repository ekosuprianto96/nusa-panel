<script setup lang="ts">
/**
 * Card Component
 * 
 * Container component untuk grouping content
 * 
 * @example
 * <Card>
 *   <CardHeader>
 *     <CardTitle>Title</CardTitle>
 *     <CardDescription>Description</CardDescription>
 *   </CardHeader>
 *   <CardContent>
 *     Content here
 *   </CardContent>
 *   <CardFooter>
 *     Footer actions
 *   </CardFooter>
 * </Card>
 */

import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

interface CardProps {
  /** Additional classes */
  class?: string
  /** Hover effect */
  hoverable?: boolean
  /** Clickable appearance */
  clickable?: boolean
}

const props = withDefaults(defineProps<CardProps>(), {
  hoverable: false,
  clickable: false
})

const cardClasses = computed(() => {
  return cn(
    'rounded-xl border bg-card text-card-foreground shadow-sm',
    props.hoverable && 'transition-shadow hover:shadow-md',
    props.clickable && 'cursor-pointer transition-all hover:shadow-md hover:border-primary/50',
    props.class
  )
})
</script>

<template>
  <div :class="cardClasses">
    <slot />
  </div>
</template>
