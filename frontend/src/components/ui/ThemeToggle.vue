<script setup lang="ts">
import { computed } from 'vue'
import { useThemeStore } from '@/stores/theme'
import { Sun, Moon } from 'lucide-vue-next'

/**
 * ThemeToggle Component
 * Tombol toggle untuk beralih antara tema light dan dark
 */
const themeStore = useThemeStore()

const isDark = computed(() => themeStore.theme === 'dark')

/**
 * Handler untuk toggle tema
 */
const handleToggle = (): void => {
  themeStore.toggleTheme()
}
</script>

<template>
  <button
    @click="handleToggle"
    :title="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'"
    class="p-3 rounded-xl transition-all duration-300 group relative overflow-hidden"
    :class="[
      isDark 
        ? 'bg-slate-800 text-amber-400 hover:bg-slate-700' 
        : 'text-slate-400 hover:bg-slate-100 hover:text-blue-600'
    ]"
  >
    <!-- Sun Icon (Light Mode) -->
    <Sun 
      v-if="isDark" 
      class="w-5 h-5 transition-transform duration-300 group-hover:rotate-12" 
    />
    <!-- Moon Icon (Dark Mode) -->
    <Moon 
      v-else 
      class="w-5 h-5 transition-transform duration-300 group-hover:-rotate-12" 
    />
    
    <!-- Glow effect -->
    <div 
      v-if="isDark"
      class="absolute inset-0 bg-amber-400/10 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity"
    />
  </button>
</template>
