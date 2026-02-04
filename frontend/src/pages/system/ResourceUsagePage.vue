<script setup lang="ts">
/**
 * ResourceUsagePage - Resource Usage Dashboard
 */
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { systemService } from '@/services/system.service'
import { Gauge, Cpu, MemoryStick, HardDrive, Activity, TrendingUp, TrendingDown, RefreshCw, Clock } from 'lucide-vue-next'



const stats = ref({
    cpuUsage: 42.5,
    cpuPeak: 78.2,
    memoryUsed: 12.4,
    memoryTotal: 32,
    diskUsed: 186,
    diskTotal: 500,
    networkIn: 12.4,
    networkOut: 8.2,
    uptime: '14d 06h 22m'
})

const processes = ref([
    { name: 'nginx', cpu: 2.1, memory: 128 },
    { name: 'php-fpm', cpu: 8.4, memory: 512 },
    { name: 'mysql', cpu: 12.3, memory: 1024 },
    { name: 'redis', cpu: 1.2, memory: 64 },
    { name: 'node', cpu: 4.5, memory: 256 }
])

const getPercentage = (used: number, total: number) => ((used / total) * 100).toFixed(1)

const getBarColor = (percentage: number) => {
    if (percentage > 80) return 'bg-red-500'
    if (percentage > 60) return 'bg-amber-500'
    return 'bg-emerald-500'
}

const refreshData = async () => {
    try {
        const res = await systemService.getResourceUsage()
        const data = res.data.data
        
        // Map backend data to stats
        stats.value.cpuUsage = parseFloat(data.cpu.toFixed(1))
        // Assuming Backend returns Bytes. frontend expects GB for display but keeping mock defaults for Total for now.
        const memUsedGB = parseFloat((data.memory / (1024 * 1024 * 1024)).toFixed(2))
        const diskUsedGB = parseFloat((data.disk / (1024 * 1024 * 1024)).toFixed(2))
        
        stats.value.memoryUsed = memUsedGB
         stats.value.diskUsed = diskUsedGB
         
         // Mock Network and Peak since backend doesn't provide them yet
         stats.value.networkIn = parseFloat((Math.random() * 20).toFixed(1))
         stats.value.networkOut = parseFloat((Math.random() * 10).toFixed(1))

         // Map processes
         processes.value = data.processes.map((p: any) => ({
             name: p.name,
             cpu: p.cpu,
             memory: parseFloat((p.memory / (1024 * 1024)).toFixed(1)) // Convert bytes to MB
         }))

    } catch (e) {
        console.error('Failed to fetch resource usage', e)
    }
}

onMounted(() => {
    refreshData()
})
</script>

<template>
<MainLayout>
    <div class="space-y-6">
        <!-- Header -->
        <div>
            <div class="flex items-center gap-2 mb-4">
                <router-link to="/dashboard/system" class="text-slate-500 text-sm hover:text-primary">System Tools</router-link>
                <span class="text-slate-400">/</span>
                <span class="text-[#0d131b] dark:text-white text-sm font-medium">Resource Usage</span>
            </div>
            <div class="flex justify-between items-end">
                <div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black leading-tight tracking-tight">Resource Usage</h2>
                    <p class="text-slate-500 dark:text-slate-400 text-base mt-2">Real-time analytics for CPU, RAM, and disk I/O consumption.</p>
                </div>
                <div class="flex gap-2 items-center">
                    <div class="flex items-center gap-2 bg-slate-100 dark:bg-slate-800 px-3 py-2 rounded-lg text-xs font-bold text-slate-500">
                        <Clock :size="14" /> Refresh: 5s
                    </div>
                    <button @click="refreshData" class="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-primary/90 transition-colors">
                        <RefreshCw :size="16" /> Refresh
                    </button>
                </div>
            </div>
        </div>

        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <!-- CPU -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-start mb-4">
                    <div class="p-2 bg-primary/10 rounded-lg">
                        <Cpu :size="20" class="text-primary" />
                    </div>
                    <span :class="['text-sm font-bold flex items-center gap-1', stats.cpuUsage > 60 ? 'text-red-500' : 'text-emerald-500']">
                        <TrendingUp v-if="stats.cpuUsage > 60" :size="14" />
                        <TrendingDown v-else :size="14" />
                        {{ stats.cpuUsage > 60 ? '+5.2%' : '-2.1%' }}
                    </span>
                </div>
                <p class="text-3xl font-black text-[#0d131b] dark:text-white mb-1">{{ stats.cpuUsage }}%</p>
                <p class="text-xs text-slate-500 uppercase tracking-wider font-bold">CPU Usage</p>
                <div class="mt-4 h-2 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
                    <div :class="['h-full transition-all', getBarColor(stats.cpuUsage)]" :style="{ width: stats.cpuUsage + '%' }"></div>
                </div>
                <p class="text-[10px] text-slate-400 mt-2">Peak: {{ stats.cpuPeak }}%</p>
            </div>

            <!-- Memory -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-start mb-4">
                    <div class="p-2 bg-blue-500/10 rounded-lg">
                        <MemoryStick :size="20" class="text-blue-500" />
                    </div>
                    <span class="text-sm font-bold text-emerald-500 flex items-center gap-1">
                        <TrendingDown :size="14" /> -0.8%
                    </span>
                </div>
                <p class="text-3xl font-black text-[#0d131b] dark:text-white mb-1">{{ stats.memoryUsed }} GB</p>
                <p class="text-xs text-slate-500 uppercase tracking-wider font-bold">Memory Usage</p>
                <div class="mt-4 h-2 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
                    <div class="h-full bg-blue-500 transition-all" :style="{ width: getPercentage(stats.memoryUsed, stats.memoryTotal) + '%' }"></div>
                </div>
                <p class="text-[10px] text-slate-400 mt-2">Total: {{ stats.memoryTotal }} GB</p>
            </div>

            <!-- Disk -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-start mb-4">
                    <div class="p-2 bg-purple-500/10 rounded-lg">
                        <HardDrive :size="20" class="text-purple-500" />
                    </div>
                    <span class="text-sm font-bold text-amber-500 flex items-center gap-1">
                        <TrendingUp :size="14" /> +1.2%
                    </span>
                </div>
                <p class="text-3xl font-black text-[#0d131b] dark:text-white mb-1">{{ stats.diskUsed }} GB</p>
                <p class="text-xs text-slate-500 uppercase tracking-wider font-bold">Disk Usage</p>
                <div class="mt-4 h-2 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden">
                    <div class="h-full bg-purple-500 transition-all" :style="{ width: getPercentage(stats.diskUsed, stats.diskTotal) + '%' }"></div>
                </div>
                <p class="text-[10px] text-slate-400 mt-2">Total: {{ stats.diskTotal }} GB</p>
            </div>

            <!-- Network -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-start mb-4">
                    <div class="p-2 bg-emerald-500/10 rounded-lg">
                        <Activity :size="20" class="text-emerald-500" />
                    </div>
                </div>
                <div class="flex items-baseline gap-2 mb-1">
                    <p class="text-3xl font-black text-[#0d131b] dark:text-white">{{ stats.networkIn }}</p>
                    <span class="text-sm text-slate-500">MB/s</span>
                </div>
                <p class="text-xs text-slate-500 uppercase tracking-wider font-bold">Network I/O</p>
                <div class="mt-4 flex gap-4">
                    <div class="flex-1">
                        <p class="text-[10px] text-slate-400 mb-1">IN</p>
                        <p class="text-sm font-bold text-emerald-500">↓ {{ stats.networkIn }} MB/s</p>
                    </div>
                    <div class="flex-1">
                        <p class="text-[10px] text-slate-400 mb-1">OUT</p>
                        <p class="text-sm font-bold text-blue-500">↑ {{ stats.networkOut }} MB/s</p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Charts Section -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- CPU Chart -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">CPU History (Last Hour)</h3>
                    <span class="text-xs text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded">Live</span>
                </div>
                <div class="h-48 w-full bg-slate-50 dark:bg-slate-800 rounded-lg flex items-end justify-around px-2 py-4">
                    <div v-for="(val, i) in [35, 42, 38, 55, 48, 72, 65, 45, 52, 48, 42, stats.cpuUsage]" :key="i" class="flex-1 mx-0.5 rounded-t transition-all" :class="getBarColor(val)" :style="{ height: val + '%' }"></div>
                </div>
            </div>

            <!-- Memory Chart -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-center mb-6">
                    <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Memory History (Last Hour)</h3>
                    <span class="text-xs text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded">Live</span>
                </div>
                <div class="h-48 w-full bg-slate-50 dark:bg-slate-800 rounded-lg flex items-end justify-around px-2 py-4">
                    <div v-for="(val, i) in [38, 40, 42, 41, 39, 38, 40, 42, 38, 41, 39, (stats.memoryUsed / stats.memoryTotal * 100)]" :key="i" class="flex-1 mx-0.5 bg-blue-500 rounded-t transition-all" :style="{ height: val + '%' }"></div>
                </div>
            </div>
        </div>

        <!-- Top Processes -->
        <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden">
            <div class="px-6 py-4 border-b border-slate-100 dark:border-slate-800 flex justify-between items-center">
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Top Processes by Resource</h3>
                <router-link to="/dashboard/system/process-manager" class="text-xs text-primary font-bold hover:underline">View All Processes →</router-link>
            </div>
            <table class="w-full text-left">
                <thead>
                    <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                        <th class="px-6 py-3 text-xs font-bold text-slate-500 uppercase tracking-wider">Process</th>
                        <th class="px-6 py-3 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">CPU %</th>
                        <th class="px-6 py-3 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Memory</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                    <tr v-for="proc in processes" :key="proc.name" class="hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors">
                        <td class="px-6 py-4 font-bold text-[#0d131b] dark:text-white">{{ proc.name }}</td>
                        <td class="px-6 py-4 text-right">
                            <div class="flex items-center justify-end gap-2">
                                <div class="w-20 h-2 bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden">
                                    <div :class="['h-full', getBarColor(proc.cpu * 5)]" :style="{ width: Math.min(proc.cpu * 5, 100) + '%' }"></div>
                                </div>
                                <span class="font-mono text-sm">{{ proc.cpu }}%</span>
                            </div>
                        </td>
                        <td class="px-6 py-4 text-right font-mono text-sm text-slate-600 dark:text-slate-400">{{ proc.memory }} MB</td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- System Info -->
        <div class="bg-primary/5 dark:bg-primary/10 border border-primary/20 rounded-xl p-6">
            <div class="flex items-center gap-4">
                <div class="p-3 bg-primary/10 rounded-xl">
                    <Gauge :size="24" class="text-primary" />
                </div>
                <div>
                    <p class="text-sm font-bold text-[#0d131b] dark:text-white">System Uptime: {{ stats.uptime }}</p>
                    <p class="text-xs text-slate-600 dark:text-slate-400">Server has been running without restart since the last kernel update.</p>
                </div>
            </div>
        </div>
    </div>
</MainLayout>
</template>
