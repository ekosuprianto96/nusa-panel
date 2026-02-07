<script setup lang="ts">
/**
 * DnsTrackerPage - DNS Tracker & Error Logs Viewer
 */
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { systemService } from '@/services/system.service'
import { Globe, Search, RefreshCw, Home, Flag, AlertCircle, Download, Trash2, Settings } from 'lucide-vue-next'

const isLoading = ref(false)
const domain = ref('')
const recordType = ref('A')
const recordTypes = ['A Record', 'AAAA', 'CNAME', 'MX', 'TXT']
const activeLogFilter = ref('critical')
const router = useRouter()

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
])

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

const getLevelVariant = (level: string): 'destructive' | 'warning' | 'secondary' => {
    switch (level) {
        case 'CRITICAL': return 'destructive'
        case 'WARNING': return 'warning'
        default: return 'secondary'
    }
}

const runTrace = async () => {
    if (!domain.value.trim()) {
        showToast('Please enter a domain or IP', 'error')
        return
    }
    showToast(`Running diagnostic for ${domain.value}...`, 'info')
    isLoading.value = true
    lookupResults.value = []
    traceRoute.value = []

    try {
        const [dnsRes, traceRes] = await Promise.allSettled([
            systemService.dnsLookup(domain.value, recordType.value),
            systemService.traceRoute(domain.value)
        ])

        if (dnsRes.status === 'fulfilled') {
            const raw = dnsRes.value.data.data
            lookupResults.value = raw.split('\n').filter(l => l.trim()).map((line) => ({
                type: recordType.value,
                value: line,
                ttl: 300
            }))
            if (lookupResults.value.length === 0) {
                 lookupResults.value.push({ type: recordType.value, value: 'No records found', ttl: 0 })
            }
        } else {
             showToast('DNS Lookup failed', 'error')
        }

        if (traceRes.status === 'fulfilled') {
            const lines = traceRes.value.data.data
            traceRoute.value = lines.map((line: string, index: number) => {
                 const parts = line.trim().split(/\s+/)
                 const hopNum = parseInt(parts[0] ?? '', 10) || (index + 1)
                 const name = parts.length > 1 ? (parts[1] || 'Unknown') : 'Unknown'
                 const latency = parts.length > 3 ? (parts[3] || '*') + ' ' + (parts[4] || 'ms') : '*'
                 return { hop: hopNum, name: name, latency: latency, status: 'complete' }
            })
        } else {
             showToast('Trace Route failed', 'error')
        }
        
        showToast('Diagnostic completed', 'success')
    } catch (e: any) {
        showToast('Diagnostic failed', 'error')
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
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-destructive text-destructive-foreground' : 'bg-primary text-primary-foreground']">{{ toast.message }}</div>
    </div>

    <div class="space-y-6">
        <!-- Header -->
        <div>
            <AppBreadcrumb
                class="mb-4"
                :items="[
                    { label: 'System Tools', icon: Settings, onClick: () => router.push('/dashboard/system') },
                    { label: 'DNS & Logs', current: true }
                ]"
            />
            <div class="flex justify-between items-end">
                <div>
                    <h2 class="text-foreground text-3xl font-black tracking-tight">DNS Tracker & Error Logs Viewer</h2>
                    <p class="text-muted-foreground text-sm mt-1">Diagnostic networking tools and system event monitoring</p>
                </div>
                <Button variant="outline">
                    <RefreshCw :size="16" class="mr-2" /> Refresh View
                </Button>
            </div>
        </div>

        <!-- Track DNS Section -->
        <Card class="rounded-xl overflow-hidden">
            <CardHeader class="bg-muted/30 flex-row items-center justify-between">
                <CardTitle class="flex items-center gap-2">
                    <Search :size="18" class="text-primary" /> Track DNS & Trace Route
                </CardTitle>
                <Badge variant="default">Live Diagnostic</Badge>
            </CardHeader>
            <CardContent class="p-6 space-y-8">
                <div class="flex flex-wrap gap-4 items-end">
                    <div class="flex-1 min-w-[300px]">
                        <label class="block text-sm font-medium text-muted-foreground mb-2">Domain Name or IP Address</label>
                        <div class="relative">
                            <Globe :size="20" class="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground" />
                            <Input v-model="domain" class="pl-12 h-12" placeholder="e.g. google.com" />
                        </div>
                    </div>
                    <div class="w-48">
                        <label class="block text-sm font-medium text-muted-foreground mb-2">Record Type</label>
                        <select v-model="recordType" class="w-full bg-background border border-border rounded-lg px-4 py-3 focus:ring-2 focus:ring-primary focus:border-primary">
                            <option v-for="rt in recordTypes" :key="rt">{{ rt }}</option>
                        </select>
                    </div>
                    <Button @click="runTrace" class="h-12">
                        <Search :size="18" class="mr-2" /> Run Trace
                    </Button>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    <!-- DNS Lookup Results -->
                    <div class="lg:col-span-1 space-y-4">
                        <p class="text-xs font-bold uppercase text-muted-foreground tracking-widest">Lookup Results</p>
                        <div class="space-y-3">
                            <div v-for="result in lookupResults" :key="result.type" class="bg-muted p-4 rounded-lg border border-border">
                                <div class="flex justify-between items-start mb-1">
                                    <span class="text-xs font-bold text-primary">{{ result.type }}</span>
                                    <span class="text-[10px] text-muted-foreground">{{ result.ttl ? `TTL: ${result.ttl}` : `Priority: ${result.priority}` }}</span>
                                </div>
                                <p class="font-mono text-sm text-foreground">{{ result.value }}</p>
                            </div>
                        </div>
                    </div>

                    <!-- Trace Route Visualizer -->
                    <div class="lg:col-span-2 space-y-4">
                        <p class="text-xs font-bold uppercase text-muted-foreground tracking-widest">Trace Route Path</p>
                        <div class="bg-muted rounded-lg border border-border p-4 overflow-x-auto">
                            <div class="flex items-center gap-0">
                                <div v-for="(hop, index) in traceRoute" :key="hop.hop" class="flex items-center">
                                    <div class="flex flex-col items-center min-w-[120px]">
                                        <div :class="['size-8 rounded-full flex items-center justify-center mb-2 z-10', hop.status === 'complete' ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/30' : hop.status === 'pending' ? 'bg-primary/20 border border-primary/50' : 'bg-muted border border-border']">
                                            <Home v-if="hop.hop === 1" :size="16" />
                                            <Flag v-else-if="hop.hop === traceRoute.length" :size="16" :class="hop.status === 'waiting' ? 'text-muted-foreground' : ''" />
                                            <span v-else class="text-xs font-bold">{{ hop.hop }}</span>
                                        </div>
                                        <span class="text-[10px] font-bold text-foreground">{{ hop.name }}</span>
                                        <span class="text-[10px] text-muted-foreground">{{ hop.latency }}</span>
                                    </div>
                                    <div v-if="index < traceRoute.length - 1" :class="['h-0.5 flex-1 min-w-[40px]', hop.status === 'complete' ? 'bg-primary' : hop.status === 'pending' ? 'border-t-2 border-dashed border-border' : 'bg-muted']"></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>

        <!-- Error Logs Section -->
        <Card class="rounded-xl overflow-hidden flex-1 min-h-[400px] flex flex-col">
            <CardHeader class="bg-muted/30 flex-row flex-wrap items-center justify-between gap-4">
                <div class="flex items-center gap-4">
                    <CardTitle class="flex items-center gap-2">
                        <AlertCircle :size="18" class="text-destructive" /> System Error Logs
                    </CardTitle>
                    <div class="flex bg-muted p-1 rounded-lg border border-border">
                        <button @click="filterLogs('critical')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'critical' ? 'bg-destructive/20 text-destructive' : 'text-muted-foreground hover:text-foreground']">Critical</button>
                        <button @click="filterLogs('warning')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'warning' ? 'bg-amber-500/20 text-amber-500' : 'text-muted-foreground hover:text-foreground']">Warning</button>
                        <button @click="filterLogs('notice')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'notice' ? 'bg-blue-500/20 text-blue-500' : 'text-muted-foreground hover:text-foreground']">Notice</button>
                        <button @click="filterLogs('all')" :class="['px-3 py-1.5 rounded-md text-xs font-bold', activeLogFilter === 'all' ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground']">All</button>
                    </div>
                </div>
                <div class="flex items-center gap-3">
                    <div class="relative">
                        <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                        <Input class="pl-9 w-64" placeholder="Search logs..." />
                    </div>
                    <Button variant="ghost" size="icon" class="text-destructive"><Trash2 :size="18" /></Button>
                    <Button variant="ghost" size="icon"><Download :size="18" /></Button>
                </div>
            </CardHeader>

            <div class="flex-1 bg-muted/30 p-4 font-mono text-[13px] leading-relaxed overflow-y-auto max-h-96">
                <div class="space-y-1">
                    <div v-for="log in filteredLogs" :key="log.line" class="flex gap-4 group hover:bg-background p-1 rounded transition-colors">
                        <span class="text-muted-foreground/50 shrink-0 select-none">{{ log.line }}</span>
                        <span class="text-muted-foreground shrink-0">[{{ log.time }}]</span>
                        <Badge :variant="getLevelVariant(log.level)" class="h-fit">{{ log.level }}</Badge>
                        <span class="text-foreground">{{ log.message }}</span>
                    </div>
                    <div class="flex gap-4 group p-1 border-l-2 border-primary animate-pulse">
                        <span class="text-muted-foreground/50 shrink-0 select-none">{{ logs.length + 1 }}</span>
                        <span class="text-muted-foreground shrink-0">[{{ new Date().toISOString().slice(0, 19).replace('T', ' ') }}]</span>
                        <Badge variant="secondary">WAITING</Badge>
                        <span class="text-muted-foreground italic">Listening for system events...</span>
                    </div>
                </div>
            </div>

            <div class="px-6 py-3 border-t border-border flex justify-between items-center text-xs text-muted-foreground font-medium bg-muted/30">
                <div class="flex items-center gap-4">
                    <span class="flex items-center gap-1.5">
                        <span class="size-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"></span>
                        Live Stream Active
                    </span>
                    <span>Displaying {{ filteredLogs.length }} of {{ logs.length }} logs</span>
                </div>
            </div>
        </Card>
    </div>
</MainLayout>
</template>
