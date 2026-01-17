<script setup lang="ts">
/**
 * EmailPage - Halaman manajemen email (cPanel-like)
 * 
 * Features:
 * - Email Accounts (CRUD, quota, password)
 * - Forwarders
 * - Autoresponders
 * - Webmail access
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { 
  Mail, Plus, Trash2, AlertCircle, X, Check,
  Forward, RefreshCcw, HardDrive, ExternalLink,
  User, Settings, Key, Eye, EyeOff,
  Calendar, MessageSquare, Inbox, Send
} from 'lucide-vue-next'
import { emailService, domainService } from '@/services'

// ==========================================
// STATE
// ==========================================
const accounts = ref<any[]>([])
const forwarders = ref<any[]>([])
const autoresponders = ref<any[]>([])
const domains = ref<any[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)
const activeTab = ref<'accounts' | 'forwarders' | 'autoresponders' | 'webmail'>('accounts')

// Modal states
const showAddAccountModal = ref(false)
const showEditAccountModal = ref(false)
const showDeleteAccountModal = ref(false)
const showAddForwarderModal = ref(false)
const showDeleteForwarderModal = ref(false)
const showAddAutoresponderModal = ref(false)
const showDeleteAutoresponderModal = ref(false)
const showChangePasswordModal = ref(false)

// Form data - Account
const selectedDomainId = ref('')
const newUsername = ref('')
const newPassword = ref('')
const showPassword = ref(false)
const newQuotaMb = ref(1024)

// Form data - Forwarder
const forwarderSource = ref('')
const forwarderDest = ref('')
const fwdDomainId = ref('')

// Form data - Autoresponder
const arAccountId = ref('')
const arSubject = ref('')
const arBody = ref('')
const arStartDate = ref('')
const arEndDate = ref('')

// Selected items
const selectedAccount = ref<any>(null)
const selectedForwarder = ref<any>(null)
const selectedAutoresponder = ref<any>(null)

// Quota options
const quotaOptions = [
  { value: 256, label: '256 MB' },
  { value: 512, label: '512 MB' },
  { value: 1024, label: '1 GB' },
  { value: 2048, label: '2 GB' },
  { value: 5120, label: '5 GB' },
  { value: 10240, label: '10 GB' },
  { value: 0, label: 'Unlimited' },
]

// Stats
const stats = computed(() => ({
  totalAccounts: accounts.value.length,
  usedQuota: accounts.value.reduce((sum, a) => sum + (a.used_mb || 0), 0),
  totalQuota: accounts.value.reduce((sum, a) => sum + (a.quota_mb || 0), 0),
  activeForwarders: forwarders.value.length,
  activeAutoresponders: autoresponders.value.filter(a => a.is_active).length
}))

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Fetch all data
 */
const fetchData = async (): Promise<void> => {
  try {
    isLoading.value = true
    error.value = null
    const [accRes, fwdRes, arRes, domRes] = await Promise.all([
      emailService.listAccounts(),
      emailService.listForwarders(),
      emailService.listAutoresponders(),
      domainService.listDomains(1, 50)
    ])
    accounts.value = accRes.data.data || []
    forwarders.value = fwdRes.data.data || []
    autoresponders.value = arRes.data.data || []
    domains.value = domRes.data.data?.items || []
    if (domains.value.length > 0 && !selectedDomainId.value) {
      selectedDomainId.value = domains.value[0]?.id || ''
      fwdDomainId.value = domains.value[0]?.id || ''
    }
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat data email'
  } finally {
    isLoading.value = false
  }
}

/**
 * Show success message
 */
const showSuccess = (msg: string): void => {
  successMsg.value = msg
  setTimeout(() => successMsg.value = null, 3000)
}

/**
 * Create email account
 */
const createAccount = async (): Promise<void> => {
  if (!newUsername.value.trim() || !newPassword.value || !selectedDomainId.value) {
    error.value = 'Semua field harus diisi'
    return
  }
  try {
    isLoading.value = true
    await emailService.createAccount({
      domain_id: selectedDomainId.value,
      username: newUsername.value.trim(),
      password: newPassword.value,
      quota_mb: newQuotaMb.value
    })
    showAddAccountModal.value = false
    resetAccountForm()
    showSuccess('Akun email berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat akun email'
  } finally {
    isLoading.value = false
  }
}

/**
 * Update account quota
 */
const updateAccountQuota = async (): Promise<void> => {
  if (!selectedAccount.value) return
  try {
    isLoading.value = true
    await emailService.updateAccount(selectedAccount.value.id, {
      quota_mb: newQuotaMb.value
    })
    showEditAccountModal.value = false
    selectedAccount.value = null
    showSuccess('Quota berhasil diupdate')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal update quota'
  } finally {
    isLoading.value = false
  }
}

/**
 * Change password
 */
const changePassword = async (): Promise<void> => {
  if (!selectedAccount.value || !newPassword.value) return
  if (newPassword.value.length < 8) {
    error.value = 'Password minimal 8 karakter'
    return
  }
  try {
    isLoading.value = true
    error.value = null
    await emailService.updateAccount(selectedAccount.value.id, {
      password: newPassword.value
    })
    showChangePasswordModal.value = false
    selectedAccount.value = null
    newPassword.value = ''
    showSuccess('Password berhasil diubah')
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal mengubah password'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete account
 */
const deleteAccount = async (): Promise<void> => {
  if (!selectedAccount.value) return
  try {
    isLoading.value = true
    await emailService.deleteAccount(selectedAccount.value.id)
    showDeleteAccountModal.value = false
    selectedAccount.value = null
    showSuccess('Akun email berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus akun'
  } finally {
    isLoading.value = false
  }
}

/**
 * Create forwarder
 */
const createForwarder = async (): Promise<void> => {
  if (!forwarderSource.value.trim() || !forwarderDest.value.trim() || !fwdDomainId.value) {
    error.value = 'Semua field harus diisi'
    return
  }
  try {
    isLoading.value = true
    await emailService.createForwarder({
      domain_id: fwdDomainId.value,
      source_username: forwarderSource.value.trim(),
      destination_email: forwarderDest.value.trim()
    })
    showAddForwarderModal.value = false
    forwarderSource.value = ''
    forwarderDest.value = ''
    showSuccess('Forwarder berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat forwarder'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete forwarder
 */
const deleteForwarder = async (): Promise<void> => {
  if (!selectedForwarder.value) return
  try {
    isLoading.value = true
    await emailService.deleteForwarder(selectedForwarder.value.id)
    showDeleteForwarderModal.value = false
    selectedForwarder.value = null
    showSuccess('Forwarder berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus forwarder'
  } finally {
    isLoading.value = false
  }
}

/**
 * Create autoresponder
 */
const createAutoresponder = async (): Promise<void> => {
  if (!arAccountId.value || !arSubject.value.trim() || !arBody.value.trim()) {
    error.value = 'Subject dan body harus diisi'
    return
  }
  try {
    isLoading.value = true
    await emailService.createAutoresponder({
      email_account_id: arAccountId.value,
      subject: arSubject.value.trim(),
      body: arBody.value.trim(),
      start_date: arStartDate.value || undefined,
      end_date: arEndDate.value || undefined
    })
    showAddAutoresponderModal.value = false
    resetAutoresponderForm()
    showSuccess('Autoresponder berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat autoresponder'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete autoresponder
 */
const deleteAutoresponder = async (): Promise<void> => {
  if (!selectedAutoresponder.value) return
  try {
    isLoading.value = true
    await emailService.deleteAutoresponder(selectedAutoresponder.value.id)
    showDeleteAutoresponderModal.value = false
    selectedAutoresponder.value = null
    showSuccess('Autoresponder berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus autoresponder'
  } finally {
    isLoading.value = false
  }
}

/**
 * Reset forms
 */
const resetAccountForm = (): void => {
  newUsername.value = ''
  newPassword.value = ''
  newQuotaMb.value = 1024
  showPassword.value = false
}

const resetAutoresponderForm = (): void => {
  arAccountId.value = ''
  arSubject.value = ''
  arBody.value = ''
  arStartDate.value = ''
  arEndDate.value = ''
}

/**
 * Generate password
 */
const generatePassword = (): void => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&'
  let pass = ''
  for (let i = 0; i < 16; i++) {
    pass += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  newPassword.value = pass
  showPassword.value = true
}

/**
 * Format bytes
 */
const formatMb = (mb: number): string => {
  if (mb === 0) return 'Unlimited'
  if (mb < 1024) return mb + ' MB'
  return (mb / 1024).toFixed(1) + ' GB'
}

// ==========================================
// MODAL OPENERS
// ==========================================

const openEditAccount = (account: any): void => {
  selectedAccount.value = account
  newQuotaMb.value = account.quota_mb || 1024
  showEditAccountModal.value = true
}

const openChangePassword = (account: any): void => {
  selectedAccount.value = account
  newPassword.value = ''
  showChangePasswordModal.value = true
}

const openDeleteAccount = (account: any): void => {
  selectedAccount.value = account
  showDeleteAccountModal.value = true
}

const openDeleteForwarder = (fwd: any): void => {
  selectedForwarder.value = fwd
  showDeleteForwarderModal.value = true
}

const openDeleteAutoresponder = (ar: any): void => {
  selectedAutoresponder.value = ar
  showDeleteAutoresponderModal.value = true
}

onMounted(() => {
  fetchData()
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
              <Mail class="w-5 h-5 text-primary" />
            </div>
            Email Manager
          </h1>
          <p class="text-muted-foreground mt-1 text-sm">Manage email accounts, forwarders, and autoresponders.</p>
        </div>
        
        <div class="flex gap-2">
          <button 
            v-if="activeTab === 'accounts'"
            @click="showAddAccountModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-all shadow-lg shadow-primary/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Add Account
          </button>
          <button 
            v-if="activeTab === 'forwarders'"
            @click="showAddForwarderModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-indigo-600/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Add Forwarder
          </button>
          <button 
            v-if="activeTab === 'autoresponders'"
            @click="showAddAutoresponderModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-amber-600 hover:bg-amber-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-amber-600/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Add Autoresponder
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex flex-wrap gap-1 bg-card border border-border p-1 rounded-xl">
        <button 
          v-for="tab in [
            { id: 'accounts', label: 'Email Accounts', icon: Inbox },
            { id: 'forwarders', label: 'Forwarders', icon: Forward },
            { id: 'autoresponders', label: 'Autoresponders', icon: RefreshCcw },
            { id: 'webmail', label: 'Webmail', icon: ExternalLink }
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

      <!-- Stats -->
      <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
        <div v-for="stat in [
          { label: 'Accounts', val: stats.totalAccounts, icon: User, color: 'text-primary' },
          { label: 'Used', val: formatMb(stats.usedQuota), icon: HardDrive, color: 'text-blue-500' },
          { label: 'Quota', val: formatMb(stats.totalQuota), icon: HardDrive, color: 'text-emerald-500' },
          { label: 'Forwarders', val: stats.activeForwarders, icon: Forward, color: 'text-indigo-500' },
          { label: 'Autoresponders', val: stats.activeAutoresponders, icon: RefreshCcw, color: 'text-amber-500' }
        ]" :key="stat.label" class="bg-card border border-border p-4 rounded-xl">
          <div class="flex items-center gap-2 mb-2">
            <component :is="stat.icon" class="w-4 h-4" :class="stat.color" />
            <p class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">{{ stat.label }}</p>
          </div>
          <p class="text-xl font-bold text-foreground">{{ stat.val }}</p>
        </div>
      </div>

      <!-- Content -->
      <div class="bg-card border border-border rounded-2xl overflow-hidden min-h-[400px]">
        <!-- Loading -->
        <div v-if="isLoading" class="p-20 flex flex-col items-center justify-center space-y-4">
          <div class="w-10 h-10 border-4 border-border border-t-primary rounded-full animate-spin" />
          <p class="text-muted-foreground font-medium text-xs uppercase tracking-wider">Loading...</p>
        </div>

        <!-- Error -->
        <div v-else-if="error && !accounts.length" class="p-20 text-center">
          <AlertCircle class="w-16 h-16 text-destructive/30 mx-auto mb-6" />
          <h3 class="text-lg font-bold text-foreground">Error</h3>
          <p class="text-muted-foreground mt-2 text-sm">{{ error }}</p>
          <button @click="fetchData" class="mt-6 px-6 py-2.5 bg-foreground text-background rounded-lg font-medium">Retry</button>
        </div>

        <!-- Accounts Tab -->
        <div v-else-if="activeTab === 'accounts'">
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">Email Accounts</span>
            <button @click="showAddAccountModal = true" class="text-xs text-primary hover:underline">+ Add New</button>
          </div>
          <div v-if="accounts.length === 0" class="p-20 text-center">
            <Inbox class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
            <h3 class="text-lg font-bold text-foreground">No email accounts</h3>
            <p class="text-muted-foreground mt-2 text-sm">Create your first email account.</p>
          </div>
          <table v-else class="w-full text-left">
            <thead>
              <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                <th class="px-6 py-4">Email</th>
                <th class="px-6 py-4">Quota</th>
                <th class="px-6 py-4">Used</th>
                <th class="px-6 py-4">Status</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr v-for="account in accounts" :key="account.id" class="group hover:bg-muted/50 transition-all">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-primary/10 rounded-xl flex items-center justify-center text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-all">
                      <Mail class="w-5 h-5" />
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-foreground">{{ account.email_address }}</p>
                      <p class="text-[10px] text-muted-foreground mt-0.5">Created {{ account.created_at?.split('T')[0] }}</p>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <span class="text-sm font-medium text-foreground">{{ formatMb(account.quota_mb) }}</span>
                </td>
                <td class="px-6 py-4">
                  <div class="w-24">
                    <div class="flex justify-between text-[10px] mb-1">
                      <span class="text-muted-foreground">{{ formatMb(account.used_mb || 0) }}</span>
                      <span class="text-muted-foreground">{{ account.quota_mb > 0 ? Math.round((account.used_mb || 0) / account.quota_mb * 100) : 0 }}%</span>
                    </div>
                    <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                      <div class="h-full bg-primary rounded-full transition-all" :style="{ width: (account.quota_mb > 0 ? Math.min((account.used_mb || 0) / account.quota_mb * 100, 100) : 0) + '%' }" />
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <div :class="['inline-flex items-center gap-1 px-2 py-1 rounded-lg text-[10px] font-semibold uppercase', account.is_active ? 'bg-emerald-500/10 text-emerald-600 border border-emerald-500/20' : 'bg-destructive/10 text-destructive border border-destructive/20']">
                    <span class="w-1.5 h-1.5 rounded-full" :class="account.is_active ? 'bg-emerald-500' : 'bg-destructive'" />
                    {{ account.is_active ? 'Active' : 'Disabled' }}
                  </div>
                </td>
                <td class="px-6 py-4 text-right">
                  <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-all">
                    <a href="https://webmail.example.com" target="_blank" class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Webmail">
                      <ExternalLink class="w-4 h-4" />
                    </a>
                    <button @click="openChangePassword(account)" class="p-2 text-muted-foreground hover:text-emerald-600 hover:bg-emerald-500/10 rounded-lg transition-all" title="Change Password">
                      <Key class="w-4 h-4" />
                    </button>
                    <button @click="openEditAccount(account)" class="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-all" title="Edit Quota">
                      <Settings class="w-4 h-4" />
                    </button>
                    <button @click="openDeleteAccount(account)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all" title="Delete">
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Forwarders Tab -->
        <div v-else-if="activeTab === 'forwarders'">
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">Email Forwarders</span>
            <button @click="showAddForwarderModal = true" class="text-xs text-primary hover:underline">+ Add New</button>
          </div>
          <div v-if="forwarders.length === 0" class="p-20 text-center">
            <Forward class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
            <h3 class="text-lg font-bold text-foreground">No forwarders</h3>
            <p class="text-muted-foreground mt-2 text-sm">Forward emails to external addresses.</p>
          </div>
          <table v-else class="w-full text-left">
            <thead>
              <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                <th class="px-6 py-4">Source</th>
                <th class="px-6 py-4">Destination</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr v-for="fwd in forwarders" :key="fwd.id" class="group hover:bg-muted/50 transition-all">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-indigo-500/10 rounded-xl flex items-center justify-center text-indigo-600">
                      <Send class="w-5 h-5" />
                    </div>
                    <p class="text-sm font-semibold text-foreground">{{ fwd.source_email }}</p>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <div class="flex items-center gap-2">
                    <Forward class="w-4 h-4 text-muted-foreground" />
                    <p class="text-sm text-foreground">{{ fwd.destination_email }}</p>
                  </div>
                </td>
                <td class="px-6 py-4 text-right">
                  <button @click="openDeleteForwarder(fwd)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all opacity-0 group-hover:opacity-100">
                    <Trash2 class="w-4 h-4" />
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Autoresponders Tab -->
        <div v-else-if="activeTab === 'autoresponders'">
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">Autoresponders</span>
            <button @click="showAddAutoresponderModal = true" class="text-xs text-primary hover:underline">+ Add New</button>
          </div>
          <div v-if="autoresponders.length === 0" class="p-20 text-center">
            <RefreshCcw class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
            <h3 class="text-lg font-bold text-foreground">No autoresponders</h3>
            <p class="text-muted-foreground mt-2 text-sm">Set up vacation or out-of-office replies.</p>
          </div>
          <div v-else class="p-6 grid grid-cols-1 md:grid-cols-2 gap-4">
            <div v-for="ar in autoresponders" :key="ar.id" class="bg-muted/50 border border-border p-5 rounded-xl group hover:border-primary/30 transition-all">
              <div class="flex items-start justify-between mb-4">
                <div class="flex items-center gap-3">
                  <div :class="['w-10 h-10 rounded-xl flex items-center justify-center', ar.is_active ? 'bg-emerald-500/10 text-emerald-600' : 'bg-muted text-muted-foreground']">
                    <RefreshCcw class="w-5 h-5" />
                  </div>
                  <div>
                    <p class="text-sm font-semibold text-foreground">{{ ar.email_address }}</p>
                    <span :class="['text-[9px] font-semibold uppercase tracking-wider', ar.is_active ? 'text-emerald-600' : 'text-muted-foreground']">
                      {{ ar.is_active ? 'Active' : 'Inactive' }}
                    </span>
                  </div>
                </div>
                <button @click="openDeleteAutoresponder(ar)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all opacity-0 group-hover:opacity-100">
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
              <div class="space-y-2">
                <div class="flex items-center gap-2 text-sm">
                  <MessageSquare class="w-4 h-4 text-muted-foreground" />
                  <span class="text-foreground font-medium">{{ ar.subject }}</span>
                </div>
                <div v-if="ar.start_date || ar.end_date" class="flex items-center gap-2 text-xs text-muted-foreground">
                  <Calendar class="w-3.5 h-3.5" />
                  {{ ar.start_date?.split('T')[0] || 'Now' }} → {{ ar.end_date?.split('T')[0] || 'No end' }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Webmail Tab -->
        <div v-else>
          <div class="p-4 border-b border-border">
            <span class="text-sm font-semibold text-foreground">Webmail Access</span>
          </div>
          <div class="p-6">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <a href="https://roundcube.example.com" target="_blank" class="bg-muted/50 border border-border p-6 rounded-xl hover:border-primary/30 transition-all group">
                <div class="w-12 h-12 bg-blue-500/10 rounded-xl flex items-center justify-center text-blue-600 mb-4 group-hover:bg-blue-500 group-hover:text-white transition-all">
                  <Mail class="w-6 h-6" />
                </div>
                <h3 class="text-lg font-semibold text-foreground">Roundcube</h3>
                <p class="text-sm text-muted-foreground mt-1">Modern webmail interface</p>
              </a>
              <a href="https://rainloop.example.com" target="_blank" class="bg-muted/50 border border-border p-6 rounded-xl hover:border-primary/30 transition-all group">
                <div class="w-12 h-12 bg-emerald-500/10 rounded-xl flex items-center justify-center text-emerald-600 mb-4 group-hover:bg-emerald-500 group-hover:text-white transition-all">
                  <Mail class="w-6 h-6" />
                </div>
                <h3 class="text-lg font-semibold text-foreground">RainLoop</h3>
                <p class="text-sm text-muted-foreground mt-1">Lightweight webmail</p>
              </a>
              <a href="https://squirrelmail.example.com" target="_blank" class="bg-muted/50 border border-border p-6 rounded-xl hover:border-primary/30 transition-all group">
                <div class="w-12 h-12 bg-amber-500/10 rounded-xl flex items-center justify-center text-amber-600 mb-4 group-hover:bg-amber-500 group-hover:text-white transition-all">
                  <Mail class="w-6 h-6" />
                </div>
                <h3 class="text-lg font-semibold text-foreground">SquirrelMail</h3>
                <p class="text-sm text-muted-foreground mt-1">Classic webmail</p>
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== MODALS ==================== -->

    <!-- Add Account Modal -->
    <div v-if="showAddAccountModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddAccountModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Create Email Account</h3>
          <button @click="showAddAccountModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4" />{{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Domain</label>
            <select v-model="selectedDomainId" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option v-for="domain in domains" :key="domain.id" :value="domain.id">{{ domain.domain_name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Username</label>
            <div class="flex items-center gap-2">
              <input v-model="newUsername" type="text" placeholder="user" class="flex-1 bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
              <span class="text-sm text-muted-foreground">@{{ domains.find(d => d.id === selectedDomainId)?.domain_name || 'domain' }}</span>
            </div>
          </div>
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-foreground">Password</label>
              <button @click="generatePassword" type="button" class="text-xs text-primary hover:underline">Generate</button>
            </div>
            <div class="relative">
              <input v-model="newPassword" :type="showPassword ? 'text' : 'password'" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 pr-12 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
              <button @click="showPassword = !showPassword" type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                <component :is="showPassword ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Quota</label>
            <select v-model="newQuotaMb" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option v-for="opt in quotaOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showAddAccountModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createAccount" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Create</button>
        </div>
      </div>
    </div>

    <!-- Edit Account Quota Modal -->
    <div v-if="showEditAccountModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showEditAccountModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Edit Quota</h3>
          <button @click="showEditAccountModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <p class="text-sm text-muted-foreground mb-4">Update quota for <span class="font-semibold text-foreground">{{ selectedAccount?.email_address }}</span></p>
        <div>
          <label class="block text-sm font-medium text-foreground mb-2">Quota</label>
          <select v-model="newQuotaMb" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
            <option v-for="opt in quotaOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showEditAccountModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="updateAccountQuota" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Save</button>
        </div>
      </div>
    </div>

    <!-- Change Password Modal -->
    <div v-if="showChangePasswordModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showChangePasswordModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Change Password</h3>
          <button @click="showChangePasswordModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4" />{{ error }}
        </div>
        <p class="text-sm text-muted-foreground mb-4">Change password for <span class="font-semibold text-foreground">{{ selectedAccount?.email_address }}</span></p>
        <div>
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium text-foreground">New Password</label>
            <button @click="generatePassword" type="button" class="text-xs text-primary hover:underline">Generate</button>
          </div>
          <div class="relative">
            <input v-model="newPassword" :type="showPassword ? 'text' : 'password'" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 pr-12 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
            <button @click="showPassword = !showPassword" type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
              <component :is="showPassword ? EyeOff : Eye" class="w-4 h-4" />
            </button>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showChangePasswordModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="changePassword" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Change</button>
        </div>
      </div>
    </div>

    <!-- Add Forwarder Modal -->
    <div v-if="showAddForwarderModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddForwarderModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Add Forwarder</h3>
          <button @click="showAddForwarderModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4" />{{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Domain</label>
            <select v-model="fwdDomainId" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option v-for="domain in domains" :key="domain.id" :value="domain.id">{{ domain.domain_name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Source</label>
            <div class="flex items-center gap-2">
              <input v-model="forwarderSource" type="text" placeholder="info" class="flex-1 bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
              <span class="text-sm text-muted-foreground">@{{ domains.find(d => d.id === fwdDomainId)?.domain_name || 'domain' }}</span>
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Forward To</label>
            <input v-model="forwarderDest" type="email" placeholder="external@gmail.com" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showAddForwarderModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createForwarder" class="px-4 py-2 text-sm font-medium bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors">Create</button>
        </div>
      </div>
    </div>

    <!-- Add Autoresponder Modal -->
    <div v-if="showAddAutoresponderModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddAutoresponderModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-lg p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Add Autoresponder</h3>
          <button @click="showAddAutoresponderModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4" />{{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Email Account</label>
            <select v-model="arAccountId" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="">-- Select Account --</option>
              <option v-for="acc in accounts" :key="acc.id" :value="acc.id">{{ acc.email_address }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Subject</label>
            <input v-model="arSubject" type="text" placeholder="Out of Office" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Message</label>
            <textarea v-model="arBody" rows="4" placeholder="I am currently out of office..." class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none resize-none" />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-foreground mb-2">Start Date</label>
              <input v-model="arStartDate" type="date" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
            </div>
            <div>
              <label class="block text-sm font-medium text-foreground mb-2">End Date</label>
              <input v-model="arEndDate" type="date" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showAddAutoresponderModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createAutoresponder" class="px-4 py-2 text-sm font-medium bg-amber-600 text-white rounded-lg hover:bg-amber-700 transition-colors">Create</button>
        </div>
      </div>
    </div>

    <!-- Delete Modals -->
    <div v-if="showDeleteAccountModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteAccountModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Email Account</h3>
        <p class="text-sm text-muted-foreground mb-6">Are you sure you want to delete <span class="font-semibold text-foreground">{{ selectedAccount?.email_address }}</span>? All emails will be permanently deleted.</p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteAccountModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteAccount" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
        </div>
      </div>
    </div>

    <div v-if="showDeleteForwarderModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteForwarderModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Forwarder</h3>
        <p class="text-sm text-muted-foreground mb-6">Delete forwarder from <span class="font-semibold text-foreground">{{ selectedForwarder?.source_email }}</span>?</p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteForwarderModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteForwarder" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
        </div>
      </div>
    </div>

    <div v-if="showDeleteAutoresponderModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteAutoresponderModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Autoresponder</h3>
        <p class="text-sm text-muted-foreground mb-6">Delete autoresponder for <span class="font-semibold text-foreground">{{ selectedAutoresponder?.email_address }}</span>?</p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteAutoresponderModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteAutoresponder" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in {
  animation: animate-in 0.4s ease-out;
}
@keyframes animate-in {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
