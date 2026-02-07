<script setup lang="ts">
/**
 * Input Component
 * 
 * Text input dengan variant system dan v-model support
 * 
 * @example
 * <Input v-model="email" placeholder="Enter email" />
 * <Input v-model="password" type="password" variant="error" />
 * <Input v-model="name" size="lg" />
 */

import { useVModel } from '@vueuse/core'
import { computed } from 'vue'
import { cn } from '@/design-system/utils/cn'
import { inputVariants, type InputVariants } from './input.variants'

// ============================================
// PROPS
// ============================================

interface InputProps {
  /** Visual variant */
  variant?: InputVariants['variant']
  /** Size variant */
  size?: InputVariants['size']
  /** Default value (uncontrolled) */
  defaultValue?: string | number
  /** Model value (v-model) */
  modelValue?: string | number
  /** Additional classes */
  class?: string
  /** Input type */
  type?: string
}

const props = withDefaults(defineProps<InputProps>(), {
  variant: 'default',
  size: 'default',
  type: 'text'
})

// ============================================
// EMITS
// ============================================

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
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

const inputClasses = computed(() => {
  return cn(
    inputVariants({ variant: props.variant, size: props.size }),
    props.class
  )
})
</script>

<template>
  <input
    v-model="modelValue"
    :type="type"
    :class="inputClasses"
  >
</template>
