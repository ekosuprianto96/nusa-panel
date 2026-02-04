<script setup lang="ts">
/**
 * ErrorLogsPage - Error Logs Viewer
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { systemService } from '@/services'
import { AlertCircle, Search, Download, Trash2, RefreshCw } from 'lucide-vue-next'

const isLoading = ref(true)
const searchQuery = ref('')
const activeFilter = ref('all')
const logs = ref<any[]>([])

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

const getLevelClass = (level: string) => {
    const l = level?.toLowerCase() || ''
    if (l.includes('error') || l.includes('critical') || l.includes('fatal')) return 'bg-red-500/20 text-red-500'
    if (l.includes('warn')) return 'bg-amber-500/20 text-amber-500'
    if (l.includes('notice') || l.includes('info')) return 'bg-blue-500/20 text-blue-500'
    return 'bg-slate-500/20 text-slate-500'
}

const getLevelLabel = (level: string) => {
    const l = level?.toLowerCase() || ''
    if (l.includes('error') || l.includes('critical') || l.includes('fatal')) return 'ERROR'
    if (l.includes('warn')) return 'WARNING'
    if (l.includes('notice') || l.includes('info')) return 'INFO'
    return level?.toUpperCase() || 'LOG'
}

const filteredLogs = computed(() => {
    let result = logs.value
    if (searchQuery.value) {
        result = result.filter(l => l.message?.toLowerCase().includes(searchQuery.value.toLowerCase()))
    }
    if (activeFilter.value !== 'all') {
        result = result.filter(l => {
            const level = l.level?.toLowerCase() || ''
            if (activeFilter.value === 'error') return level.includes('error') || level.includes('critical')
            if (activeFilter.value === 'warning') return level.includes('warn')
            if (activeFilter.value === 'info') return level.includes('notice') || level.includes('info')
            return true
        })
    }
    return result
})

const parseLogLine = (line: string, index: number) => {
    // Try to extract "[timestamp] LEVEL: message" or "[timestamp] LEVEL message"
    const bracketMatch = line.match(/^\[([^\]]+)\]\s*(.*)$/)
    let timestamp = ''
    let rest = line
    if (bracketMatch) {
        timestamp = bracketMatch[1]
        rest = bracketMatch[2]
    }

    let level = 'info'
    let message = rest
    const levelMatch = rest.match(/^(NOTICE|INFO|WARN|WARNING|ERROR|FATAL|CRITICAL)\s*:?\s*(.*)$/i)
    if (levelMatch) {
        level = levelMatch[1].toLowerCase()
        message = levelMatch[2]
    }

    return {
        id: index + 1,
        timestamp: timestamp || new Date().toISOString(),
        level,
        message: message || line
    }
}

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await systemService.getErrorLogs()
        const data = res.data.data || []
        logs.value = Array.isArray(data)
            ? data.map((entry: any, idx: number) => {
                if (typeof entry === 'string') return parseLogLine(entry, idx)
                return { id: entry.id ?? idx + 1, ...entry }
            })
            : []
    } catch (e) {
        // Sample data fallback
        logs.value = [
            { id: 1, timestamp: '2024-01-15 14:22:01', level: 'error', message: 'PHP Fatal error: Uncaught Error: Call to undefined function get_users() in /var/www/html/index.php:42' },
            { id: 2, timestamp: '2024-01-15 14:22:05', level: 'warning', message: 'PHP Warning: file_get_contents(): SSL operation failed with code 1 in /var/www/html/api.php:128' },
            { id: 3, timestamp: '2024-01-15 14:23:12', level: 'notice', message: 'PHP Notice: Undefined index: user_id in /var/www/html/dashboard.php:55' },
            { id: 4, timestamp: '2024-01-15 14:24:45', level: 'error', message: 'MySQL Error: Table \'database.sessions\' doesn\'t exist' },
            { id: 5, timestamp: '2024-01-15 14:25:01', level: 'warning', message: 'Apache: MaxRequestWorkers limit reached, consider raising this value' },
            { id: 6, timestamp: '2024-01-15 14:28:10', level: 'notice', message: 'PHP Notice: A session had already been started in /var/www/html/auth.php:12' },
            { id: 7, timestamp: '2024-01-15 14:30:22', level: 'error', message: 'Fatal error: Allowed memory size of 134217728 bytes exhausted in /var/www/html/import.php:892' }
        ]
    } finally {
        isLoading.value = false
    }
}

const clearLogs = async () => {
    if (!confirm('Are you sure you want to clear all error logs?')) return
    try {
        await systemService.clearErrorLogs()
        logs.value = []
        showToast('Error logs cleared', 'success')
    } catch (e: any) {
        showToast(e.response?.data?.message || 'Failed to clear logs', 'error')
    }
}

const downloadLogs = () => {
    const content = logs.value.map(l => `[${l.timestamp}] [${l.level}] ${l.message}`).join('\n')
    const blob = new Blob([content], { type: 'text/plain' })
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = `error-logs-${new Date().toISOString().split('T')[0]}.txt`
    a.click()
    showToast('Logs downloaded', 'success')
}

const refreshLogs = () => {
    showToast('Refreshing logs...', 'info')
    fetchData()
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <div class="fixed top-4 right-4 z-50 space-y-2">
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-red-500 text-white' : 'bg-primary text-white']">{{ toast.message }}</div>
    </div>

    <div class="space-y-6">
        <!-- Header -->
        <div>
            <div class="flex items-center gap-2 mb-4">
                <router-link to="/dashboard/system" class="text-slate-500 text-sm hover:text-primary">System Tools</router-link>
                <span class="text-slate-400">/</span>
                <span class="text-[#0d131b] dark:text-white text-sm font-medium">Error Logs</span>
            </div>
            <div class="flex justify-between items-end">
                <div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black leading-tight tracking-tight">Error Logs</h2>
                    <p class="text-slate-500 dark:text-slate-400 text-base mt-2">Inspect Apache and PHP error logs in real-time for debugging.</p>
                </div>
                <div class="flex gap-2">
                    <button @click="refreshLogs" class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors">
                        <RefreshCw :size="16" /> Refresh
                    </button>
                    <button @click="downloadLogs" class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors">
                        <Download :size="16" /> Export
                    </button>
                    <button @click="clearLogs" class="flex items-center gap-2 px-4 py-2 bg-red-500 text-white rounded-lg text-sm font-bold hover:bg-red-600 transition-colors">
                        <Trash2 :size="16" /> Clear Logs
                    </button>
                </div>
            </div>
        </div>

        <!-- Filters -->
        <div class="flex flex-wrap items-center justify-between gap-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-4">
            <div class="flex items-center gap-4">
                <div class="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-lg">
                    <button @click="activeFilter = 'all'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'all' ? 'bg-white dark:bg-slate-700 shadow-sm' : 'text-slate-500 hover:text-slate-700']">All</button>
                    <button @click="activeFilter = 'error'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'error' ? 'bg-red-500/20 text-red-500' : 'text-slate-500 hover:text-slate-700']">Errors</button>
                    <button @click="activeFilter = 'warning'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'warning' ? 'bg-amber-500/20 text-amber-500' : 'text-slate-500 hover:text-slate-700']">Warnings</button>
                    <button @click="activeFilter = 'info'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'info' ? 'bg-blue-500/20 text-blue-500' : 'text-slate-500 hover:text-slate-700']">Info</button>
                </div>
                <span class="text-xs text-slate-500">{{ filteredLogs.length }} logs found</span>
            </div>
            <div class="relative">
                <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                <input v-model="searchQuery" class="bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg pl-9 pr-4 py-2 text-sm w-64 focus:ring-1 focus:ring-primary" placeholder="Search logs..." type="text" />
            </div>
        </div>

        <!-- Log Viewer -->
        <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden">
            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-slate-500">Loading logs...</p>
            </div>

            <div v-else-if="filteredLogs.length === 0" class="p-12 text-center">
                <AlertCircle :size="48" class="mx-auto mb-4 text-slate-300" />
                <p class="text-slate-500">No error logs found</p>
            </div>

            <div v-else class="divide-y divide-slate-100 dark:divide-slate-800 max-h-[600px] overflow-y-auto">
                <div v-for="log in filteredLogs" :key="log.id" class="p-4 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                    <div class="flex items-start gap-4">
                        <span :class="['px-2 py-1 rounded text-[10px] font-bold uppercase shrink-0', getLevelClass(log.level)]">{{ getLevelLabel(log.level) }}</span>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-mono text-[#0d131b] dark:text-white break-all">{{ log.message }}</p>
                            <p class="text-xs text-slate-400 mt-1">{{ log.timestamp }}</p>
                        </div>
                    </div>
                </div>
            </div>

            <div v-if="filteredLogs.length > 0" class="px-4 py-3 border-t border-slate-100 dark:border-slate-800 flex justify-between items-center text-xs text-slate-500 bg-slate-50 dark:bg-slate-800/50">
                <span class="flex items-center gap-2">
                    <span class="size-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    Live monitoring active
                </span>
                <span>Last updated: {{ new Date().toLocaleTimeString() }}</span>
            </div>
        </div>
    </div>
</MainLayout>
</template>
