<script setup lang="ts">
import MainLayout from '@/layouts/MainLayout.vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { computed } from 'vue'
import { 
  Folder, Database, Globe, Mail, 
  Activity, HardDrive
} from 'lucide-vue-next'
import { features } from '@/config/features'

const router = useRouter()
const authStore = useAuthStore()

const navigateTo = (link?: string): void => {
  if (link) {
    router.push(link)
  }
}

/**
 * Real Resource Stats from Auth Store
 */
const resourceStats = computed(() => {
  const usage = authStore.user?.usage
  return [
    { 
      label: 'Disk Usage', 
      val: usage ? `${(usage.disk_used_mb / 1024).toFixed(2)} GB` : '0 GB', 
      total: usage ? `${(usage.disk_limit_mb / 1024).toFixed(0)} GB` : '0 GB',
      icon: HardDrive, 
      color: 'text-blue-500', 
      bg: 'bg-blue-500/10'
    },
    { 
      label: 'Bandwidth', 
      val: usage ? `${(usage.bandwidth_used_mb / 1024).toFixed(2)} GB` : '0 GB', 
      total: usage ? `${(usage.bandwidth_limit_mb / 1024).toFixed(0)} GB` : '0 GB',
      icon: Activity, 
      color: 'text-emerald-500', 
      bg: 'bg-emerald-500/10'
    },
    { 
      label: 'Databases', 
      val: usage ? `${usage.databases_count}` : '0', 
      total: usage ? `${usage.databases_limit}` : '0',
      icon: Database, 
      color: 'text-amber-500', 
      bg: 'bg-amber-500/10'
    },
    { 
      label: 'Email Accounts', 
      val: usage ? `${usage.email_accounts_count}` : '0', 
      total: usage ? `${usage.email_accounts_limit}` : '0',
      icon: Mail, 
      color: 'text-pink-500', 
      bg: 'bg-pink-500/10'
    },
  ]
})

/**
 * Quick Actions
 */
const quickActions = [
  { title: 'File Manager', icon: Folder, link: '/dashboard/files', color: 'text-blue-600', bg: 'bg-blue-100 dark:bg-blue-500/20' },
  { title: 'phpMyAdmin', icon: Database, link: '/dashboard/databases', color: 'text-orange-600', bg: 'bg-orange-100 dark:bg-orange-500/20' },
  { title: 'Email Accounts', icon: Mail, link: '/dashboard/emails', color: 'text-pink-600', bg: 'bg-pink-100 dark:bg-pink-500/20' },
  { title: 'Subdomains', icon: Globe, link: '/dashboard/domains', color: 'text-emerald-600', bg: 'bg-emerald-100 dark:bg-emerald-500/20' },
]
</script>

<template>
  <MainLayout>
    <div class="space-y-8 animate-in pb-10">
      
      <!-- Welcome & Stats Row -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Welcome Card -->
        <div class="lg:col-span-2 relative overflow-hidden bg-card border border-border rounded-2xl p-6 flex flex-col justify-between">
          <div class="relative z-10">
            <h1 class="text-2xl font-bold text-foreground">Welcome back, {{ authStore.user?.username || 'User' }}!</h1>
            <p class="text-muted-foreground mt-2 text-sm max-w-md">
              Your server is currently running smoothly. You have used {{ resourceStats[0]?.val }} of {{ resourceStats[0]?.total }} disk space.
            </p>
            
            <!-- Quick Actions -->
            <div class="mt-8">
              <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">Quick Actions</h3>
              <div class="flex flex-wrap gap-3">
                <button 
                  v-for="action in quickActions"
                  :key="action.title"
                  @click="navigateTo(action.link)"
                  class="flex items-center gap-2 px-4 py-2.5 bg-background border border-border rounded-lg hover:border-primary/50 transition-all hover:shadow-sm"
                >
                  <div :class="['p-1.5 rounded-md', action.bg, action.color]">
                    <component :is="action.icon" class="w-4 h-4" />
                  </div>
                  <span class="text-sm font-medium">{{ action.title }}</span>
                </button>
              </div>
            </div>
          </div>
          
          <!-- Decorative Blob -->
          <div class="absolute top-0 right-0 w-64 h-64 bg-primary/5 rounded-full blur-3xl -translate-y-1/2 translate-x-1/2" />
        </div>

        <!-- Stats Grid -->
        <div class="grid grid-cols-2 gap-3">
          <div 
            v-for="stat in resourceStats" 
            :key="stat.label"
            class="bg-card border border-border p-4 rounded-2xl flex flex-col justify-center gap-2 hover:border-primary/20 transition-colors"
          >
            <div class="flex items-center gap-2">
              <div :class="['p-1.5 rounded-lg', stat.bg, stat.color]">
                <component :is="stat.icon" class="w-4 h-4" />
              </div>
              <span class="text-[10px] font-semibold text-muted-foreground uppercase truncate">{{ stat.label }}</span>
            </div>
            <div>
              <p class="text-lg font-bold text-foreground">{{ stat.val }}</p>
              <p class="text-xs text-muted-foreground">of {{ stat.total }}</p>
            </div>
          </div>
        </div>
      </div>

      <hr class="border-border/50" />

      <!-- Feature Grid -->
      <div v-for="section in features" :key="section.title" class="space-y-4">
        <h3 class="text-base font-bold text-foreground flex items-center gap-2">
          <component :is="section.icon" class="w-4 h-4 text-primary" />
          {{ section.title }}
        </h3>
        
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-6 gap-4">
          <div 
            v-for="card in section.cards" 
            :key="card.title"
            @click="navigateTo(card.link)"
            class="group bg-card border border-border p-4 rounded-xl hover:shadow-lg hover:shadow-primary/5 hover:-translate-y-1 transition-all duration-300 cursor-pointer flex flex-col gap-3 relative overflow-hidden"
          >
            <div :class="['w-10 h-10 rounded-lg flex items-center justify-center bg-background border border-border group-hover:border-primary/20 transition-colors', card.color]">
              <component :is="card.icon" class="w-6 h-6" />
            </div>
            
            <div>
              <h4 class="font-semibold text-xs text-foreground group-hover:text-primary transition-colors line-clamp-1">{{ card.title }}</h4>
              <p class="text-[10px] text-muted-foreground mt-0.5 line-clamp-1">{{ card.description }}</p>
            </div>
          </div>
        </div>
      </div>

    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in {
  animation: fadeUp 0.5s ease-out forwards;
}

@keyframes fadeUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style> */

