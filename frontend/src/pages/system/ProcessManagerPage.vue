<script setup lang="ts">
/**
 * ProcessManagerPage - PM2 Process Management
 * 
 * Features:
 * - Real-time process list (PM2)
 * - Start/Stop/Restart/Delete processes
 * - Environment Variable Management (.env editor)
 * - Live Log Stream (UI Only)
 */
import { ref, onMounted, onUnmounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { nodejsService, type Pm2Process } from '@/services/nodejs.service'
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
const envPath = ref('')  // Default path resolved from user
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
    const running = processes.value.filter(p => p.status === 'online').length
    const errored = processes.value.filter(p => p.status === 'errored').length
    const stopped = total - running - errored
    return { total, running, errored, stopped }
})

const filteredProcesses = computed(() => {
    if (!searchQuery.value) return processes.value
    const q = searchQuery.value.toLowerCase()
    return processes.value.filter(p => 
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
        // Keep previous data on error to avoid flickering, or show toast if critical
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
        // Parse "KEY=VALUE" strings
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
    // PM2 returns uptime as timestamp (start time)
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
    
    // Simulate log stream
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
                <h2 class="text-[#0d131b] dark:text-white text-4xl font-black leading-tight tracking-tight">PM2 Process Manager</h2>
                <p class="text-slate-500 dark:text-slate-400 text-base max-w-lg leading-relaxed mt-2">
                    Monitor and control your Node.js application processes in real-time with PM2.
                </p>
            </div>

            <!-- Stats Cards -->
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                <!-- Total -->
                <div class="bg-white dark:bg-slate-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm flex items-center justify-between">
                    <div>
                        <p class="text-xs font-bold text-slate-500 uppercase tracking-wider mb-1">Total Processes</p>
                        <h3 class="text-2xl font-black text-[#0d131b] dark:text-white">{{ stats.total }}</h3>
                    </div>
                    <div class="bg-slate-100 dark:bg-slate-800 p-2 rounded-lg text-slate-500">
                        <List :size="20" />
                    </div>
                </div>
                <!-- Running -->
                <div class="bg-white dark:bg-slate-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm flex items-center justify-between">
                    <div>
                        <p class="text-xs font-bold text-emerald-600 uppercase tracking-wider mb-1">Running</p>
                        <h3 class="text-2xl font-black text-emerald-600 dark:text-white">{{ stats.running }}</h3>
                    </div>
                    <div class="bg-emerald-50 dark:bg-emerald-900/20 p-2 rounded-lg text-emerald-600">
                        <CheckCircle :size="20" />
                    </div>
                </div>
                <!-- Errored -->
                <div class="bg-white dark:bg-slate-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm flex items-center justify-between">
                    <div>
                        <p class="text-xs font-bold text-rose-600 uppercase tracking-wider mb-1">Errored</p>
                        <h3 class="text-2xl font-black text-rose-600 dark:text-white">{{ stats.errored }}</h3>
                    </div>
                    <div class="bg-rose-50 dark:bg-rose-900/20 p-2 rounded-lg text-rose-600">
                        <AlertCircle :size="20" />
                    </div>
                </div>
                <!-- Stopped -->
                <div class="bg-white dark:bg-slate-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm flex items-center justify-between">
                    <div>
                        <p class="text-xs font-bold text-slate-500 uppercase tracking-wider mb-1">Stopped</p>
                        <h3 class="text-2xl font-black text-slate-400 dark:text-white">{{ stats.stopped }}</h3>
                    </div>
                    <div class="bg-slate-100 dark:bg-slate-800 p-2 rounded-lg text-slate-400">
                        <StopCircle :size="20" />
                    </div>
                </div>
            </div>

            <!-- Controls -->
            <div class="bg-white dark:bg-slate-900 p-6 rounded-xl border border-slate-200 dark:border-slate-800 shadow-sm mb-6 flex flex-wrap items-center justify-between gap-4">
                <div class="flex items-center gap-4">
                    <button @click="handleRestartAll" class="bg-primary hover:bg-primary/90 text-white font-bold py-2.5 px-5 rounded-lg flex items-center gap-2 transition-all shadow-md">
                        <RefreshCw :size="18" /> Restart All
                    </button>
                    <button @click="handleStopAll" class="bg-rose-600 hover:bg-rose-700 text-white font-bold py-2.5 px-5 rounded-lg flex items-center gap-2 transition-all shadow-md">
                        <Square :size="18" fill="currentColor" /> Stop All
                    </button>
                </div>
                <div class="flex items-center gap-3">
                    <button disabled class="opacity-50 cursor-not-allowed bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 font-bold py-2.5 px-5 rounded-lg flex items-center gap-2 transition-all">
                        <Plus :size="18" /> New Process
                    </button>
                </div>
            </div>

            <!-- Process List -->
            <div class="mb-10">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-4">
                    <h3 class="text-[#0d131b] dark:text-white text-xl font-bold tracking-tight">Process List</h3>
                    <div class="w-full md:w-96 relative group">
                        <div class="absolute inset-y-0 left-0 flex items-center pl-3.5 pointer-events-none text-slate-400 group-focus-within:text-primary transition-colors">
                            <Search :size="20" />
                        </div>
                        <input v-model="searchQuery" class="block w-full h-11 pl-11 pr-4 rounded-lg border-none bg-white dark:bg-slate-800 shadow-sm ring-1 ring-slate-200 dark:ring-slate-700 focus:ring-2 focus:ring-primary text-[#0d131b] dark:text-white placeholder:text-slate-400 text-sm font-medium" placeholder="Search processes..." type="text"/>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
                    <table class="w-full text-left border-collapse min-w-[800px]">
                        <thead>
                            <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Status</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Name</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">PID</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Uptime</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">CPU</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Memory</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Restarts</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                            <tr v-if="isLoading" class="animate-pulse">
                                <td colspan="8" class="px-6 py-8 text-center text-slate-500">Loading processes...</td>
                            </tr>
                            <tr v-else-if="filteredProcesses.length === 0">
                                <td colspan="8" class="px-6 py-8 text-center text-slate-500">No processes found</td>
                            </tr>
                            <tr v-for="proc in filteredProcesses" :key="proc.pm_id" class="hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors">
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <span :class="['size-2.5 rounded-full', proc.status === 'online' ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400']"></span>
                                        <span :class="['text-xs font-bold uppercase tracking-tighter', proc.status === 'online' ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-500']">{{ proc.status }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4"><span class="font-bold text-[#0d131b] dark:text-white">{{ proc.name }}</span></td>
                                <td class="px-6 py-4 font-mono text-xs text-slate-500">{{ proc.pid }}</td>
                                <td class="px-6 py-4 text-sm text-slate-600 dark:text-slate-400">{{ formatUptime(proc.uptime) }}</td>
                                <td class="px-6 py-4"><span class="text-xs font-bold text-slate-700 dark:text-slate-300">{{ proc.cpu }}%</span></td>
                                <td class="px-6 py-4"><span class="text-xs font-bold text-slate-700 dark:text-slate-300">{{ formatMemory(proc.memory) }}</span></td>
                                <td class="px-6 py-4"><span class="px-2 py-1 text-[10px] font-bold rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">{{ proc.restarts }}</span></td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex justify-end gap-2">
                                        <button v-if="proc.status === 'online'" @click="handleAction('stop', proc.name)" class="p-2 text-slate-400 hover:text-amber-500 transition-colors" title="Stop">
                                            <Square :size="18" fill="currentColor" />
                                        </button>
                                        <button v-else @click="handleAction('start', proc.name)" class="p-2 text-emerald-500 hover:text-emerald-600 transition-colors" title="Start">
                                            <Play :size="18" fill="currentColor" />
                                        </button>
                                        <button @click="handleAction('restart', proc.name)" class="p-2 text-slate-400 hover:text-primary transition-colors" title="Restart">
                                            <RefreshCw :size="18" />
                                        </button>
                                        <button @click="handleAction('delete', proc.name)" class="p-2 text-slate-400 hover:text-rose-500 transition-colors" title="Delete">
                                            <Trash2 :size="18" />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Environment Variables -->
            <div class="mb-10">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-4">
                    <div class="flex flex-col">
                        <h3 class="text-[#0d131b] dark:text-white text-xl font-bold tracking-tight">Environment Variables</h3>
                        <p class="text-slate-500 dark:text-slate-400 text-xs mt-1">Configure secrets and runtime settings for your application.</p>
                    </div>
                    <div class="flex gap-2">
                        <input v-model="envPath" type="text" placeholder="/path/to/project" class="h-9 px-3 rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs" />
                        <button @click="fetchEnvVars" class="bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 text-slate-700 dark:text-white px-3 py-2 rounded-lg text-xs font-bold">Load</button>
                        <button @click="showAddEnvModal = true" class="bg-primary text-white font-bold py-2 px-4 rounded-lg text-sm flex items-center gap-2 hover:bg-primary/90 transition-all shadow-sm">
                            <Plus :size="18" /> Add Variable
                        </button>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Key</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Value</th>
                                <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                            <tr v-if="envVars.length === 0"><td colspan="3" class="px-6 py-4 text-center text-sm text-slate-500">No variables loaded. Enter path and click Load.</td></tr>
                            <tr v-for="v in envVars" :key="v.key" class="hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors">
                                <td class="px-6 py-4">
                                    <code class="text-xs font-bold text-slate-700 dark:text-slate-300 bg-slate-100 dark:bg-slate-800 px-1.5 py-0.5 rounded">{{ v.key }}</code>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <span class="text-sm font-mono text-slate-600 dark:text-slate-400">{{ v.hidden ? '••••••••' : v.value }}</span>
                                        <button @click="v.hidden = !v.hidden" class="text-slate-400 hover:text-primary transition-colors">
                                            <component :is="v.hidden ? Eye : EyeOff" :size="16" />
                                        </button>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex justify-end gap-2">
                                        <button @click="deleteEnvVar(v.key)" class="p-2 text-slate-400 hover:text-rose-500 transition-colors" title="Delete">
                                            <Trash2 :size="18" />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

             <!-- Live Log Stream -->
             <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                    <h3 class="text-[#0d131b] dark:text-white text-xl font-bold tracking-tight">Live Log Stream</h3>
                    <span class="bg-primary/10 text-primary text-[10px] font-bold px-2 py-0.5 rounded uppercase">PM2</span>
                </div>
                <div class="flex gap-2">
                    <button @click="logs = []" class="text-xs font-bold text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 flex items-center gap-1 transition-colors">
                        <Ban :size="14" /> Clear
                    </button>
                </div>
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
            <footer class="p-8 text-center text-slate-400 text-sm">
                © 2024 WebHost Hosting Solutions. PM2 Integration | Node.js
            </footer>
        </div>
    </div>

    <!-- Add Env Modal -->
    <div v-if="showAddEnvModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
        <div class="bg-white dark:bg-slate-900 rounded-xl shadow-2xl w-full max-w-md p-6 border border-slate-200 dark:border-slate-800">
            <h3 class="text-lg font-bold mb-4 text-[#0d131b] dark:text-white">Add Environment Variable</h3>
            <div class="space-y-4">
                <div>
                    <label class="block text-xs font-bold text-slate-500 mb-1">Key</label>
                    <input v-model="newEnv.key" type="text" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800" placeholder="API_KEY">
                </div>
                <div>
                    <label class="block text-xs font-bold text-slate-500 mb-1">Value</label>
                    <input v-model="newEnv.value" type="text" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800" placeholder="secret_value">
                </div>
                <div class="flex justify-end gap-2 mt-6">
                    <button @click="showAddEnvModal = false" class="px-4 py-2 text-slate-500 hover:text-slate-700 font-bold text-sm">Cancel</button>
                    <button @click="saveEnvVar" class="px-4 py-2 bg-primary text-white rounded-lg font-bold text-sm">Save</button>
                </div>
            </div>
        </div>
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
