<script setup lang="ts">
/**
 * DomainsPage - Halaman manajemen domain (cPanel-like)
 * 
 * Features:
 * - Domain management
 * - Subdomains
 * - SSL/TLS certificates
 * - DNS Zone Editor
 * - Redirects (301/302)
 * - Domain Aliases (Parked Domains)
 */
import { ref, onMounted, computed, watch } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { 
  Globe, Plus, Trash2, 
  AlertCircle, X, Lock,
  FileText, Check,
  Server, Edit, ChevronDown, Layers, CornerUpRight
} from 'lucide-vue-next'
import { domainService } from '@/services'
import type { DomainResponse, SubdomainResponse, DnsRecordResponse } from '@/types'

// ==========================================
// STATE
// ==========================================
const domains = ref<DomainResponse[]>([])
const subdomains = ref<SubdomainResponse[]>([])
const dnsRecords = ref<DnsRecordResponse[]>([])
const redirects = ref<any[]>([])
const aliases = ref<any[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)
const activeTab = ref<'domains' | 'subdomains' | 'ssl' | 'dns' | 'redirects' | 'aliases'>('domains')

// Selected domain for sub-features
const selectedDomainId = ref<string>('')

// Modal states
const showAddDomainModal = ref(false)
const showDeleteDomainModal = ref(false)
const showAddSubdomainModal = ref(false)
const showDeleteSubdomainModal = ref(false)
const showAddDnsModal = ref(false)
const showEditDnsModal = ref(false)
const showDeleteDnsModal = ref(false)
const showRequestSslModal = ref(false)
const showAddRedirectModal = ref(false)
const showDeleteRedirectModal = ref(false)
const showAddAliasModal = ref(false)
const showDeleteAliasModal = ref(false)

// Form data - Domain
const newDomainName = ref('')
const newDocumentRoot = ref('')

// Form data - Subdomain
const newSubdomainName = ref('')
const newSubdomainDocRoot = ref('')

// Form data - DNS
const newDnsType = ref('A')
const newDnsName = ref('')
const newDnsValue = ref('')
const newDnsTtl = ref(14400)
const newDnsPriority = ref(10)
const editingDnsRecord = ref<DnsRecordResponse | null>(null)

// Form data - Redirect
const newRedirectPath = ref('/')
const newRedirectDest = ref('')
const newRedirectType = ref<'301' | '302'>('301')

// Form data - Alias
const newAliasDomain = ref('')

// Selected items for delete
const selectedDomain = ref<DomainResponse | null>(null)
const selectedSubdomain = ref<SubdomainResponse | null>(null)
const selectedDnsRecord = ref<DnsRecordResponse | null>(null)
const selectedRedirect = ref<any>(null)
const selectedAlias = ref<any>(null)

// DNS Record types
const dnsTypes = [
  { value: 'A', label: 'A (IPv4 Address)', placeholder: '192.168.1.1' },
  { value: 'AAAA', label: 'AAAA (IPv6 Address)', placeholder: '2001:db8::1' },
  { value: 'CNAME', label: 'CNAME (Alias)', placeholder: 'www.example.com' },
  { value: 'MX', label: 'MX (Mail Exchange)', placeholder: 'mail.example.com' },
  { value: 'TXT', label: 'TXT (Text Record)', placeholder: 'v=spf1 include:...' },
  { value: 'NS', label: 'NS (Name Server)', placeholder: 'ns1.example.com' },
  { value: 'SRV', label: 'SRV (Service)', placeholder: '0 5 5269 xmpp.example.com' },
  { value: 'CAA', label: 'CAA (Cert Authority)', placeholder: '0 issue "letsencrypt.org"' },
]

// Computed
const selectedDomainObj = computed(() => 
  domains.value.find(d => d.id === selectedDomainId.value)
)

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Fetch domains
 */
const fetchDomains = async (): Promise<void> => {
  try {
    isLoading.value = true
    error.value = null
    const response = await domainService.listDomains(1, 50)
    domains.value = response.data.data.items || []
    if (domains.value.length > 0 && !selectedDomainId.value) {
      selectedDomainId.value = domains.value[0]?.id || ''
    }
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat domain'
  } finally {
    isLoading.value = false
  }
}

/**
 * Fetch subfeatures based on active tab
 */
const fetchSubFeatures = async (): Promise<void> => {
  if (!selectedDomainId.value) return
  try {
    if (activeTab.value === 'subdomains') {
      const res = await domainService.listSubdomains(selectedDomainId.value)
      subdomains.value = res.data.data || []
    } else if (activeTab.value === 'dns') {
      const res = await domainService.listDnsRecords(selectedDomainId.value)
      dnsRecords.value = res.data.data || []
    } else if (activeTab.value === 'redirects') {
      const res = await domainService.listRedirects(selectedDomainId.value)
      redirects.value = res.data.data || []
    } else if (activeTab.value === 'aliases') {
      const res = await domainService.listAliases(selectedDomainId.value)
      aliases.value = res.data.data || []
    }
  } catch (err: any) {
    console.error(`Failed to fetch ${activeTab.value}:`, err)
  }
}

// Watchers
watch(selectedDomainId, fetchSubFeatures)
watch(activeTab, fetchSubFeatures)

const showSuccess = (msg: string): void => {
  successMsg.value = msg
  setTimeout(() => successMsg.value = null, 3000)
}

// --- Domain CRUD ---
const createDomain = async (): Promise<void> => {
  if (!newDomainName.value.trim()) return
  try {
    isLoading.value = true
    await domainService.createDomain({
      domain_name: newDomainName.value.trim(),
      document_root: newDocumentRoot.value.trim() || undefined
    })
    showAddDomainModal.value = false
    newDomainName.value = ''
    newDocumentRoot.value = ''
    showSuccess('Domain berhasil ditambahkan')
    await fetchDomains()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menambahkan domain'
  } finally {
    isLoading.value = false
  }
}

const deleteDomain = async (): Promise<void> => {
  if (!selectedDomain.value) return
  try {
    isLoading.value = true
    await domainService.deleteDomain(selectedDomain.value.id)
    showDeleteDomainModal.value = false
    selectedDomain.value = null
    showSuccess('Domain berhasil dihapus')
    await fetchDomains()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus domain'
  } finally {
    isLoading.value = false
  }
}

// --- Subdomain CRUD ---
const createSubdomain = async (): Promise<void> => {
  if (!newSubdomainName.value.trim() || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.createSubdomain(selectedDomainId.value, {
      subdomain_name: newSubdomainName.value.trim(),
      document_root: newSubdomainDocRoot.value.trim() || undefined
    })
    showAddSubdomainModal.value = false
    newSubdomainName.value = ''
    newSubdomainDocRoot.value = ''
    showSuccess('Subdomain berhasil dibuat')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat subdomain'
  } finally {
    isLoading.value = false
  }
}

const deleteSubdomain = async (): Promise<void> => {
  if (!selectedSubdomain.value || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.deleteSubdomain(selectedDomainId.value, selectedSubdomain.value.id)
    showDeleteSubdomainModal.value = false
    selectedSubdomain.value = null
    showSuccess('Subdomain berhasil dihapus')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus subdomain'
  } finally {
    isLoading.value = false
  }
}

// --- DNS CRUD ---
const createDnsRecord = async (): Promise<void> => {
  if (!newDnsName.value.trim() || !newDnsValue.value.trim() || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.createDnsRecord(selectedDomainId.value, {
      record_type: newDnsType.value,
      name: newDnsName.value.trim(),
      value: newDnsValue.value.trim(),
      ttl: newDnsTtl.value,
      priority: newDnsType.value === 'MX' ? newDnsPriority.value : undefined
    })
    showAddDnsModal.value = false
    resetDnsForm()
    showSuccess('DNS record berhasil dibuat')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat DNS record'
  } finally {
    isLoading.value = false
  }
}

const updateDnsRecord = async (): Promise<void> => {
  if (!editingDnsRecord.value || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.updateDnsRecord(selectedDomainId.value, editingDnsRecord.value.id, {
      value: newDnsValue.value.trim(),
      ttl: newDnsTtl.value,
      priority: newDnsType.value === 'MX' ? newDnsPriority.value : undefined
    })
    showEditDnsModal.value = false
    editingDnsRecord.value = null
    resetDnsForm()
    showSuccess('DNS record berhasil diupdate')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal update DNS record'
  } finally {
    isLoading.value = false
  }
}

const deleteDnsRecord = async (): Promise<void> => {
  if (!selectedDnsRecord.value || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.deleteDnsRecord(selectedDomainId.value, selectedDnsRecord.value.id)
    showDeleteDnsModal.value = false
    selectedDnsRecord.value = null
    showSuccess('DNS record berhasil dihapus')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus DNS record'
  } finally {
    isLoading.value = false
  }
}

const resetDnsForm = (): void => {
  newDnsType.value = 'A'
  newDnsName.value = ''
  newDnsValue.value = ''
  newDnsTtl.value = 14400
  newDnsPriority.value = 10
}

// --- Redirects CRUD ---
const createRedirect = async (): Promise<void> => {
  if (!newRedirectDest.value.trim() || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.createRedirect(selectedDomainId.value, {
      source_path: newRedirectPath.value.trim(),
      destination_url: newRedirectDest.value.trim(),
      type: newRedirectType.value
    })
    showAddRedirectModal.value = false
    newRedirectPath.value = '/'
    newRedirectDest.value = ''
    showSuccess('Redirect berhasil dibuat')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat redirect'
  } finally {
    isLoading.value = false
  }
}

const deleteRedirect = async (): Promise<void> => {
  if (!selectedRedirect.value || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.deleteRedirect(selectedDomainId.value, selectedRedirect.value.id)
    showDeleteRedirectModal.value = false
    selectedRedirect.value = null
    showSuccess('Redirect berhasil dihapus')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus redirect'
  } finally {
    isLoading.value = false
  }
}

// --- Aliases CRUD ---
const createAlias = async (): Promise<void> => {
  if (!newAliasDomain.value.trim() || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.createAlias(selectedDomainId.value, {
      alias_domain: newAliasDomain.value.trim()
    })
    showAddAliasModal.value = false
    newAliasDomain.value = ''
    showSuccess('Alias berhasil ditambahkan')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat alias'
  } finally {
    isLoading.value = false
  }
}

const deleteAlias = async (): Promise<void> => {
  if (!selectedAlias.value || !selectedDomainId.value) return
  try {
    isLoading.value = true
    await domainService.deleteAlias(selectedDomainId.value, selectedAlias.value.id)
    showDeleteAliasModal.value = false
    selectedAlias.value = null
    showSuccess('Alias berhasil dihapus')
    await fetchSubFeatures()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus alias'
  } finally {
    isLoading.value = false
  }
}

// Modal Toggle Helpers
const openDeleteDomain = (domain: DomainResponse) => { selectedDomain.value = domain; showDeleteDomainModal.value = true }
const openDeleteSubdomain = (sub: SubdomainResponse) => { selectedSubdomain.value = sub; showDeleteSubdomainModal.value = true }
const openEditDns = (record: DnsRecordResponse) => {
  editingDnsRecord.value = record
  newDnsType.value = record.record_type
  newDnsName.value = record.name
  newDnsValue.value = record.value
  newDnsTtl.value = record.ttl || 14400
  newDnsPriority.value = record.priority || 10
  showEditDnsModal.value = true
}
const openDeleteDns = (record: DnsRecordResponse) => { selectedDnsRecord.value = record; showDeleteDnsModal.value = true }
const openDeleteRedirect = (redirect: any) => { selectedRedirect.value = redirect; showDeleteRedirectModal.value = true }
const openDeleteAlias = (alias: any) => { selectedAlias.value = alias; showDeleteAliasModal.value = true }

onMounted(() => {
  fetchDomains()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-7xl mx-auto space-y-6 animate-in">
      <!-- Success Message -->
      <div v-if="successMsg" class="fixed top-4 right-4 bg-emerald-500 text-white px-6 py-3 rounded-xl shadow-lg z-50 flex items-center gap-2 animate-in">
        <Check class="w-5 h-5" />
        {{ successMsg }}
      </div>

      <!-- Header -->
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 class="text-2xl font-bold text-foreground tracking-tight flex items-center gap-3">
            <div class="p-2 bg-primary/10 rounded-xl">
              <Globe class="w-5 h-5 text-primary" />
            </div>
            Domain Manager
          </h1>
          <p class="text-muted-foreground mt-1 text-sm">Manage domains, subdomains, SSL, redirects, and DNS.</p>
        </div>
        
        <div class="flex gap-2">
          <button v-if="activeTab === 'domains'" @click="showAddDomainModal = true" class="btn-primary">
            <Plus class="w-4 h-4" /> Add Domain
          </button>
          <button v-if="activeTab === 'subdomains' && selectedDomainId" @click="showAddSubdomainModal = true" class="btn-primary bg-indigo-600 hover:bg-indigo-700 shadow-indigo-600/20">
            <Plus class="w-4 h-4" /> Add Subdomain
          </button>
          <button v-if="activeTab === 'ssl' && selectedDomainId" @click="showRequestSslModal = true" class="btn-primary bg-emerald-600 hover:bg-emerald-700 shadow-emerald-600/20">
            <Lock class="w-4 h-4" /> Request SSL
          </button>
          <button v-if="activeTab === 'dns' && selectedDomainId" @click="showAddDnsModal = true" class="btn-primary bg-blue-600 hover:bg-blue-700 shadow-blue-600/20">
            <Plus class="w-4 h-4" /> Add Record
          </button>
          <button v-if="activeTab === 'redirects' && selectedDomainId" @click="showAddRedirectModal = true" class="btn-primary bg-amber-600 hover:bg-amber-700 shadow-amber-600/20">
            <CornerUpRight class="w-4 h-4" /> Add Redirect
          </button>
          <button v-if="activeTab === 'aliases' && selectedDomainId" @click="showAddAliasModal = true" class="btn-primary bg-purple-600 hover:bg-purple-700 shadow-purple-600/20">
            <Layers class="w-4 h-4" /> Add Alias
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex flex-wrap gap-1 bg-card border border-border p-1 rounded-xl">
        <button 
          v-for="tab in [
            { id: 'domains', label: 'Domains', icon: Globe },
            { id: 'subdomains', label: 'Subdomains', icon: Server },
            { id: 'aliases', label: 'Aliases', icon: Layers },
            { id: 'redirects', label: 'Redirects', icon: CornerUpRight },
            { id: 'ssl', label: 'SSL/TLS', icon: Lock },
            { id: 'dns', label: 'DNS Zone', icon: FileText }
          ]"
          :key="tab.id"
          @click="activeTab = tab.id as any"
          :class="[
            'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all',
            activeTab === tab.id ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted'
          ]"
        >
          <component :is="tab.icon" class="w-4 h-4" />
          {{ tab.label }}
        </button>
      </div>

      <!-- Domain Selector -->
      <div v-if="activeTab !== 'domains' && domains.length > 0" class="bg-card border border-border p-4 rounded-xl flex items-center gap-4">
        <label class="text-sm font-medium text-foreground whitespace-nowrap">Managing Domain:</label>
        <div class="relative flex-1 max-w-md">
          <select v-model="selectedDomainId" class="w-full bg-muted border border-border rounded-lg px-4 py-2.5 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none appearance-none">
            <option v-for="domain in domains" :key="domain.id" :value="domain.id">{{ domain.domain_name }}</option>
          </select>
          <ChevronDown class="w-4 h-4 text-muted-foreground absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none" />
        </div>
      </div>

      <!-- Content Area -->
      <div class="bg-card border border-border rounded-2xl overflow-hidden min-h-[400px]">
        <!-- Loading -->
        <div v-if="isLoading" class="p-20 flex flex-col items-center justify-center space-y-4">
          <div class="w-10 h-10 border-4 border-border border-t-primary rounded-full animate-spin" />
          <p class="text-muted-foreground font-medium text-xs uppercase tracking-wider">Loading...</p>
        </div>

        <!-- Error -->
        <div v-else-if="error && !domains.length" class="p-20 text-center">
          <AlertCircle class="w-16 h-16 text-destructive/30 mx-auto mb-6" />
          <h3>Error</h3>
          <p class="text-muted-foreground mt-2">{{ error }}</p>
          <button @click="fetchDomains" class="mt-4 btn-secondary">Retry</button>
        </div>

        <div v-else>
          <!-- Domains Tab -->
          <div v-if="activeTab === 'domains'">
             <div v-if="domains.length === 0" class="empty-state">
              <Globe class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold">No domains</h3>
              <p class="text-muted-foreground">Add your first domain to get started.</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="table-header">
                  <th class="px-6 py-4">Domain</th>
                  <th class="px-6 py-4">Root</th>
                  <th class="px-6 py-4">SSL</th>
                  <th class="px-6 py-4">Status</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="domain in domains" :key="domain.id" class="group hover:bg-muted/50 transition-all">
                  <td class="px-6 py-4 font-medium">{{ domain.domain_name }}</td>
                  <td class="px-6 py-4 text-sm text-muted-foreground">{{ domain.document_root }}</td>
                  <td class="px-6 py-4">
                    <span :class="['badge', domain.ssl_enabled ? 'badge-success' : 'badge-muted']">
                      {{ domain.ssl_enabled ? 'Active' : 'No SSL' }}
                    </span>
                  </td>
                  <td class="px-6 py-4">
                    <span :class="['badge', domain.is_active ? 'badge-success' : 'badge-danger']">
                      {{ domain.is_active ? 'Active' : 'Inactive' }}
                    </span>
                  </td>
                  <td class="px-6 py-4 text-right">
                    <button @click="openDeleteDomain(domain)" class="icon-btn-danger group-hover:opacity-100 opacity-0"><Trash2 class="w-4 h-4" /></button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Subdomains Tab -->
          <div v-else-if="activeTab === 'subdomains'">
            <div v-if="subdomains.length === 0" class="empty-state">
              <Server class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3>No subdomains</h3>
              <p>Create subdomains like blog.{{ selectedDomainObj?.domain_name }}</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="table-header">
                  <th class="px-6 py-4">Subdomain</th>
                  <th class="px-6 py-4">Root</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="sub in subdomains" :key="sub.id" class="group hover:bg-muted/50">
                  <td class="px-6 py-4 font-medium">{{ sub.subdomain_name }}.{{ selectedDomainObj?.domain_name }}</td>
                  <td class="px-6 py-4 text-sm text-muted-foreground">{{ sub.document_root }}</td>
                  <td class="px-6 py-4 text-right">
                    <button @click="openDeleteSubdomain(sub)" class="icon-btn-danger group-hover:opacity-100 opacity-0"><Trash2 class="w-4 h-4" /></button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Redirects Tab -->
          <div v-else-if="activeTab === 'redirects'">
            <div v-if="redirects.length === 0" class="empty-state">
              <CornerUpRight class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3>No redirects</h3>
              <p>Create URL redirects (301/302).</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="table-header">
                  <th class="px-6 py-4">Source Path</th>
                  <th class="px-6 py-4">Target URL</th>
                  <th class="px-6 py-4">Type</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="red in redirects" :key="red.id" class="group hover:bg-muted/50">
                  <td class="px-6 py-4 font-mono text-sm">{{ red.source_path }}</td>
                  <td class="px-6 py-4 text-sm text-blue-500">{{ red.destination_url }}</td>
                  <td class="px-6 py-4">
                    <span class="badge badge-muted">{{ red.type }}</span>
                  </td>
                  <td class="px-6 py-4 text-right">
                    <button @click="openDeleteRedirect(red)" class="icon-btn-danger group-hover:opacity-100 opacity-0"><Trash2 class="w-4 h-4" /></button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Aliases Tab -->
          <div v-else-if="activeTab === 'aliases'">
            <div v-if="aliases.length === 0" class="empty-state">
              <Layers class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3>No aliases</h3>
              <p>Park other domains on top of {{ selectedDomainObj?.domain_name }}</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="table-header">
                  <th class="px-6 py-4">Alias Domain</th>
                  <th class="px-6 py-4">Target</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="alias in aliases" :key="alias.id" class="group hover:bg-muted/50">
                  <td class="px-6 py-4 font-medium">{{ alias.alias_domain }}</td>
                  <td class="px-6 py-4 text-sm text-muted-foreground">Points to {{ selectedDomainObj?.domain_name }}</td>
                  <td class="px-6 py-4 text-right">
                    <button @click="openDeleteAlias(alias)" class="icon-btn-danger group-hover:opacity-100 opacity-0"><Trash2 class="w-4 h-4" /></button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- DNS Tab -->
          <div v-else-if="activeTab === 'dns'">
            <div v-if="dnsRecords.length === 0" class="empty-state">
              <FileText class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3>No DNS records</h3>
              <p>Add A, CNAME, MX records and more.</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="table-header">
                  <th class="px-6 py-4">Type</th>
                  <th class="px-6 py-4">Name</th>
                  <th class="px-6 py-4">Value</th>
                  <th class="px-6 py-4">TTL</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="record in dnsRecords" :key="record.id" class="group hover:bg-muted/50">
                  <td class="px-6 py-4"><span class="badge badge-muted">{{ record.record_type }}</span></td>
                  <td class="px-6 py-4 text-sm">{{ record.name }}</td>
                  <td class="px-6 py-4 text-sm font-mono truncate max-w-[200px]">{{ record.value }}</td>
                  <td class="px-6 py-4 text-xs text-muted-foreground">{{ record.ttl }}s</td>
                  <td class="px-6 py-4 text-right flex justify-end gap-1">
                    <button @click="openEditDns(record)" class="icon-btn opacity-0 group-hover:opacity-100"><Edit class="w-4 h-4" /></button>
                    <button @click="openDeleteDns(record)" class="icon-btn-danger opacity-0 group-hover:opacity-100"><Trash2 class="w-4 h-4" /></button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- SSL Tab -->
          <div v-else-if="activeTab === 'ssl'">
            <div class="p-6">
               <div class="bg-muted/50 border border-border rounded-xl p-6">
                <div class="flex items-center justify-between mb-4">
                  <div class="flex items-center gap-3">
                    <div :class="['w-12 h-12 rounded-xl flex items-center justify-center', selectedDomainObj?.ssl_enabled ? 'bg-emerald-500/10 text-emerald-600' : 'bg-amber-500/10 text-amber-600']">
                      <Lock class="w-6 h-6" />
                    </div>
                    <div>
                      <h3 class="text-lg font-semibold text-foreground">{{ selectedDomainObj?.domain_name }}</h3>
                      <p class="text-sm text-muted-foreground">SSL Certificate Status</p>
                    </div>
                  </div>
                  <div :class="['badge', selectedDomainObj?.ssl_enabled ? 'badge-success' : 'badge-warning']">
                    {{ selectedDomainObj?.ssl_enabled ? 'SSL Active' : 'No SSL' }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <!-- Add Domain -->
    <div v-if="showAddDomainModal" class="modal-overlay" @click.self="showAddDomainModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Add Domain</h3>
          <button @click="showAddDomainModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="form-label">Domain Name</label>
            <input v-model="newDomainName" type="text" placeholder="example.com" class="form-input" />
          </div>
          <div>
            <label class="form-label">Document Root (Optional)</label>
            <input v-model="newDocumentRoot" type="text" placeholder="/public_html" class="form-input" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddDomainModal = false" class="btn-ghost">Cancel</button>
          <button @click="createDomain" class="btn-primary">Add Domain</button>
        </div>
      </div>
    </div>

    <!-- Add Alias Modal -->
    <div v-if="showAddAliasModal" class="modal-overlay" @click.self="showAddAliasModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Add Parked Domain (Alias)</h3>
          <button @click="showAddAliasModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="form-label">Alias Domain Name</label>
            <input v-model="newAliasDomain" type="text" placeholder="example.net" class="form-input" />
            <p class="text-xs text-muted-foreground mt-1">This domain will point to <strong>{{ selectedDomainObj?.domain_name }}</strong></p>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddAliasModal = false" class="btn-ghost">Cancel</button>
          <button @click="createAlias" class="btn-primary">Add Alias</button>
        </div>
      </div>
    </div>

    <!-- Add Redirect Modal -->
    <div v-if="showAddRedirectModal" class="modal-overlay" @click.self="showAddRedirectModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Add Redirect</h3>
          <button @click="showAddRedirectModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="form-label">Type</label>
            <select v-model="newRedirectType" class="form-input">
              <option value="301">Permanent (301)</option>
              <option value="302">Temporary (302)</option>
            </select>
          </div>
          <div>
            <label class="form-label">Source Path</label>
             <div class="flex items-center gap-2">
              <span class="text-sm text-muted-foreground">/</span>
              <input v-model="newRedirectPath" type="text" placeholder="old-page" class="form-input" />
            </div>
          </div>
          <div>
            <label class="form-label">Target URL</label>
            <input v-model="newRedirectDest" type="text" placeholder="https://google.com" class="form-input" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddRedirectModal = false" class="btn-ghost">Cancel</button>
          <button @click="createRedirect" class="btn-primary">Add Redirect</button>
        </div>
      </div>
    </div>

    <!-- Confirms (Reusable style) -->
    <div v-if="showDeleteDomainModal" class="modal-overlay" @click.self="showDeleteDomainModal = false">
      <div class="modal-content animate-in">
        <h3>Delete Domain?</h3>
        <div class="modal-footer">
          <button @click="showDeleteDomainModal = false" class="btn-ghost">Cancel</button>
          <button @click="deleteDomain" class="btn-destroy">Delete</button>
        </div>
      </div>
    </div>

    <!-- Add Subdomain Modal -->
    <div v-if="showAddSubdomainModal" class="modal-overlay" @click.self="showAddSubdomainModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Add Subdomain</h3>
          <button @click="showAddSubdomainModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="form-label">Subdomain</label>
            <div class="flex items-center gap-2">
              <input v-model="newSubdomainName" type="text" placeholder="blog" class="form-input flex-1" />
              <span class="text-sm text-muted-foreground">.{{ selectedDomainObj?.domain_name }}</span>
            </div>
          </div>
          <div>
            <label class="form-label">Document Root</label>
            <input v-model="newSubdomainDocRoot" type="text" placeholder="/blog" class="form-input" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddSubdomainModal = false" class="btn-ghost">Cancel</button>
          <button @click="createSubdomain" class="btn-primary">Create</button>
        </div>
      </div>
    </div>

    <!-- Add DNS Modal -->
    <div v-if="showAddDnsModal" class="modal-overlay" @click.self="showAddDnsModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Add DNS Record</h3>
          <button @click="showAddDnsModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="form-label">Type</label>
              <select v-model="newDnsType" class="form-input">
                <option v-for="t in dnsTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
              </select>
            </div>
            <div>
              <label class="form-label">TTL</label>
               <input v-model="newDnsTtl" type="number" class="form-input" />
            </div>
          </div>
          <div>
            <label class="form-label">Name</label>
            <input v-model="newDnsName" type="text" placeholder="@" class="form-input" />
          </div>
           <div>
            <label class="form-label">Value</label>
            <input v-model="newDnsValue" type="text" class="form-input" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showAddDnsModal = false" class="btn-ghost">Cancel</button>
          <button @click="createDnsRecord" class="btn-primary">Add Record</button>
        </div>
      </div>
    </div>

    <!-- Delete Subdomain -->
    <div v-if="showDeleteSubdomainModal" class="modal-overlay" @click.self="showDeleteSubdomainModal = false">
      <div class="modal-content animate-in">
        <h3>Delete Subdomain?</h3>
        <p class="text-sm text-muted-foreground mt-2">Are you sure you want to delete {{ selectedSubdomain?.subdomain_name }}?</p>
        <div class="modal-footer">
          <button @click="showDeleteSubdomainModal = false" class="btn-ghost">Cancel</button>
          <button @click="deleteSubdomain" class="btn-destroy">Delete</button>
        </div>
      </div>
    </div>

    <!-- Delete DNS -->
    <div v-if="showDeleteDnsModal" class="modal-overlay" @click.self="showDeleteDnsModal = false">
      <div class="modal-content animate-in">
        <h3>Delete DNS Record?</h3>
         <p class="text-sm text-muted-foreground mt-2">Delete {{ selectedDnsRecord?.name }} ({{ selectedDnsRecord?.record_type }})?</p>
        <div class="modal-footer">
          <button @click="showDeleteDnsModal = false" class="btn-ghost">Cancel</button>
          <button @click="deleteDnsRecord" class="btn-destroy">Delete</button>
        </div>
      </div>
    </div>

    <!-- Edit DNS -->
    <div v-if="showEditDnsModal" class="modal-overlay" @click.self="showEditDnsModal = false">
      <div class="modal-content animate-in">
         <div class="modal-header">
          <h3>Edit DNS Record</h3>
          <button @click="showEditDnsModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="space-y-4">
           <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="form-label">Type</label>
              <select v-model="newDnsType" class="form-input" disabled>
                <option v-for="t in dnsTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
              </select>
            </div>
            <div>
              <label class="form-label">TTL</label>
               <input v-model="newDnsTtl" type="number" class="form-input" />
            </div>
          </div>
          <div>
            <label class="form-label">Name</label>
            <input v-model="newDnsName" type="text" class="form-input" disabled />
          </div>
           <div>
            <label class="form-label">Value</label>
            <input v-model="newDnsValue" type="text" class="form-input" />
          </div>
        </div>
        <div class="modal-footer">
          <button @click="showEditDnsModal = false" class="btn-ghost">Cancel</button>
          <button @click="updateDnsRecord" class="btn-primary">Save Changes</button>
        </div>
      </div>
    </div>

    <!-- Delete Redirect -->
    <div v-if="showDeleteRedirectModal" class="modal-overlay" @click.self="showDeleteRedirectModal = false">
      <div class="modal-content animate-in">
        <h3>Delete Redirect?</h3>
        <div class="modal-footer">
          <button @click="showDeleteRedirectModal = false" class="btn-ghost">Cancel</button>
          <button @click="deleteRedirect" class="btn-destroy">Delete</button>
        </div>
      </div>
    </div>

    <!-- Delete Alias -->
    <div v-if="showDeleteAliasModal" class="modal-overlay" @click.self="showDeleteAliasModal = false">
      <div class="modal-content animate-in">
        <h3>Delete Alias?</h3>
        <div class="modal-footer">
          <button @click="showDeleteAliasModal = false" class="btn-ghost">Cancel</button>
          <button @click="deleteAlias" class="btn-destroy">Delete</button>
        </div>
      </div>
    </div>

    <!-- Request SSL -->
     <div v-if="showRequestSslModal" class="modal-overlay" @click.self="showRequestSslModal = false">
      <div class="modal-content animate-in">
        <div class="modal-header">
          <h3>Request SSL Certificate</h3>
          <button @click="showRequestSslModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <p class="text-sm text-foreground mb-4">Install specific SSL for <strong>{{ selectedDomainObj?.domain_name }}</strong>?</p>
        <div class="bg-muted p-4 rounded-lg mb-4">
            <p class="text-xs text-muted-foreground">Provider: <strong>Let's Encrypt (Free)</strong></p>
            <p class="text-xs text-muted-foreground mt-1">Validity: <strong>90 Days (Auto-renew)</strong></p>
        </div>
        <div class="modal-footer">
          <button @click="showRequestSslModal = false" class="btn-ghost">Cancel</button>
          <button @click="showRequestSslModal = false; showSuccess('SSL Requested (Mock)')" class="btn-primary bg-emerald-600 hover:bg-emerald-700">Install SSL</button>
        </div>
      </div>
    </div>

  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeIn 0.3s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

/* Utility Classes for clean template */
.btn-primary { @apply flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-all shadow-lg shadow-primary/20 active:scale-[0.98] text-sm; }
.btn-secondary { @apply px-5 py-2.5 bg-muted hover:bg-muted/80 text-foreground rounded-xl font-medium transition-all text-sm; }
.btn-ghost { @apply px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors; }
.btn-destroy { @apply px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors; }
.icon-btn { @apply p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all; }
.icon-btn-danger { @apply p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all; }

.modal-overlay { @apply fixed inset-0 bg-black/50 flex items-center justify-center z-50; }
.modal-content { @apply bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6; }
.modal-header { @apply flex items-center justify-between mb-6; }
.modal-header h3 { @apply text-lg font-semibold text-foreground; }
.modal-footer { @apply flex justify-end gap-3 mt-6; }

.form-label { @apply block text-sm font-medium text-foreground mb-2; }
.form-input { @apply w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none; }

.empty-state { @apply p-20 text-center; }
.empty-state h3 { @apply text-lg font-bold text-foreground; }
.empty-state p { @apply text-muted-foreground mt-2 text-sm; }

.table-header { @apply bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider; }
.badge { @apply inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-semibold uppercase; }
.badge-success { @apply bg-emerald-500/10 text-emerald-600 border border-emerald-500/20; }
.badge-muted { @apply bg-muted text-muted-foreground border border-border; }
.badge-warning { @apply bg-amber-500/10 text-amber-600 border border-amber-500/20; }
.badge-danger { @apply bg-destructive/10 text-destructive border border-destructive/20; }
</style>
