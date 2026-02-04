<script setup lang="ts">
/**
 * SslTlsPage - SSL/TLS Status Management
 * 
 * Features:
 * - Domain SSL status table
 * - AutoSSL functionality
 * - Certificate renewal
 * - Stats overview
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { domainService, securityService } from '@/services'
import { 
    Shield, Search, RefreshCw, ChevronRight, Lock, CheckCircle, 
    AlertTriangle, Globe
} from 'lucide-vue-next'

// State
const domains = ref<any[]>([])
const isLoading = ref(true)
const searchQuery = ref('')
const sortBy = ref('domain_name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isRunningAutoSsl = ref(false)

// Toast
import { useToastStore } from '@/stores/toast'

const toast = useToastStore()

// Computed
const filteredDomains = computed(() => {
    let list = domains.value
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase()
        list = list.filter(d => d.domain_name?.toLowerCase().includes(q))
    }
    return list.sort((a, b) => {
        const aVal = a[sortBy.value] || ''
        const bVal = b[sortBy.value] || ''
        return sortOrder.value === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal)
    })
})

const stats = computed(() => {
    const total = domains.value.length
    const active = domains.value.filter(d => d.ssl_status === 'active' || d.ssl_enabled).length
    const expiring = domains.value.filter(d => {
        if (!d.ssl_expiry_at) return false
        const expiry = new Date(d.ssl_expiry_at)
        const now = new Date()
        const diffDays = Math.ceil((expiry.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
        return diffDays <= 30 && diffDays > 0
    }).length
    return { total, active, expiring, pending: total - active }
})

// Methods
const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await domainService.listDomains()
        domains.value = res.data.data || []
    } catch (e) {
        console.error('Failed to load domains:', e)
        toast.error('Failed to load domains')
    } finally {
        isLoading.value = false
    }
}

const runAutoSsl = async () => {
    isRunningAutoSsl.value = true
    try {
        await securityService.runAutoSsl()
        toast.success('AutoSSL completed successfully')
        await fetchData()
    } catch (e) {
        toast.error('AutoSSL process failed')
    } finally {
        isRunningAutoSsl.value = false
    }
}

const renewCertificate = async (domainId: string) => {
    try {
        await securityService.renewSsl(domainId)
        toast.success('Certificate renewal initiated')
        await fetchData()
    } catch (e) {
        toast.error('Failed to renew certificate')
    }
}

const toggleSort = (field: string) => {
    if (sortBy.value === field) {
        sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
    } else {
        sortBy.value = field
        sortOrder.value = 'asc'
    }
}

const getSslStatusClass = (domain: any) => {
    if (domain.ssl_status === 'active' || domain.ssl_enabled) {
        return 'bg-success/10 text-success'
    }
    if (domain.ssl_status === 'expiring') {
        return 'bg-warning/10 text-warning'
    }
    return 'bg-slate-100 text-slate-500 dark:bg-slate-800'
}

const getSslStatusText = (domain: any) => {
    if (domain.ssl_status === 'active' || domain.ssl_enabled) return 'Active'
    if (domain.ssl_status === 'expiring') return 'Expiring'
    return 'Inactive'
}

onMounted(fetchData)
</script>

<template>
    <MainLayout>


        <div class="space-y-8">
            <!-- Page Header -->
            <div class="flex flex-wrap justify-between items-center gap-4">
                <div class="flex flex-col gap-2">
                    <nav class="flex items-center gap-2 text-xs font-medium text-slate-400 mb-1">
                        <router-link to="/dashboard/security" class="hover:text-primary transition-colors">Security Center</router-link>
                        <ChevronRight :size="12" />
                        <span class="text-slate-600 dark:text-slate-300">SSL/TLS Status</span>
                    </nav>
                    <h2 class="text-3xl font-black leading-tight tracking-tight text-[#0d131b] dark:text-white flex items-center gap-3">
                        <Lock :size="32" class="text-primary" />
                        SSL/TLS Status
                    </h2>
                </div>
                <button @click="runAutoSsl" :disabled="isRunningAutoSsl"
                    class="flex items-center gap-2 px-5 py-2.5 bg-primary text-white hover:bg-primary/90 rounded-lg text-sm font-bold transition-all shadow-lg shadow-primary/20 disabled:opacity-50">
                    <RefreshCw :size="18" :class="{ 'animate-spin': isRunningAutoSsl }" />
                    {{ isRunningAutoSsl ? 'Running...' : 'Run AutoSSL' }}
                </button>
            </div>

            <!-- Stats Grid -->
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                            <Globe :size="20" class="text-primary" />
                        </div>
                        <div>
                            <p class="text-2xl font-black">{{ stats.total }}</p>
                            <p class="text-xs text-slate-500">Total Domains</p>
                        </div>
                    </div>
                </div>
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-success/10 rounded-lg flex items-center justify-center">
                            <CheckCircle :size="20" class="text-success" />
                        </div>
                        <div>
                            <p class="text-2xl font-black">{{ stats.active }}</p>
                            <p class="text-xs text-slate-500">SSL Active</p>
                        </div>
                    </div>
                </div>
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-warning/10 rounded-lg flex items-center justify-center">
                            <AlertTriangle :size="20" class="text-warning" />
                        </div>
                        <div>
                            <p class="text-2xl font-black">{{ stats.expiring }}</p>
                            <p class="text-xs text-slate-500">Expiring Soon</p>
                        </div>
                    </div>
                </div>
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 bg-slate-100 dark:bg-slate-800 rounded-lg flex items-center justify-center">
                            <Shield :size="20" class="text-slate-500" />
                        </div>
                        <div>
                            <p class="text-2xl font-black">{{ stats.pending }}</p>
                            <p class="text-xs text-slate-500">Pending</p>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Domains Table -->
            <section>
                <div class="flex items-center justify-between mb-6">
                    <h3 class="text-[22px] font-bold leading-tight tracking-tight">Domain SSL Status</h3>
                    <div class="relative">
                        <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                        <input v-model="searchQuery" type="text" placeholder="Search domains..."
                            class="pl-10 pr-4 py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg text-xs focus:ring-primary focus:border-primary w-64" />
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
                    <!-- Loading State -->
                    <div v-if="isLoading" class="p-12 text-center">
                        <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                        <p class="text-slate-500">Loading domains...</p>
                    </div>

                    <!-- Empty State -->
                    <div v-else-if="filteredDomains.length === 0" class="p-12 text-center">
                        <Globe :size="48" class="mx-auto mb-4 text-slate-300" />
                        <p class="text-slate-500">No domains found</p>
                    </div>

                    <!-- Table -->
                    <template v-else>
                        <table class="w-full text-left">
                            <thead>
                                <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                                    <th @click="toggleSort('domain_name')" class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider cursor-pointer hover:text-primary">
                                        Domain Name
                                    </th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">SSL Status</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Expiry Date</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                                <tr v-for="domain in filteredDomains" :key="domain.id" class="hover:bg-slate-50/50 dark:hover:bg-slate-800/30 transition-colors">
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-2">
                                            <Lock :size="16" class="text-primary" />
                                            <span class="text-sm font-medium">{{ domain.domain_name }}</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4">
                                        <span :class="['px-2 py-1 rounded text-[10px] font-bold uppercase', getSslStatusClass(domain)]">
                                            {{ getSslStatusText(domain) }}
                                        </span>
                                    </td>
                                    <td class="px-6 py-4 text-sm text-slate-500">
                                        {{ domain.ssl_expiry_at?.split('T')[0] || 'N/A' }}
                                    </td>
                                    <td class="px-6 py-4 text-right">
                                        <button @click="renewCertificate(domain.id)"
                                            class="px-3 py-1.5 border border-slate-200 dark:border-slate-700 hover:border-primary hover:text-primary rounded text-xs font-bold transition-all">
                                            Renew
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </template>
                </div>
            </section>
        </div>
    </MainLayout>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
    transition: opacity 0.3s ease, transform 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
    opacity: 0;
    transform: translateY(-10px);
}
</style>
