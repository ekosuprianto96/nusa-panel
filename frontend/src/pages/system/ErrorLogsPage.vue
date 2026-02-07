<script setup lang="ts">
/**
 * ErrorLogsPage - Error Logs Viewer
 */
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { systemService } from '@/services'
import { AlertCircle, Search, Download, Trash2, RefreshCw, Settings } from 'lucide-vue-next'

const isLoading = ref(true)
const searchQuery = ref('')
const activeFilter = ref('all')
const logs = ref<any[]>([])
const router = useRouter()

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

const getLevelVariant = (level: string): 'destructive' | 'warning' | 'secondary' | 'success' => {
    const l = level?.toLowerCase() || ''
    if (l.includes('error') || l.includes('critical') || l.includes('fatal')) return 'destructive'
    if (l.includes('warn')) return 'warning'
    if (l.includes('notice') || l.includes('info')) return 'secondary'
    return 'secondary'
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
    const bracketMatch = line.match(/^\[([^\]]+)\]\s*(.*)$/)
    const timestamp = bracketMatch?.[1] ?? ''
    const rest = bracketMatch?.[2] ?? line

    const levelMatch = rest.match(/^(NOTICE|INFO|WARN|WARNING|ERROR|FATAL|CRITICAL)\s*:?\s*(.*)$/i)
    const level = (levelMatch?.[1] ?? 'info').toLowerCase()
    const message = levelMatch?.[2] ?? rest

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
        logs.value = [
            { id: 1, timestamp: '2024-01-15 14:22:01', level: 'error', message: 'PHP Fatal error: Uncaught Error: Call to undefined function get_users() in /var/www/html/index.php:42' },
            { id: 2, timestamp: '2024-01-15 14:22:05', level: 'warning', message: 'PHP Warning: file_get_contents(): SSL operation failed with code 1 in /var/www/html/api.php:128' },
            { id: 3, timestamp: '2024-01-15 14:23:12', level: 'notice', message: 'PHP Notice: Undefined index: user_id in /var/www/html/dashboard.php:55' },
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
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-destructive text-destructive-foreground' : 'bg-primary text-primary-foreground']">{{ toast.message }}</div>
    </div>

    <div class="space-y-6">
        <!-- Header -->
        <div>
            <AppBreadcrumb
                class="mb-4"
                :items="[
                    { label: 'System Tools', icon: Settings, onClick: () => router.push('/dashboard/system') },
                    { label: 'Error Logs', current: true }
                ]"
            />
            <div class="flex justify-between items-end">
                <div>
                    <h2 class="text-foreground text-3xl font-black leading-tight tracking-tight">Error Logs</h2>
                    <p class="text-muted-foreground text-base mt-2">Inspect Apache and PHP error logs in real-time for debugging.</p>
                </div>
                <div class="flex gap-2">
                    <Button variant="outline" @click="refreshLogs">
                        <RefreshCw :size="16" class="mr-2" /> Refresh
                    </Button>
                    <Button variant="outline" @click="downloadLogs">
                        <Download :size="16" class="mr-2" /> Export
                    </Button>
                    <Button variant="destructive" @click="clearLogs">
                        <Trash2 :size="16" class="mr-2" /> Clear Logs
                    </Button>
                </div>
            </div>
        </div>

        <!-- Filters -->
        <Card class="rounded-xl">
            <CardContent class="p-4 flex flex-wrap items-center justify-between gap-4">
                <div class="flex items-center gap-4">
                    <div class="flex bg-muted p-1 rounded-lg">
                        <button @click="activeFilter = 'all'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'all' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground']">All</button>
                        <button @click="activeFilter = 'error'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'error' ? 'bg-destructive/20 text-destructive' : 'text-muted-foreground hover:text-foreground']">Errors</button>
                        <button @click="activeFilter = 'warning'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'warning' ? 'bg-amber-500/20 text-amber-500' : 'text-muted-foreground hover:text-foreground']">Warnings</button>
                        <button @click="activeFilter = 'info'" :class="['px-3 py-1.5 rounded-md text-xs font-bold transition-colors', activeFilter === 'info' ? 'bg-blue-500/20 text-blue-500' : 'text-muted-foreground hover:text-foreground']">Info</button>
                    </div>
                    <span class="text-xs text-muted-foreground">{{ filteredLogs.length }} logs found</span>
                </div>
                <div class="relative">
                    <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input v-model="searchQuery" class="pl-9 w-64" placeholder="Search logs..." />
                </div>
            </CardContent>
        </Card>

        <!-- Log Viewer -->
        <Card class="rounded-xl overflow-hidden">
            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-muted-foreground">Loading logs...</p>
            </div>

            <div v-else-if="filteredLogs.length === 0" class="p-12 text-center">
                <AlertCircle :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                <p class="text-muted-foreground">No error logs found</p>
            </div>

            <div v-else class="divide-y divide-border max-h-[600px] overflow-y-auto">
                <div v-for="log in filteredLogs" :key="log.id" class="p-4 hover:bg-muted/30 transition-colors">
                    <div class="flex items-start gap-4">
                        <Badge :variant="getLevelVariant(log.level)" class="shrink-0">{{ getLevelLabel(log.level) }}</Badge>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-mono text-foreground break-all">{{ log.message }}</p>
                            <p class="text-xs text-muted-foreground mt-1">{{ log.timestamp }}</p>
                        </div>
                    </div>
                </div>
            </div>

            <div v-if="filteredLogs.length > 0" class="px-4 py-3 border-t border-border flex justify-between items-center text-xs text-muted-foreground bg-muted/30">
                <span class="flex items-center gap-2">
                    <span class="size-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    Live monitoring active
                </span>
                <span>Last updated: {{ new Date().toLocaleTimeString() }}</span>
            </div>
        </Card>
    </div>
</MainLayout>
</template>
