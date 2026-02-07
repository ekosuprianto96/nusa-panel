<script setup lang="ts">
/**
 * Button Component
 * 
 * Reusable button dengan variant system, loading state, dan slot support
 * 
 * @example
 * <Button variant="default" size="lg">Click me</Button>
 * <Button variant="destructive" :loading="isLoading">Delete</Button>
 * <Button variant="outline" as-child>
 *   <router-link to="/home">Go Home</router-link>
 * </Button>
 */

import { computed, useAttrs } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import { cn } from '@/design-system/utils/cn'
import { buttonVariants, type ButtonVariants } from './button.variants'

// ============================================
// PROPS
// ============================================

interface ButtonProps {
  /** Visual variant */
  variant?: ButtonVariants['variant']
  /** Size variant */
  size?: ButtonVariants['size']
  /** Show loading spinner */
  loading?: boolean
  /** Disabled state */
  disabled?: boolean
  /** Render as child element (untuk wrapping links, etc) */
  asChild?: boolean
  /** Additional classes */
  class?: string
}

const props = withDefaults(defineProps<ButtonProps>(), {
  variant: 'default',
  size: 'default',
  loading: false,
  disabled: false,
  asChild: false
})

// ============================================
// COMPUTED
// ============================================

const attrs = useAttrs()

const buttonClasses = computed(() => {
  return cn(
    buttonVariants({ variant: props.variant, size: props.size }),
    props.loading && 'cursor-wait',
    props.class
  )
})

const isDisabled = computed(() => props.disabled || props.loading)
</script>

<template>
  <!-- Render slot content directly when asChild is true -->
  <slot v-if="asChild" v-bind="{ class: buttonClasses, disabled: isDisabled, ...attrs }" />
  
  <!-- Default button rendering -->
  <button
    v-else
    :class="buttonClasses"
    :disabled="isDisabled"
    v-bind="attrs"
  >
    <!-- Loading spinner -->
    <Loader2 
      v-if="loading" 
      class="animate-spin" 
      :class="{ 'mr-2': $slots.default }"
    />
    
    <!-- Button content -->
    <slot />
  </button>
</template>
