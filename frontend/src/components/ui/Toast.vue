<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-vue-next'
import type { ToastType } from '@/stores/toast'

const props = defineProps<{
  id: string
  message: string
  type: ToastType
}>()

const emit = defineEmits<{
  (e: 'close', id: string): void
}>()

const isVisible = ref(false)

onMounted(() => {
  // Trigger enter animation
  setTimeout(() => isVisible.value = true, 50)
})

const close = () => {
  isVisible.value = false
  setTimeout(() => emit('close', props.id), 300) // Wait for exit animation
}

const styles = computed(() => {
  switch (props.type) {
    case 'success':
      return 'bg-emerald-50 border-emerald-200 text-emerald-800 dark:bg-emerald-900/30 dark:border-emerald-500/30 dark:text-emerald-400'
    case 'error':
      return 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/30 dark:border-red-500/30 dark:text-red-400'
    case 'warning':
      return 'bg-amber-50 border-amber-200 text-amber-800 dark:bg-amber-900/30 dark:border-amber-500/30 dark:text-amber-400'
    default:
      return 'bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/30 dark:border-blue-500/30 dark:text-blue-400'
  }
})

const icon = computed(() => {
  switch (props.type) {
    case 'success': return CheckCircle
    case 'error': return AlertCircle
    case 'warning': return AlertTriangle
    default: return Info
  }
})
</script>

<template>
  <div 
    class="flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg border transition-all duration-300 transform pointer-events-auto min-w-[300px] max-w-md"
    :class="[
      styles,
      isVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'
    ]"
  >
    <component :is="icon" class="w-5 h-5 flex-shrink-0 mt-0.5" />
    <div class="flex-1 text-sm font-medium">{{ message }}</div>
    <button @click="close" class="opacity-70 hover:opacity-100 transition-opacity">
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
