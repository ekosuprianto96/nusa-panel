<script setup lang="ts">
/**
 * Textarea Component
 * 
 * Multi-line text input dengan styling konsisten
 * 
 * @example
 * <Textarea v-model="message" placeholder="Enter your message" />
 * <Textarea v-model="bio" :rows="6" />
 */

import { useVModel } from '@vueuse/core'
import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'

// ============================================
// PROPS
// ============================================

interface TextareaProps {
  /** Default value (uncontrolled) */
  defaultValue?: string
  /** Model value (v-model) */
  modelValue?: string
  /** Additional classes */
  class?: string
  /** Number of visible rows */
  rows?: number
  /** Disable resize */
  noResize?: boolean
}

const props = withDefaults(defineProps<TextareaProps>(), {
  rows: 4,
  noResize: false
})

// ============================================
// EMITS
// ============================================

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string): void
}>()

// ============================================
// STATE
// ============================================

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue
})

// ============================================
// COMPUTED
// ============================================

const textareaClasses = computed(() => {
  return cn(
    'flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2',
    'text-base ring-offset-background placeholder:text-muted-foreground',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
    'disabled:cursor-not-allowed disabled:opacity-50',
    'md:text-sm',
    props.noResize && 'resize-none',
    props.class
  )
})
</script>

<template>
  <textarea
    v-model="modelValue"
    :rows="rows"
    :class="textareaClasses"
  />
</template>
