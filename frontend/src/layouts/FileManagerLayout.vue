<script setup lang="ts">
/**
 * FileManagerLayout - Layout khusus untuk File Manager standalone
 * Memiliki slim sidebar dengan icon-only navigation
 */
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import ThemeToggle from '@/components/ui/ThemeToggle.vue'
import FtpAccountsPanel from '@/components/files/FtpAccountsPanel.vue'
import { 
  Home, Folder, Server, Settings, LogOut
} from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()

/** FTP Panel visibility */
const showFtpPanel = ref(false)

/**
 * Handle logout user
 */
const handleLogout = (): void => {
  authStore.logout()
  router.push('/auth/login')
}

/**
 * Navigasi ke dashboard
 */
const goToDashboard = (): void => {
  window.open('/dashboard', '_blank')
}

/**
 * Open FTP Panel
 */
const openFtpPanel = (): void => {
  showFtpPanel.value = true
}

/**
 * Sidebar navigation items
 * Terminal menu dihapus - diakses dari Dashboard
 */
const sidebarItems = [
  { name: 'Home', icon: Home, action: goToDashboard, active: false },
  { name: 'Files', icon: Folder, action: () => {}, active: true },
  { name: 'FTP', icon: Server, action: openFtpPanel, active: false },
  { name: 'Settings', icon: Settings, action: () => {}, active: false },
]
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background text-foreground font-sans transition-colors duration-300">
    <!-- Slim Icon Sidebar -->
    <aside class="w-16 flex-shrink-0 bg-card border-r border-border flex flex-col items-center py-4 gap-2 transition-colors duration-300">
      <!-- Logo -->
      <div class="w-10 h-10 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-lg shadow-primary/20 mb-4">
        <Folder class="w-5 h-5" />
      </div>

      <!-- Nav Items -->
      <nav class="flex-1 flex flex-col items-center gap-2">
        <button
          v-for="item in sidebarItems"
          :key="item.name"
          @click="item.action"
          :title="item.name"
          :class="[
            'w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-200 relative group',
            item.active 
              ? 'bg-primary/10 text-primary' 
              : 'text-muted-foreground hover:bg-muted hover:text-primary'
          ]"
        >
          <component :is="item.icon" class="w-5 h-5" />
          <!-- Active indicator -->
          <div 
            v-if="item.active" 
            class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-5 bg-primary rounded-r-full"
          />
          <!-- Tooltip -->
          <div class="absolute left-full ml-2 px-2 py-1 bg-foreground text-background text-xs rounded opacity-0 group-hover:opacity-100 pointer-events-none whitespace-nowrap z-50 transition-opacity">
            {{ item.name }}
          </div>
        </button>
      </nav>

      <!-- Bottom actions -->
      <div class="flex flex-col items-center gap-2">
        <ThemeToggle />
        <button 
          @click="handleLogout"
          title="Logout"
          class="w-10 h-10 rounded-xl flex items-center justify-center text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all"
        >
          <LogOut class="w-5 h-5" />
        </button>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 overflow-hidden">
      <slot />
    </main>

    <!-- FTP Panel -->
    <FtpAccountsPanel 
      :visible="showFtpPanel" 
      @close="showFtpPanel = false" 
    />
  </div>
</template>

<style scoped>
.animate-in {
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
