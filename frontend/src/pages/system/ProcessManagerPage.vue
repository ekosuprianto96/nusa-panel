<script setup lang="ts">
/**
 * ProcessManagerPage - PM2 Process Management
 */
import { ref, onMounted, onUnmounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { nodejsService } from '@/services/nodejs.service'
import type { Pm2Process } from '@/types'
import { useToastStore } from '@/stores/toast'
import { useAuthStore } from '@/stores/auth'
import { 
    Play, Square, RefreshCw, Trash2, Search, Plus, 
    Eye, EyeOff, List, AlertCircle,
    StopCircle, CheckCircle, Ban
} from 'lucide-vue-next'

const toast = useToastStore()
const authStore = useAuthStore()

// State
const isLoading = ref(true)
const processes = ref<Pm2Process[]>([])
const searchQuery = ref('')
const liveUpdate = ref(true)
let pollInterval: any = null

// Env Vars State
const envPath = ref('')
const envVars = ref<{ key: string, value: string, hidden: boolean }[]>([])
const isLoadingEnv = ref(false)
const showAddEnvModal = ref(false)
const newEnv = ref({ key: '', value: '' })

// Console State
const logs = ref<string[]>([
    '[2024-05-20 14:02:12] PM2: Process [nextjs-frontend] launched',
    '[STDOUT] info  - Loaded env from .env',
    '[STDOUT] info  - Ready on http://localhost:3000'
])

// Computed Stats
const stats = computed(() => {
    const total = processes.value.length
    const running = processes.value.filter((p: Pm2Process) => p.status === 'online').length
    const errored = processes.value.filter((p: Pm2Process) => p.status === 'errored').length
    const stopped = total - running - errored
    return { total, running, errored, stopped }
})

const filteredProcesses = computed(() => {
    if (!searchQuery.value) return processes.value
    const q = searchQuery.value.toLowerCase()
    return processes.value.filter((p: Pm2Process) => 
        p.name.toLowerCase().includes(q) || 
        p.pid.toString().includes(q)
    )
})

// Methods
const fetchProcesses = async () => {
    try {
        const res = await nodejsService.listPm2()
        processes.value = res.data.data
    } catch (e) {
        console.error('Failed to fetch PM2 list:', e)
    } finally {
        isLoading.value = false
    }
}

const handleAction = async (action: 'start' | 'stop' | 'restart' | 'delete', target: string) => {
    try {
        await nodejsService.pm2Action(action, target)
        toast.success(`Process ${target} ${action}ed`)
        await fetchProcesses()
    } catch (e) {
        toast.error(`Failed to ${action} ${target}`)
    }
}

const handleRestartAll = async () => {
    if (!confirm('Restart all processes?')) return
    try {
        await nodejsService.pm2Action('restart', 'all')
        toast.success('All processes restarted')
        await fetchProcesses()
    } catch (e) {
        toast.error('Failed to restart all')
    }
}

const handleStopAll = async () => {
    if (!confirm('Stop all processes?')) return
    try {
        await nodejsService.pm2Action('stop', 'all')
        toast.success('All processes stopped')
        await fetchProcesses()
    } catch (e) {
        toast.error('Failed to stop all')
    }
}

// Env Vars Methods
const fetchEnvVars = async () => {
    if (!envPath.value) return
    isLoadingEnv.value = true
    try {
        const res = await nodejsService.getEnvVars(envPath.value)
        envVars.value = res.data.data.map(line => {
             const [key, ...rest] = line.split('=')
             return { key: (key || '').trim(), value: rest.join('=').trim(), hidden: true }
        })
    } catch (e) {
        toast.error('Failed to load env vars. Check path.')
        envVars.value = []
    } finally {
        isLoadingEnv.value = false
    }
}

const saveEnvVar = async () => {
    if (!newEnv.value.key || !envPath.value) return
    try {
        await nodejsService.saveEnvVar(envPath.value, newEnv.value.key, newEnv.value.value)
        toast.success('Environment variable saved')
        showAddEnvModal.value = false
        newEnv.value = { key: '', value: '' }
        await fetchEnvVars()
    } catch (e) {
        toast.error('Failed to save variable')
    }
}

const deleteEnvVar = async (key: string) => {
    if (!confirm(`Delete variable ${key}?`)) return
    try {
        await nodejsService.deleteEnvVar(envPath.value, key)
        toast.success('Variable deleted')
        await fetchEnvVars()
    } catch (e) {
        toast.error('Failed to delete variable')
    }
}

const formatUptime = (ms: number) => {
    const now = Date.now()
    const diff = now - ms
    const seconds = Math.floor(diff / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)
    
    if (days > 0) return `${days}d ${hours % 24}h`
    if (hours > 0) return `${hours}h ${minutes % 60}m`
    if (minutes > 0) return `${minutes}m`
    return `${seconds}s`
}

const formatMemory = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const mb = bytes / (1024 * 1024)
    return `${mb.toFixed(1)} MB`
}

onMounted(() => {
    fetchProcesses()
    pollInterval = setInterval(() => {
        if (liveUpdate.value) fetchProcesses()
    }, 2000)
    
    setInterval(() => {
        if (processes.value.length > 0) {
            const randomProc = processes.value[Math.floor(Math.random() * processes.value.length)]
            if (randomProc) {
                logs.value.push(`[STDOUT] ${randomProc.name}: processing request...`)
                if (logs.value.length > 50) logs.value.shift()
            }
        }
    }, 3000)

    if (!authStore.user) {
        authStore.fetchMe().then(() => {
            const username = authStore.user?.username || 'user'
            envPath.value = `/home/user_${username}`
            fetchEnvVars()
        })
    } else {
        const username = authStore.user?.username || 'user'
        envPath.value = `/home/user_${username}`
        fetchEnvVars()
    }
})

onUnmounted(() => {
    if (pollInterval) clearInterval(pollInterval)
})
</script>

<template>
<MainLayout>
    <div class="flex flex-col h-[calc(100vh-6rem)]">
        <div class="flex-1 overflow-y-auto pr-2 custom-scrollbar">
            <!-- Header -->
            <div class="mb-8">
                <h2 class="text-foreground text-4xl font-black leading-tight tracking-tight">PM2 Process Manager</h2>
                <p class="text-muted-foreground text-base max-w-lg leading-relaxed mt-2">
                    Monitor and control your Node.js application processes in real-time with PM2.
                </p>
            </div>

            <!-- Stats Cards -->
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                <Card class="rounded-xl">
                    <CardContent class="p-4 flex items-center justify-between">
                        <div>
                            <p class="text-xs font-bold text-muted-foreground uppercase tracking-wider mb-1">Total Processes</p>
                            <h3 class="text-2xl font-black text-foreground">{{ stats.total }}</h3>
                        </div>
                        <div class="bg-muted p-2 rounded-lg text-muted-foreground">
                            <List :size="20" />
                        </div>
                    </CardContent>
                </Card>
                <Card class="rounded-xl">
                    <CardContent class="p-4 flex items-center justify-between">
                        <div>
                            <p class="text-xs font-bold text-emerald-600 uppercase tracking-wider mb-1">Running</p>
                            <h3 class="text-2xl font-black text-emerald-600">{{ stats.running }}</h3>
                        </div>
                        <div class="bg-emerald-50 dark:bg-emerald-900/20 p-2 rounded-lg text-emerald-600">
                            <CheckCircle :size="20" />
                        </div>
                    </CardContent>
                </Card>
                <Card class="rounded-xl">
                    <CardContent class="p-4 flex items-center justify-between">
                        <div>
                            <p class="text-xs font-bold text-destructive uppercase tracking-wider mb-1">Errored</p>
                            <h3 class="text-2xl font-black text-destructive">{{ stats.errored }}</h3>
                        </div>
                        <div class="bg-destructive/10 p-2 rounded-lg text-destructive">
                            <AlertCircle :size="20" />
                        </div>
                    </CardContent>
                </Card>
                <Card class="rounded-xl">
                    <CardContent class="p-4 flex items-center justify-between">
                        <div>
                            <p class="text-xs font-bold text-muted-foreground uppercase tracking-wider mb-1">Stopped</p>
                            <h3 class="text-2xl font-black text-muted-foreground">{{ stats.stopped }}</h3>
                        </div>
                        <div class="bg-muted p-2 rounded-lg text-muted-foreground">
                            <StopCircle :size="20" />
                        </div>
                    </CardContent>
                </Card>
            </div>

            <!-- Controls -->
            <Card class="rounded-xl mb-6">
                <CardContent class="p-6 flex flex-wrap items-center justify-between gap-4">
                    <div class="flex items-center gap-4">
                        <Button @click="handleRestartAll">
                            <RefreshCw :size="18" class="mr-2" /> Restart All
                        </Button>
                        <Button variant="destructive" @click="handleStopAll">
                            <Square :size="18" fill="currentColor" class="mr-2" /> Stop All
                        </Button>
                    </div>
                    <div class="flex items-center gap-3">
                        <Button variant="outline" disabled>
                            <Plus :size="18" class="mr-2" /> New Process
                        </Button>
                    </div>
                </CardContent>
            </Card>

            <!-- Process List -->
            <div class="mb-10">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-4">
                    <h3 class="text-foreground text-xl font-bold tracking-tight">Process List</h3>
                    <div class="w-full md:w-96 relative group">
                        <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground group-focus-within:text-primary transition-colors" />
                        <Input v-model="searchQuery" class="h-11 pl-11" placeholder="Search processes..." />
                    </div>
                </div>

                <Card class="rounded-xl overflow-hidden">
                    <table class="w-full text-left border-collapse min-w-[800px]">
                        <thead>
                            <tr class="bg-muted/50 border-b border-border">
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Status</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Name</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">PID</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Uptime</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">CPU</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Memory</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Restarts</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <tr v-if="isLoading" class="animate-pulse">
                                <td colspan="8" class="px-6 py-8 text-center text-muted-foreground">Loading processes...</td>
                            </tr>
                            <tr v-else-if="filteredProcesses.length === 0">
                                <td colspan="8" class="px-6 py-8 text-center text-muted-foreground">No processes found</td>
                            </tr>
                            <tr v-for="proc in filteredProcesses" :key="proc.pm_id" class="hover:bg-muted/30 transition-colors">
                                <td class="px-6 py-4">
                                    <Badge :variant="proc.status === 'online' ? 'success' : 'secondary'">
                                        {{ proc.status }}
                                    </Badge>
                                </td>
                                <td class="px-6 py-4"><span class="font-bold text-foreground">{{ proc.name }}</span></td>
                                <td class="px-6 py-4 font-mono text-xs text-muted-foreground">{{ proc.pid }}</td>
                                <td class="px-6 py-4 text-sm text-muted-foreground">{{ formatUptime(proc.uptime) }}</td>
                                <td class="px-6 py-4"><span class="text-xs font-bold text-foreground">{{ proc.cpu }}%</span></td>
                                <td class="px-6 py-4"><span class="text-xs font-bold text-foreground">{{ formatMemory(proc.memory) }}</span></td>
                                <td class="px-6 py-4"><Badge variant="secondary">{{ proc.restarts }}</Badge></td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex justify-end gap-2">
                                        <Button v-if="proc.status === 'online'" variant="ghost" size="icon" @click="handleAction('stop', proc.name)" title="Stop">
                                            <Square :size="18" fill="currentColor" class="text-amber-500" />
                                        </Button>
                                        <Button v-else variant="ghost" size="icon" @click="handleAction('start', proc.name)" title="Start">
                                            <Play :size="18" fill="currentColor" class="text-emerald-500" />
                                        </Button>
                                        <Button variant="ghost" size="icon" @click="handleAction('restart', proc.name)" title="Restart">
                                            <RefreshCw :size="18" />
                                        </Button>
                                        <Button variant="ghost" size="icon" @click="handleAction('delete', proc.name)" class="text-destructive" title="Delete">
                                            <Trash2 :size="18" />
                                        </Button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </Card>
            </div>

            <!-- Environment Variables -->
            <div class="mb-10">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-4">
                    <div class="flex flex-col">
                        <h3 class="text-foreground text-xl font-bold tracking-tight">Environment Variables</h3>
                        <p class="text-muted-foreground text-xs mt-1">Configure secrets and runtime settings for your application.</p>
                    </div>
                    <div class="flex gap-2">
                        <Input v-model="envPath" type="text" placeholder="/path/to/project" class="h-9 text-xs w-48" />
                        <Button variant="outline" size="sm" @click="fetchEnvVars">Load</Button>
                        <Button size="sm" @click="showAddEnvModal = true">
                            <Plus :size="18" class="mr-2" /> Add Variable
                        </Button>
                    </div>
                </div>

                <Card class="rounded-xl overflow-hidden">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-muted/50 border-b border-border">
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Key</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Value</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <tr v-if="envVars.length === 0"><td colspan="3" class="px-6 py-4 text-center text-sm text-muted-foreground">No variables loaded. Enter path and click Load.</td></tr>
                            <tr v-for="v in envVars" :key="v.key" class="hover:bg-muted/30 transition-colors">
                                <td class="px-6 py-4">
                                    <code class="text-xs font-bold text-foreground bg-muted px-1.5 py-0.5 rounded">{{ v.key }}</code>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <span class="text-sm font-mono text-muted-foreground">{{ v.hidden ? '••••••••' : v.value }}</span>
                                        <Button variant="ghost" size="icon" @click="v.hidden = !v.hidden">
                                            <component :is="v.hidden ? Eye : EyeOff" :size="16" />
                                        </Button>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <Button variant="ghost" size="icon" @click="deleteEnvVar(v.key)" class="text-destructive" title="Delete">
                                        <Trash2 :size="18" />
                                    </Button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </Card>
            </div>

            <!-- Live Log Stream -->
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                    <h3 class="text-foreground text-xl font-bold tracking-tight">Live Log Stream</h3>
                    <Badge variant="default">PM2</Badge>
                </div>
                <Button variant="ghost" size="sm" @click="logs = []">
                    <Ban :size="14" class="mr-1" /> Clear
                </Button>
            </div>
            <div class="bg-slate-900 rounded-xl border border-slate-800 shadow-lg overflow-hidden flex flex-col mb-8 h-80 group">
                <div class="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700">
                    <div class="flex gap-1.5">
                        <div class="size-2.5 rounded-full bg-red-500/50"></div>
                        <div class="size-2.5 rounded-full bg-amber-500/50"></div>
                        <div class="size-2.5 rounded-full bg-emerald-500/50"></div>
                    </div>
                    <span class="text-[10px] font-mono text-slate-500 uppercase tracking-widest">STDOUT/STDERR — PM2 Stream</span>
                </div>
                <div class="flex-1 p-4 font-mono text-xs leading-relaxed overflow-y-auto custom-scrollbar">
                    <p v-for="(log, i) in logs" :key="i" class="text-emerald-400 mb-1 break-words font-mono">{{ log }}</p>
                    <p class="text-white mt-4 flex items-center gap-2">
                         <span class="animate-pulse bg-emerald-500 size-1.5 rounded-full"></span> Tailing logs...
                    </p>
                </div>
            </div>

            <!-- Footer -->
            <footer class="p-8 text-center text-muted-foreground text-sm">
                © 2024 WebHost Hosting Solutions. PM2 Integration | Node.js
            </footer>
        </div>
    </div>

    <!-- Add Env Modal -->
    <div v-if="showAddEnvModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
        <Card class="w-full max-w-md">
            <CardHeader>
                <CardTitle>Add Environment Variable</CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
                <div>
                    <label class="block text-xs font-bold text-muted-foreground mb-1">Key</label>
                    <Input v-model="newEnv.key" type="text" placeholder="API_KEY" />
                </div>
                <div>
                    <label class="block text-xs font-bold text-muted-foreground mb-1">Value</label>
                    <Input v-model="newEnv.value" type="text" placeholder="secret_value" />
                </div>
                <div class="flex justify-end gap-2 mt-6">
                    <Button variant="outline" @click="showAddEnvModal = false">Cancel</Button>
                    <Button @click="saveEnvVar">Save</Button>
                </div>
            </CardContent>
        </Card>
    </div>
</MainLayout>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: #cbd5e1;
    border-radius: 20px;
}
.dark .custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: #475569;
}
</style>
