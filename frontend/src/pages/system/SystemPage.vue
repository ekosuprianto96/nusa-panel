<script setup lang="ts">
/**
 * SystemPage - System Tools Dashboard
 * 
 * Modern dashboard with tool cards linking to dedicated sub-pages:
 * - Cron Jobs
 * - Track DNS
 * - Process Manager
 * - PHP Manager
 * - NPM Manager
 * - Error Logs
 * - Resource Usage
 */
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { systemService } from '@/services'
import { 
    Clock, Globe, Activity, Code, Package, AlertTriangle, Gauge,
    Search, Grid3X3, Filter, CheckCircle, RefreshCw, TrendingUp
} from 'lucide-vue-next'

const isLoading = ref(true)
const searchQuery = ref('')
const stats = ref({
    cronJobs: 0,
    backups: 0,
    phpVersion: '8.2',
    storageUsed: '0 B'
})

const recentActivity = ref([
    { type: 'success', title: 'Cron Job Executed', desc: 'daily_backup.sh finished successfully', time: '2 mins ago' },
    { type: 'update', title: 'NPM Package Updated', desc: 'Updated react-dom in shop-v2 project', time: '18 mins ago' },
    { type: 'update', title: 'PHP Version Changed', desc: 'Updated to PHP 8.3 for example.com', time: '45 mins ago' }
])

const tools = [
    { 
        id: 'cron', 
        name: 'Cron Jobs', 
        description: 'Schedule automated scripts and recurring tasks for your applications.',
        icon: Clock,
        route: '/dashboard/system/cron-jobs'
    },
    { 
        id: 'dns', 
        name: 'Track DNS', 
        description: 'Lookup records and trace the routing path of your domain names.',
        icon: Globe,
        route: '/dashboard/system/dns-tracker'
    },
    { 
        id: 'process', 
        name: 'Process Manager', 
        description: 'Monitor and terminate active system processes to free up resources.',
        icon: Activity,
        route: '/dashboard/system/process-manager'
    },
    { 
        id: 'php', 
        name: 'PHP Manager', 
        description: 'Select between PHP 7.4, 8.1, and 8.3 environments globally or per-domain.',
        icon: Code,
        route: '/dashboard/system/php-manager'
    },
    { 
        id: 'npm', 
        name: 'NPM Manager', 
        description: 'Manage Node.js packages and dependencies for your web applications.',
        icon: Package,
        route: '/dashboard/system/npm-manager'
    },
    { 
        id: 'logs', 
        name: 'Error Logs', 
        description: 'Inspect Apache and PHP error logs in real-time for debugging.',
        icon: AlertTriangle,
        route: '/dashboard/system/error-logs'
    },
    { 
        id: 'resource', 
        name: 'Resource Usage', 
        description: 'Real-time analytics for CPU, RAM, and disk I/O consumption.',
        icon: Gauge,
        route: '/dashboard/system/resource-usage'
    }
]

const filteredTools = ref(tools)

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const fetchData = async () => {
    isLoading.value = true
    try {
        const [cronRes, backupRes, phpRes] = await Promise.all([
            systemService.listCronJobs().catch(() => ({ data: { data: [] } })),
            systemService.listBackups().catch(() => ({ data: { data: [] } })),
            systemService.getCurrentPhpVersion().catch(() => ({ data: { data: '8.2' } }))
        ])
        
        const cronJobs = cronRes.data.data || []
        const backups = backupRes.data.data || []
        
        stats.value = {
            cronJobs: cronJobs.length,
            backups: backups.length,
            phpVersion: phpRes.data.data || '8.2',
            storageUsed: formatBytes(backups.reduce((acc: number, b: any) => acc + (b.size_bytes || 0), 0))
        }
    } catch (e) {
        console.error('Failed to load system data:', e)
    } finally {
        isLoading.value = false
    }
}

const handleSearch = () => {
    if (!searchQuery.value) {
        filteredTools.value = tools
        return
    }
    filteredTools.value = tools.filter(t => 
        t.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        t.description.toLowerCase().includes(searchQuery.value.toLowerCase())
    )
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <div class="space-y-8">
        <!-- Page Header -->
        <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
            <div class="flex flex-col gap-2">
                <h2 class="text-4xl font-black leading-tight tracking-tight text-[#0d131b] dark:text-white">System Tools</h2>
                <p class="text-slate-500 dark:text-slate-400 text-base max-w-lg leading-relaxed">
                    Optimize and maintain your server environment with professional-grade management utilities.
                </p>
            </div>
            <div class="w-full md:w-80">
                <label class="relative block group">
                    <div class="absolute inset-y-0 left-0 flex items-center pl-3.5 pointer-events-none text-slate-400 group-focus-within:text-primary transition-colors">
                        <Search :size="20" />
                    </div>
                    <input 
                        v-model="searchQuery"
                        @input="handleSearch"
                        class="block w-full h-12 pl-11 pr-4 rounded-xl border-none bg-white dark:bg-slate-800 shadow-sm ring-1 ring-slate-200 dark:ring-slate-700 focus:ring-2 focus:ring-primary text-slate-900 dark:text-white placeholder:text-slate-400 text-sm font-medium transition-all" 
                        placeholder="Search system utilities..." 
                        type="text"
                    />
                </label>
            </div>
        </div>

        <!-- Stats Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-2 mb-3">
                    <Clock :size="16" class="text-primary" />
                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Cron Jobs</p>
                </div>
                <p class="text-2xl font-black text-[#0d131b] dark:text-white">{{ stats.cronJobs }}</p>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-2 mb-3">
                    <RefreshCw :size="16" class="text-emerald-500" />
                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Backups</p>
                </div>
                <p class="text-2xl font-black text-[#0d131b] dark:text-white">{{ stats.backups }}</p>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-2 mb-3">
                    <Code :size="16" class="text-blue-500" />
                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">PHP Version</p>
                </div>
                <p class="text-2xl font-black text-[#0d131b] dark:text-white">{{ stats.phpVersion }}</p>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-2 mb-3">
                    <TrendingUp :size="16" class="text-purple-500" />
                    <p class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">Storage Used</p>
                </div>
                <p class="text-2xl font-black text-[#0d131b] dark:text-white">{{ stats.storageUsed }}</p>
            </div>
        </div>

        <!-- Tools Section Header -->
        <div class="flex items-center justify-between">
            <h3 class="text-[#0d131b] dark:text-white text-xl font-bold tracking-tight">Management Utilities</h3>
            <div class="flex gap-2">
                <button class="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary transition-colors">
                    <Filter :size="16" />
                </button>
                <button class="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary transition-colors">
                    <Grid3X3 :size="16" />
                </button>
            </div>
        </div>

        <!-- Loading State -->
        <div v-if="isLoading" class="flex justify-center py-20">
            <div class="animate-spin w-10 h-10 border-3 border-primary border-t-transparent rounded-full"></div>
        </div>

        <!-- Tools Grid -->
        <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            <router-link 
                v-for="tool in filteredTools" 
                :key="tool.id"
                :to="tool.route"
                class="group flex flex-col p-6 rounded-xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-sm hover:shadow-md hover:border-primary/30 dark:hover:border-primary/50 transition-all hover:-translate-y-0.5"
            >
                <div class="mb-5 size-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary group-hover:bg-primary group-hover:text-white transition-all">
                    <component :is="tool.icon" :size="24" />
                </div>
                <div class="flex flex-col gap-1.5">
                    <h4 class="text-[#0d131b] dark:text-white text-lg font-bold leading-tight">{{ tool.name }}</h4>
                    <p class="text-slate-500 dark:text-slate-400 text-sm font-normal leading-normal">{{ tool.description }}</p>
                </div>
            </router-link>
        </div>

        <!-- Recent System Activity -->
        <div class="mt-12">
            <h3 class="text-[#0d131b] dark:text-white text-xl font-bold tracking-tight pb-4">Recent System Activity</h3>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden divide-y divide-slate-100 dark:divide-slate-800 shadow-sm">
                <div 
                    v-for="(activity, index) in recentActivity" 
                    :key="index"
                    class="flex items-center gap-4 p-4 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                >
                    <div :class="[
                        'size-8 rounded-full flex items-center justify-center',
                        activity.type === 'success' ? 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600' : 'bg-blue-100 dark:bg-blue-900/30 text-blue-600'
                    ]">
                        <CheckCircle v-if="activity.type === 'success'" :size="16" />
                        <RefreshCw v-else :size="16" />
                    </div>
                    <div class="flex-1">
                        <p class="text-sm font-medium text-[#0d131b] dark:text-white">{{ activity.title }}</p>
                        <p class="text-xs text-slate-500">{{ activity.desc }}</p>
                    </div>
                    <span class="text-xs text-slate-400 font-medium">{{ activity.time }}</span>
                </div>
            </div>
        </div>
    </div>
</MainLayout>
</template>
