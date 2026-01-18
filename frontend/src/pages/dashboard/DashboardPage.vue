<script setup lang="ts">
import MainLayout from '@/layouts/MainLayout.vue'
import { useRouter } from 'vue-router'
import { 
  Folder, Database, Globe, Mail, Shield, 
  FileText, Image as ImageIcon, Lock, 
  Save, Cloud, ChevronRight, Settings2, 
  Activity, MailPlus, Zap, ShieldCheck, 
  ShieldAlert, Cpu, HardDrive
} from 'lucide-vue-next'

const router = useRouter()

/**
 * Interface untuk card pada dashboard
 */
interface DashboardCard {
  title: string
  icon: any
  color: string
  bgLight: string
  bgDark: string
  link?: string
  description?: string
}

/**
 * Interface untuk section dashboard
 */
interface DashboardSection {
  title: string
  icon: any
  cards: DashboardCard[]
}

/**
 * Definisi section dan card untuk dashboard
 */
const sections: DashboardSection[] = [
  {
    title: 'Files',
    icon: Folder,
    cards: [
      { title: 'File Manager', icon: FileText, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/files', description: 'Explore files' },
      { title: 'Images', icon: ImageIcon, color: 'text-orange-600 dark:text-orange-400', bgLight: 'bg-orange-50', bgDark: 'dark:bg-orange-500/10', link: '/dashboard/files', description: 'Optimize assets' },
      { title: 'Directory Privacy', icon: Lock, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/files', description: 'Secure folders' },
      { title: 'Backup Wizard', icon: Save, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/system', description: 'Save your data' },
      { title: 'FTP Accounts', icon: Cloud, color: 'text-purple-600 dark:text-purple-400', bgLight: 'bg-purple-50', bgDark: 'dark:bg-purple-500/10', link: '/dashboard/files', description: 'FTP management' },
    ]
  },
  {
    title: 'Databases',
    icon: Database,
    cards: [
      { title: 'phpMyAdmin', icon: Settings2, color: 'text-orange-600 dark:text-orange-400', bgLight: 'bg-orange-50', bgDark: 'dark:bg-orange-500/10', description: 'Database UI' },
      { title: 'MySQL Databases', icon: Database, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/databases', description: 'Manage DBs' },
      { title: 'Redis Manager', icon: Zap, color: 'text-red-600 dark:text-red-400', bgLight: 'bg-red-50', bgDark: 'dark:bg-red-500/10', link: '/dashboard/redis', description: 'In-memory cache' },
    ]
  },
  {
    title: 'Domains',
    icon: Globe,
    cards: [
      { title: 'Domains', icon: Globe, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/domains', description: 'Main domains' },
      { title: 'Addon Domains', icon: ChevronRight, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/domains', description: 'Extra domains' },
      { title: 'Subdomains', icon: ChevronRight, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/domains', description: 'Sub-level webs' },
    ]
  },
  {
    title: 'Email',
    icon: Mail,
    cards: [
      { title: 'Email Accounts', icon: MailPlus, color: 'text-pink-600 dark:text-pink-400', bgLight: 'bg-pink-50', bgDark: 'dark:bg-pink-500/10', link: '/dashboard/emails', description: 'Professional mail' },
      { title: 'Forwarders', icon: ChevronRight, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/emails', description: 'Mail routing' },
      { title: 'Autoresponders', icon: Zap, color: 'text-amber-600 dark:text-amber-400', bgLight: 'bg-amber-50', bgDark: 'dark:bg-amber-500/10', link: '/dashboard/emails', description: 'Auto reply' },
    ]
  },
  {
    title: 'Security',
    icon: Shield,
    cards: [
      { title: 'SSH Access', icon: Lock, color: 'text-slate-600 dark:text-slate-400', bgLight: 'bg-slate-100', bgDark: 'dark:bg-slate-500/10', link: '/dashboard/security', description: 'Terminal keys' },
      { title: 'IP Blocker', icon: ShieldAlert, color: 'text-red-600 dark:text-red-400', bgLight: 'bg-red-50', bgDark: 'dark:bg-red-500/10', link: '/dashboard/security', description: 'Deny access' },
      { title: 'SSL/TLS', icon: ShieldCheck, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/security', description: 'Security certs' },
    ]
  },
  {
    title: 'Software',
    icon: Cloud,
    cards: [
      { title: 'App Installer', icon: Cloud, color: 'text-purple-600 dark:text-purple-400', bgLight: 'bg-purple-50', bgDark: 'dark:bg-purple-500/10', link: '/dashboard/apps', description: 'One-click install' },
      { title: 'Web Server', icon: Activity, color: 'text-green-600 dark:text-green-400', bgLight: 'bg-green-50', bgDark: 'dark:bg-green-500/10', link: '/dashboard/web-server', description: 'Nginx & PHP' },
      { title: 'PHP Selector', icon: Settings2, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/system', description: 'Change version' },
    ]
  }
]

/**
 * Statistik resource sistem
 */
const resourceStats = [
  { label: 'CPU Usage', val: '12%', icon: Cpu, color: 'bg-blue-500' },
  { label: 'RAM Usage', val: '2.4 GB', icon: Activity, color: 'bg-emerald-500' },
  { label: 'Disk Space', val: '45.2 GB', icon: HardDrive, color: 'bg-indigo-500' },
  { label: 'Bandwidth', val: '128 GB', icon: Zap, color: 'bg-amber-500' },
]

/**
 * Handle navigasi ke halaman tertentu
 */
const navigateTo = (link?: string): void => {
  if (link) {
    router.push(link)
  }
}
</script>

<template>
  <MainLayout>
    <div class="space-y-8 animate-in">
      
      <!-- Hero Section -->
      <div class="relative overflow-hidden bg-card border border-border rounded-2xl p-8 transition-colors duration-300">
        <div class="relative z-10 flex flex-col lg:flex-row lg:items-center justify-between gap-6">
          <div class="max-w-xl">
            <h1 class="text-2xl font-bold text-foreground leading-tight">Welcome back to NusaPanel!</h1>
            <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
              Your server performance is currently stable. All services are running smoothly.
            </p>
            <div class="flex items-center gap-3 mt-6">
              <button class="px-5 py-2 bg-primary text-primary-foreground rounded-lg font-medium text-sm shadow-lg shadow-primary/20 hover:bg-primary/90 transition-all active:scale-[0.98]">
                View Reports
              </button>
              <button class="px-5 py-2 bg-muted text-foreground rounded-lg font-medium text-sm hover:bg-muted/80 transition-all active:scale-[0.98]">
                Panel Settings
              </button>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div 
              v-for="stat in resourceStats" 
              :key="stat.label" 
              class="bg-muted/50 border border-border p-3 rounded-xl w-32 transition-colors duration-300"
            >
              <div class="flex items-center gap-1.5 mb-1">
                <div :class="['w-1.5 h-1.5 rounded-full', stat.color]" />
                <span class="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider">{{ stat.label }}</span>
              </div>
              <p class="text-lg font-bold text-foreground">{{ stat.val }}</p>
            </div>
          </div>
        </div>

        <!-- Decorative -->
        <div class="absolute top-[-20%] right-[-10%] w-[40%] h-[40%] bg-primary/5 rounded-full blur-[80px]" />
        <div class="absolute bottom-[-20%] left-[-10%] w-[40%] h-[40%] bg-indigo-600/5 rounded-full blur-[80px]" />
      </div>

      <!-- Categories -->
      <div v-for="section in sections" :key="section.title" class="space-y-4">
        <div class="flex items-center gap-2 ml-1">
          <div class="p-1.5 bg-primary/10 rounded-lg">
            <component :is="section.icon" class="w-4 h-4 text-primary" />
          </div>
          <h3 class="text-base font-bold text-foreground tracking-tight">{{ section.title }}</h3>
        </div>
        
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          <div 
            v-for="card in section.cards" 
            :key="card.title"
            @click="navigateTo(card.link)"
            class="group bg-card border border-border p-5 rounded-xl hover:shadow-lg hover:shadow-primary/5 hover:-translate-y-0.5 transition-all duration-300 cursor-pointer flex flex-col gap-3 relative overflow-hidden"
          >
            <!-- Icon -->
            <div :class="[
              'w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-300 group-hover:scale-105',
              card.bgLight, card.bgDark, card.color
            ]">
              <component :is="card.icon" class="w-5 h-5" />
            </div>
            
            <!-- Title & Description -->
            <div>
              <h4 class="font-semibold text-sm text-foreground group-hover:text-primary transition-colors">{{ card.title }}</h4>
              <p class="text-[10px] text-muted-foreground mt-0.5 uppercase tracking-wide">{{ card.description }}</p>
            </div>

            <!-- Arrow -->
            <div class="absolute right-4 bottom-4 opacity-0 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all">
              <ChevronRight class="w-4 h-4 text-primary" />
            </div>
          </div>
        </div>
      </div>

    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in {
  animation: animate-in 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes animate-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
