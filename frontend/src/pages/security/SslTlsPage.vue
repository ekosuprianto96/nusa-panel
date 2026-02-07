<script setup lang="ts">
/**
 * SslTlsPage - SSL/TLS Status Management
 */
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { domainService, securityService } from '@/services'
import { Shield, Search, RefreshCw, Lock, CheckCircle, AlertTriangle, Globe } from 'lucide-vue-next'

const router = useRouter()
const domains = ref<any[]>([])
const isLoading = ref(true)
const searchQuery = ref('')
const sortBy = ref('domain_name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const isRunningAutoSsl = ref(false)

import { useToastStore } from '@/stores/toast'
const toast = useToastStore()

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

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await domainService.listDomains()
        domains.value = res.data.data || []
    } catch (e) {
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

const getSslStatusVariant = (domain: any): 'success' | 'warning' | 'secondary' => {
    if (domain.ssl_status === 'active' || domain.ssl_enabled) return 'success'
    if (domain.ssl_status === 'expiring') return 'warning'
    return 'secondary'
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
        <!-- Header -->
        <div class="flex flex-wrap justify-between items-center gap-4">
            <div>
                <AppBreadcrumb
                    class="mb-4"
                    :items="[
                        { label: 'Security Center', icon: Shield, onClick: () => router.push('/dashboard/security') },
                        { label: 'SSL/TLS Status', current: true }
                    ]"
                />
                <h2 class="text-3xl font-black leading-tight tracking-tight text-foreground flex items-center gap-3">
                    <Lock :size="32" class="text-primary" />
                    SSL/TLS Status
                </h2>
            </div>
            <Button @click="runAutoSsl" :disabled="isRunningAutoSsl">
                <RefreshCw :size="18" :class="['mr-2', { 'animate-spin': isRunningAutoSsl }]" />
                {{ isRunningAutoSsl ? 'Running...' : 'Run AutoSSL' }}
            </Button>
        </div>

        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Card class="rounded-xl">
                <CardContent class="p-5 flex items-center gap-3">
                    <div class="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center">
                        <Globe :size="20" class="text-primary" />
                    </div>
                    <div>
                        <p class="text-2xl font-black text-foreground">{{ stats.total }}</p>
                        <p class="text-xs text-muted-foreground">Total Domains</p>
                    </div>
                </CardContent>
            </Card>
            <Card class="rounded-xl">
                <CardContent class="p-5 flex items-center gap-3">
                    <div class="w-10 h-10 bg-emerald-500/10 rounded-lg flex items-center justify-center">
                        <CheckCircle :size="20" class="text-emerald-500" />
                    </div>
                    <div>
                        <p class="text-2xl font-black text-foreground">{{ stats.active }}</p>
                        <p class="text-xs text-muted-foreground">SSL Active</p>
                    </div>
                </CardContent>
            </Card>
            <Card class="rounded-xl">
                <CardContent class="p-5 flex items-center gap-3">
                    <div class="w-10 h-10 bg-amber-500/10 rounded-lg flex items-center justify-center">
                        <AlertTriangle :size="20" class="text-amber-500" />
                    </div>
                    <div>
                        <p class="text-2xl font-black text-foreground">{{ stats.expiring }}</p>
                        <p class="text-xs text-muted-foreground">Expiring Soon</p>
                    </div>
                </CardContent>
            </Card>
            <Card class="rounded-xl">
                <CardContent class="p-5 flex items-center gap-3">
                    <div class="w-10 h-10 bg-muted rounded-lg flex items-center justify-center">
                        <Shield :size="20" class="text-muted-foreground" />
                    </div>
                    <div>
                        <p class="text-2xl font-black text-foreground">{{ stats.pending }}</p>
                        <p class="text-xs text-muted-foreground">Pending</p>
                    </div>
                </CardContent>
            </Card>
        </div>

        <!-- Domains Table -->
        <section>
            <div class="flex items-center justify-between mb-6">
                <h3 class="text-[22px] font-bold leading-tight tracking-tight text-foreground">Domain SSL Status</h3>
                <div class="relative">
                    <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input v-model="searchQuery" class="pl-10 w-64" placeholder="Search domains..." />
                </div>
            </div>

            <Card class="rounded-xl overflow-hidden">
                <!-- Loading -->
                <div v-if="isLoading" class="p-12 text-center">
                    <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                    <p class="text-muted-foreground">Loading domains...</p>
                </div>

                <!-- Empty -->
                <div v-else-if="filteredDomains.length === 0" class="p-12 text-center">
                    <Globe :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                    <p class="text-muted-foreground">No domains found</p>
                </div>

                <!-- Table -->
                <template v-else>
                    <table class="w-full text-left">
                        <thead>
                            <tr class="bg-muted/50 border-b border-border">
                                <th @click="toggleSort('domain_name')" class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider cursor-pointer hover:text-primary">Domain Name</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">SSL Status</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Expiry Date</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <tr v-for="domain in filteredDomains" :key="domain.id" class="hover:bg-muted/30 transition-colors">
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <Lock :size="16" class="text-primary" />
                                        <span class="text-sm font-medium text-foreground">{{ domain.domain_name }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4">
                                    <Badge :variant="getSslStatusVariant(domain)">{{ getSslStatusText(domain) }}</Badge>
                                </td>
                                <td class="px-6 py-4 text-sm text-muted-foreground">{{ domain.ssl_expiry_at?.split('T')[0] || 'N/A' }}</td>
                                <td class="px-6 py-4 text-right">
                                    <Button variant="outline" size="sm" @click="renewCertificate(domain.id)">Renew</Button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </template>
            </Card>
        </section>
    </div>
</MainLayout>
</template>
