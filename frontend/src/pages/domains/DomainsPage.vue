<script setup lang="ts">
/**
 * DomainsPage - Comprehensive Domain Management
 * Full CRUD with Modals and Toast Notifications
 */
import { ref, onMounted, computed, watch } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { domainService } from '@/services'
import type { DomainResponse, SubdomainResponse, DnsRecordResponse } from '@/types'

// ==========================================
// STATE
// ==========================================
const currentView = ref<'dashboard' | 'dns' | 'subdomains' | 'redirects' | 'ddns'>('dashboard')
const selectedDomainId = ref<string>('')
const domains = ref<DomainResponse[]>([])
const subdomains = ref<SubdomainResponse[]>([])
const dnsRecords = ref<DnsRecordResponse[]>([])
const redirects = ref<any[]>([])
const isLoading = ref(false)
const isSaving = ref(false)
const searchQuery = ref('')

// DDNS (Mock)
const ddnsHosts = ref([
    { hostname: 'vpn.example.com', id: 'ddns-7421', ip: '192.168.1.45', updated: '5m ago' },
    { hostname: 'nas.example.com', id: 'ddns-8832', ip: '82.45.12.102', updated: '2h ago' }
])

// ==========================================
// TOAST SYSTEM
// ==========================================
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

// ==========================================
// MODALS
// ==========================================
const showDomainModal = ref(false)
const showSubdomainModal = ref(false)
const showDnsModal = ref(false)
const showRedirectModal = ref(false)
const showDeleteConfirm = ref(false)
const deleteTarget = ref<{ type: string; id: string; name: string } | null>(null)

// Form Data
const domainForm = ref({ domain_name: '', document_root: '' })
const subdomainForm = ref({ subdomain_name: '', document_root: '' })
const dnsForm = ref({ record_type: 'A', name: '', value: '', ttl: 14400, priority: 10 })
const redirectForm = ref({ source_path: '', destination_url: '', type: '301' as '301' | '302' })

// Reset Forms
const resetForms = () => {
    domainForm.value = { domain_name: '', document_root: '' }
    subdomainForm.value = { subdomain_name: '', document_root: '' }
    dnsForm.value = { record_type: 'A', name: '', value: '', ttl: 14400, priority: 10 }
    redirectForm.value = { source_path: '', destination_url: '', type: '301' }
}

// ==========================================
// COMPUTED
// ==========================================
const selectedDomainObj = computed(() => domains.value.find(d => d.id === selectedDomainId.value))
const domainNameForTitle = computed(() => selectedDomainObj.value?.domain_name || 'Select Domain')

const stats = computed(() => ({
    total: domains.value.length,
    ssl_active: domains.value.filter(d => d.ssl_enabled).length,
    auto_renew: domains.value.filter(d => d.is_active).length
}))

const filteredDomains = computed(() => {
    if (!searchQuery.value) return domains.value
    const q = searchQuery.value.toLowerCase()
    return domains.value.filter(d => d.domain_name.toLowerCase().includes(q))
})

const filteredSubdomains = computed(() => {
    if (!searchQuery.value) return subdomains.value
    const q = searchQuery.value.toLowerCase()
    return subdomains.value.filter(s => s.subdomain_name.toLowerCase().includes(q))
})

const filteredDns = computed(() => {
    if (!searchQuery.value) return dnsRecords.value
    const q = searchQuery.value.toLowerCase()
    return dnsRecords.value.filter(r => r.name.toLowerCase().includes(q) || r.record_type.toLowerCase().includes(q))
})

const filteredRedirects = computed(() => {
     if (!searchQuery.value) return redirects.value
     const q = searchQuery.value.toLowerCase()
     return redirects.value.filter(r => r.source_path?.toLowerCase().includes(q))
})

// ==========================================
// NAVIGATION
// ==========================================
const navigateTo = (view: typeof currentView.value, domainId?: string) => {
    currentView.value = view
    searchQuery.value = ''
    if (domainId) {
        selectedDomainId.value = domainId
    } else if (!selectedDomainId.value && domains.value.length > 0 && view !== 'dashboard' && view !== 'ddns') {
        selectedDomainId.value = domains.value[0]?.id ?? ''
    }
}

// ==========================================
// DATA FETCHING
// ==========================================
const fetchDomains = async () => {
    isLoading.value = true
    try {
        const res = await domainService.listDomains(1, 100)
        domains.value = res.data.data || []
        if (domains.value.length > 0 && !selectedDomainId.value) {
            selectedDomainId.value = domains.value[0]?.id ?? ''
        }
    } catch (e) { console.error(e); showToast('Failed to load domains', 'error') } 
    finally { isLoading.value = false }
}

const fetchSubFeatures = async () => {
    if (!selectedDomainId.value) return
    isLoading.value = true
    try {
        if (currentView.value === 'dns') {
            const res = await domainService.listDnsRecords(selectedDomainId.value)
            dnsRecords.value = res.data.data || []
        } else if (currentView.value === 'subdomains') {
            const res = await domainService.listSubdomains(selectedDomainId.value)
            subdomains.value = res.data.data || []
        } else if (currentView.value === 'redirects') {
            const res = await domainService.listRedirects(selectedDomainId.value)
            redirects.value = res.data.data || []
        }
    } catch (e) { console.error(e) } 
    finally { isLoading.value = false }
}

// ==========================================
// CRUD OPERATIONS
// ==========================================
// --- Domain ---
const createDomain = async () => {
    if (!domainForm.value.domain_name) return showToast('Domain name is required', 'error')
    isSaving.value = true
    try {
        await domainService.createDomain(domainForm.value)
        showToast('Domain created successfully!', 'success')
        showDomainModal.value = false
        resetForms()
        fetchDomains()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed to create domain', 'error') }
    finally { isSaving.value = false }
}

const deleteDomain = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteDomain(id)
        showToast('Domain deleted', 'success')
        fetchDomains()
    } catch (e) { showToast('Failed to delete domain', 'error') }
    finally { isSaving.value = false; showDeleteConfirm.value = false; deleteTarget.value = null }
}

// --- Subdomain ---
const createSubdomain = async () => {
    if (!subdomainForm.value.subdomain_name || !selectedDomainId.value) return showToast('Fill all fields', 'error')
    isSaving.value = true
    try {
        await domainService.createSubdomain(selectedDomainId.value, subdomainForm.value)
        showToast('Subdomain created!', 'success')
        showSubdomainModal.value = false
        resetForms()
        fetchSubFeatures()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed', 'error') }
    finally { isSaving.value = false }
}

const deleteSubdomain = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteSubdomain(selectedDomainId.value, id)
        showToast('Subdomain deleted', 'success')
        fetchSubFeatures()
    } catch (e) { showToast('Failed to delete', 'error') }
    finally { isSaving.value = false; showDeleteConfirm.value = false; deleteTarget.value = null }
}

// --- DNS ---
const createDnsRecord = async () => {
    if (!dnsForm.value.name || !dnsForm.value.value) return showToast('Fill all fields', 'error')
    isSaving.value = true
    try {
        await domainService.createDnsRecord(selectedDomainId.value, dnsForm.value)
        showToast('DNS Record added!', 'success')
        showDnsModal.value = false
        resetForms()
        fetchSubFeatures()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed', 'error') }
    finally { isSaving.value = false }
}

const deleteDnsRecord = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteDnsRecord(selectedDomainId.value, id)
        showToast('DNS Record deleted', 'success')
        fetchSubFeatures()
    } catch (e) { showToast('Failed to delete', 'error') }
    finally { isSaving.value = false; showDeleteConfirm.value = false; deleteTarget.value = null }
}

// --- Redirects ---
const createRedirect = async () => {
    if (!redirectForm.value.source_path || !redirectForm.value.destination_url) return showToast('Fill all fields', 'error')
    isSaving.value = true
    try {
        await domainService.createRedirect(selectedDomainId.value, redirectForm.value)
        showToast('Redirect created!', 'success')
        showRedirectModal.value = false
        resetForms()
        fetchSubFeatures()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed', 'error') }
    finally { isSaving.value = false }
}

const deleteRedirect = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteRedirect(selectedDomainId.value, id)
        showToast('Redirect deleted', 'success')
        fetchSubFeatures()
    } catch (e) { showToast('Failed to delete', 'error') }
    finally { isSaving.value = false; showDeleteConfirm.value = false; deleteTarget.value = null }
}

// --- Delete Confirmation ---
const confirmDelete = (type: string, id: string, name: string) => {
    deleteTarget.value = { type, id, name }
    showDeleteConfirm.value = true
}

const executeDelete = async () => {
    if (!deleteTarget.value) return
    const { type, id } = deleteTarget.value
    if (type === 'domain') await deleteDomain(id)
    else if (type === 'subdomain') await deleteSubdomain(id)
    else if (type === 'dns') await deleteDnsRecord(id)
    else if (type === 'redirect') await deleteRedirect(id)
}

// --- Utility ---
const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    showToast(`Copied: ${text}`, 'info')
}

// Watchers
watch(currentView, () => {
    if (currentView.value !== 'dashboard' && currentView.value !== 'ddns') fetchSubFeatures()
})
watch(selectedDomainId, fetchSubFeatures)

// Lifecycle
onMounted(() => { fetchDomains() })

</script>

<template>
<MainLayout>
    <div class="font-display text-[#0d131b] dark:text-slate-200">
        
        <!-- =========================================================================
             VIEW: DASHBOARD (OVERVIEW)
             ========================================================================= -->
        <div v-if="currentView === 'dashboard'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
             <!-- PageHeading & Search -->
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                <div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Domains Management</h2>
                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Register, transfer, and manage your domain settings.</p>
                </div>
                <div class="flex flex-col sm:flex-row gap-3">
                    <!-- SearchBar -->
                    <div class="relative w-full sm:w-64">
                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <span class="material-symbols-outlined text-[#4c6c9a] text-xl">search</span>
                        </div>
                        <input v-model="searchQuery" class="block w-full pl-10 pr-3 py-2 border-none bg-white dark:bg-slate-800 rounded-lg text-sm focus:ring-2 focus:ring-primary shadow-sm dark:text-white placeholder:text-[#4c6c9a]" placeholder="Search domains..." type="text"/>
                    </div>
                    <button @click="showDomainModal = true" class="flex items-center justify-center gap-2 bg-primary text-white px-5 py-2 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md">
                        <span class="material-symbols-outlined text-lg">add_circle</span>
                        <span>Create New Domain</span>
                    </button>
                </div>
            </div>

            <!-- Main Layout Grid -->
            <div class="grid grid-cols-12 gap-6">
                <!-- Left Column -->
                <div class="col-span-12 lg:col-span-9 space-y-6">
                    <!-- Stats Section -->
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="flex flex-col gap-2 rounded-xl p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 shadow-sm">
                            <div class="flex justify-between items-start">
                                <p class="text-[#4c6c9a] dark:text-slate-400 text-sm font-medium uppercase tracking-wider">Total Domains</p>
                                <span class="material-symbols-outlined text-primary bg-primary/10 p-1.5 rounded-lg text-lg">globe</span>
                            </div>
                            <p class="text-[#0d131b] dark:text-white text-3xl font-bold leading-tight">{{ stats.total }}</p>
                            <p class="text-[#07883b] text-xs font-semibold flex items-center gap-1">
                                <span class="material-symbols-outlined text-sm">trending_up</span> +0% from last month
                            </p>
                        </div>
                        <div class="flex flex-col gap-2 rounded-xl p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 shadow-sm">
                            <div class="flex justify-between items-start">
                                <p class="text-[#4c6c9a] dark:text-slate-400 text-sm font-medium uppercase tracking-wider">Active SSL</p>
                                <span class="material-symbols-outlined text-green-500 bg-green-500/10 p-1.5 rounded-lg text-lg">verified_user</span>
                            </div>
                            <p class="text-[#0d131b] dark:text-white text-3xl font-bold leading-tight">{{ stats.ssl_active }} protected</p>
                             <p class="text-[#07883b] text-xs font-semibold flex items-center gap-1">
                                <span class="material-symbols-outlined text-sm">trending_up</span> All good
                            </p>
                        </div>
                        <div class="flex flex-col gap-2 rounded-xl p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 shadow-sm">
                            <div class="flex justify-between items-start">
                                <p class="text-[#4c6c9a] dark:text-slate-400 text-sm font-medium uppercase tracking-wider">Active Domains</p>
                                <span class="material-symbols-outlined text-orange-500 bg-orange-500/10 p-1.5 rounded-lg text-lg">autorenew</span>
                            </div>
                            <p class="text-[#0d131b] dark:text-white text-3xl font-bold leading-tight">{{ stats.auto_renew }} active</p>
                             <p class="text-[#4c6c9a] text-xs font-semibold flex items-center gap-1">
                                <span class="material-symbols-outlined text-sm">info</span> {{ stats.total - stats.auto_renew }} inactive
                            </p>
                        </div>
                    </div>

                    <!-- Domains List Section -->
                    <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                        <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Domains List</h2>
                            <button class="text-[#4c6c9a] hover:text-primary transition-colors">
                                <span class="material-symbols-outlined">filter_list</span>
                            </button>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-slate-50 dark:bg-slate-900/50">
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Domain Name</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Doc Root</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">SSL Status</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                    <tr v-for="domain in filteredDomains" :key="domain.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex items-center gap-3">
                                                <div class="size-2 rounded-full" :class="domain.is_active ? 'bg-green-500' : 'bg-red-500'"></div>
                                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ domain.domain_name }}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <span class="text-primary hover:underline text-sm font-medium cursor-pointer">{{ domain.document_root }}</span>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <span v-if="domain.ssl_enabled" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 border border-green-200 dark:border-green-800">
                                                <span class="material-symbols-outlined text-xs">check_circle</span> Active
                                            </span>
                                            <span v-else class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold bg-slate-100 text-slate-600 dark:bg-slate-700 dark:text-slate-300 border border-slate-200 dark:border-slate-600">
                                                <span class="material-symbols-outlined text-xs">lock_open</span> No SSL
                                            </span>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-right">
                                            <div class="flex justify-end gap-2">
                                                 <button @click="navigateTo('dns', domain.id)" class="text-[#4c6c9a] hover:text-primary p-1" title="DNS">
                                                    <span class="material-symbols-outlined text-lg">dns</span>
                                                </button>
                                                <button @click="navigateTo('subdomains', domain.id)" class="text-[#4c6c9a] hover:text-primary p-1" title="Subdomains">
                                                    <span class="material-symbols-outlined text-lg">layers</span>
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <!-- Quick Actions Grid -->
                    <div class="space-y-4">
                        <h3 class="text-[#0d131b] dark:text-white text-lg font-bold px-1">Quick Actions</h3>
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <button @click="navigateTo('subdomains')" class="flex flex-col items-center justify-center p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl hover:border-primary/50 hover:shadow-md transition-all group">
                                <span class="material-symbols-outlined text-primary text-3xl mb-3 group-hover:scale-110 transition-transform">add_link</span>
                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">Create Subdomain</span>
                            </button>
                            <button @click="navigateTo('dns')" class="flex flex-col items-center justify-center p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl hover:border-primary/50 hover:shadow-md transition-all group">
                                <span class="material-symbols-outlined text-purple-500 text-3xl mb-3 group-hover:scale-110 transition-transform">dns</span>
                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">Zone Editor (DNS)</span>
                            </button>
                            <button @click="navigateTo('redirects')" class="flex flex-col items-center justify-center p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl hover:border-primary/50 hover:shadow-md transition-all group">
                                <span class="material-symbols-outlined text-orange-500 text-3xl mb-3 group-hover:scale-110 transition-transform">shortcut</span>
                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">Domain Redirects</span>
                            </button>
                            <button @click="navigateTo('ddns')" class="flex flex-col items-center justify-center p-6 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl hover:border-primary/50 hover:shadow-md transition-all group">
                                <span class="material-symbols-outlined text-teal-500 text-3xl mb-3 group-hover:scale-110 transition-transform">dynamic_form</span>
                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">Dynamic DNS</span>
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Right Column (Nameserver Information) -->
                <div class="col-span-12 lg:col-span-3">
                    <div class="sticky top-8 space-y-6">
                        <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl p-6 shadow-sm">
                            <h3 class="text-[#0d131b] dark:text-white text-sm font-bold uppercase tracking-wider mb-4 flex items-center gap-2">
                                <span class="material-symbols-outlined text-primary text-xl">info</span>
                                Nameserver Info
                            </h3>
                            <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mb-4">Point your domains to these nameservers to use our hosting services.</p>
                            <div class="space-y-4">
                                <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                    <p class="text-[10px] uppercase font-bold text-[#4c6c9a] mb-1">Primary Nameserver</p>
                                    <div class="flex items-center justify-between gap-2">
                                        <code class="text-xs font-mono font-semibold text-primary">ns1.hostpanel.com</code>
                                        <button @click="copyToClipboard('ns1.hostpanel.com')" class="text-[#4c6c9a] hover:text-primary transition-colors">
                                            <span class="material-symbols-outlined text-lg">content_copy</span>
                                        </button>
                                    </div>
                                </div>
                                <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                    <p class="text-[10px] uppercase font-bold text-[#4c6c9a] mb-1">Secondary Nameserver</p>
                                    <div class="flex items-center justify-between gap-2">
                                        <code class="text-xs font-mono font-semibold text-primary">ns2.hostpanel.com</code>
                                        <button @click="copyToClipboard('ns2.hostpanel.com')" class="text-[#4c6c9a] hover:text-primary transition-colors">
                                            <span class="material-symbols-outlined text-lg">content_copy</span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- =========================================================================
             VIEW: ZONE EDITOR (DNS)
             ========================================================================= -->
        <div v-else-if="currentView === 'dns'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
             <nav class="flex items-center gap-2 text-sm mb-6 text-[#4c6c9a] dark:text-slate-400">
                <a @click="navigateTo('dashboard')" class="hover:text-primary transition-colors flex items-center gap-1 cursor-pointer">
                    <span class="material-symbols-outlined text-lg">globe</span>
                    Domains
                </a>
                <span class="material-symbols-outlined text-sm">chevron_right</span>
                <div class="relative">
                     <select v-model="selectedDomainId" class="appearance-none bg-transparent border-none text-[#0d131b] dark:text-white font-medium pr-6 py-0 focus:ring-0 cursor-pointer">
                         <option v-for="d in domains" :key="d.id" :value="d.id">Zone Editor: {{ d.domain_name }}</option>
                     </select>
                </div>
            </nav>

            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                <div>
                     <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight flex items-center gap-3">
                        Zone Editor
                    </h2>
                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Manage DNS records to control how traffic is routed for {{ domainNameForTitle }}.</p>
                </div>
                <div class="flex flex-col sm:flex-row gap-3">
                    <div class="relative w-full sm:w-64">
                         <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <span class="material-symbols-outlined text-[#4c6c9a] text-xl">search</span>
                        </div>
                        <input v-model="searchQuery" class="block w-full pl-10 pr-3 py-2 border-none bg-white dark:bg-slate-800 rounded-lg text-sm focus:ring-2 focus:ring-primary shadow-sm dark:text-white placeholder:text-[#4c6c9a]" placeholder="Filter records..." type="text"/>
                    </div>
                    <button @click="showDnsModal = true" class="flex items-center justify-center gap-2 bg-primary text-white px-5 py-2 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md">
                        <span class="material-symbols-outlined text-lg">add</span>
                        <span>Add Record</span>
                    </button>
                </div>
            </div>

            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden mb-8">
                <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                     <div class="flex items-center gap-4">
                        <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Manage Records</h2>
                        <div class="flex gap-2">
                             <!-- Simple filters could be added here affecting searchQuery or a filter var -->
                            <button @click="searchQuery = ''" class="px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-100 text-[#4c6c9a] dark:bg-slate-700 dark:text-slate-300 hover:bg-primary/10 hover:text-primary transition-all">All</button>
                            <button @click="searchQuery = 'A'" class="px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-100 text-[#4c6c9a] dark:bg-slate-700 dark:text-slate-300 hover:bg-primary/10 hover:text-primary transition-all">A</button>
                            <button @click="searchQuery = 'CNAME'" class="px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-100 text-[#4c6c9a] dark:bg-slate-700 dark:text-slate-300 hover:bg-primary/10 hover:text-primary transition-all">CNAME</button>
                            <button @click="searchQuery = 'MX'" class="px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-100 text-[#4c6c9a] dark:bg-slate-700 dark:text-slate-300 hover:bg-primary/10 hover:text-primary transition-all">MX</button>
                        </div>
                    </div>
                </div>
                 <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-slate-50 dark:bg-slate-900/50">
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Type</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Name</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">TTL</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Record / Value</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                             <tr v-for="record in filteredDns" :key="record.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class="inline-block px-2 py-0.5 rounded text-[10px] font-black border"
                                        :class="{
                                            'bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/40 dark:text-blue-300 dark:border-blue-800': record.record_type === 'A',
                                            'bg-purple-100 text-purple-700 border-purple-200 dark:bg-purple-900/40 dark:text-purple-300 dark:border-purple-800': record.record_type === 'CNAME',
                                            'bg-orange-100 text-orange-700 border-orange-200 dark:bg-orange-900/40 dark:text-orange-300 dark:border-orange-800': record.record_type === 'MX',
                                            'bg-green-100 text-green-700 border-green-200 dark:bg-green-900/40 dark:text-green-300 dark:border-green-800': record.record_type === 'TXT',
                                        }">
                                        {{ record.record_type }}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-[#0d131b] dark:text-white">{{ record.name }}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-[#4c6c9a] dark:text-slate-400">{{ record.ttl || 14400 }}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-[#0d131b] dark:text-white truncate max-w-[300px]">{{ record.value }}</td>
                                <td class="px-6 py-4 whitespace-nowrap text-right space-x-2">
                                    <button class="text-primary hover:bg-primary/10 p-1.5 rounded transition-colors" title="Edit Record">
                                        <span class="material-symbols-outlined text-lg">edit</span>
                                    </button>
                                    <button @click="confirmDelete('dns', record.id, record.name)" class="text-[#e73908] hover:bg-red-50 dark:hover:bg-red-900/20 p-1.5 rounded transition-colors" title="Delete Record">
                                        <span class="material-symbols-outlined text-lg">delete</span>
                                    </button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                 </div>
            </div>
            
             <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="bg-blue-50/50 dark:bg-blue-900/10 border border-blue-100 dark:border-blue-900/30 rounded-xl p-6">
                    <div class="flex items-start gap-4">
                        <span class="material-symbols-outlined text-primary">info</span>
                        <div>
                            <h4 class="text-[#0d131b] dark:text-white font-bold text-sm mb-1">Propagation Time</h4>
                            <p class="text-[#4c6c9a] dark:text-slate-400 text-xs leading-relaxed">
                                DNS changes can take up to 48 hours to fully propagate worldwide, although most changes are visible within a few hours. 
                            </p>
                        </div>
                    </div>
                </div>
                 <div class="bg-orange-50/50 dark:bg-orange-900/10 border border-orange-100 dark:border-orange-900/30 rounded-xl p-6">
                    <div class="flex items-start gap-4">
                        <span class="material-symbols-outlined text-orange-500">warning</span>
                        <div>
                            <h4 class="text-[#0d131b] dark:text-white font-bold text-sm mb-1">Advanced Tool</h4>
                            <p class="text-[#4c6c9a] dark:text-slate-400 text-xs leading-relaxed">
                                Misconfiguring DNS records can lead to your website or email services becoming unavailable. Proceed with caution.
                            </p>
                        </div>
                    </div>
                </div>
             </div>
        </div>

        <!-- =========================================================================
             VIEW: SUBDOMAINS
             ========================================================================= -->
        <div v-else-if="currentView === 'subdomains'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
             <div class="mb-8">
                <div class="flex items-center gap-2 text-[#4c6c9a] text-xs font-medium mb-2 uppercase tracking-widest">
                    <a @click="navigateTo('dashboard')" class="hover:text-primary transition-colors cursor-pointer">Domains</a>
                    <span class="material-symbols-outlined text-[10px]">chevron_right</span>
                    <span class="text-[#0d131b] dark:text-white">Subdomain Management</span>
                </div>
                <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Subdomains</h2>
                <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Create and manage sub-sections of your existing domains.</p>
            </div>

            <div class="grid grid-cols-12 gap-8">
                 <div class="col-span-12 lg:col-span-4 space-y-6">
                    <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                        <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700">
                            <h3 class="text-[#0d131b] dark:text-white text-lg font-bold">Create a Subdomain</h3>
                        </div>
                        <div class="p-6">
                             <form class="space-y-4">
                                <div>
                                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2" for="subdomain">Subdomain</label>
                                    <div class="flex">
                                        <input class="block w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-l-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white" id="subdomain" placeholder="e.g. blog" type="text"/>
                                        <div class="flex items-center px-4 bg-slate-50 dark:bg-slate-800 border-y border-r border-[#cfd9e7] dark:border-slate-700 rounded-r-lg text-sm text-[#4c6c9a]">.</div>
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2" for="domain">Domain</label>
                                    <select v-model="selectedDomainId" class="block w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white" id="domain">
                                        <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                                    </select>
                                </div>
                                <button @click="showSubdomainModal = true" type="button" class="w-full flex items-center justify-center gap-2 bg-primary text-white px-5 py-3 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md mt-6">
                                    <span class="material-symbols-outlined text-lg">add_circle</span>
                                    <span>Create Subdomain</span>
                                </button>
                             </form>
                        </div>
                    </div>
                 </div>

                 <div class="col-span-12 lg:col-span-8">
                     <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                        <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Current Subdomains</h2>
                             <div class="flex items-center gap-3">
                                <div class="relative hidden sm:block">
                                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                        <span class="material-symbols-outlined text-[#4c6c9a] text-lg">search</span>
                                    </div>
                                    <input v-model="searchQuery" class="block w-48 pl-9 pr-3 py-1.5 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-xs focus:ring-1 focus:ring-primary dark:text-white" placeholder="Filter list..." type="text"/>
                                </div>
                            </div>
                        </div>
                        <div class="overflow-x-auto">
                             <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-slate-50 dark:bg-slate-900/50">
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Subdomain</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Document Root</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                     <tr v-for="sub in filteredSubdomains" :key="sub.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex items-center gap-3">
                                                <div class="size-2 rounded-full bg-green-500"></div>
                                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ sub.subdomain_name }}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex items-center gap-2 text-sm text-[#4c6c9a] dark:text-slate-400 font-medium">
                                                <span class="material-symbols-outlined text-base">folder_open</span>
                                                <a class="text-primary hover:underline" href="#">{{ sub.document_root }}</a>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-right">
                                            <div class="flex items-center justify-end gap-2">
                                                 <button @click="confirmDelete('subdomain', sub.id, sub.subdomain_name)" class="flex items-center justify-center p-1 rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all">
                                                    <span class="material-symbols-outlined text-xl">delete</span>
                                                </button>
                                            </div>
                                        </td>
                                     </tr>
                                </tbody>
                             </table>
                        </div>
                     </div>
                 </div>
            </div>
        </div>

        <!-- =========================================================================
             VIEW: REDIRECTS
             ========================================================================= -->
        <div v-else-if="currentView === 'redirects'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                <div>
                    <div class="flex items-center gap-2 text-primary mb-1">
                        <a @click="navigateTo('dashboard')" class="text-xs font-bold hover:underline cursor-pointer">Domains Management</a>
                        <span class="material-symbols-outlined text-xs">chevron_right</span>
                        <span class="text-xs font-bold text-[#4c6c9a]">Redirects</span>
                    </div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Domain Redirects</h2>
                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Send visitors of one domain or page to a different URL automatically.</p>
                </div>
            </div>

            <div class="grid grid-cols-12 gap-6">
                <div class="col-span-12 lg:col-span-8 space-y-6">
                     <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                        <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center gap-2">
                            <span class="material-symbols-outlined text-primary">add_link</span>
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Add a Redirect</h2>
                        </div>
                        <div class="p-6">
                            <form @submit.prevent="createRedirect" class="space-y-6">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    <div class="space-y-2">
                                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider">Type</label>
                                        <select v-model="redirectForm.type" class="block w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white">
                                            <option value="301">301 Permanent</option>
                                            <option value="302">302 Temporary</option>
                                        </select>
                                    </div>
                                    <div class="space-y-2">
                                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider">Domain</label>
                                         <select v-model="selectedDomainId" class="block w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white">
                                            <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                                        </select>
                                    </div>
                                </div>
                                <div class="space-y-2">
                                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider">Source Path *</label>
                                    <div class="relative">
                                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                            <span class="text-[#4c6c9a] text-sm">/</span>
                                        </div>
                                        <input v-model="redirectForm.source_path" class="block w-full px-4 py-2 pl-6 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white" placeholder="old-page" type="text"/>
                                    </div>
                                </div>
                                <div class="space-y-2">
                                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider">Destination URL *</label>
                                    <div class="relative">
                                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                            <span class="material-symbols-outlined text-[#4c6c9a] text-lg">link</span>
                                        </div>
                                        <input v-model="redirectForm.destination_url" class="block w-full px-4 py-2 pl-10 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white" placeholder="https://new-destination.com" type="url"/>
                                    </div>
                                </div>
                                <div class="flex items-center gap-4 pt-2">
                                    <button type="submit" :disabled="isSaving" class="flex items-center justify-center gap-2 bg-primary text-white px-6 py-2.5 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md disabled:opacity-50">
                                        <span class="material-symbols-outlined text-lg">add</span>
                                        {{ isSaving ? 'Adding...' : 'Add Redirect' }}
                                    </button>
                                </div>
                            </form>
                        </div>
                     </div>

                     <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                         <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Current Redirects</h2>
                            <div class="relative">
                                <span class="material-symbols-outlined absolute left-2.5 top-1/2 -translate-y-1/2 text-[#4c6c9a] text-lg">search</span>
                                <input v-model="searchQuery" class="pl-9 py-1.5 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-xs w-48" placeholder="Search redirects..." type="text"/>
                            </div>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-slate-50 dark:bg-slate-900/50">
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Domain/Path</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Type</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Destination</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                    <tr v-for="redirect in filteredRedirects" :key="redirect.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex flex-col">
                                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ redirect.source_path }}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <span class="px-2 py-1 rounded bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 text-[10px] font-bold border border-blue-100 dark:border-blue-800">{{ redirect.type }}</span>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap">
                                            <div class="flex items-center gap-2 max-w-[200px] truncate">
                                                <span class="material-symbols-outlined text-sm text-[#4c6c9a]">subdirectory_arrow_right</span>
                                                <a class="text-sm text-primary hover:underline truncate" href="#">{{ redirect.destination_url }}</a>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4 whitespace-nowrap text-right">
                                            <button @click="confirmDelete('redirect', redirect.id, redirect.source_path)" class="text-red-500 hover:text-red-700 p-2 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-all">
                                                <span class="material-symbols-outlined">delete</span>
                                            </button>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                     </div>
                </div>
            </div>
        </div>

        <!-- =========================================================================
             VIEW: DYNAMIC DNS (Mock)
             ========================================================================= -->
        <div v-else-if="currentView === 'ddns'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
             <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                <div>
                     <div class="flex items-center gap-2 text-primary mb-2">
                        <a @click="navigateTo('dashboard')" class="text-sm font-medium hover:underline flex items-center gap-1 cursor-pointer">
                             <span class="material-symbols-outlined text-base">arrow_back</span> Back to Domains
                        </a>
                    </div>
                    <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Dynamic DNS</h2>
                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Automatically update your DNS records when your IP address changes.</p>
                </div>
            </div>

            <div class="grid grid-cols-12 gap-6">
                <div class="col-span-12 lg:col-span-8 space-y-6">
                     <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800 rounded-xl p-5 flex gap-4">
                        <span class="material-symbols-outlined text-blue-600 dark:text-blue-400">lightbulb</span>
                         <div>
                            <h4 class="text-blue-900 dark:text-blue-200 text-sm font-bold mb-1">How it works</h4>
                            <p class="text-blue-800/80 dark:text-blue-300/80 text-sm leading-relaxed">
                                Dynamic DNS (DDNS) allows you to point a domain or subdomain to a computer with a changing IP address. 
                            </p>
                        </div>
                     </div>

                     <!-- Mock Table -->
                     <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                        <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Existing DDNS Hosts</h2>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-slate-50 dark:bg-slate-900/50">
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Hostname</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">ID / Key</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Current IP</th>
                                        <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                    <tr v-for="host in ddnsHosts" :key="host.id">
                                        <td class="px-6 py-4">
                                            <div class="flex flex-col">
                                                <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ host.hostname }}</span>
                                            </div>
                                        </td>
                                        <td class="px-6 py-4">
                                            <div class="flex items-center gap-2">
                                                <code class="text-xs bg-slate-100 dark:bg-slate-900 px-2 py-1 rounded font-mono text-primary">{{ host.id }}</code>
                                            </div>
                                        </td>
                                         <td class="px-6 py-4">
                                            <div class="flex items-center gap-2">
                                                <span class="size-2 rounded-full bg-green-500"></span>
                                                <span class="text-sm font-medium text-[#0d131b] dark:text-slate-300">{{ host.ip }}</span>
                                            </div>
                                            <span class="text-[10px] text-[#4c6c9a] uppercase font-bold">Updated {{ host.updated }}</span>
                                        </td>
                                        <td class="px-6 py-4 text-right">
                                             <button class="p-1.5 text-[#4c6c9a] hover:text-red-500 hover:bg-red-50 rounded">
                                                <span class="material-symbols-outlined text-lg">delete</span>
                                            </button>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                     </div>
                </div>
            </div>
        </div>

    </div>

    <!-- =========================================================================
         MODALS & TOASTS
         ========================================================================= -->
    
    <!-- Toast Notifications -->
    <div class="fixed top-4 right-4 z-50 space-y-2">
        <div v-for="toast in toasts" :key="toast.id" 
            class="flex items-center gap-3 px-4 py-3 rounded-lg shadow-lg animate-in slide-in-from-right-5 duration-300"
            :class="{
                'bg-green-600 text-white': toast.type === 'success',
                'bg-red-600 text-white': toast.type === 'error',
                'bg-blue-600 text-white': toast.type === 'info'
            }">
            <span class="material-symbols-outlined text-lg">
                {{ toast.type === 'success' ? 'check_circle' : toast.type === 'error' ? 'error' : 'info' }}
            </span>
            <span class="text-sm font-medium">{{ toast.message }}</span>
        </div>
    </div>

    <!-- Modal Backdrop -->
    <div v-if="showDomainModal || showSubdomainModal || showDnsModal || showRedirectModal || showDeleteConfirm" 
        @click="showDomainModal = false; showSubdomainModal = false; showDnsModal = false; showRedirectModal = false; showDeleteConfirm = false"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 animate-in fade-in duration-200" />

    <!-- Create Domain Modal -->
    <div v-if="showDomainModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-md animate-in zoom-in-95 duration-200">
            <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Create New Domain</h3>
                <button @click="showDomainModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                    <span class="material-symbols-outlined">close</span>
                </button>
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Domain Name *</label>
                    <input v-model="domainForm.domain_name" type="text" placeholder="example.com" 
                        class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                </div>
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Document Root (optional)</label>
                    <input v-model="domainForm.document_root" type="text" placeholder="/public_html" 
                        class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                </div>
            </div>
            <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                <button @click="showDomainModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a] hover:text-[#0d131b]">Cancel</button>
                <button @click="createDomain" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                    {{ isSaving ? 'Creating...' : 'Create Domain' }}
                </button>
            </div>
        </div>
    </div>

    <!-- Create Subdomain Modal -->
    <div v-if="showSubdomainModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-md animate-in zoom-in-95 duration-200">
            <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Create Subdomain</h3>
                <button @click="showSubdomainModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                    <span class="material-symbols-outlined">close</span>
                </button>
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Subdomain *</label>
                    <div class="flex">
                        <input v-model="subdomainForm.subdomain_name" type="text" placeholder="blog" 
                            class="flex-1 px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-l-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                        <span class="px-4 py-2 bg-slate-100 dark:bg-slate-700 border-y border-r border-[#cfd9e7] dark:border-slate-700 rounded-r-lg text-sm text-[#4c6c9a]">.{{ domainNameForTitle }}</span>
                    </div>
                </div>
            </div>
            <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                <button @click="showSubdomainModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a]">Cancel</button>
                <button @click="createSubdomain" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                    {{ isSaving ? 'Creating...' : 'Create Subdomain' }}
                </button>
            </div>
        </div>
    </div>

    <!-- Create DNS Record Modal -->
    <div v-if="showDnsModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-lg animate-in zoom-in-95 duration-200">
            <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Add DNS Record</h3>
                <button @click="showDnsModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                    <span class="material-symbols-outlined">close</span>
                </button>
            </div>
            <div class="p-6 space-y-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Type *</label>
                        <select v-model="dnsForm.record_type" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm">
                            <option>A</option><option>AAAA</option><option>CNAME</option><option>MX</option><option>TXT</option><option>NS</option><option>SRV</option><option>CAA</option>
                        </select>
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">TTL</label>
                        <input v-model.number="dnsForm.ttl" type="number" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                    </div>
                </div>
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Name *</label>
                    <input v-model="dnsForm.name" type="text" placeholder="@" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                </div>
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Value *</label>
                    <input v-model="dnsForm.value" type="text" placeholder="192.168.1.1" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                </div>
                <div v-if="dnsForm.record_type === 'MX'">
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Priority</label>
                    <input v-model.number="dnsForm.priority" type="number" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                </div>
            </div>
            <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                <button @click="showDnsModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a]">Cancel</button>
                <button @click="createDnsRecord" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                    {{ isSaving ? 'Adding...' : 'Add Record' }}
                </button>
            </div>
        </div>
    </div>

    <!-- Create Redirect Modal -->
    <div v-if="showRedirectModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-md animate-in zoom-in-95 duration-200">
            <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Add Redirect</h3>
                <button @click="showRedirectModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                    <span class="material-symbols-outlined">close</span>
                </button>
            </div>
            <div class="p-6 space-y-4">
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Type</label>
                    <select v-model="redirectForm.type" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm">
                        <option value="301">301 Permanent</option>
                        <option value="302">302 Temporary</option>
                    </select>
                </div>
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Source Path *</label>
                    <input v-model="redirectForm.source_path" type="text" placeholder="/old-page" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                </div>
                <div>
                    <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Destination URL *</label>
                    <input v-model="redirectForm.destination_url" type="url" placeholder="https://new-site.com" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm dark:text-white" />
                </div>
            </div>
            <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                <button @click="showRedirectModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a]">Cancel</button>
                <button @click="createRedirect" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                    {{ isSaving ? 'Creating...' : 'Add Redirect' }}
                </button>
            </div>
        </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteConfirm" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-sm animate-in zoom-in-95 duration-200">
            <div class="p-6 text-center">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                    <span class="material-symbols-outlined text-red-600 dark:text-red-400 text-3xl">warning</span>
                </div>
                <h3 class="text-lg font-bold text-[#0d131b] dark:text-white mb-2">Confirm Delete</h3>
                <p class="text-sm text-[#4c6c9a] dark:text-slate-400 mb-6">
                    Are you sure you want to delete <strong>{{ deleteTarget?.name }}</strong>? This action cannot be undone.
                </p>
                <div class="flex justify-center gap-3">
                    <button @click="showDeleteConfirm = false; deleteTarget = null" class="px-5 py-2 border border-[#cfd9e7] dark:border-slate-700 rounded-lg text-sm font-medium text-[#4c6c9a]">Cancel</button>
                    <button @click="executeDelete" :disabled="isSaving" class="px-5 py-2 bg-red-600 text-white rounded-lg text-sm font-bold hover:bg-red-700 disabled:opacity-50">
                        {{ isSaving ? 'Deleting...' : 'Delete' }}
                    </button>
                </div>
            </div>
        </div>
    </div>

</MainLayout>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=block');
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&display=swap');

/* Force font family override for this page */
.font-display {
    font-family: 'Inter', sans-serif;
}
.material-symbols-outlined {
  font-variation-settings:
  'FILL' 0,
  'wght' 400,
  'GRAD' 0,
  'opsz' 24
}
</style>
