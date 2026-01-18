<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import ThemeToggle from '@/components/ui/ThemeToggle.vue'
import { 
  BarChart3, Folder, Database, Globe, 
  Mail, Shield, Search, LogOut, Plus,
  Settings
} from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()
const searchQuery = ref('')

onMounted(async () => {
  if (!authStore.user && authStore.accessToken) {
    await authStore.fetchMe()
  }
})

/**
 * Handle user logout
 */
const handleLogout = (): void => {
  authStore.logout()
  router.push('/auth/login')
}

/**
 * Item navigasi utama
 */
interface NavItem {
  name: string
  icon: any
  path: string
  openNewTab?: boolean
}

const mainNav: NavItem[] = [
  { name: 'Dashboard', icon: BarChart3, path: '/dashboard' },
]

const managementNav: NavItem[] = [
  { name: 'Files', icon: Folder, path: '/file-manager', openNewTab: true },
  { name: 'Databases', icon: Database, path: '/dashboard/databases' },
  { name: 'Domains', icon: Globe, path: '/dashboard/domains' },
  { name: 'Email', icon: Mail, path: '/dashboard/emails' },
  { name: 'Security', icon: Shield, path: '/dashboard/security' },
  { name: 'System', icon: Settings, path: '/dashboard/system' },
]

/**
 * Handle navigasi item - bisa open new tab atau router push
 */
const handleNavClick = (item: NavItem): void => {
  if (item.openNewTab) {
    window.open(item.path, '_blank')
  } else {
    router.push(item.path)
  }
}

/**
 * Statistik resource usage dari user data
 */
const stats = computed(() => {
  const usage = authStore.user?.usage
  return [
    { 
      label: 'Disk Usage', 
      value: usage ? `${(usage.disk_used_mb / 1024).toFixed(1)} GB / ${(usage.disk_limit_mb / 1024).toFixed(0)} GB` : '0 / 0 GB', 
      color: 'bg-blue-500', 
      percent: usage ? (usage.disk_used_mb / usage.disk_limit_mb) * 100 : 0 
    },
    { 
      label: 'Bandwidth', 
      value: usage ? `${(usage.bandwidth_used_mb / 1024).toFixed(0)} GB / ${(usage.bandwidth_limit_mb / 1024).toFixed(0)} GB` : '0 / 0 GB', 
      color: 'bg-emerald-500', 
      percent: usage ? (usage.bandwidth_used_mb / usage.bandwidth_limit_mb) * 100 : 0 
    },
    { 
      label: 'MySQL Disk Usage', 
      value: usage ? `${usage.databases_count} DBs` : '0 DBs', 
      color: 'bg-amber-500', 
      percent: usage ? (usage.databases_count / usage.databases_limit) * 100 : 0 
    },
    { 
      label: 'Email Accounts', 
      value: usage ? `${usage.email_accounts_count} / ${usage.email_accounts_limit}` : '0 / 0', 
      color: 'bg-pink-500', 
      percent: usage ? (usage.email_accounts_count / usage.email_accounts_limit) * 100 : 0 
    },
  ]
})

/**
 * Info server (demo data, bisa diganti dari API)
 */
const serverInfo = [
  { label: 'STATUS', value: 'Operational', type: 'badge' },
  { label: 'Domain', value: authStore.user?.email?.split('@')[1] || 'mysite.com', type: 'text' },
  { label: 'IP Address', value: '192.168.1.42', type: 'text' },
  { label: 'Server Name', value: 'srv-alpha-01', type: 'text' }
]

/**
 * Breadcrumb dari current route
 */
const breadcrumb = computed(() => {
  const path = router.currentRoute.value.path
  const parts = path.split('/').filter(p => p)
  return parts.map((p, i) => ({
    name: p.charAt(0).toUpperCase() + p.slice(1),
    isLast: i === parts.length - 1
  }))
})
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background text-foreground font-sans transition-colors duration-300">
    
    <!-- Slim Left Sidebar -->
    <aside class="w-20 flex-shrink-0 bg-card border-r border-border flex flex-col hidden lg:flex items-center py-6 gap-8 transition-colors duration-300">
      <!-- Logo -->
      <div class="flex items-center justify-center">
        <div class="w-12 h-12 bg-primary rounded-xl flex items-center justify-center text-primary-foreground shadow-lg shadow-primary/20">
          <svg class="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
        </div>
      </div>

      <!-- Nav Items -->
      <nav class="flex-1 flex flex-col items-center gap-4">
        <button
          v-for="item in [...mainNav, ...managementNav]"
          :key="item.name"
          @click="handleNavClick(item)"
          :title="item.name"
          :class="[
            'p-3 rounded-xl transition-all duration-200 group relative',
            router.currentRoute.value.path.startsWith(item.path) 
              ? 'bg-primary/10 text-primary' 
              : 'text-muted-foreground hover:bg-muted hover:text-primary'
          ]"
        >
          <component :is="item.icon" class="w-5 h-5" />
          <!-- Active Indicator -->
          <div 
            v-if="router.currentRoute.value.path.startsWith(item.path)" 
            class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-primary rounded-r-full"
          />
        </button>
      </nav>

      <!-- Bottom Actions -->
      <div class="flex flex-col items-center gap-3">
        <!-- Theme Toggle -->
        <ThemeToggle />
        
        <!-- Logout -->
        <button 
          @click="handleLogout"
          title="Logout"
          class="p-3 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-xl transition-all"
        >
          <LogOut class="w-5 h-5" />
        </button>
      </div>
    </aside>

    <!-- Main Section -->
    <main class="flex-1 flex flex-col min-w-0 overflow-hidden relative">
      <!-- Header -->
      <header class="h-16 flex items-center justify-between px-8 shrink-0 z-20 border-b border-border bg-card transition-colors duration-300">
        <!-- Breadcrumb -->
        <div class="flex items-center gap-2 text-sm">
          <span class="text-muted-foreground">Home</span>
          <template v-for="(crumb, idx) in breadcrumb" :key="idx">
            <span class="text-muted-foreground/50">/</span>
            <span :class="crumb.isLast ? 'text-foreground font-semibold' : 'text-muted-foreground'">
              {{ crumb.name }}
            </span>
          </template>
        </div>

        <!-- Center Search -->
        <div class="max-w-xl w-full mx-8 hidden md:block">
          <div class="relative w-full group">
            <Search class="w-4 h-4 absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <input
              v-model="searchQuery"
              class="w-full bg-muted border border-border rounded-xl pl-10 pr-12 py-2.5 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none placeholder:text-muted-foreground transition-all"
              placeholder="Find functions quickly (e.g. 'MySQL', 'Email')..."
              type="text"
            />
            <div class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground/50 pointer-events-none px-1.5 py-0.5 bg-muted rounded border border-border text-[10px] font-medium">
              ⌘K
            </div>
          </div>
        </div>

        <!-- User Profile -->
        <div class="flex items-center gap-3">
          <div class="text-right hidden sm:block">
            <p class="text-sm font-semibold text-foreground leading-none">{{ authStore.user?.username || 'Admin' }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">{{ authStore.user?.email || 'admin@example.com' }}</p>
          </div>
          <div class="w-9 h-9 bg-primary/10 text-primary rounded-full flex items-center justify-center text-sm font-bold border border-primary/20 transition-transform hover:scale-105 cursor-pointer">
            {{ authStore.user?.username?.substring(0, 2).toUpperCase() || 'AM' }}
          </div>
        </div>
      </header>

      <!-- Content Area -->
      <div class="flex-1 flex overflow-hidden">
        <div class="flex-1 overflow-y-auto p-8 custom-scrollbar bg-background transition-colors duration-300">
          <slot />
        </div>

        <!-- Right Server Info Panel -->
        <aside class="w-72 flex-shrink-0 bg-card border-l border-border p-6 overflow-y-auto hidden xl:block custom-scrollbar transition-colors duration-300">
          <!-- Server Information -->
          <div class="mb-8">
            <div class="flex items-center gap-2 mb-6">
              <div class="p-1.5 bg-primary/10 rounded-lg">
                <BarChart3 class="w-4 h-4 text-primary" />
              </div>
              <h3 class="text-sm font-bold text-foreground">Server Information</h3>
            </div>
            
            <div class="space-y-3">
              <div v-for="info in serverInfo" :key="info.label" class="flex items-center justify-between text-xs">
                <span class="text-muted-foreground">{{ info.label }}</span>
                <span 
                  v-if="info.type === 'badge'" 
                  class="bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 px-2.5 py-1 rounded-full font-medium flex items-center gap-1.5 border border-emerald-500/20"
                >
                  <span class="w-1.5 h-1.5 bg-emerald-500 rounded-full animate-pulse" />
                  {{ info.value }}
                </span>
                <span v-else class="text-foreground font-medium font-mono text-xs">{{ info.value }}</span>
              </div>
            </div>
          </div>

          <!-- Resource Usage -->
          <div>
            <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider mb-4">Resource Usage</h4>
            <div class="space-y-5">
              <div v-for="stat in stats" :key="stat.label">
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-xs text-muted-foreground">{{ stat.label }}</span>
                  <span class="text-xs font-semibold text-foreground">{{ stat.value }}</span>
                </div>
                <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                  <div 
                    :class="['h-full rounded-full transition-all duration-700', stat.color]" 
                    :style="{ width: `${Math.min(stat.percent, 100)}%` }"
                  />
                </div>
                <p class="text-[10px] text-muted-foreground mt-1">{{ Math.round(stat.percent || 0) }}% Used</p>
              </div>
            </div>

            <button class="w-full mt-8 py-3 px-4 bg-primary text-primary-foreground text-xs font-semibold rounded-xl hover:bg-primary/90 transition-all shadow-lg shadow-primary/20 active:scale-[0.98] flex items-center justify-center gap-2">
              <Plus class="w-4 h-4" />
              Create New Database
            </button>
          </div>
        </aside>
      </div>
    </main>
  </div>
</template>

<style>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: hsl(var(--border));
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: hsl(var(--muted-foreground) / 0.3);
}
</style>
