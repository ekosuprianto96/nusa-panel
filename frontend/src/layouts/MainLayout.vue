<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import ThemeToggle from '@/components/ui/ThemeToggle.vue'
import { 
  Mail, Shield, Search, LogOut,
  Settings, Menu, Folder, Database, Globe,
  LayoutDashboard, Server, Zap, Users, Package

} from 'lucide-vue-next'

import { features, type DashboardCard } from '@/config/features'
import { adminFeatures } from '@/config/admin-features'
import ToastContainer from '@/components/ui/ToastContainer.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const userRole = computed(() => authStore.user?.role || 'user')
const isAdmin = computed(() => ['admin', 'reseller'].includes(userRole.value))

const isSidebarOpen = ref(true)
const searchQuery = ref('')
const isMobile = ref(false)

onMounted(async () => {
  checkScreenSize()
  window.addEventListener('resize', checkScreenSize)
  
  if (!authStore.user && authStore.accessToken) {
    await authStore.fetchMe()
  }
})

const checkScreenSize = () => {
  isMobile.value = window.innerWidth < 1024
  if (isMobile.value) {
    isSidebarOpen.value = false
  } else {
    isSidebarOpen.value = true
  }
}

const toggleSidebar = () => {
  isSidebarOpen.value = !isSidebarOpen.value
}

const handleLogout = (): void => {
  authStore.logout()
  router.push('/auth/login')
}

/**
 * Search Logic
 */
const filteredFeatures = computed(() => {
  if (!searchQuery.value) return []
  
  const query = searchQuery.value.toLowerCase() // English consistency
  const results: DashboardCard[] = []
  const activeFeatures = isAdmin.value ? adminFeatures : features

  activeFeatures.forEach(section => {
    section.cards.forEach(card => {
      if (
        card.title.toLowerCase().includes(query) || 
        (card.description && card.description.toLowerCase().includes(query))
      ) {
        results.push(card)
      }
    })
  })

  return results
})

const handleSearchClick = (feature: DashboardCard) => {
  if (feature.link) {
    router.push(feature.link)
    searchQuery.value = '' // Clear search after navigation
  }
}

// Navigation structure
interface NavGroup {
  title: string
  items: NavItem[]
}

interface NavItem {
  name: string
  icon: any
  path: string
  openNewTab?: boolean
  badge?: string
  roles?: string[]  // Roles that can access this menu. Empty/undefined = all roles
}

const navigation: NavGroup[] = [
  ...(isAdmin.value
    ? [
        {
          title: 'Overview',
          items: [
            { name: 'Admin Dashboard', icon: LayoutDashboard, path: '/admin' },
          ],
        },
        {
          title: 'Administration',
          items: [
            { name: 'Users', icon: Users, path: '/dashboard/users', roles: ['admin', 'reseller'] },
            { name: 'Packages', icon: Package, path: '/dashboard/packages', roles: ['admin'] },
          ],
        },
        {
          title: 'Server',
          items: [
            { name: 'Web Server', icon: Zap, path: '/dashboard/web-server', roles: ['admin', 'reseller'] },
          ],
        },
      ]
    : [
        {
          title: 'Overview',
          items: [
            { name: 'Dashboard', icon: LayoutDashboard, path: '/dashboard' },
          ],
        },
        {
          title: 'Management',
          items: [
            { name: 'File Manager', icon: Folder, path: '/file-manager', openNewTab: true },
            { name: 'Databases', icon: Database, path: '/dashboard/databases' },
            { name: 'Domains', icon: Globe, path: '/dashboard/domains' },
            { name: 'Email Accounts', icon: Mail, path: '/dashboard/emails' },
          ],
        },
        {
          title: 'Server & Security',
          items: [
            { name: 'Security', icon: Shield, path: '/dashboard/security' },
            { name: 'System', icon: Settings, path: '/dashboard/system' },
            { name: 'Apps', icon: Server, path: '/dashboard/apps' },
          ],
        },
      ])
]

// Filter navigation based on user role
const filteredNavigation = computed(() => {
  const role = userRole.value
  return navigation
    .map(group => ({
      ...group,
      items: group.items.filter(item => {
        // If no roles specified, show to all
        if (!item.roles || item.roles.length === 0) return true
        // Check if user role is in allowed roles
        return item.roles.includes(role)
      })
    }))
    .filter(group => group.items.length > 0)  // Remove empty groups
})

const handleNavClick = (item: NavItem): void => {
  if (item.openNewTab) {
    window.open(item.path, '_blank')
  } else {
    router.push(item.path)
    if (isMobile.value) isSidebarOpen.value = false
  }
}



// Breadcrumbs
const breadcrumb = computed(() => {
  const path = route.path
  if (path === '/dashboard') return [{ name: 'Dashboard', isLast: true }]
  if (path === '/admin') return [{ name: 'Admin', isLast: true }]
  
  const isAdminRoute = path.startsWith('/admin')
  const parts = path.split('/').filter(p => p && p !== 'dashboard' && p !== 'admin')
  return [
    { name: isAdminRoute ? 'Admin' : 'Dashboard', isLast: false },
    ...parts.map((p, i) => ({
      name: p.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' '),
      isLast: i === parts.length - 1
    }))
  ]
})
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background text-foreground font-sans">
    
    <!-- Sidebar Backdrop (Mobile) -->
    <div 
      v-if="isMobile && isSidebarOpen" 
      class="fixed inset-0 bg-background/80 backdrop-blur-sm z-30 lg:hidden"
      @click="isSidebarOpen = false"
    />

    <!-- Sidebar -->
    <aside 
      :class="[
        'fixed lg:static inset-y-0 left-0 z-40 bg-card border-r border-border transition-all duration-300 flex flex-col',
        isSidebarOpen ? 'w-64 translate-x-0' : 'w-0 lg:w-20 -translate-x-full lg:translate-x-0'
      ]"
    >
      <!-- Logo -->
      <div class="h-16 flex items-center px-6 border-b border-border shadow-sm">
        <div class="flex items-center gap-3 overflow-hidden">
          <div class="w-8 h-8 rounded-lg bg-primary flex-shrink-0 flex items-center justify-center text-primary-foreground font-bold shadow-lg shadow-primary/20">
            N
          </div>
          <span 
            :class="['font-bold text-lg tracking-tight transition-opacity duration-300', isSidebarOpen ? 'opacity-100' : 'opacity-0 lg:hidden']"
          >
            NusaPanel
          </span>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 overflow-y-auto py-6 px-3 custom-scrollbar flex flex-col gap-6">
        <div v-for="group in filteredNavigation" :key="group.title">
          <h3 
            v-if="isSidebarOpen"
            class="px-3 mb-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider animate-in fade-in"
          >
            {{ group.title }}
          </h3>
          <div class="space-y-1">
            <button
              v-for="item in group.items"
              :key="item.name"
              @click="handleNavClick(item)"
              :title="!isSidebarOpen ? item.name : ''"
              :class="[
                'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 group relative',
                route.path.startsWith(item.path) && item.path !== '/' 
                  ? 'bg-primary/10 text-primary' 
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground'
              ]"
            >
              <component :is="item.icon" class="w-5 h-5 flex-shrink-0" />
              <span 
                :class="['whitespace-nowrap transition-all duration-300', isSidebarOpen ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-4 absolute left-10 lg:hidden']"
              >
                {{ item.name }}
              </span>
              
              <!-- Active Indicator (Collapsed) -->
              <div 
                v-if="!isSidebarOpen && route.path.startsWith(item.path)" 
                class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-primary rounded-r-full"
              />
            </button>
          </div>
        </div>
      </nav>

      <!-- User Profile (Bottom) -->
      <div class="p-4 border-t border-border mt-auto">
        <div 
          class="flex items-center gap-3 p-2 rounded-xl hover:bg-muted transition-colors cursor-pointer"
          @click="toggleSidebar"
        >
          <div class="w-9 h-9 rounded-full bg-gradient-to-tr from-primary to-purple-500 flex items-center justify-center text-white font-bold text-xs ring-2 ring-background">
            {{ authStore.user?.username?.substring(0, 2).toUpperCase() || 'US' }}
          </div>
          <div :class="['flex-1 overflow-hidden transition-all duration-300', isSidebarOpen ? 'opacity-100 w-auto' : 'opacity-0 w-0 lg:hidden']">
            <p class="text-sm font-semibold truncate">{{ authStore.user?.username || 'User' }}</p>
            <p class="text-xs text-muted-foreground truncate">{{ authStore.user?.email || 'user@example.com' }}</p>
          </div>
          <ThemeToggle v-if="isSidebarOpen" />
        </div>
      </div>
    </aside>

    <!-- Main Content wrapper -->
    <div class="flex-1 flex flex-col min-w-0 bg-background transition-all duration-300">
      
      <!-- Top Header -->
      <header class="h-16 border-b border-border bg-card/50 backdrop-blur sticky top-0 z-20 px-4 lg:px-8 flex items-center justify-between gap-4">
        
        <!-- Left: Toggle & Breadcrumbs -->
        <div class="flex items-center gap-4">
          <button 
            @click="toggleSidebar"
            class="p-2 rounded-lg hover:bg-muted text-muted-foreground transition-colors"
          >
            <Menu class="w-5 h-5" />
          </button>

          <div class="hidden md:flex items-center gap-2 text-sm">
            <template v-for="(crumb, idx) in breadcrumb" :key="idx">
              <span v-if="idx > 0" class="text-muted-foreground/40">/</span>
              <span :class="crumb.isLast ? 'text-foreground font-medium' : 'text-muted-foreground'">
                {{ crumb.name }}
              </span>
            </template>
          </div>
        </div>

        <!-- Right: Search & Server Info -->
        <div class="flex items-center gap-4">
          <!-- Server IP Badge -->
          <div class="hidden lg:flex items-center px-3 py-1.5 bg-muted/50 rounded-full border border-border gap-2">
            <div class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span class="text-xs font-mono text-muted-foreground">{{ authStore.user?.username ? '192.168.1.42' : 'Connecting...' }}</span>
          </div>

          <!-- Search -->
          <div class="relative hidden sm:block">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input 
              v-model="searchQuery"
              type="text" 
              placeholder="Search features..." 
              class="pl-9 pr-4 py-1.5 bg-background border border-border rounded-lg text-sm focus:ring-1 focus:ring-primary focus:border-primary outline-none w-64 transition-all"
            >
            <div class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
              <kbd class="hidden md:inline-flex h-5 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
                <span class="text-xs">⌘</span>K
              </kbd>
            </div>

            <!-- Search Results Dropdown -->
            <div 
              v-if="searchQuery && filteredFeatures.length > 0"
              class="absolute top-full left-0 right-0 mt-2 bg-card border border-border rounded-xl shadow-xl overflow-hidden z-50 max-h-96 overflow-y-auto"
            >
              <div class="p-2 space-y-1">
                <button
                  v-for="feature in filteredFeatures"
                  :key="feature.title"
                  @click="handleSearchClick(feature)"
                  class="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-muted text-left transition-colors group"
                >
                  <div :class="['p-1.5 rounded-md flex-shrink-0', feature.bgLight || 'bg-background', feature.color]">
                    <component :is="feature.icon" class="w-4 h-4" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium text-foreground truncate group-hover:text-primary transition-colors">
                      {{ feature.title }}
                    </p>
                    <p class="text-[10px] text-muted-foreground truncate">
                      {{ feature.description }}
                    </p>
                  </div>
                </button>
              </div>
            </div>
             <div 
              v-else-if="searchQuery"
              class="absolute top-full left-0 right-0 mt-2 bg-card border border-border rounded-xl shadow-xl z-50 p-4 text-center text-sm text-muted-foreground"
            >
              No results found.
            </div>
          </div>

          <!-- Logout -->
          <button 
            @click="handleLogout"
            class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
            title="Logout"
          >
            <LogOut class="w-5 h-5" />
          </button>
        </div>
      </header>

      <!-- Main Scrollable Area -->
      <main class="flex-1 overflow-y-auto p-4 lg:p-8 custom-scrollbar">
        <div class="max-w-7xl mx-auto space-y-8">
          <slot />
        </div>
      </main>

    </div>
  </div>
  <ToastContainer />
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: hsl(var(--muted-foreground) / 0.2);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: hsl(var(--muted-foreground) / 0.4);
}
</style>

