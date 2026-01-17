<script setup lang="ts">
/**
 * SystemPage - Halaman system tools (cPanel-like)
 * 
 * Features:
 * - Cron Jobs with presets
 * - Backups scheduling & restore
 * - PHP version selector
 * - Error logs viewer
 * - Service status
 */
import { ref, onMounted, reactive } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { 
  Settings, Clock, Save, HardDrive, 
  AlertCircle, Plus, Trash2, X, Check,
  Server, Download, Globe, Play,
  FileText, RefreshCw
} from 'lucide-vue-next'
import { systemService } from '@/services'

// ==========================================
// STATE
// ==========================================
const cronJobs = ref<any[]>([])
const backups = ref<any[]>([])
const services = ref<any[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)
const activeTab = ref<'cron' | 'backups' | 'php' | 'logs' | 'services'>('cron')

// Modal states
const showAddCronModal = ref(false)
const showDeleteCronModal = ref(false)
const showCreateBackupModal = ref(false)
const showDeleteBackupModal = ref(false)
const showScheduleBackupModal = ref(false)

// Form data - Cron
const cronPreset = ref('')
const cronSchedule = ref('0 * * * *')
const cronCommand = ref('')
const cronDescription = ref('')

// Form data - Backup
const backupType = ref<'full' | 'database' | 'homedir'>('full')
const backupDesc = ref('')
const backupSchedule = ref('daily')

// PHP version
const currentPhpVersion = ref('8.2')
const availablePhpVersions: string[] = reactive([])

// Error logs
const errorLogs = ref<string[]>([])

// Selected items
const selectedCron = ref<any>(null)
const selectedBackup = ref<any>(null)

// Cron presets
const cronPresets = [
  { label: 'Every minute', value: '* * * * *' },
  { label: 'Every 5 minutes', value: '*/5 * * * *' },
  { label: 'Every hour', value: '0 * * * *' },
  { label: 'Every 6 hours', value: '0 */6 * * *' },
  { label: 'Daily at midnight', value: '0 0 * * *' },
  { label: 'Daily at 3am', value: '0 3 * * *' },
  { label: 'Weekly (Sunday)', value: '0 0 * * 0' },
  { label: 'Monthly (1st)', value: '0 0 1 * *' },
  { label: 'Custom', value: '' }
]

// Backup schedule options
const backupScheduleOptions = [
  { value: 'manual', label: 'Manual only' },
  { value: 'daily', label: 'Daily at 3:00 AM' },
  { value: 'weekly', label: 'Weekly (Sunday)' },
  { value: 'monthly', label: 'Monthly (1st)' }
]

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Fetch data dari API
 */
const fetchData = async (): Promise<void> => {
  try {
    isLoading.value = true
    error.value = null
    // Load all data parallel
    const [cronRes, backRes, servRes, phpVerRes, currentPhpRes, logsRes] = await Promise.all([
      systemService.listCronJobs(),
      systemService.listBackups(),
      systemService.getServicesStatus().catch(() => ({ data: { data: [] } })),
      systemService.getPhpVersions().catch(() => ({ data: { data: ['8.2'] } })),
      systemService.getCurrentPhpVersion().catch(() => ({ data: { data: '8.2' } })),
      systemService.getErrorLogs().catch(() => ({ data: { data: [] } }))
    ])

    cronJobs.value = cronRes.data.data || []
    backups.value = backRes.data.data || []
    services.value = servRes.data.data || [] // @ts-ignore
    availablePhpVersions.splice(0, availablePhpVersions.length, ...phpVerRes.data.data) // @ts-ignore
    currentPhpVersion.value = currentPhpRes.data.data // @ts-ignore
    errorLogs.value = logsRes.data.data
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat data sistem'
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
 * Create cron job
 */
const createCronJob = async (): Promise<void> => {
  if (!cronCommand.value.trim()) {
    error.value = 'Command harus diisi'
    return
  }
  try {
    isLoading.value = true
    await systemService.createCronJob({
      schedule: cronSchedule.value,
      command: cronCommand.value.trim(),
      description: cronDescription.value.trim() || undefined
    })
    showAddCronModal.value = false
    resetCronForm()
    showSuccess('Cron job berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat cron job'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete cron job
 */
const deleteCronJob = async (): Promise<void> => {
  if (!selectedCron.value) return
  try {
    isLoading.value = true
    await systemService.deleteCronJob(selectedCron.value.id)
    showDeleteCronModal.value = false
    selectedCron.value = null
    showSuccess('Cron job berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus cron job'
  } finally {
    isLoading.value = false
  }
}

/**
 * Create backup
 */
const createBackup = async (): Promise<void> => {
  try {
    isLoading.value = true
    await systemService.createBackup({
      backup_type: backupType.value,
      description: backupDesc.value.trim() || undefined
    })
    showCreateBackupModal.value = false
    backupDesc.value = ''
    showSuccess('Backup sedang dibuat...')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat backup'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete backup
 */
const deleteBackup = async (): Promise<void> => {
  if (!selectedBackup.value) return
  try {
    isLoading.value = true
    await systemService.deleteBackup(selectedBackup.value.id)
    showDeleteBackupModal.value = false
    selectedBackup.value = null
    showSuccess('Backup berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus backup'
  } finally {
    isLoading.value = false
  }
}

/**
 * Change PHP version (mock)
 */
/**
 * Change PHP version
 */
const changePhpVersion = async (version: string): Promise<void> => {
  try {
    isLoading.value = true
    await systemService.changePhpVersion(version)
    currentPhpVersion.value = version
    showSuccess(`PHP version changed to ${version}`)
  } catch (err: any) {
    error.value = err.response?.data?.message || `Gagal mengubah versi PHP ke ${version}`
  } finally {
    isLoading.value = false
  }
}

/**
 * Clear error logs
 */
const clearLogs = async (): Promise<void> => {
  try {
    isLoading.value = true
    await systemService.clearErrorLogs()
    errorLogs.value = []
    showSuccess('Error logs cleared')
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus logs'
  } finally {
    isLoading.value = false
  }
}

/**
 * Reset forms
 */
const resetCronForm = (): void => {
  cronPreset.value = ''
  cronSchedule.value = '0 * * * *'
  cronCommand.value = ''
  cronDescription.value = ''
}

/**
 * Handle preset change
 */
const onPresetChange = (): void => {
  if (cronPreset.value) {
    cronSchedule.value = cronPreset.value
  }
}

/**
 * Format bytes
 */
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// Modal openers
const openDeleteCron = (cron: any): void => {
  selectedCron.value = cron
  showDeleteCronModal.value = true
}

const openDeleteBackup = (backup: any): void => {
  selectedBackup.value = backup
  showDeleteBackupModal.value = true
}

// State untuk tracking service yang sedang diproses
const serviceControlling = ref<string | null>(null)

/**
 * Control service (start/stop/restart)
 */
const controlService = async (serviceName: string, action: 'start' | 'stop' | 'restart'): Promise<void> => {
  try {
    serviceControlling.value = serviceName
    error.value = null
    await systemService.controlService(serviceName, action)
    showSuccess(`Service ${serviceName} berhasil di-${action}`)
    // Refresh service list
    const servRes = await systemService.getServicesStatus()
    services.value = servRes.data.data || []
  } catch (err: any) {
    error.value = err.response?.data?.message || `Gagal ${action} service ${serviceName}`
  } finally {
    serviceControlling.value = null
  }
}

/**
 * Refresh service status
 */
const refreshServices = async (): Promise<void> => {
  try {
    isLoading.value = true
    const servRes = await systemService.getServicesStatus()
    services.value = servRes.data.data || []
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat status services'
  } finally {
    isLoading.value = false
  }
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
              <Settings class="w-5 h-5 text-primary" />
            </div>
            System Tools
          </h1>
          <p class="text-muted-foreground mt-1 text-sm">Manage cron jobs, backups, PHP, and system services.</p>
        </div>
        
        <div class="flex gap-2">
          <button 
            v-if="activeTab === 'cron'"
            @click="showAddCronModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-all shadow-lg shadow-primary/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Add Cron Job
          </button>
          <button 
            v-if="activeTab === 'backups'"
            @click="showCreateBackupModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-emerald-600/20 active:scale-[0.98] text-sm"
          >
            <Save class="w-4 h-4" />
            Create Backup
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex flex-wrap gap-1 bg-card border border-border p-1 rounded-xl">
        <button 
          v-for="tab in [
            { id: 'cron', label: 'Cron Jobs', icon: Clock },
            { id: 'backups', label: 'Backups', icon: Save },
            { id: 'php', label: 'PHP Version', icon: Globe },
            { id: 'logs', label: 'Error Logs', icon: FileText },
            { id: 'services', label: 'Services', icon: Server }
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
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div v-for="stat in [
          { label: 'Cron Jobs', val: cronJobs.length, icon: Clock, color: 'text-primary' },
          { label: 'Backups', val: backups.length, icon: Save, color: 'text-emerald-500' },
          { label: 'PHP Version', val: currentPhpVersion, icon: Globe, color: 'text-blue-500' },
          { label: 'Storage Used', val: formatBytes(backups.reduce((acc, b) => acc + (b.size_bytes || 0), 0)), icon: HardDrive, color: 'text-purple-500' }
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
        <div v-else-if="error && !cronJobs.length" class="p-20 text-center">
          <AlertCircle class="w-16 h-16 text-destructive/30 mx-auto mb-6" />
          <h3 class="text-lg font-bold text-foreground">System Error</h3>
          <p class="text-muted-foreground mt-2 text-sm">{{ error }}</p>
          <button @click="fetchData" class="mt-6 px-6 py-2.5 bg-foreground text-background rounded-lg font-medium">Retry</button>
        </div>

        <div v-else>
          <!-- Cron Jobs Tab -->
          <div v-if="activeTab === 'cron'">
            <div class="p-4 border-b border-border flex items-center justify-between">
              <span class="text-sm font-semibold text-foreground">Scheduled Tasks</span>
              <button @click="showAddCronModal = true" class="text-xs text-primary hover:underline">+ Add New</button>
            </div>
            <div v-if="cronJobs.length === 0" class="p-20 text-center">
              <Clock class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold text-foreground">No cron jobs</h3>
              <p class="text-muted-foreground mt-2 text-sm">Schedule automated tasks.</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                  <th class="px-6 py-4">Schedule</th>
                  <th class="px-6 py-4">Command</th>
                  <th class="px-6 py-4">Status</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="cron in cronJobs" :key="cron.id" class="group hover:bg-muted/50 transition-all">
                  <td class="px-6 py-4">
                    <div class="flex items-center gap-3">
                      <div class="w-10 h-10 bg-primary/10 rounded-xl flex items-center justify-center text-primary">
                        <Clock class="w-5 h-5" />
                      </div>
                      <div>
                        <p class="text-xs font-mono font-semibold text-foreground">{{ cron.schedule }}</p>
                        <p class="text-[10px] text-muted-foreground mt-0.5">{{ cron.description || 'No description' }}</p>
                      </div>
                    </div>
                  </td>
                  <td class="px-6 py-4">
                    <p class="text-xs font-mono text-foreground bg-muted p-2 rounded-lg border border-border truncate max-w-[250px]">{{ cron.command }}</p>
                  </td>
                  <td class="px-6 py-4">
                    <span :class="['text-[10px] font-semibold uppercase px-2 py-1 rounded-lg', cron.is_active !== false ? 'bg-emerald-500/10 text-emerald-600' : 'bg-muted text-muted-foreground']">
                      {{ cron.is_active !== false ? 'Active' : 'Paused' }}
                    </span>
                  </td>
                  <td class="px-6 py-4 text-right">
                    <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-all">
                      <button class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Run Now">
                        <Play class="w-4 h-4" />
                      </button>
                      <button @click="openDeleteCron(cron)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all" title="Delete">
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Backups Tab -->
          <div v-else-if="activeTab === 'backups'">
            <div class="p-4 border-b border-border flex items-center justify-between">
              <span class="text-sm font-semibold text-foreground">System Backups</span>
              <div class="flex gap-3">
                <button @click="showScheduleBackupModal = true" class="text-xs text-muted-foreground hover:text-foreground">Schedule</button>
                <button @click="showCreateBackupModal = true" class="text-xs text-primary hover:underline">+ Create Now</button>
              </div>
            </div>
            <div v-if="backups.length === 0" class="p-20 text-center">
              <Save class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold text-foreground">No backups</h3>
              <p class="text-muted-foreground mt-2 text-sm">Create backups to protect your data.</p>
            </div>
            <table v-else class="w-full text-left">
              <thead>
                <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                  <th class="px-6 py-4">Backup</th>
                  <th class="px-6 py-4">Type</th>
                  <th class="px-6 py-4">Size</th>
                  <th class="px-6 py-4">Date</th>
                  <th class="px-6 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border">
                <tr v-for="backup in backups" :key="backup.id" class="group hover:bg-muted/50 transition-all">
                  <td class="px-6 py-4">
                    <div class="flex items-center gap-3">
                      <div class="w-10 h-10 bg-emerald-500/10 rounded-xl flex items-center justify-center text-emerald-600">
                        <Save class="w-5 h-5" />
                      </div>
                      <div>
                        <p class="text-sm font-semibold text-foreground">{{ backup.filename || 'backup_' + backup.id?.slice(0,8) + '.tar.gz' }}</p>
                        <p class="text-[10px] text-muted-foreground mt-0.5">{{ backup.description || 'No description' }}</p>
                      </div>
                    </div>
                  </td>
                  <td class="px-6 py-4">
                    <span class="text-xs font-medium text-foreground bg-muted px-2 py-1 rounded-lg border border-border uppercase">{{ backup.backup_type }}</span>
                  </td>
                  <td class="px-6 py-4 text-sm text-muted-foreground">{{ formatBytes(backup.size_bytes || 0) }}</td>
                  <td class="px-6 py-4 text-xs text-muted-foreground">{{ backup.created_at?.split('T')[0] }}</td>
                  <td class="px-6 py-4 text-right">
                    <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-all">
                      <button class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Download">
                        <Download class="w-4 h-4" />
                      </button>
                      <button class="p-2 text-muted-foreground hover:text-emerald-600 hover:bg-emerald-500/10 rounded-lg transition-all" title="Restore">
                        <RefreshCw class="w-4 h-4" />
                      </button>
                      <button @click="openDeleteBackup(backup)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all" title="Delete">
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- PHP Version Tab -->
          <div v-else-if="activeTab === 'php'">
            <div class="p-4 border-b border-border">
              <span class="text-sm font-semibold text-foreground">PHP Version Selector</span>
            </div>
            <div class="p-6">
              <div class="bg-muted/50 border border-border rounded-xl p-6 mb-6">
                <div class="flex items-center gap-4 mb-4">
                  <div class="w-12 h-12 bg-indigo-500/10 rounded-xl flex items-center justify-center text-indigo-600">
                    <Globe class="w-6 h-6" />
                  </div>
                  <div>
                    <h3 class="text-lg font-semibold text-foreground">Current PHP Version</h3>
                    <p class="text-sm text-muted-foreground">PHP {{ currentPhpVersion }} is currently active</p>
                  </div>
                </div>
              </div>
              <h4 class="text-sm font-semibold text-foreground mb-4">Select PHP Version</h4>
              <div class="grid grid-cols-2 md:grid-cols-5 gap-3">
                <button 
                  v-for="version in availablePhpVersions" 
                  :key="version"
                  @click="changePhpVersion(version)"
                  :class="[
                    'p-4 border rounded-xl text-center transition-all',
                    currentPhpVersion === version 
                      ? 'border-primary bg-primary/10 text-primary' 
                      : 'border-border bg-muted/50 text-foreground hover:border-primary/30'
                  ]"
                >
                  <p class="text-lg font-bold">{{ version }}</p>
                  <p class="text-[10px] text-muted-foreground mt-1">
                    {{ version === '8.3' ? 'Latest' : version === '8.2' ? 'Recommended' : 'Supported' }}
                  </p>
                </button>
              </div>
              <div class="mt-6 bg-amber-500/10 border border-amber-500/20 p-4 rounded-xl">
                <p class="text-sm text-amber-700 dark:text-amber-400">
                  ⚠️ Changing PHP version may affect your applications. Make sure your code is compatible.
                </p>
              </div>
            </div>
          </div>

          <!-- Error Logs Tab -->
          <div v-else-if="activeTab === 'logs'">
            <div class="p-4 border-b border-border flex items-center justify-between">
              <span class="text-sm font-semibold text-foreground">Error Logs</span>
              <button @click="clearLogs" class="text-xs text-primary hover:underline">Clear Logs</button>
            </div>
            <div v-if="errorLogs.length === 0" class="p-20 text-center">
              <FileText class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold text-foreground">No errors</h3>
              <p class="text-muted-foreground mt-2 text-sm">No error logs found.</p>
            </div>
            <div v-else class="p-4 font-mono text-xs bg-zinc-900 text-zinc-100 overflow-x-auto max-h-96">
              <div v-for="(log, idx) in errorLogs" :key="idx" class="py-1 border-b border-zinc-800 last:border-0">
                <span :class="log.includes('Fatal') || log.includes('error') ? 'text-red-400' : 'text-amber-400'">{{ log }}</span>
              </div>
            </div>
          </div>

          <!-- Services Tab -->
          <div v-else>
            <div class="p-4 border-b border-border flex items-center justify-between">
              <span class="text-sm font-semibold text-foreground">System Services</span>
              <button @click="refreshServices" class="text-xs text-primary hover:underline flex items-center gap-1">
                <RefreshCw class="w-3 h-3" />
                Refresh
              </button>
            </div>
            <div v-if="services.length === 0" class="p-20 text-center">
              <Server class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold text-foreground">No services data</h3>
              <p class="text-muted-foreground mt-2 text-sm">Admin access required to view services.</p>
            </div>
            <div v-else class="p-6 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div v-for="service in services" :key="service.name" class="bg-muted/50 border border-border p-4 rounded-xl hover:border-primary/30 transition-all">
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center gap-3">
                    <div :class="['w-10 h-10 rounded-xl flex items-center justify-center', service.status === 'active' ? 'bg-emerald-500/10 text-emerald-600' : service.status === 'failed' ? 'bg-destructive/10 text-destructive' : 'bg-amber-500/10 text-amber-600']">
                      <Server class="w-5 h-5" />
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-foreground">{{ service.name }}</p>
                      <p :class="['text-[10px] mt-0.5 uppercase font-medium', service.status === 'active' ? 'text-emerald-600' : service.status === 'failed' ? 'text-destructive' : 'text-amber-600']">{{ service.status }}</p>
                    </div>
                  </div>
                  <div :class="['w-3 h-3 rounded-full', service.status === 'active' ? 'bg-emerald-500' : service.status === 'failed' ? 'bg-destructive' : 'bg-amber-500']" />
                </div>
                <div class="grid grid-cols-2 gap-2 text-xs text-muted-foreground mb-3">
                  <div>
                    <span class="block text-[10px] uppercase font-medium">Uptime</span>
                    {{ service.uptime || 'N/A' }}
                  </div>
                  <div>
                    <span class="block text-[10px] uppercase font-medium">Memory</span>
                    {{ service.memory_usage || 'N/A' }}
                  </div>
                </div>
                <div class="flex gap-2">
                  <button 
                    v-if="service.status === 'active'"
                    @click="controlService(service.name, 'restart')"
                    :disabled="serviceControlling === service.name"
                    class="flex-1 px-3 py-2 bg-primary/10 hover:bg-primary/20 text-primary rounded-lg text-xs font-medium transition-all disabled:opacity-50"
                  >
                    {{ serviceControlling === service.name ? 'Processing...' : 'Restart' }}
                  </button>
                  <button 
                    v-if="service.status !== 'active'"
                    @click="controlService(service.name, 'start')"
                    :disabled="serviceControlling === service.name"
                    class="flex-1 px-3 py-2 bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-600 rounded-lg text-xs font-medium transition-all disabled:opacity-50"
                  >
                    {{ serviceControlling === service.name ? 'Starting...' : 'Start' }}
                  </button>
                  <button 
                    v-if="service.status === 'active'"
                    @click="controlService(service.name, 'stop')"
                    :disabled="serviceControlling === service.name"
                    class="px-3 py-2 bg-destructive/10 hover:bg-destructive/20 text-destructive rounded-lg text-xs font-medium transition-all disabled:opacity-50"
                  >
                    Stop
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== MODALS ==================== -->

    <!-- Add Cron Modal -->
    <div v-if="showAddCronModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAddCronModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-lg p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Add Cron Job</h3>
          <button @click="showAddCronModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4" />{{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Common Settings</label>
            <select v-model="cronPreset" @change="onPresetChange" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="">-- Select Preset --</option>
              <option v-for="preset in cronPresets" :key="preset.value" :value="preset.value">{{ preset.label }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Schedule (Cron syntax)</label>
            <input v-model="cronSchedule" type="text" placeholder="* * * * *" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
            <p class="text-[10px] text-muted-foreground mt-1">minute hour day month weekday</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Command</label>
            <input v-model="cronCommand" type="text" placeholder="/usr/bin/php /home/user/script.php" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Description (optional)</label>
            <input v-model="cronDescription" type="text" placeholder="Daily cleanup task" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showAddCronModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createCronJob" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Create</button>
        </div>
      </div>
    </div>

    <!-- Create Backup Modal -->
    <div v-if="showCreateBackupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showCreateBackupModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Create Backup</h3>
          <button @click="showCreateBackupModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Backup Type</label>
            <div class="grid grid-cols-3 gap-2">
              <button 
                v-for="type in [
                  { value: 'full', label: 'Full', icon: HardDrive },
                  { value: 'database', label: 'Database', icon: Server },
                  { value: 'homedir', label: 'Files', icon: Save }
                ]" 
                :key="type.value"
                @click="backupType = type.value as any"
                :class="[
                  'p-3 border rounded-lg text-center transition-all',
                  backupType === type.value ? 'border-primary bg-primary/10 text-primary' : 'border-border bg-muted text-muted-foreground hover:border-primary/30'
                ]"
              >
                <component :is="type.icon" class="w-5 h-5 mx-auto mb-1" />
                <p class="text-xs font-medium">{{ type.label }}</p>
              </button>
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Description (optional)</label>
            <input v-model="backupDesc" type="text" placeholder="Before major update" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showCreateBackupModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createBackup" class="px-4 py-2 text-sm font-medium bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors">Create Backup</button>
        </div>
      </div>
    </div>

    <!-- Schedule Backup Modal -->
    <div v-if="showScheduleBackupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showScheduleBackupModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Backup Schedule</h3>
          <button @click="showScheduleBackupModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Schedule</label>
            <select v-model="backupSchedule" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option v-for="opt in backupScheduleOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Backup Type</label>
            <select v-model="backupType" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="full">Full Backup</option>
              <option value="database">Database Only</option>
              <option value="homedir">Home Directory Only</option>
            </select>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showScheduleBackupModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="showScheduleBackupModal = false" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Save Schedule</button>
        </div>
      </div>
    </div>

    <!-- Delete Modals -->
    <div v-if="showDeleteCronModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteCronModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Cron Job</h3>
        <p class="text-sm text-muted-foreground mb-6">Are you sure you want to delete this scheduled task?</p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteCronModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteCronJob" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
        </div>
      </div>
    </div>

    <div v-if="showDeleteBackupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteBackupModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Backup</h3>
        <p class="text-sm text-muted-foreground mb-6">Are you sure you want to delete this backup? This action cannot be undone.</p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteBackupModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteBackup" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
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
