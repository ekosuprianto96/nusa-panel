<script setup lang="ts">
import { useToastStore } from '@/stores/toast'
import Toast from './Toast.vue'

const toastStore = useToastStore()
</script>

<template>
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none p-4">
    <Toast 
      v-for="toast in toastStore.toasts" 
      :key="toast.id"
      :id="toast.id"
      :message="toast.message"
      :type="toast.type"
      @close="toastStore.remove"
    />
  </div>
</template>
