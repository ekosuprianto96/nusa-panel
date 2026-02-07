<script setup lang="ts">
/**
 * DomainsPage - Comprehensive Domain Management
 * Full CRUD with Modals and Toast Notifications
 */
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { domainService } from '@/services'
import type { DomainResponse, SubdomainResponse, DnsRecordResponse, DdnsHostResponse } from '@/types'

// ==========================================
// STATE
// ==========================================
const currentView = ref<'dashboard' | 'dns' | 'subdomains' | 'redirects' | 'aliases' | 'ddns'>('dashboard')
const selectedDomainId = ref<string>('')
const domains = ref<DomainResponse[]>([])
const subdomains = ref<SubdomainResponse[]>([])
const dnsRecords = ref<DnsRecordResponse[]>([])
const redirects = ref<any[]>([])
const aliases = ref<any[]>([])
const isLoading = ref(false)
const isSaving = ref(false)
const searchQuery = ref('')
const openMenuId = ref<string | null>(null)
const menuPosition = ref<{ top: number; left: number } | null>(null)

// DDNS (Mock)
const ddnsHosts = ref<DdnsHostResponse[]>([])
const ddnsForm = ref({ hostname: '', description: '' })
const lastCreatedDdnsKey = ref<string | null>(null)

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
const showAliasModal = ref(false)
const showEditDomainModal = ref(false)
const showDeleteConfirm = ref(false)
const deleteTarget = ref<{ type: string; id: string; name: string } | null>(null)

// Form Data
const domainForm = ref({ domain_name: '', document_root: '' })
const editDomainForm = ref({ document_root: '', is_active: true })
const editDomainId = ref<string>('')
const subdomainForm = ref({ subdomain_name: '', document_root: '' })
const dnsForm = ref({ record_type: 'A', name: '', value: '', ttl: 14400, priority: 10 })
const redirectForm = ref({ source_path: '', destination_url: '', type: '301' as '301' | '302' })
const aliasForm = ref({ alias_domain: '' })

// Reset Forms
const resetForms = () => {
    domainForm.value = { domain_name: '', document_root: '' }
    editDomainForm.value = { document_root: '', is_active: true }
    editDomainId.value = ''
    subdomainForm.value = { subdomain_name: '', document_root: '' }
    dnsForm.value = { record_type: 'A', name: '', value: '', ttl: 14400, priority: 10 }
    redirectForm.value = { source_path: '', destination_url: '', type: '301' }
    aliasForm.value = { alias_domain: '' }
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

const toggleMenu = (id: string, event?: MouseEvent) => {
    if (openMenuId.value === id) {
        openMenuId.value = null
        menuPosition.value = null
        return
    }
    openMenuId.value = id
    if (event?.currentTarget instanceof HTMLElement) {
        const rect = event.currentTarget.getBoundingClientRect()
        const menuWidth = 176
        const padding = 8
        let left = rect.right - menuWidth
        const maxLeft = window.innerWidth - menuWidth - padding
        left = Math.min(Math.max(left, padding), maxLeft)
        menuPosition.value = { top: rect.bottom + 8, left }
    }
}

const closeMenu = () => {
    openMenuId.value = null
    menuPosition.value = null
}


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

const filteredAliases = computed(() => {
    if (!searchQuery.value) return aliases.value
    const q = searchQuery.value.toLowerCase()
    return aliases.value.filter(a => a.alias_domain?.toLowerCase().includes(q))
})

// ==========================================
// NAVIGATION
// ==========================================
const navigateTo = (view: typeof currentView.value, domainId?: string) => {
    currentView.value = view
    searchQuery.value = ''
    if (domainId) {
        selectedDomainId.value = domainId
    } else if (!selectedDomainId.value && domains.value.length > 0 && view !== 'dashboard') {
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
        } else if (currentView.value === 'aliases') {
            const res = await domainService.listAliases(selectedDomainId.value)
            aliases.value = res.data.data || []
        } else if (currentView.value === 'ddns') {
            const res = await domainService.listDdnsHosts(selectedDomainId.value)
            ddnsHosts.value = res.data.data || []
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

const openEditDomain = (domain: DomainResponse) => {
    editDomainId.value = domain.id
    editDomainForm.value = {
        document_root: domain.document_root || '',
        is_active: domain.is_active
    }
    showEditDomainModal.value = true
}

const updateDomain = async () => {
    if (!editDomainId.value) return
    isSaving.value = true
    try {
        const payload: { document_root?: string; is_active?: boolean } = {
            is_active: editDomainForm.value.is_active
        }
        if (editDomainForm.value.document_root?.trim()) {
            payload.document_root = editDomainForm.value.document_root.trim()
        }
        await domainService.updateDomain(editDomainId.value, payload)
        showToast('Domain updated', 'success')
        showEditDomainModal.value = false
        resetForms()
        fetchDomains()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed to update domain', 'error') }
    finally { isSaving.value = false }
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

// --- DDNS ---
const createDdnsHost = async () => {
    if (!ddnsForm.value.hostname || !selectedDomainId.value) return showToast('Hostname is required', 'error')
    isSaving.value = true
    try {
        const res = await domainService.createDdnsHost(selectedDomainId.value, {
            hostname: ddnsForm.value.hostname,
            description: ddnsForm.value.description || undefined,
        })
        lastCreatedDdnsKey.value = res.data.data.api_key
        showToast('DDNS host created. Copy the key from the table.', 'success')
        ddnsForm.value = { hostname: '', description: '' }
        fetchSubFeatures()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed', 'error') }
    finally { isSaving.value = false }
}

const deleteDdnsHost = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteDdnsHost(selectedDomainId.value, id)
        showToast('DDNS host deleted', 'success')
        fetchSubFeatures()
    } catch (e) { showToast('Failed to delete', 'error') }
    finally { isSaving.value = false; showDeleteConfirm.value = false; deleteTarget.value = null }
}

// --- Aliases ---
const createAlias = async () => {
    if (!aliasForm.value.alias_domain || !selectedDomainId.value) return showToast('Alias domain is required', 'error')
    isSaving.value = true
    try {
        await domainService.createAlias(selectedDomainId.value, aliasForm.value)
        showToast('Alias created!', 'success')
        showAliasModal.value = false
        resetForms()
        fetchSubFeatures()
    } catch (e: any) { showToast(e.response?.data?.message || 'Failed', 'error') }
    finally { isSaving.value = false }
}

const deleteAlias = async (id: string) => {
    isSaving.value = true
    try {
        await domainService.deleteAlias(selectedDomainId.value, id)
        showToast('Alias deleted', 'success')
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
    else if (type === 'ddns') await deleteDdnsHost(id)
    else if (type === 'alias') await deleteAlias(id)
}

// --- Utility ---
const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    showToast(`Copied: ${text}`, 'info')
}

const formatDate = (value: string) => {
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return date.toLocaleDateString()
}

const formatDateTime = (value?: string | null) => {
    if (!value) return '—'
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return value
    return date.toLocaleString()
}

// Watchers
watch(currentView, () => {
    if (currentView.value !== 'dashboard') fetchSubFeatures()
})
watch(selectedDomainId, fetchSubFeatures)

// Lifecycle
const onDocClick = () => { closeMenu() }
const onWindowResize = () => { closeMenu() }
const onWindowScroll = () => { closeMenu() }
onMounted(() => {
    fetchDomains()
    document.addEventListener('click', onDocClick)
    window.addEventListener('resize', onWindowResize)
    window.addEventListener('scroll', onWindowScroll, true)
})
onUnmounted(() => {
    document.removeEventListener('click', onDocClick)
    window.removeEventListener('resize', onWindowResize)
    window.removeEventListener('scroll', onWindowScroll, true)
})

</script>

<template>
    <MainLayout>
        <div class="text-foreground">
            
            <!-- =========================================================================
                VIEW: DASHBOARD (OVERVIEW)
                ========================================================================= -->
            <div v-if="currentView === 'dashboard'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
            <div class="mx-auto py-8 px-4 lg:px-8 text-[15px]">
                <div class="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-10">
                    <div>
                        <h2 class="text-3xl font-bold text-foreground">Domains</h2>
                        <p class="text-base text-muted-foreground mt-2">Manage your connected assets and registrar settings.</p>
                    </div>
                        <div class="flex flex-col sm:flex-row gap-4">
                            <div class="relative w-full sm:w-72">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <span class="material-symbols-outlined text-muted-foreground text-lg">search</span>
                                </div>
                            <input v-model="searchQuery" class="block w-full pl-10 pr-3 py-2.5 bg-background border border-border rounded-lg text-base focus:ring-1 focus:ring-primary focus:border-primary placeholder:text-muted-foreground" placeholder="Find a domain..." type="text"/>
                        </div>
                        <button @click="showDomainModal = true" class="flex items-center justify-center gap-2 bg-primary text-white px-6 py-2.5 rounded-lg text-base font-semibold hover:bg-primary/90 transition-all shadow-sm">
                            <span class="material-symbols-outlined text-lg">add</span>
                            <span>Register New</span>
                        </button>
                        </div>
                    </div>

                    <div class="grid grid-cols-12 gap-10">
                        <div class="col-span-12 lg:col-span-9 space-y-12">
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div class="bg-card border border-border p-5 rounded-2xl flex flex-col justify-center gap-2">
                                <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Total Managed</span>
                                <p class="text-2xl font-bold text-foreground">{{ stats.total }}</p>
                            </div>
                            <div class="bg-card border border-border p-5 rounded-2xl flex flex-col justify-center gap-2">
                                <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Active SSL</span>
                                <p class="text-2xl font-bold text-foreground">{{ stats.ssl_active }}</p>
                                <p class="text-sm text-muted-foreground">of {{ stats.total }}</p>
                            </div>
                            <div class="bg-card border border-border p-5 rounded-2xl flex flex-col justify-center gap-2">
                                <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Auto-Renewals</span>
                                <p class="text-2xl font-bold text-foreground">{{ stats.auto_renew }}</p>
                            </div>
                        </div>

                            <div class="space-y-4">
                            <div class="flex items-center justify-between px-1">
                                <h3 class="text-lg font-bold text-foreground">Domains List</h3>
                                <button class="text-muted-foreground hover:text-primary transition-colors flex items-center gap-1.5 text-sm font-medium">
                                    <span class="material-symbols-outlined text-sm">tune</span>
                                    <span>Filter</span>
                                </button>
                            </div>
                            <div class="overflow-x-auto bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm">
                                <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-slate-50 dark:bg-slate-900/50">
                                            <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Domain &amp; SSL</th>
                                            <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Document Root</th>
                                            <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">System Alias</th>
                                            <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Settings</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                        <tr v-for="domain in filteredDomains" :key="domain.id" class="action-row group">
                                            <td class="px-6 py-4">
                                                <div class="flex flex-col gap-1">
                                                    <div class="flex items-center gap-2">
                                                        <span class="text-base font-semibold text-[#0d131b] dark:text-white">{{ domain.domain_name }}</span>
                                                        <span v-if="!domain.is_active" class="flex items-center text-[10px] font-bold text-red-600 bg-red-50 dark:bg-red-900/20 px-1.5 py-0.5 rounded-full">
                                                            <span class="material-symbols-outlined text-[12px] mr-1">warning</span>Inactive
                                                        </span>
                                                        <span v-else-if="domain.ssl_enabled" class="flex items-center text-[10px] font-bold text-green-600 bg-green-50 dark:bg-green-900/20 px-1.5 py-0.5 rounded-full">
                                                            <span class="material-symbols-outlined text-[12px] mr-1">check_circle</span>SSL
                                                        </span>
                                                        <span v-else class="flex items-center text-[10px] font-bold text-[#4c6c9a] bg-slate-100 dark:bg-slate-700 px-1.5 py-0.5 rounded-full">
                                                            <span class="material-symbols-outlined text-[12px] mr-1">history</span>Pending
                                                        </span>
                                                    </div>
                                                    <span class="text-xs text-[#4c6c9a] dark:text-slate-400 uppercase tracking-tighter">Created {{ formatDate(domain.created_at) }}</span>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <span
                                                    class="text-primary hover:bg-primary/5 px-2 py-1 -ml-2 rounded text-sm font-semibold transition-colors"
                                                    :title="domain.document_root || '—'"
                                                >
                                                    {{ domain.document_root || '—' }}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 text-sm text-[#4c6c9a] dark:text-slate-400 font-medium">{{ domain.ssl_provider || domain.ssl_status || '—' }}</td>
                                            <td class="px-6 py-4">
                                                <div class="relative flex items-center justify-end">
                                                    <button
                                                        @click.stop="toggleMenu(domain.id, $event)"
                                                        class="p-1.5 text-[#4c6c9a] hover:text-primary transition-colors rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800"
                                                        title="Actions"
                                                    >
                                                        <span class="material-symbols-outlined text-lg">more_horiz</span>
                                                    </button>

                                                    <teleport to="body">
                                                        <div
                                                            v-if="openMenuId === domain.id"
                                                            @click.stop
                                                            class="fixed w-44 bg-white dark:bg-slate-900 border border-[#cfd9e7] dark:border-slate-800 rounded-xl shadow-lg z-[100] overflow-hidden"
                                                            :style="menuPosition ? { top: menuPosition.top + 'px', left: menuPosition.left + 'px' } : {}"
                                                        >
                                                            <button @click="openEditDomain(domain); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-[#4c6c9a] dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800">
                                                                <span class="material-symbols-outlined text-sm">edit_note</span>
                                                                <span>Edit</span>
                                                            </button>
                                                            <button @click="navigateTo('dns', domain.id); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-[#4c6c9a] dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800">
                                                                <span class="material-symbols-outlined text-sm">dns</span>
                                                                <span>DNS</span>
                                                            </button>
                                                            <button @click="navigateTo('subdomains', domain.id); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-[#4c6c9a] dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800">
                                                                <span class="material-symbols-outlined text-sm">layers</span>
                                                                <span>Subdomains</span>
                                                            </button>
                                                            <button @click="navigateTo('redirects', domain.id); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-[#4c6c9a] dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800">
                                                                <span class="material-symbols-outlined text-sm">shortcut</span>
                                                                <span>Redirects</span>
                                                            </button>
                                                            <button @click="navigateTo('aliases', domain.id); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-[#4c6c9a] dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800">
                                                                <span class="material-symbols-outlined text-sm">ios_share</span>
                                                                <span>Aliases</span>
                                                            </button>
                                                            <div class="h-px bg-[#cfd9e7] dark:bg-slate-800"></div>
                                                            <button @click="confirmDelete('domain', domain.id, domain.domain_name); closeMenu()" class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20">
                                                                <span class="material-symbols-outlined text-sm">delete</span>
                                                                <span>Delete</span>
                                                            </button>
                                                        </div>
                                                    </teleport>
                                                </div>
                                            </td>
                                        </tr>
                                    </tbody>
                                </table>
                                <div v-if="filteredDomains.length === 0" class="p-10 text-center text-sm text-[#4c6c9a] dark:text-slate-400">
                                    No domains found.
                                </div>
                            </div>
                            </div>

                            <div class="pt-6">
                                <h3 class="text-base font-bold text-foreground mb-6">Utilities</h3>
                                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                                    <button @click="navigateTo('subdomains')" class="group flex items-center gap-3 px-4 py-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-sm">
                                        <div class="p-1.5 rounded-md bg-muted text-primary">
                                            <span class="material-symbols-outlined text-base">add_link</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="text-sm font-medium text-foreground group-hover:text-primary transition-colors">Subdomains</p>
                                            <p class="text-[11px] text-muted-foreground">Create new</p>
                                        </div>
                                    </button>
                                    <button @click="navigateTo('dns')" class="group flex items-center gap-3 px-4 py-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-sm">
                                        <div class="p-1.5 rounded-md bg-muted text-primary">
                                            <span class="material-symbols-outlined text-base">dns</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="text-sm font-medium text-foreground group-hover:text-primary transition-colors">Zone Editor</p>
                                            <p class="text-[11px] text-muted-foreground">DNS records</p>
                                        </div>
                                    </button>
                                    <button @click="navigateTo('redirects')" class="group flex items-center gap-3 px-4 py-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-sm">
                                        <div class="p-1.5 rounded-md bg-muted text-primary">
                                            <span class="material-symbols-outlined text-base">shortcut</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="text-sm font-medium text-foreground group-hover:text-primary transition-colors">Redirects</p>
                                            <p class="text-[11px] text-muted-foreground">HTTP mapping</p>
                                        </div>
                                    </button>
                                    <button @click="navigateTo('ddns')" class="group flex items-center gap-3 px-4 py-3 bg-card border border-border rounded-xl hover:border-primary/50 transition-all hover:shadow-sm">
                                        <div class="p-1.5 rounded-md bg-muted text-primary">
                                            <span class="material-symbols-outlined text-base">dynamic_form</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="text-sm font-medium text-foreground group-hover:text-primary transition-colors">Dynamic DNS</p>
                                            <p class="text-[11px] text-muted-foreground">API driven</p>
                                        </div>
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div class="col-span-12 lg:col-span-3">
                            <div class="sticky top-12 space-y-6">
                                <div class="bg-white dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl p-7 shadow-[0_8px_30px_rgb(0,0,0,0.02)]">
                                    <h3 class="text-slate-900 dark:text-white text-[11px] font-bold uppercase tracking-widest mb-6">Nameservers</h3>
                                    <div class="space-y-6">
                                        <div>
                                            <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2">Primary</p>
                                            <div class="flex items-center justify-between group cursor-pointer bg-slate-50 dark:bg-slate-900/50 p-3 rounded-xl">
                                                <code class="text-xs font-semibold text-primary">ns1.hostpanel.com</code>
                                                <button @click="copyToClipboard('ns1.hostpanel.com')" class="material-symbols-outlined text-slate-300 group-hover:text-primary transition-colors text-lg">content_copy</button>
                                            </div>
                                        </div>
                                        <div>
                                            <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2">Secondary</p>
                                            <div class="flex items-center justify-between group cursor-pointer bg-slate-50 dark:bg-slate-900/50 p-3 rounded-xl">
                                                <code class="text-xs font-semibold text-primary">ns2.hostpanel.com</code>
                                                <button @click="copyToClipboard('ns2.hostpanel.com')" class="material-symbols-outlined text-slate-300 group-hover:text-primary transition-colors text-lg">content_copy</button>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="mt-8 pt-8 border-t border-slate-100 dark:border-slate-700">
                                        <div class="p-4 bg-primary/5 rounded-xl border border-primary/10">
                                            <p class="text-[11px] font-bold text-primary mb-2 flex items-center gap-2">
                                                <span class="material-symbols-outlined text-base">lightbulb</span>
                                                Pro Tip
                                            </p>
                                            <p class="text-xs text-slate-600 dark:text-slate-400 leading-relaxed">
                                                Point your domains here to automatically sync SSL &amp; DNS.
                                            </p>
                                            <a class="mt-3 block text-xs font-bold text-primary hover:underline" href="#">View Setup Guide →</a>
                                        </div>
                                    </div>
                                </div>
                                <div class="px-2">
                                    <button class="w-full py-3 text-xs font-bold text-slate-400 hover:text-primary transition-colors text-center border border-dashed border-slate-100 dark:border-slate-800 rounded-xl">
                                        Request Custom NS
                                    </button>
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
             <div class="flex flex-wrap items-center gap-3 mb-6">
                <AppBreadcrumb
                    :items="[
                        { label: 'Domains', icon: 'globe', onClick: () => navigateTo('dashboard') },
                        { label: 'Zone Editor', current: true }
                    ]"
                />
                <div class="relative">
                     <select v-model="selectedDomainId" class="appearance-none bg-transparent border-none text-foreground font-medium pr-6 py-0 focus:ring-0 cursor-pointer">
                         <option v-for="d in domains" :key="d.id" :value="d.id">Zone Editor: {{ d.domain_name }}</option>
                     </select>
                </div>
            </div>

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
                <AppBreadcrumb
                    class="mb-2"
                    :items="[
                        { label: 'Domains', icon: 'globe', onClick: () => navigateTo('dashboard') },
                        { label: 'Subdomain Management', current: true }
                    ]"
                />
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
                <div class="max-w-[1200px] mx-auto py-8">
                    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                    <div>
                        <AppBreadcrumb
                            class="mb-1"
                            :items="[
                                { label: 'Domains', icon: 'globe', onClick: () => navigateTo('dashboard') },
                                { label: 'Redirects', current: true }
                            ]"
                        />
                        <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Domain Redirects</h2>
                        <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">Send visitors of one domain or page to a different URL automatically.</p>
                    </div>
                        <div class="flex flex-col sm:flex-row gap-3">
                            <button class="flex items-center justify-center gap-2 bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 text-slate-500 px-5 py-2 rounded-lg text-sm font-bold hover:bg-slate-50 dark:hover:bg-slate-800/70 transition-all shadow-sm">
                                <span class="material-symbols-outlined text-lg">help_outline</span>
                                <span>Guide</span>
                            </button>
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
                                    <form @submit.prevent="createRedirect" class="space-y-5">
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
                                            <div class="space-y-2">
                                                <label class="text-sm font-bold text-slate-500 dark:text-slate-400">Type</label>
                                                <select v-model="redirectForm.type" class="block w-full h-10 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-sm focus:border-primary focus:ring-primary shadow-sm">
                                                    <option value="301">301 Permanent</option>
                                                    <option value="302">302 Temporary</option>
                                                </select>
                                                <p class="text-[10px] text-slate-500">301 tells browsers the site moved forever.</p>
                                            </div>
                                            <div class="space-y-2 md:col-span-2">
                                                <label class="text-sm font-bold text-slate-500 dark:text-slate-400">Redirect From</label>
                                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                                    <select v-model="selectedDomainId" class="block w-full h-10 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-sm focus:border-primary focus:ring-primary shadow-sm">
                                                        <option disabled value="">Select domain</option>
                                                        <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                                                    </select>
                                                    <div class="relative">
                                                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                                            <span class="text-slate-500 text-sm">/</span>
                                                        </div>
                                                        <input v-model="redirectForm.source_path" class="block w-full h-10 pl-6 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-sm focus:border-primary focus:ring-primary shadow-sm" placeholder="path (optional)" type="text"/>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-sm font-bold text-slate-500 dark:text-slate-400">Redirects To (Destination URL)</label>
                                            <div class="relative">
                                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                                    <span class="material-symbols-outlined text-slate-500 text-lg">link</span>
                                                </div>
                                                <input v-model="redirectForm.destination_url" class="block w-full h-10 pl-10 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-sm focus:border-primary focus:ring-primary shadow-sm" placeholder="https://new-destination.com" type="url"/>
                                            </div>
                                            <p class="text-[10px] text-slate-500">Must include the protocol (http:// or https://).</p>
                                        </div>
                                        <div class="flex flex-wrap items-center gap-4 pt-1">
                                            <button type="submit" :disabled="isSaving" class="bg-primary text-white px-6 py-2.5 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md disabled:opacity-50">
                                                {{ isSaving ? 'Adding...' : 'Add Redirect' }}
                                            </button>
                                            <button type="reset" @click="redirectForm = { source_path: '', destination_url: '', type: '301' }" class="text-slate-500 text-sm font-semibold hover:text-[#0d131b] dark:hover:text-white transition-colors">
                                                Clear Form
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            </div>

                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                                <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                                    <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Current Redirects</h2>
                                    <div class="relative">
                                        <span class="material-symbols-outlined absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500 text-lg">search</span>
                                        <input v-model="searchQuery" class="pl-9 py-1.5 rounded-lg border-[#cfd9e7] dark:border-slate-700 dark:bg-slate-900 text-xs w-48" placeholder="Search redirects..." type="text"/>
                                    </div>
                                </div>
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-slate-50 dark:bg-slate-900/50">
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Domain/Path</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Type</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Destination</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                            <tr v-for="redirect in filteredRedirects" :key="redirect.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <div class="flex flex-col">
                                                        <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ selectedDomainObj?.domain_name || 'Domain' }}{{ redirect.source_path ? `/${redirect.source_path}` : '' }}</span>
                                                        <span class="text-[10px] text-slate-500">Wildcard: {{ redirect.is_wildcard ? 'Yes' : 'No' }}</span>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <span :class="[redirect.type === '302' ? 'bg-amber-50 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 border-amber-100 dark:border-amber-800' : 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 border-blue-100 dark:border-blue-800', 'px-2 py-1 rounded text-[10px] font-bold border']">{{ redirect.type }} {{ redirect.type === '302' ? 'Temporary' : 'Permanent' }}</span>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <div class="flex items-center gap-2 max-w-[200px] truncate">
                                                        <span class="material-symbols-outlined text-sm text-slate-500">subdirectory_arrow_right</span>
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
                                    <div v-if="filteredRedirects.length === 0" class="p-10 text-center text-sm text-slate-400">No redirects found.</div>
                                </div>
                            </div>
                        </div>

                        <div class="col-span-12 lg:col-span-4 space-y-6">
                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl p-6 shadow-sm">
                                <h3 class="text-[#0d131b] dark:text-white text-sm font-bold uppercase tracking-wider mb-4 flex items-center gap-2">
                                    <span class="material-symbols-outlined text-primary text-xl">info</span>
                                    About Redirects
                                </h3>
                                <div class="space-y-4">
                                    <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                        <h4 class="text-xs font-bold text-primary mb-1">301 Permanent</h4>
                                        <p class="text-xs text-slate-500 leading-relaxed">Recommended for SEO. Passes ranking power to the new URL. The browser caches this redirect.</p>
                                    </div>
                                    <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                        <h4 class="text-xs font-bold text-orange-500 mb-1">302 Temporary</h4>
                                        <p class="text-xs text-slate-500 leading-relaxed">Use for maintenance or limited-time offers. Search engines keep the original URL in the index.</p>
                                    </div>
                                </div>
                            </div>

                            <div class="bg-primary/5 border border-primary/20 rounded-xl p-6">
                                <h4 class="text-primary text-sm font-bold mb-2">Wildcard Redirects</h4>
                                <p class="text-slate-500 dark:text-slate-400 text-xs leading-relaxed mb-4">
                                    Enable wildcard redirects to point all files in a directory to the same file name in the new destination.
                                </p>
                                <a class="text-primary text-xs font-bold flex items-center gap-1 hover:underline" href="#">
                                    Advanced Settings <span class="material-symbols-outlined text-sm">settings_input_component</span>
                                </a>
                            </div>

                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl p-6 shadow-sm">
                                <h3 class="text-[#0d131b] dark:text-white text-sm font-bold uppercase tracking-wider mb-4">Quick Stats</h3>
                                <div class="grid grid-cols-2 gap-4 text-center">
                                    <div class="p-3 border border-slate-100 dark:border-slate-700 rounded-lg">
                                        <p class="text-2xl font-bold text-primary">{{ redirects.length }}</p>
                                        <p class="text-[10px] uppercase text-slate-500 font-bold">Active</p>
                                    </div>
                                    <div class="p-3 border border-slate-100 dark:border-slate-700 rounded-lg">
                                        <p class="text-2xl font-bold text-[#0d131b] dark:text-white">50</p>
                                        <p class="text-[10px] uppercase text-slate-500 font-bold">Limit</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- =========================================================================
                VIEW: DOMAIN ALIASES
                ========================================================================= -->
            <div v-else-if="currentView === 'aliases'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                    <div>
                        <AppBreadcrumb
                            class="mb-1"
                            :items="[
                                { label: 'Domains', icon: 'globe', onClick: () => navigateTo('dashboard') },
                                { label: 'Aliases', current: true }
                            ]"
                        />
                        <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Domain Aliases</h2>
                        <p class="text-[#4c6c9a] dark:text-slate-400 text-sm mt-1">Point multiple domains to the same website.</p>
                    </div>
                    <div class="flex flex-col sm:flex-row gap-3">
                        <div class="relative w-full sm:w-64">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <span class="material-symbols-outlined text-[#4c6c9a] text-xl">search</span>
                            </div>
                            <input v-model="searchQuery" class="block w-full pl-10 pr-3 py-2 border-none bg-white dark:bg-slate-800 rounded-lg text-sm focus:ring-2 focus:ring-primary shadow-sm dark:text-white placeholder:text-[#4c6c9a]" placeholder="Search aliases..." type="text"/>
                        </div>
                        <button @click="showAliasModal = true" class="flex items-center justify-center gap-2 bg-primary text-white px-5 py-2 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md">
                            <span class="material-symbols-outlined text-lg">add</span>
                            <span>Add Alias</span>
                        </button>
                    </div>
                </div>

                <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                    <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Aliases for</h2>
                            <select v-model="selectedDomainId" class="appearance-none bg-transparent border-none text-[#0d131b] dark:text-white font-medium pr-6 py-0 focus:ring-0 cursor-pointer">
                                <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                            </select>
                        </div>
                    </div>
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="bg-slate-50 dark:bg-slate-900/50">
                                    <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Alias Domain</th>
                                    <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                <tr v-for="alias in filteredAliases" :key="alias.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-semibold text-[#0d131b] dark:text-white">{{ alias.alias_domain }}</td>
                                    <td class="px-6 py-4 whitespace-nowrap text-right">
                                        <button @click="confirmDelete('alias', alias.id, alias.alias_domain)" class="text-[#e73908] hover:bg-red-50 dark:hover:bg-red-900/20 p-1.5 rounded transition-colors" title="Delete Alias">
                                            <span class="material-symbols-outlined text-lg">delete</span>
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                        <div v-if="filteredAliases.length === 0" class="p-10 text-center text-sm text-[#4c6c9a] dark:text-slate-400">
                            No aliases found for this domain.
                        </div>
                    </div>
                </div>
            </div>

            <!-- =========================================================================
                VIEW: DYNAMIC DNS (Mock)
                ========================================================================= -->
            <div v-else-if="currentView === 'ddns'" class="animate-in fade-in slide-in-from-bottom-2 duration-500">
                <div class="max-w-[1200px] mx-auto py-8">
                    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8">
                    <div>
                        <AppBreadcrumb
                            class="mb-2"
                            :items="[
                                { label: 'Domains', icon: 'globe', onClick: () => navigateTo('dashboard') },
                                { label: 'Dynamic DNS', current: true }
                            ]"
                        />
                        <h2 class="text-[#0d131b] dark:text-white text-3xl font-black tracking-tight">Dynamic DNS</h2>
                        <p class="text-slate-500 dark:text-slate-400 text-sm mt-1">Automatically update your DNS records when your IP address changes.</p>
                    </div>
                    </div>

                    <div class="grid grid-cols-12 gap-6">
                        <div class="col-span-12 lg:col-span-8 space-y-6">
                            <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800 rounded-xl p-5 flex gap-4">
                                <span class="material-symbols-outlined text-blue-600 dark:text-blue-400">lightbulb</span>
                                <div>
                                    <h4 class="text-blue-900 dark:text-blue-200 text-sm font-bold mb-1">How it works</h4>
                                    <p class="text-blue-800/80 dark:text-blue-300/80 text-sm leading-relaxed">
                                        Dynamic DNS (DDNS) allows you to point a domain or subdomain to a computer with a changing IP address. Use the generated ID and Key in your DDNS client to keep your records synchronized.
                                    </p>
                                </div>
                            </div>

                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                                <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700">
                                    <h3 class="text-[#0d131b] dark:text-white text-lg font-bold">Create a Dynamic DNS Host</h3>
                                </div>
                                <div class="p-6">
                                    <form class="space-y-4" @submit.prevent="createDdnsHost">
                                        <div class="space-y-1.5">
                                            <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">Domain</label>
                                            <select v-model="selectedDomainId" class="block w-full h-10 rounded-lg border-[#cfd9e7] dark:border-slate-600 bg-white dark:bg-slate-900 text-sm focus:ring-1 focus:ring-primary focus:border-primary dark:text-white">
                                                <option disabled value="">Select domain</option>
                                                <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                                            </select>
                                        </div>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div class="space-y-1.5">
                                                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">Host Name</label>
                                                <div class="flex">
                                                    <input v-model="ddnsForm.hostname" class="block w-full h-10 px-3 border border-[#cfd9e7] dark:border-slate-600 bg-white dark:bg-slate-900 rounded-l-lg text-sm focus:ring-1 focus:ring-primary focus:border-primary dark:text-white" placeholder="home-server" type="text"/>
                                                    <span class="inline-flex items-center px-3 rounded-r-lg border border-l-0 border-[#cfd9e7] dark:border-slate-600 bg-slate-50 dark:bg-slate-800 text-slate-500 text-sm">.{{ selectedDomainObj?.domain_name || 'domain.com' }}</span>
                                                </div>
                                            </div>
                                            <div class="space-y-1.5">
                                                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider">Description (Optional)</label>
                                                <input v-model="ddnsForm.description" class="block w-full h-10 px-3 border border-[#cfd9e7] dark:border-slate-600 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-1 focus:ring-primary focus:border-primary dark:text-white" placeholder="My local development machine" type="text"/>
                                            </div>
                                        </div>
                                        <div class="flex justify-end pt-2">
                                            <button :disabled="isSaving || !selectedDomainId" class="bg-primary text-white px-6 py-2 rounded-lg text-sm font-bold hover:bg-blue-700 transition-all shadow-md flex items-center gap-2 disabled:opacity-50" type="submit">
                                                <span class="material-symbols-outlined text-lg">add</span>
                                                Create Host
                                            </button>
                                        </div>
                                    </form>
                                    <div v-if="lastCreatedDdnsKey" class="mt-4 p-3 bg-primary/5 border border-primary/20 rounded-lg">
                                        <p class="text-xs font-bold text-primary mb-1">DDNS Key (save this)</p>
                                        <div class="flex items-center justify-between gap-2">
                                            <code class="text-xs font-mono font-semibold text-primary break-all">{{ lastCreatedDdnsKey }}</code>
                                            <button class="text-slate-500 hover:text-primary" @click="copyToClipboard(lastCreatedDdnsKey)">
                                                <span class="material-symbols-outlined text-sm">content_copy</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden">
                                <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                                    <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Existing DDNS Hosts</h2>
                                </div>
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-slate-50 dark:bg-slate-900/50">
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Hostname</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">ID / Key</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Current IP</th>
                                                <th class="px-6 py-4 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                                            <tr v-for="host in ddnsHosts" :key="host.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                                <td class="px-6 py-4">
                                                    <div class="flex flex-col">
                                                        <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ host.hostname }}</span>
                                                        <span class="text-xs text-slate-500 dark:text-slate-400">{{ host.description || 'Auto-managed' }}</span>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4">
                                                    <div class="flex items-center gap-2">
                                                        <code class="text-xs bg-slate-100 dark:bg-slate-900 px-2 py-1 rounded font-mono text-primary">{{ host.id }}</code>
                                                        <button class="text-slate-500 hover:text-primary" @click="copyToClipboard(host.id)">
                                                            <span class="material-symbols-outlined text-sm">content_copy</span>
                                                        </button>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4">
                                                    <div class="flex items-center gap-2">
                                                        <span class="size-2 rounded-full bg-green-500"></span>
                                                        <span class="text-sm font-medium text-[#0d131b] dark:text-slate-300">{{ host.last_ip || '—' }}</span>
                                                    </div>
                                                    <span class="text-[10px] text-slate-500 uppercase font-bold">Updated {{ formatDateTime(host.last_updated_at) }}</span>
                                                </td>
                                                <td class="px-6 py-4 text-right">
                                                    <div class="flex justify-end gap-2">
                                                        <button class="p-1.5 text-slate-500 hover:text-primary hover:bg-primary/5 rounded">
                                                            <span class="material-symbols-outlined text-lg">edit</span>
                                                        </button>
                                                        <button @click="confirmDelete('ddns', host.id, host.hostname)" class="p-1.5 text-slate-500 hover:text-red-500 hover:bg-red-50 rounded">
                                                            <span class="material-symbols-outlined text-lg">delete</span>
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>

                        <div class="col-span-12 lg:col-span-4 space-y-6">
                            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl p-6 shadow-sm">
                                <h3 class="text-[#0d131b] dark:text-white text-sm font-bold uppercase tracking-wider mb-4 flex items-center gap-2">
                                    <span class="material-symbols-outlined text-primary text-xl">settings_ethernet</span>
                                    Client Configuration
                                </h3>
                                <p class="text-slate-500 dark:text-slate-400 text-sm mb-4">Use these settings in your DDNS client software (e.g., ddclient, Inadyn).</p>
                                <div class="space-y-4">
                                    <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                        <p class="text-[10px] uppercase font-bold text-slate-500 mb-1">Update URL</p>
                                        <div class="flex items-center justify-between gap-2">
                                            <code class="text-[11px] font-mono font-semibold text-primary overflow-hidden text-ellipsis">https://ddns.hostpanel.com/update</code>
                                            <button class="text-slate-500 hover:text-primary shrink-0" @click="copyToClipboard('https://ddns.hostpanel.com/update')">
                                                <span class="material-symbols-outlined text-lg">content_copy</span>
                                            </button>
                                        </div>
                                    </div>
                                    <div class="p-3 bg-slate-50 dark:bg-slate-900 rounded-lg border border-[#cfd9e7] dark:border-slate-700">
                                        <p class="text-[10px] uppercase font-bold text-slate-500 mb-1">Protocol</p>
                                        <p class="text-xs font-semibold text-[#0d131b] dark:text-white">DYNDNS2 / HTTP-AUTH</p>
                                    </div>
                                </div>
                                <div class="mt-6 pt-6 border-t border-slate-100 dark:border-slate-700">
                                    <h4 class="text-xs font-bold text-[#0d131b] dark:text-white uppercase mb-2">Supported Clients</h4>
                                    <div class="flex flex-wrap gap-2">
                                        <span class="px-2 py-1 bg-slate-100 dark:bg-slate-700 rounded text-[10px] font-bold text-slate-500 dark:text-slate-300">ddclient</span>
                                        <span class="px-2 py-1 bg-slate-100 dark:bg-slate-700 rounded text-[10px] font-bold text-slate-500 dark:text-slate-300">Inadyn</span>
                                        <span class="px-2 py-1 bg-slate-100 dark:bg-slate-700 rounded text-[10px] font-bold text-slate-500 dark:text-slate-300">pfSense</span>
                                        <span class="px-2 py-1 bg-slate-100 dark:bg-slate-700 rounded text-[10px] font-bold text-slate-500 dark:text-slate-300">MikroTik</span>
                                    </div>
                                </div>
                            </div>

                            <div class="rounded-xl overflow-hidden bg-primary/5 border border-primary/20 p-5">
                                <h4 class="text-primary text-sm font-bold mb-2">Automation Help</h4>
                                <p class="text-slate-500 dark:text-slate-400 text-xs leading-relaxed mb-4">Want to update via a simple curl command? Check our script examples for Linux and Windows.</p>
                                <a class="text-primary text-xs font-bold flex items-center gap-1 hover:underline" href="#">
                                    View API Documentation <span class="material-symbols-outlined text-sm">open_in_new</span>
                                </a>
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
        <div v-if="showDomainModal || showEditDomainModal || showSubdomainModal || showDnsModal || showRedirectModal || showAliasModal || showDeleteConfirm" 
            @click="showDomainModal = false; showEditDomainModal = false; showSubdomainModal = false; showDnsModal = false; showRedirectModal = false; showAliasModal = false; showDeleteConfirm = false"
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

        <!-- Edit Domain Modal -->
        <div v-if="showEditDomainModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-md animate-in zoom-in-95 duration-200">
                <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                    <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Edit Domain</h3>
                    <button @click="showEditDomainModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                        <span class="material-symbols-outlined">close</span>
                    </button>
                </div>
                <div class="p-6 space-y-4">
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Document Root</label>
                        <input v-model="editDomainForm.document_root" type="text" placeholder="/public_html" 
                            class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                    </div>
                    <div class="flex items-center justify-between rounded-lg border border-[#cfd9e7] dark:border-slate-700 px-4 py-3">
                        <div>
                            <p class="text-sm font-semibold text-[#0d131b] dark:text-white">Active Domain</p>
                            <p class="text-xs text-[#4c6c9a] dark:text-slate-400">Disable to pause routing and services.</p>
                        </div>
                        <button @click="editDomainForm.is_active = !editDomainForm.is_active" :class="['relative inline-flex h-6 w-11 items-center rounded-full transition-colors', editDomainForm.is_active ? 'bg-green-600' : 'bg-slate-300 dark:bg-slate-700']">
                            <span :class="['inline-block h-5 w-5 transform rounded-full bg-white border border-slate-300 shadow transition-transform', editDomainForm.is_active ? 'translate-x-5' : 'translate-x-0.5']" />
                        </button>
                    </div>
                </div>
                <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                    <button @click="showEditDomainModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a] hover:text-[#0d131b]">Cancel</button>
                    <button @click="updateDomain" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                        {{ isSaving ? 'Saving...' : 'Save Changes' }}
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

        <!-- Create Alias Modal -->
        <div v-if="showAliasModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-md animate-in zoom-in-95 duration-200">
                <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                    <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Add Domain Alias</h3>
                    <button @click="showAliasModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                        <span class="material-symbols-outlined">close</span>
                    </button>
                </div>
                <div class="p-6 space-y-4">
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Alias Domain *</label>
                        <input v-model="aliasForm.alias_domain" type="text" placeholder="alias-example.com"
                            class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Target Domain</label>
                        <select v-model="selectedDomainId" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary focus:border-primary dark:text-white">
                            <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                        </select>
                    </div>
                </div>
                <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                    <button @click="showAliasModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a]">Cancel</button>
                    <button @click="createAlias" :disabled="isSaving" class="px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50">
                        {{ isSaving ? 'Creating...' : 'Add Alias' }}
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

<style scoped>
.hover-actions {
    opacity: 1;
    transition: opacity 0.2s ease-in-out;
}

@media (hover: hover) {
    .hover-actions {
        opacity: 0;
    }

    .action-row:hover .hover-actions {
        opacity: 1;
    }
}
</style>

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
