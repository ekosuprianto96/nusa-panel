<script setup lang="ts">
/**
 * UserDetailPage - Detailed User Overview (Admin/Reseller)
 */
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import { appService, domainService, packageService, userService } from '@/services'
import type { DomainResponse, Package, UserResourceUsage, UserResponse } from '@/types'
import { useToastStore } from '@/stores/toast'
import {
    ArrowLeft,
    Activity,
    Package as PackageIcon,
    Globe,
    HardDrive,
    Database,
    Mail,
    Server,
    RefreshCw,
    ShieldCheck,
    ShieldAlert,
    Clock,
    Users
} from 'lucide-vue-next'

type AppInstallation = {
    id?: string
    app_type?: string
    domain_id?: string
    domain_name?: string
    version?: string
    status?: string
    installed_at?: string
}

const route = useRoute()
const router = useRouter()
const toast = useToastStore()

const user = ref<UserResponse | null>(null)
const usage = ref<UserResourceUsage | null>(null)
const packages = ref<Package[]>([])
const domains = ref<DomainResponse[]>([])
const installations = ref<AppInstallation[]>([])
const isLoading = ref(true)
const isRefreshing = ref(false)
const domainAction = ref<Record<string, boolean>>({})
const isUpdatingPackage = ref(false)
const packageSelection = ref<string>('')

const userId = computed(() => String(route.params.id || ''))

const selectedPackage = computed(() => {
    if (!user.value?.package_id) return null
    return packages.value.find(p => p.id === user.value?.package_id) || null
})

const fullName = computed(() => user.value?.full_name || user.value?.username || '-')

const statusBadgeClass = computed(() => {
    const status = user.value?.status || 'inactive'
    if (status === 'active') return 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400'
    if (status === 'blocked') return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
    return 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'
})

const roleBadgeClass = computed(() => {
    const role = user.value?.role || 'user'
    if (role === 'admin') return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
    if (role === 'reseller') return 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
    return 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
})

const formatDate = (date?: string | null) => {
    if (!date) return '-'
    return new Date(date).toLocaleDateString('id-ID', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
    })
}

const formatMB = (mb: number) => {
    if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`
    return `${mb.toFixed(0)} MB`
}

const percent = (used: number, limit: number) => {
    if (!limit || limit <= 0) return 0
    return Math.min(100, Math.round((used / limit) * 100))
}

type UsageUnit = 'mb' | 'count'

const usageItems = computed(() => {
    const u = usage.value
    if (!u) return []
    return [
        {
            label: 'Storage',
            used: u.disk_used_mb,
            limit: u.disk_limit_mb,
            icon: HardDrive,
            color: 'bg-purple-500',
            unit: 'mb' as UsageUnit
        },
        {
            label: 'Bandwidth',
            used: u.bandwidth_used_mb,
            limit: u.bandwidth_limit_mb,
            icon: Activity,
            color: 'bg-emerald-500',
            unit: 'mb' as UsageUnit
        },
        {
            label: 'Domains',
            used: u.domains_count,
            limit: u.domains_limit,
            icon: Globe,
            color: 'bg-blue-500',
            unit: 'count' as UsageUnit
        },
        {
            label: 'Databases',
            used: u.databases_count,
            limit: u.databases_limit,
            icon: Database,
            color: 'bg-amber-500',
            unit: 'count' as UsageUnit
        },
        {
            label: 'Email Accounts',
            used: u.email_accounts_count,
            limit: u.email_accounts_limit,
            icon: Mail,
            color: 'bg-pink-500',
            unit: 'count' as UsageUnit
        }
    ]
})

const formatUsageValue = (value: number, unit: UsageUnit) => {
    if (unit === 'count') return String(value)
    return formatMB(value)
}

const packageChangeWarning = computed(() => {
    if (!usage.value || !packageSelection.value) return null
    const pkg = packages.value.find(p => p.id === packageSelection.value)
    if (!pkg) return null

    const warnings: string[] = []
    if (pkg.disk_quota_mb > 0 && usage.value.disk_used_mb > pkg.disk_quota_mb) {
        warnings.push('Disk usage exceeds package quota')
    }
    if (pkg.bandwidth_mb > 0 && usage.value.bandwidth_used_mb > pkg.bandwidth_mb) {
        warnings.push('Bandwidth usage exceeds package quota')
    }
    if (pkg.max_domains > 0 && usage.value.domains_count > pkg.max_domains) {
        warnings.push('Domain count exceeds package limit')
    }
    if (pkg.max_databases > 0 && usage.value.databases_count > pkg.max_databases) {
        warnings.push('Database count exceeds package limit')
    }
    if (pkg.max_email_accounts > 0 && usage.value.email_accounts_count > pkg.max_email_accounts) {
        warnings.push('Email accounts exceed package limit')
    }

    if (warnings.length === 0) return null
    return warnings.join(' · ')
})

const filteredInstallations = computed(() => {
    if (installations.value.length === 0) return []
    if (domains.value.length === 0) return installations.value

    const domainIds = new Set(domains.value.map(d => d.id))
    const domainNames = new Set(domains.value.map(d => d.domain_name))

    return installations.value.filter(i => {
        if (i.domain_id && domainIds.has(i.domain_id)) return true
        if (i.domain_name && domainNames.has(i.domain_name)) return true
        return false
    })
})

const fetchUser = async () => {
    const res = await userService.getUser(userId.value)
    user.value = res.data.data
    packageSelection.value = user.value?.package_id || ''
}

const fetchUsage = async () => {
    const res = await userService.getUsage(userId.value)
    usage.value = res.data.data
}

const fetchPackages = async () => {
    const res = await packageService.listPackages()
    packages.value = res.data.data || []
}

const fetchDomains = async () => {
    const res = await domainService.listDomainsByUser(userId.value, 1, 200)
    domains.value = res.data.data || []
}

const toggleDomainStatus = async (domain: DomainResponse) => {
    if (domainAction.value[domain.id]) return
    domainAction.value = { ...domainAction.value, [domain.id]: true }
    try {
        const nextStatus = !domain.is_active
        await domainService.updateDomainStatusAdmin(domain.id, { is_active: nextStatus })
        domains.value = domains.value.map(d =>
            d.id === domain.id ? { ...d, is_active: nextStatus } : d
        )
        toast.success(`Domain ${nextStatus ? 'unsuspended' : 'suspended'} successfully`)
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to update domain status')
    } finally {
        domainAction.value = { ...domainAction.value, [domain.id]: false }
    }
}

const fetchInstallations = async () => {
    const res = await appService.listInstallationsByUser(userId.value)
    const data = res.data.data
    installations.value = Array.isArray(data) ? data : []
}

const loadAll = async () => {
    isLoading.value = true
    try {
        await Promise.all([
            fetchUser(),
            fetchUsage(),
            fetchPackages(),
            fetchDomains(),
            fetchInstallations()
        ])
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to load user detail')
    } finally {
        isLoading.value = false
    }
}

const refresh = async () => {
    isRefreshing.value = true
    try {
        await loadAll()
    } finally {
        isRefreshing.value = false
    }
}

const updatePackage = async () => {
    if (!packageSelection.value) {
        toast.error('Pilih paket terlebih dahulu')
        return
    }
    if (packageSelection.value === user.value?.package_id) {
        toast.error('Paket yang dipilih sama dengan paket saat ini')
        return
    }
    isUpdatingPackage.value = true
    try {
        await userService.assignPackage(userId.value, { package_id: packageSelection.value })
        await Promise.all([fetchUser(), fetchUsage()])
        toast.success('Paket user berhasil diperbarui')
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Gagal memperbarui paket user')
    } finally {
        isUpdatingPackage.value = false
    }
}

onMounted(loadAll)
</script>

<template>
<MainLayout>
    <div class="space-y-6">
        <!-- Header -->
        <div class="flex flex-wrap items-center justify-between gap-4">
            <div>
                <div class="flex items-center gap-2 text-sm text-slate-500">
                    <button @click="router.push('/dashboard/users')" class="inline-flex items-center gap-2 hover:text-primary">
                        <ArrowLeft :size="16" /> Users
                    </button>
                    <span class="text-slate-400">/</span>
                    <span class="text-[#0d131b] dark:text-white font-medium">User Detail</span>
                </div>
                <h2 class="text-3xl font-black text-[#0d131b] dark:text-white mt-2">User Overview</h2>
                <p class="text-slate-500 text-sm mt-1">Audit account, packages, limits, and service usage in one place.</p>
            </div>
            <button
                @click="refresh"
                class="inline-flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-primary/90 transition-colors"
                :disabled="isRefreshing"
            >
                <RefreshCw :size="16" :class="isRefreshing ? 'animate-spin' : ''" />
                Refresh
            </button>
        </div>

        <!-- Loading State -->
        <div v-if="isLoading" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-10 text-center">
            <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
            <p class="text-slate-500">Loading user detail...</p>
        </div>

        <template v-else>
            <!-- Profile & Quick Stats -->
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6 lg:col-span-2">
                    <div class="flex flex-wrap items-start gap-6">
                        <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-primary to-indigo-600 text-white flex items-center justify-center text-xl font-black">
                            {{ user?.username?.charAt(0).toUpperCase() || 'U' }}
                        </div>
                        <div class="flex-1 min-w-[220px]">
                            <div class="flex flex-wrap items-center gap-3">
                                <h3 class="text-2xl font-black text-[#0d131b] dark:text-white">{{ fullName }}</h3>
                                <span :class="['text-xs font-bold px-2.5 py-1 rounded-full capitalize', statusBadgeClass]">
                                    {{ user?.status || '-' }}
                                </span>
                                <span :class="['text-xs font-bold px-2.5 py-1 rounded-full capitalize', roleBadgeClass]">
                                    {{ user?.role || '-' }}
                                </span>
                            </div>
                            <p class="text-slate-500 text-sm mt-1">@{{ user?.username || '-' }}</p>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4 text-sm text-slate-500">
                                <div class="flex items-center gap-2">
                                    <Mail :size="16" class="text-slate-400" />
                                    <span>{{ user?.email || '-' }}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <Clock :size="16" class="text-slate-400" />
                                    <span>Last login: {{ formatDate(user?.last_login_at) }}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <Users :size="16" class="text-slate-400" />
                                    <span>Created: {{ formatDate(user?.created_at) }}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <ShieldCheck v-if="user?.status === 'active'" :size="16" class="text-emerald-500" />
                                    <ShieldAlert v-else :size="16" class="text-red-500" />
                                    <span>{{ user?.status === 'active' ? 'Account Active' : 'Restricted Access' }}</span>
                                </div>
                            </div>
                        </div>
                        <div class="w-full md:w-auto bg-slate-50 dark:bg-slate-800/50 rounded-xl p-4 border border-slate-200/60 dark:border-slate-800">
                            <p class="text-xs uppercase tracking-wider text-slate-500 font-bold mb-2">Company</p>
                            <p class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ user?.company || '—' }}</p>
                            <p class="text-xs text-slate-500 mt-2">{{ user?.phone || 'No phone on file' }}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                            <PackageIcon :size="18" class="text-primary" />
                            <h4 class="text-sm font-bold text-slate-600 dark:text-slate-300 uppercase tracking-wider">Package</h4>
                        </div>
                    </div>
                    <div class="mb-4 space-y-3">
                        <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider">Change Package</label>
                        <div class="flex flex-wrap gap-2">
                            <select
                                v-model="packageSelection"
                                class="flex-1 min-w-[180px] px-3 py-2 rounded-lg border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm text-slate-700 dark:text-slate-200"
                            >
                                <option value="" disabled>Pilih paket</option>
                                <option v-for="pkg in packages" :key="pkg.id" :value="pkg.id">
                                    {{ pkg.name }}
                                </option>
                            </select>
                            <button
                                @click="updatePackage"
                                class="px-4 py-2 rounded-lg bg-primary text-white text-sm font-bold hover:bg-primary/90 transition-colors disabled:opacity-60"
                                :disabled="isUpdatingPackage || !packageSelection"
                            >
                                {{ isUpdatingPackage ? 'Updating...' : 'Update' }}
                            </button>
                        </div>
                        <div v-if="packageChangeWarning" class="text-xs text-amber-600 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2">
                            {{ packageChangeWarning }}
                        </div>
                    </div>
                    <div v-if="selectedPackage" class="space-y-2">
                        <p class="text-xl font-black text-[#0d131b] dark:text-white">{{ selectedPackage.name }}</p>
                        <p class="text-xs text-slate-500">{{ selectedPackage.description || 'Standard hosting package' }}</p>
                        <div class="mt-4 space-y-3 text-sm text-slate-500">
                            <div class="flex justify-between">
                                <span>Disk Quota</span>
                                <span class="font-semibold">{{ formatMB(selectedPackage.disk_quota_mb) }}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>Bandwidth</span>
                                <span class="font-semibold">{{ formatMB(selectedPackage.bandwidth_mb) }}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>Domains</span>
                                <span class="font-semibold">{{ selectedPackage.max_domains }}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>Databases</span>
                                <span class="font-semibold">{{ selectedPackage.max_databases }}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>Email Accounts</span>
                                <span class="font-semibold">{{ selectedPackage.max_email_accounts }}</span>
                            </div>
                        </div>
                    </div>
                    <div v-else class="text-sm text-slate-500">
                        No package assigned.
                    </div>
                </div>
            </div>

            <!-- Resource Usage -->
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex items-center justify-between mb-6">
                    <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Resource Usage</h3>
                    <span class="text-xs text-slate-500">Live quota utilization</span>
                </div>
                <div v-if="usageItems.length === 0" class="text-sm text-slate-500">No usage data available.</div>
                <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <div v-for="item in usageItems" :key="item.label" class="border border-slate-200 dark:border-slate-800 rounded-xl p-4 bg-slate-50/70 dark:bg-slate-800/40">
                        <div class="flex items-center justify-between mb-3">
                            <div class="flex items-center gap-2 text-sm font-semibold text-slate-600 dark:text-slate-300">
                                <component :is="item.icon" :size="16" class="text-slate-400" />
                                {{ item.label }}
                            </div>
                            <span class="text-xs text-slate-500">{{ percent(item.used, item.limit) }}%</span>
                        </div>
                        <div class="h-2 rounded-full bg-slate-200/80 dark:bg-slate-700 overflow-hidden">
                            <div :class="['h-full transition-all', item.color]" :style="{ width: percent(item.used, item.limit) + '%' }"></div>
                        </div>
                        <div class="flex justify-between text-xs text-slate-500 mt-2">
                            <span>Used: {{ formatUsageValue(item.used, item.unit) }}</span>
                            <span>Limit: {{ formatUsageValue(item.limit, item.unit) }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Domains & Apps -->
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6 lg:col-span-2">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                            <Globe :size="18" class="text-primary" />
                            <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Domains</h3>
                        </div>
                        <span class="text-xs text-slate-500">{{ domains.length }} active domains</span>
                    </div>
                    <div v-if="domains.length === 0" class="text-sm text-slate-500">No domains found for this user.</div>
                    <div v-else class="divide-y divide-slate-100 dark:divide-slate-800">
                        <div v-for="domain in domains" :key="domain.id" class="py-3 flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <p class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ domain.domain_name }}</p>
                                <p class="text-xs text-slate-500">{{ domain.document_root }}</p>
                            </div>
                            <div class="flex flex-wrap items-center gap-2 text-xs text-slate-500">
                                <span
                                    class="px-2 py-1 rounded-full"
                                    :class="domain.ssl_enabled ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400'"
                                >
                                    {{ domain.ssl_enabled ? 'SSL Active' : 'No SSL' }}
                                </span>
                                <span
                                    class="px-2 py-1 rounded-full"
                                    :class="domain.is_active ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' : 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400'"
                                >
                                    {{ domain.is_active ? 'Active' : 'Suspended' }}
                                </span>
                                <button
                                    @click="toggleDomainStatus(domain)"
                                    class="px-3 py-1 rounded-full border text-[11px] font-bold transition-colors"
                                    :class="domain.is_active ? 'border-amber-200 text-amber-700 hover:bg-amber-50 dark:border-amber-800 dark:text-amber-300 dark:hover:bg-amber-900/20' : 'border-emerald-200 text-emerald-700 hover:bg-emerald-50 dark:border-emerald-800 dark:text-emerald-300 dark:hover:bg-emerald-900/20'"
                                    :disabled="domainAction[domain.id]"
                                >
                                    {{ domain.is_active ? 'Suspend' : 'Unsuspend' }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                    <div class="flex items-center gap-2 mb-4">
                        <Server :size="18" class="text-primary" />
                        <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Installed Apps</h3>
                    </div>
                    <div v-if="filteredInstallations.length === 0" class="text-sm text-slate-500">No applications installed.</div>
                    <div v-else class="space-y-3">
                        <div v-for="app in filteredInstallations" :key="app.id || app.domain_id || app.app_type" class="border border-slate-200 dark:border-slate-800 rounded-lg p-3 bg-slate-50/70 dark:bg-slate-800/40">
                            <div class="flex items-center justify-between">
                                <p class="text-sm font-semibold text-[#0d131b] dark:text-white capitalize">{{ app.app_type || 'App' }}</p>
                                <span class="text-xs text-slate-500">{{ app.version || 'latest' }}</span>
                            </div>
                            <p class="text-xs text-slate-500 mt-1">{{ app.domain_name || 'Unknown domain' }}</p>
                            <div class="flex items-center justify-between text-xs text-slate-400 mt-2">
                                <span>Status: {{ app.status || 'active' }}</span>
                                <span>{{ formatDate(app.installed_at || null) }}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </template>
    </div>
</MainLayout>
</template>
