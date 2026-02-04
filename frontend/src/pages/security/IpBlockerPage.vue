<script setup lang="ts">
/**
 * IpBlockerPage - IP Blocker Management
 * 
 * Features:
 * - Add IP/Range form with reason selector
 * - Blocked IPs table with search
 * - Unblock functionality
 * - Pagination
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { securityService } from '@/services'
import { 
    Shield, Search, Download, Ban, Globe, Network,
    ChevronRight
} from 'lucide-vue-next'

// State
const blockedIps = ref<any[]>([])
const isLoading = ref(true)
const searchQuery = ref('')
const currentPage = ref(1)
const itemsPerPage = 10

// Form state
const newIpAddress = ref('')
const newBlockReason = ref('Brute Force Attempt')
const isSubmitting = ref(false)

const blockReasons = [
    'Brute Force Attempt',
    'Spam/Malicious Content',
    'Unauthorized Access',
    'DDoS Protection',
    'Other / Maintenance'
]

import { useToastStore } from '@/stores/toast'

const toast = useToastStore()

// Computed
const filteredIps = computed(() => {
    if (!searchQuery.value) return blockedIps.value
    const q = searchQuery.value.toLowerCase()
    return blockedIps.value.filter(ip => 
        ip.ip_address?.toLowerCase().includes(q) ||
        ip.reason?.toLowerCase().includes(q)
    )
})

const paginatedIps = computed(() => {
    const start = (currentPage.value - 1) * itemsPerPage
    return filteredIps.value.slice(start, start + itemsPerPage)
})

const totalPages = computed(() => Math.ceil(filteredIps.value.length / itemsPerPage))

// Methods
const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await securityService.getBlockedIps()
        blockedIps.value = res.data.data || []
    } catch (e) {
        console.error('Failed to load blocked IPs:', e)
        console.error('Failed to load blocked IPs:', e)
        toast.error('Failed to load blocked IPs')
    } finally {
        isLoading.value = false
    }
}

const blockIp = async () => {
    if (!newIpAddress.value.trim()) {
        toast.error('Please enter an IP address')
        return
    }
    isSubmitting.value = true
    try {
        await securityService.blockIp({
            ip_address: newIpAddress.value,
            reason: newBlockReason.value,
            type: 'block'
        })
        toast.success('IP blocked successfully')
        newIpAddress.value = ''
        await fetchData()
    } catch (e) {
        toast.error('Failed to block IP')
    } finally {
        isSubmitting.value = false
    }
}

const unblockIp = async (id: string) => {
    try {
        await securityService.unblockIp(id)
        toast.success('IP unblocked successfully')
        await fetchData()
    } catch (e) {
        toast.error('Failed to unblock IP')
    }
}

const getReasonColor = (reason: string) => {
    const r = reason?.toLowerCase() || ''
    if (r.includes('brute') || r.includes('ddos')) return 'bg-error/10 text-error'
    if (r.includes('spam') || r.includes('malicious')) return 'bg-error/10 text-error'
    if (r.includes('unauthorized')) return 'bg-warning/10 text-warning'
    return 'bg-error/10 text-error'
}

const getCountryFlag = (country: string) => {
    const flags: Record<string, string> = {
        'US': '🇺🇸', 'RU': '🇷🇺', 'CN': '🇨🇳', 'BR': '🇧🇷', 
        'DE': '🇩🇪', 'FR': '🇫🇷', 'GB': '🇬🇧', 'IN': '🇮🇳',
        'ID': '🇮🇩', 'JP': '🇯🇵'
    }
    return flags[country] || '🌐'
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
                        <span class="text-slate-600 dark:text-slate-300">IP Blocker</span>
                    </nav>
                    <h2 class="text-3xl font-black leading-tight tracking-tight text-[#0d131b] dark:text-white flex items-center gap-3">
                        <Ban :size="32" class="text-primary" />
                        IP Blocker
                    </h2>
                </div>
            </div>

            <!-- Add IP Form Section -->
            <section>
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-8 shadow-sm">
                    <h3 class="text-xl font-bold mb-6">Add an IP Address or Range</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 items-end">
                        <div class="flex flex-col gap-2">
                            <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">IP Address or Range</label>
                            <input v-model="newIpAddress" type="text" 
                                placeholder="e.g. 192.168.0.1 or 10.0.0.0/24"
                                class="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all text-sm outline-none" />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">Reason for Block</label>
                            <div class="relative">
                                <select v-model="newBlockReason"
                                    class="w-full appearance-none px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all text-sm outline-none cursor-pointer">
                                    <option v-for="reason in blockReasons" :key="reason" :value="reason">{{ reason }}</option>
                                </select>
                                <ChevronRight :size="16" class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 rotate-90 pointer-events-none" />
                            </div>
                        </div>
                        <button @click="blockIp" :disabled="isSubmitting"
                            class="w-full py-3 bg-primary text-white hover:bg-primary/90 rounded-lg text-sm font-bold transition-all shadow-lg shadow-primary/20 flex items-center justify-center gap-2 disabled:opacity-50">
                            <Shield :size="18" />
                            Block IP
                        </button>
                    </div>
                    <p class="mt-4 text-xs text-slate-400">You can specify an individual IP address (192.168.0.1) or a range in CIDR format (192.168.0.1/24).</p>
                </div>
            </section>

            <!-- Blocked IPs Table -->
            <section>
                <div class="flex items-center justify-between mb-6">
                    <div class="flex flex-col">
                        <h3 class="text-[22px] font-bold leading-tight tracking-tight">Currently Blocked IPs</h3>
                        <p class="text-xs text-slate-500 mt-1">There are {{ blockedIps.length }} active blocks currently enforced.</p>
                    </div>
                    <div class="flex gap-2">
                        <div class="relative">
                            <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                            <input v-model="searchQuery" type="text" placeholder="Search blocked list..."
                                class="pl-10 pr-4 py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg text-xs focus:ring-primary focus:border-primary w-64" />
                        </div>
                        <button class="p-2 border border-slate-200 dark:border-slate-800 rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                            <Download :size="18" class="text-slate-500" />
                        </button>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
                    <!-- Loading State -->
                    <div v-if="isLoading" class="p-12 text-center">
                        <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                        <p class="text-slate-500">Loading blocked IPs...</p>
                    </div>

                    <!-- Empty State -->
                    <div v-else-if="filteredIps.length === 0" class="p-12 text-center">
                        <Shield :size="48" class="mx-auto mb-4 text-slate-300" />
                        <p class="text-slate-500">No blocked IPs found</p>
                    </div>

                    <!-- Table -->
                    <template v-else>
                        <table class="w-full text-left">
                            <thead>
                                <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">IP / Range</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Country</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Reason</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider">Date Added</th>
                                    <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase tracking-wider text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                                <tr v-for="ip in paginatedIps" :key="ip.id" class="hover:bg-slate-50/50 dark:hover:bg-slate-800/30 transition-colors">
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-2">
                                            <component :is="ip.ip_address?.includes('/') ? Network : Globe" :size="18" class="text-slate-400" />
                                            <span class="text-sm font-bold font-mono">{{ ip.ip_address }}</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-2">
                                            <span class="text-lg">{{ getCountryFlag(ip.country || 'US') }}</span>
                                            <span class="text-sm">{{ ip.country_name || 'Unknown' }}</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 text-sm">
                                        <span :class="['px-2 py-1 rounded text-[10px] font-bold uppercase', getReasonColor(ip.reason)]">
                                            {{ ip.reason || 'Manual Block' }}
                                        </span>
                                    </td>
                                    <td class="px-6 py-4 text-sm text-slate-500">{{ ip.created_at?.split('T')[0] || 'Unknown' }}</td>
                                    <td class="px-6 py-4 text-right">
                                        <button @click="unblockIp(ip.id)"
                                            class="px-3 py-1.5 border border-slate-200 dark:border-slate-700 hover:border-primary hover:text-primary rounded text-xs font-bold transition-all">
                                            Unblock
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>

                        <!-- Pagination -->
                        <div class="px-6 py-4 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between">
                            <p class="text-xs text-slate-500">Showing {{ paginatedIps.length }} of {{ filteredIps.length }} blocked entries</p>
                            <div class="flex gap-2">
                                <button @click="currentPage--" :disabled="currentPage === 1"
                                    class="px-3 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                                    Previous
                                </button>
                                <button @click="currentPage++" :disabled="currentPage >= totalPages"
                                    class="px-3 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-xs hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
                                    Next
                                </button>
                            </div>
                        </div>
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
