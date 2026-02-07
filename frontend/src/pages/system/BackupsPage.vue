<script setup lang="ts">
/**
 * BackupsPage - System Backup Management
 */
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, BaseModal } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { systemService } from '@/services'
import { useToastStore } from '@/stores/toast'
import { RefreshCw, Database, HardDrive, Archive, Download, Trash2, Plus, AlertTriangle, FileArchive } from 'lucide-vue-next'
import type { SystemBackup } from '@/types'

const isLoading = ref(true)
const backups = ref<SystemBackup[]>([])
const router = useRouter()
const toast = useToastStore()

// Create Backup Form
const showCreateModal = ref(false)
const createType = ref<'full' | 'database' | 'homedir'>('full')
const createDescription = ref('')
const isCreating = ref(false)

// Delete
const showDeleteModal = ref(false)
const backupToDelete = ref<SystemBackup | null>(null)
const isDeleting = ref(false)

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await systemService.listBackups()
        backups.value = res.data.data || []
    } catch (e) {
        toast.error('Failed to load backups')
    } finally {
        isLoading.value = false
    }
}

const formatBytes = (bytes: number, decimals = 2) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString()
}

const getBackupIcon = (type: string) => {
    switch(type) {
        case 'database': return Database;
        case 'homedir': return HardDrive;
        default: return Archive; // Full or other
    }
}

const createBackup = async () => {
    isCreating.value = true
    try {
        await systemService.createBackup({ 
            backup_type: createType.value,
            description: createDescription.value
        })
        toast.success('Backup created successfully')
        showCreateModal.value = false
        await fetchData()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to create backup')
    } finally {
        isCreating.value = false
    }
}

const confirmDelete = (backup: SystemBackup) => {
    backupToDelete.value = backup
    showDeleteModal.value = true
}

const deleteBackup = async () => {
    if (!backupToDelete.value) return
    isDeleting.value = true
    try {
        await systemService.deleteBackup(backupToDelete.value.id)
        toast.success('Backup deleted successfully')
        showDeleteModal.value = false
        await fetchData()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to delete backup')
    } finally {
        isDeleting.value = false
        backupToDelete.value = null
    }
}

const downloadBackup = (id: string) => {
    const url = systemService.getBackupDownloadUrl(id)
    window.open(url, '_blank')
}

const refreshTable = () => {
    toast.info('Refreshing...')
    fetchData()
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <div class="space-y-8">
        <!-- Breadcrumbs -->
        <AppBreadcrumb
            :items="[
                { label: 'System Tools', icon: Database, onClick: () => router.push('/dashboard/system') },
                { label: 'Backups', current: true }
            ]"
        />

        <!-- Page Header -->
        <div class="flex justify-between items-end">
            <div class="flex flex-col gap-2">
                <h1 class="text-foreground text-4xl font-black leading-tight tracking-tight">Backup Management</h1>
                <p class="text-muted-foreground text-base max-w-2xl">Create and manage system backups securely. Download archives or restore points.</p>
            </div>
            <div class="flex gap-2">
                <Button variant="outline" @click="refreshTable">
                    <RefreshCw :size="18" class="mr-2" /> Refresh
                </Button>
                <Button @click="showCreateModal = true">
                    <Plus :size="18" class="mr-2" /> Create Backup
                </Button>
            </div>
        </div>

        <!-- Backups Table -->
        <Card class="rounded-xl">
            <CardHeader class="bg-muted/30 flex-row items-center justify-between">
                <CardTitle>Available Backups</CardTitle>
                <Badge variant="secondary">{{ backups.length }} Archives</Badge>
            </CardHeader>

            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-muted-foreground">Loading backups...</p>
            </div>

            <div v-else-if="backups.length === 0" class="p-12 text-center">
                <FileArchive :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                <p class="text-muted-foreground">No backups found. Create one to get started.</p>
            </div>

            <div v-else class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-muted/30">
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Filename</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Type</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Size</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Date Created</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-border">
                        <tr v-for="backup in backups" :key="backup.id" class="hover:bg-muted/30 transition-colors">
                            <td class="px-6 py-4">
                                <div class="flex items-center gap-3">
                                    <component :is="getBackupIcon(backup.backup_type)" class="w-5 h-5 text-primary" />
                                    <span class="font-mono text-sm font-medium">{{ backup.filename }}</span>
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <Badge variant="outline" class="uppercase text-[10px]">{{ backup.backup_type }}</Badge>
                            </td>
                            <td class="px-6 py-4 text-sm text-foreground">
                                {{ formatBytes(backup.size_bytes) }}
                            </td>
                            <td class="px-6 py-4 text-sm text-muted-foreground">
                                {{ formatDate(backup.created_at) }}
                            </td>
                            <td class="px-6 py-4 text-right">
                                <div class="flex justify-end gap-2">
                                    <Button variant="ghost" size="icon" @click="downloadBackup(backup.id)" title="Download">
                                        <Download :size="18" />
                                    </Button>
                                    <Button variant="ghost" size="icon" @click="confirmDelete(backup)" class="text-destructive hover:text-destructive" title="Delete">
                                        <Trash2 :size="18" />
                                    </Button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </Card>
    </div>

    <!-- Create Backup Modal -->
    <BaseModal :isOpen="showCreateModal" @close="showCreateModal = false" title="Create New Backup">
        <div class="space-y-4">
            <div class="space-y-2">
                <label class="text-sm font-medium">Backup Type</label>
                <select v-model="createType" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50">
                    <option value="full">Full Backup (Files + Database)</option>
                    <option value="homedir">Home Directory Only</option>
                    <option value="database">Database Only</option>
                </select>
                <p class="text-xs text-muted-foreground" v-if="createType === 'full'">Backs up public_html and all databases.</p>
                <p class="text-xs text-muted-foreground" v-if="createType === 'homedir'">Backs up /home/user/public_html only.</p>
                <p class="text-xs text-muted-foreground" v-if="createType === 'database'">Backs up all user databases to SQL files.</p>
            </div>
            
            <div class="bg-blue-50/50 p-4 rounded-lg border border-blue-100 dark:border-blue-900/30">
                <p class="text-sm text-blue-600 dark:text-blue-400">
                    <Info class="w-4 h-4 inline mr-1 mb-0.5" />
                    Backups are stored in your home directory (/home/user/backups).
                </p>
            </div>
        </div>
        <template #footer>
            <Button variant="ghost" @click="showCreateModal = false">Cancel</Button>
            <Button @click="createBackup" :disabled="isCreating">
                <span v-if="isCreating">Creating...</span>
                <span v-else>Start Backup</span>
            </Button>
        </template>
    </BaseModal>

    <!-- Delete Confirmation Modal -->
    <BaseModal :isOpen="showDeleteModal" @close="showDeleteModal = false" title="Confirm Deletion" width="sm">
        <div class="flex items-start gap-4 py-2">
            <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center shrink-0">
                <AlertTriangle class="w-5 h-5 text-destructive" />
            </div>
            <div>
                <p class="text-sm font-medium text-foreground">Delete Backup Archive</p>
                <p class="text-sm text-muted-foreground mt-1">
                    Are you sure you want to delete <strong>{{ backupToDelete?.filename }}</strong>? This cannot be undone.
                </p>
            </div>
        </div>
        <template #footer>
            <Button variant="ghost" @click="showDeleteModal = false">Cancel</Button>
            <Button variant="destructive" @click="deleteBackup" :disabled="isDeleting">
                {{ isDeleting ? 'Deleting...' : 'Delete Backup' }}
            </Button>
        </template>
    </BaseModal>
</MainLayout>
</template>
