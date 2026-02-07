<script setup lang="ts">
/**
 * IpBlockerPage - IP Blocker Management
 */
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { securityService } from '@/services'
import { Shield, Search, Download, Ban, Globe, Network } from 'lucide-vue-next'

const router = useRouter()
const blockedIps = ref<any[]>([])
const isLoading = ref(true)
const searchQuery = ref('')
const currentPage = ref(1)
const itemsPerPage = 10

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

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await securityService.getBlockedIps()
        blockedIps.value = res.data.data || []
    } catch (e) {
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

const getReasonVariant = (reason: string): 'destructive' | 'warning' | 'secondary' => {
    const r = reason?.toLowerCase() || ''
    if (r.includes('brute') || r.includes('ddos') || r.includes('spam') || r.includes('malicious')) return 'destructive'
    if (r.includes('unauthorized')) return 'warning'
    return 'destructive'
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
        <!-- Header -->
        <div>
            <AppBreadcrumb
                class="mb-4"
                :items="[
                    { label: 'Security Center', icon: Shield, onClick: () => router.push('/dashboard/security') },
                    { label: 'IP Blocker', current: true }
                ]"
            />
            <h2 class="text-3xl font-black leading-tight tracking-tight text-foreground flex items-center gap-3">
                <Ban :size="32" class="text-primary" />
                IP Blocker
            </h2>
        </div>

        <!-- Add IP Form -->
        <Card class="rounded-xl">
            <CardHeader>
                <CardTitle>Add an IP Address or Range</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 items-end">
                    <div class="flex flex-col gap-2">
                        <label class="text-xs font-bold text-muted-foreground uppercase tracking-wider">IP Address or Range</label>
                        <Input v-model="newIpAddress" placeholder="e.g. 192.168.0.1 or 10.0.0.0/24" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="text-xs font-bold text-muted-foreground uppercase tracking-wider">Reason for Block</label>
                        <select v-model="newBlockReason" class="w-full appearance-none px-4 py-2 bg-muted border border-border rounded-lg focus:ring-2 focus:ring-primary/20 focus:border-primary text-sm">
                            <option v-for="reason in blockReasons" :key="reason" :value="reason">{{ reason }}</option>
                        </select>
                    </div>
                    <Button @click="blockIp" :disabled="isSubmitting" class="w-full">
                        <Shield :size="18" class="mr-2" /> Block IP
                    </Button>
                </div>
                <p class="mt-4 text-xs text-muted-foreground">You can specify an individual IP address (192.168.0.1) or a range in CIDR format (192.168.0.1/24).</p>
            </CardContent>
        </Card>

        <!-- Blocked IPs Table -->
        <section>
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h3 class="text-[22px] font-bold leading-tight tracking-tight text-foreground">Currently Blocked IPs</h3>
                    <p class="text-xs text-muted-foreground mt-1">There are {{ blockedIps.length }} active blocks currently enforced.</p>
                </div>
                <div class="flex gap-2">
                    <div class="relative">
                        <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                        <Input v-model="searchQuery" class="pl-10 w-64" placeholder="Search blocked list..." />
                    </div>
                    <Button variant="outline" size="icon">
                        <Download :size="18" />
                    </Button>
                </div>
            </div>

            <Card class="rounded-xl overflow-hidden">
                <!-- Loading -->
                <div v-if="isLoading" class="p-12 text-center">
                    <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                    <p class="text-muted-foreground">Loading blocked IPs...</p>
                </div>

                <!-- Empty -->
                <div v-else-if="filteredIps.length === 0" class="p-12 text-center">
                    <Shield :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                    <p class="text-muted-foreground">No blocked IPs found</p>
                </div>

                <!-- Table -->
                <template v-else>
                    <table class="w-full text-left">
                        <thead>
                            <tr class="bg-muted/50 border-b border-border">
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">IP / Range</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Country</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Reason</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Date Added</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <tr v-for="ip in paginatedIps" :key="ip.id" class="hover:bg-muted/30 transition-colors">
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <component :is="ip.ip_address?.includes('/') ? Network : Globe" :size="18" class="text-muted-foreground" />
                                        <span class="text-sm font-bold font-mono text-foreground">{{ ip.ip_address }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <span class="text-lg">{{ getCountryFlag(ip.country || 'US') }}</span>
                                        <span class="text-sm">{{ ip.country_name || 'Unknown' }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <Badge :variant="getReasonVariant(ip.reason)">{{ ip.reason || 'Manual Block' }}</Badge>
                                </td>
                                <td class="px-6 py-4 text-sm text-muted-foreground">{{ ip.created_at?.split('T')[0] || 'Unknown' }}</td>
                                <td class="px-6 py-4 text-right">
                                    <Button variant="outline" size="sm" @click="unblockIp(ip.id)">Unblock</Button>
                                </td>
                            </tr>
                        </tbody>
                    </table>

                    <!-- Pagination -->
                    <div class="px-6 py-4 border-t border-border flex items-center justify-between">
                        <p class="text-xs text-muted-foreground">Showing {{ paginatedIps.length }} of {{ filteredIps.length }} blocked entries</p>
                        <div class="flex gap-2">
                            <Button variant="outline" size="sm" @click="currentPage--" :disabled="currentPage === 1">Previous</Button>
                            <Button variant="outline" size="sm" @click="currentPage++" :disabled="currentPage >= totalPages">Next</Button>
                        </div>
                    </div>
                </template>
            </Card>
        </section>
    </div>
</MainLayout>
</template>
