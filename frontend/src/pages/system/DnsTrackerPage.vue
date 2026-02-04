<script setup lang="ts">
/**
 * DnsTrackerPage - DNS Tracker & Error Logs Viewer
 */
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { systemService } from '@/services/system.service'
import { Globe, Search, RefreshCw, Home, Flag, AlertCircle, Download, Trash2 } from 'lucide-vue-next'

const isLoading = ref(false)
const domain = ref('')
const recordType = ref('A')
const recordTypes = ['A Record', 'AAAA', 'CNAME', 'MX', 'TXT']
const activeLogFilter = ref('critical')

const lookupResults = ref([
    { type: 'A', value: '142.250.190.46', ttl: 300 },
    { type: 'MX', value: 'aspmx.l.google.com', priority: 10 }
])

const traceRoute = ref([
    { hop: 1, name: 'Local Gateway', latency: '1ms', status: 'complete' },
    { hop: 2, name: 'ISP Backbone', latency: '12ms', status: 'complete' },
    { hop: 3, name: 'IXP Node', latency: '24ms', status: 'pending' },
    { hop: 4, name: 'Target', latency: '--', status: 'waiting' }
])

const logs = ref([
    { line: 104, time: '2023-11-22 14:22:01', level: 'CRITICAL', message: 'Connection failure to primary DNS cluster cluster-us-east-1. Retrying via secondary.' },
    { line: 105, time: '2023-11-22 14:22:05', level: 'WARNING', message: 'High latency detected on hop 7 (450ms). Possible network congestion at 203.0.113.1.' },
    { line: 106, time: '2023-11-22 14:23:12', level: 'NOTICE', message: 'Cache refresh initiated for 1,422 entries in local resolver.' },
    { line: 107, time: '2023-11-22 14:24:45', level: 'CRITICAL', message: 'Out of memory exception in dns_watchdog_process. PID: 45012. Dumping core...' },
    { line: 108, time: '2023-11-22 14:25:01', level: 'NOTICE', message: 'Watchdog process restarted successfully. Auto-recovery sequence complete.' },
    { line: 109, time: '2023-11-22 14:28:10', level: 'WARNING', message: 'Certificate for secure.gateway.internal expires in 3 days.' },
    { line: 110, time: '2023-11-22 14:30:22', level: 'NOTICE', message: 'New dynamic IP assignment received from upstream provider.' }
])

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

const getLevelClass = (level: string) => {
    switch (level) {
        case 'CRITICAL': return 'bg-red-500/20 text-red-400'
        case 'WARNING': return 'bg-amber-500/20 text-amber-400'
        case 'NOTICE': return 'bg-blue-500/20 text-blue-400'
        default: return 'bg-slate-500/20 text-slate-400'
    }
}

const runTrace = async () => {
    if (!domain.value.trim()) {
        showToast('Please enter a domain or IP', 'error')
        return
    }
    showToast(`Running diagnostic for ${domain.value}...`, 'info')
    isLoading.value = true
    
    // Reset results
    lookupResults.value = []
    traceRoute.value = []

    try {
        // Run in parallel
        const [dnsRes, traceRes] = await Promise.allSettled([
            systemService.dnsLookup(domain.value, recordType.value),
            systemService.traceRoute(domain.value)
        ])

        // Handle DNS Result
        if (dnsRes.status === 'fulfilled') {
            const raw = dnsRes.value.data.data
            // Simple parsing of "dig +short" output
            // Output is usually just lines of IPs or records
            lookupResults.value = raw.split('\n').filter(l => l.trim()).map((line) => ({
                type: recordType.value,
                value: line,
                ttl: 300 // default as dig +short doesn't show TTL
            }))
            if (lookupResults.value.length === 0) {
                 lookupResults.value.push({ type: recordType.value, value: 'No records found', ttl: 0 })
            }
        } else {
             showToast('DNS Lookup failed', 'error')
        }

        // Handle Trace Result
        if (traceRes.status === 'fulfilled') {
            const lines = traceRes.value.data.data
            // Parse traceroute lines
            traceRoute.value = lines.map((line: string, index: number) => {
                // Simplified parsing: Hop number is index+1. Info is the line.
                // Regex could be better but sticking to simple for now.
                // Typical line: " 1  192.168.1.1 (192.168.1.1)  0.345 ms  0.456 ms  0.567 ms"
                 const parts = line.trim().split(/\s+/)
                 const hopNum = parseInt(parts[0]) || (index + 1)
                 const name = parts.length > 1 ? (parts[1] || 'Unknown') : 'Unknown'
                 const latency = parts.length > 3 ? (parts[3] || '*') + ' ' + (parts[4] || 'ms') : '*'
                 
                 return {
                    hop: hopNum,
                    name: name,
                    latency: latency,
                    status: 'complete'
                 }
            })
        } else {
             showToast('Trace Route failed', 'error')
        }
        
        showToast('Diagnostic completed', 'success')

    } catch (e: any) {
        showToast('Diagnostic failed', 'error')
        console.error(e)
    } finally {
        isLoading.value = false
    }
}

const filteredLogs = ref(logs.value)

const filterLogs = (filter: string) => {
    activeLogFilter.value = filter
    if (filter === 'all') {
        filteredLogs.value = logs.value
    } else {
        filteredLogs.value = logs.value.filter(l => l.level.toLowerCase() === filter.toLowerCase())
    }
}

onMounted(() => { filteredLogs.value = logs.value.filter(l => l.level === 'CRITICAL') })
</script>

<template>
<MainLayout>
    <div class="fixed top-4 right-4 z-50 space-y-2">
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-red-500 text-white' : 'bg-primary text-white']">{{ toast.message }}</div>
    </div>

    <div class="space-y-6">
        <!-- Breadcrumbs & Header -->
        <div>
            <nav class="flex items-center gap-2 mb-4">
                <router-link to="/dashboard/system" class="text-slate-500 dark:text-slate-400 text-sm hover:text-primary transition-colors">System Tools</router-link>
                <span class="text-slate-500 dark:text-slate-600 text-sm">/</span>
                <span class="text-[#0d131b] dark:text-white text-sm font-medium">DNS & Logs</span>
            </nav>
            <div class="flex justify-between items-end">
                <div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">DNS Tracker & Error Logs Viewer</h2>
                    <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">Diagnostic networking tools and system event monitoring</p>
                </div>
                <button class="bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 px-4 py-2 rounded-lg text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors flex items-center gap-2">
                    <RefreshCw :size="16" /> Refresh View
                </button>
            </div>
        </div>

        <!-- Track DNS Section -->
        <section class="bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
            <div class="p-5 border-b border-slate-100 dark:border-slate-800 flex justify-between items-center">
                <h3 class="text-lg font-bold flex items-center gap-2">
                    <Search :size="18" class="text-primary" /> Track DNS & Trace Route
                </h3>
                <span class="px-2 py-1 bg-primary/10 text-primary text-[10px] font-bold uppercase tracking-wider rounded">Live Diagnostic</span>
            </div>
            <div class="p-6">
                <div class="flex flex-wrap gap-4 items-end mb-8">
                    <div class="flex-1 min-w-[300px]">
                        <label class="block text-sm font-medium text-slate-500 dark:text-slate-400 mb-2">Domain Name or IP Address</label>
                        <div class="relative">
                            <Globe :size="20" class="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" />
                            <input v-model="domain" class="w-full bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg pl-12 pr-4 py-3 focus:ring-2 focus:ring-primary focus:border-primary" placeholder="e.g. google.com" type="text" />
                        </div>
                    </div>
                    <div class="w-48">
                        <label class="block text-sm font-medium text-slate-500 dark:text-slate-400 mb-2">Record Type</label>
                        <select v-model="recordType" class="w-full bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-4 py-3 focus:ring-2 focus:ring-primary focus:border-primary">
                            <option v-for="rt in recordTypes" :key="rt">{{ rt }}</option>
                        </select>
                    </div>
                    <button @click="runTrace" class="bg-primary hover:bg-primary/90 text-white font-bold py-3 px-8 rounded-lg transition-all shadow-lg shadow-primary/20 flex items-center gap-2">
                        <Search :size="18" /> Run Trace
                    </button>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <!-- DNS Lookup Results -->
                    <div class="lg:col-span-1 space-y-4">
                        <p class="text-xs font-bold uppercase text-slate-500 dark:text-slate-400 tracking-widest">Lookup Results</p>
                        <div class="space-y-3">
                            <div v-for="result in lookupResults" :key="result.type" class="bg-slate-50 dark:bg-slate-800 p-4 rounded-lg border border-slate-200 dark:border-slate-700">
                                <div class="flex justify-between items-start mb-1">
                                    <span class="text-xs font-bold text-primary">{{ result.type }}</span>
                                    <span class="text-[10px] text-slate-500">{{ result.ttl ? `TTL: ${result.ttl}` : `Priority: ${result.priority}` }}</span>
                                </div>
                                <p class="font-mono text-sm">{{ result.value }}</p>
                            </div>
                        </div>
                    </div>

                    <!-- Trace Route Visualizer -->
                    <div class="lg:col-span-2 space-y-4">
                        <p class="text-xs font-bold uppercase text-slate-500 dark:text-slate-400 tracking-widest">Trace Route Path</p>
                        <div class="bg-slate-50 dark:bg-slate-800 rounded-lg border border-slate-200 dark:border-slate-700 p-4 overflow-x-auto">
                            <div class="flex items-center gap-0">
                                <div v-for="(hop, index) in traceRoute" :key="hop.hop" class="flex items-center">
                                    <div class="flex flex-col items-center min-w-[120px]">
                                        <div :class="['size-8 rounded-full flex items-center justify-center mb-2 z-10', hop.status === 'complete' ? 'bg-primary text-white shadow-lg shadow-primary/30' : hop.status === 'pending' ? 'bg-primary/20 border border-primary/50' : 'bg-slate-200 dark:bg-slate-700 border border-slate-300 dark:border-slate-600']">
                                            <Home v-if="hop.hop === 1" :size="16" />
                                            <Flag v-else-if="hop.hop === traceRoute.length" :size="16" :class="hop.status === 'waiting' ? 'text-slate-400' : ''" />
                                            <span v-else class="text-xs font-bold">{{ hop.hop }}</span>
                                        </div>
                                        <span class="text-[10px] font-bold">{{ hop.name }}</span>
                                        <span class="text-[10px] text-slate-400">{{ hop.latency }}</span>
                                    </div>
                                    <div v-if="index < traceRoute.length - 1" :class="['h-0.5 flex-1 min-w-[40px]', hop.status === 'complete' ? 'bg-primary' : hop.status === 'pending' ? 'border-t-2 border-dashed border-slate-300 dark:border-slate-600' : 'bg-slate-200 dark:bg-slate-700']"></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>

        <!-- Error Logs Section -->
        <section class="flex-1 min-h-[400px] flex flex-col bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
            <div class="p-5 border-b border-slate-100 dark:border-slate-800 flex flex-wrap justify-between items-center gap-4">
                <div class="flex items-center gap-4">
                    <h3 class="text-lg font-bold flex items-center gap-2">
                        <AlertCircle :size="18" class="text-red-500" /> System Error Logs
                    </h3>
                    <div class="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-lg border border-slate-200 dark:border-slate-700">
                        <button @click="filterLogs('critical')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'critical' ? 'bg-red-500/20 text-red-400 border border-red-500/30' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white']">Critical</button>
                        <button @click="filterLogs('warning')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'warning' ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white']">Warning</button>
                        <button @click="filterLogs('notice')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'notice' ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white']">Notice</button>
                        <button @click="filterLogs('all')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'all' ? 'bg-slate-200 dark:bg-slate-600' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white']">All</button>
                    </div>
                </div>
                <div class="flex items-center gap-3">
                    <div class="relative">
                        <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                        <input class="bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg pl-9 pr-4 py-1.5 text-sm w-64 focus:ring-1 focus:ring-primary" placeholder="Search logs..." type="text" />
                    </div>
                    <button class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors text-slate-500 hover:text-red-500"><Trash2 :size="18" /></button>
                    <button class="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors text-slate-500 hover:text-primary"><Download :size="18" /></button>
                </div>
            </div>

            <div class="flex-1 bg-slate-50 dark:bg-slate-800/50 p-4 font-mono text-[13px] leading-relaxed overflow-y-auto max-h-96">
                <div class="space-y-1">
                    <div v-for="log in filteredLogs" :key="log.line" class="flex gap-4 group hover:bg-white dark:hover:bg-white/5 p-1 rounded transition-colors">
                        <span class="text-slate-400/50 shrink-0 select-none">{{ log.line }}</span>
                        <span class="text-slate-500 shrink-0">[{{ log.time }}]</span>
                        <span :class="['px-1.5 py-0.5 rounded text-[10px] font-bold h-fit mt-1', getLevelClass(log.level)]">{{ log.level }}</span>
                        <span class="text-[#0d131b] dark:text-white">{{ log.message }}</span>
                    </div>
                    <div class="flex gap-4 group p-1 border-l-2 border-primary animate-pulse">
                        <span class="text-slate-400/50 shrink-0 select-none">{{ logs.length + 1 }}</span>
                        <span class="text-slate-500 shrink-0">[{{ new Date().toISOString().slice(0, 19).replace('T', ' ') }}]</span>
                        <span class="px-1.5 py-0.5 rounded bg-slate-200 dark:bg-slate-700 text-slate-500 text-[10px] font-bold h-fit mt-1">WAITING</span>
                        <span class="text-slate-500 italic">Listening for system events...</span>
                    </div>
                </div>
            </div>

            <div class="px-6 py-3 border-t border-slate-100 dark:border-slate-800 flex justify-between items-center text-xs text-slate-500 font-medium bg-slate-50/50 dark:bg-slate-800/20">
                <div class="flex items-center gap-4">
                    <span class="flex items-center gap-1.5">
                        <span class="size-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"></span>
                        Live Stream Active
                    </span>
                    <span>Displaying {{ filteredLogs.length }} of {{ logs.length }} logs</span>
                </div>
            </div>
        </section>
    </div>
</MainLayout>
</template>
