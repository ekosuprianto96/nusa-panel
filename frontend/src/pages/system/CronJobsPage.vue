<script setup lang="ts">
/**
 * CronJobsPage - Cron Job Management
 */
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, Input, BaseModal } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { systemService } from '@/services'
import { useToastStore } from '@/stores/toast'
import { RefreshCw, Clock, Terminal, Edit, Trash2, Info, Settings, Plus, AlertTriangle, FileText, Eraser, RotateCcw } from 'lucide-vue-next'

const isLoading = ref(true)
const cronJobs = ref<any[]>([])
const router = useRouter()

// Form data
const cronPreset = ref('')
const cronMinute = ref('*')
const cronHour = ref('*')
const cronDay = ref('*')
const cronMonth = ref('*')
const cronWeekday = ref('*')
const cronCommand = ref('')

const presets = [
    { value: '', label: '-- Select a Preset --' },
    { value: '* * * * *', label: 'Every Minute ( * * * * * )' },
    { value: '*/5 * * * *', label: 'Every 5 Minutes ( */5 * * * * )' },
    { value: '0 * * * *', label: 'Every Hour ( 0 * * * * )' },
    { value: '0 0,12 * * *', label: 'Twice Daily ( 0 0,12 * * * )' },
    { value: '0 0 * * 0', label: 'Once a Week ( 0 0 * * 0 )' }
]

const toast = useToastStore()
const showDeleteModal = ref(false)
const cronToDelete = ref<string | null>(null)

// Logs
const showLogModal = ref(false)
const logsLoading = ref(false)
const logsContent = ref('')
const currentCronForLogs = ref<string | null>(null)

// Edit
const showEditModal = ref(false)
const editingCronId = ref<string | null>(null)
const editSchedule = ref('')
const editCommand = ref('')
const editDescription = ref('')

const getScheduleDescription = (schedule: string): string => {
    const parts = schedule.split(' ')
    if (parts.length !== 5) return schedule
    const [min, hour] = parts
    if (schedule === '* * * * *') return 'Every Minute'
    if (schedule === '*/5 * * * *') return 'Every 5 Minutes'
    if (schedule === '0 * * * *') return 'Every Hour'
    if (schedule === '0 0 * * *') return 'Daily at Midnight'
    if (schedule === '@reboot') return 'On System Startup'
    if (min === '0' && hour === '12' && parts[4] === '1-5') return 'Weekdays at Noon'
    return schedule
}

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await systemService.listCronJobs()
        cronJobs.value = res.data.data || []
    } catch (e) {
        toast.error('Failed to load cron jobs')
    } finally {
        isLoading.value = false
    }
}

const onPresetChange = () => {
    if (!cronPreset.value) return
    const parts = cronPreset.value.split(' ')
    if (parts.length === 5) {
        cronMinute.value = parts[0] || '*'
        cronHour.value = parts[1] || '*'
        cronDay.value = parts[2] || '*'
        cronMonth.value = parts[3] || '*'
        cronWeekday.value = parts[4] || '*'
    }
}

const createCronJob = async () => {
    if (!cronCommand.value.trim()) {
        toast.error('Command is required')
        return
    }
    const schedule = `${cronMinute.value} ${cronHour.value} ${cronDay.value} ${cronMonth.value} ${cronWeekday.value}`
    try {
        isLoading.value = true
        await systemService.createCronJob({ schedule, command: cronCommand.value.trim() })
        toast.success('Cron job created successfully')
        resetForm()
        await fetchData()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to create cron job')
    } finally {
        isLoading.value = false
    }
}

const deleteCronJob = (cronId: string) => {
    cronToDelete.value = cronId
    showDeleteModal.value = true
}

const confirmDelete = async () => {
    if (!cronToDelete.value) return
    
    try {
        isLoading.value = true
        await systemService.deleteCronJob(cronToDelete.value)
        toast.success('Cron job deleted successfully')
        showDeleteModal.value = false
        await fetchData()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to delete cron job')
    } finally {
        isLoading.value = false
        cronToDelete.value = null
    }
}

const viewLogs = async (cronId: string) => {
    currentCronForLogs.value = cronId
    logsContent.value = ''
    showLogModal.value = true
    await fetchLogs()
}

const fetchLogs = async () => {
    if (!currentCronForLogs.value) return
    logsLoading.value = true
    try {
        const res = await systemService.getCronLogs(currentCronForLogs.value)
        logsContent.value = res.data.data
    } catch (e: any) {
        toast.error('Failed to fetch logs')
        logsContent.value = 'Failed to load logs.'
    } finally {
        logsLoading.value = false
    }
}

const clearLogs = async () => {
    if (!currentCronForLogs.value) return
    try {
        logsLoading.value = true
        await systemService.clearCronLogs(currentCronForLogs.value)
        toast.success('Logs cleared')
        logsContent.value = '' 
        await fetchLogs()
    } catch (e: any) {
        toast.error('Failed to clear logs')
    } finally {
        logsLoading.value = false
    }
}



const editCronJob = (cron: any) => {
    editingCronId.value = cron.id
    editSchedule.value = cron.schedule
    editCommand.value = cron.command
    editDescription.value = cron.description || ''
    showEditModal.value = true
}

const updateCronJob = async () => {
    if (!editingCronId.value) return
    if (!editCommand.value.trim()) {
        toast.error('Command is required')
        return
    }
    
    try {
        isLoading.value = true
        await systemService.updateCronJob(editingCronId.value, {
            schedule: editSchedule.value,
            command: editCommand.value.trim(),
            description: editDescription.value
        })
        toast.success('Cron job updated')
        showEditModal.value = false
        await fetchData()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to update cron job')
    } finally {
        isLoading.value = false
    }
}

const resetForm = () => {
    cronPreset.value = ''
    cronMinute.value = '*'
    cronHour.value = '*'
    cronDay.value = '*'
    cronMonth.value = '*'
    cronWeekday.value = '*'
    cronCommand.value = ''
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
                { label: 'System Tools', icon: Settings, onClick: () => router.push('/dashboard/system') },
                { label: 'Cron Jobs', current: true }
            ]"
        />

        <!-- Page Header -->
        <div class="flex justify-between items-end">
            <div class="flex flex-col gap-2">
                <h1 class="text-foreground text-4xl font-black leading-tight tracking-tight">Cron Job Management</h1>
                <p class="text-muted-foreground text-base max-w-2xl">Schedule and manage recurring system tasks using standard cron syntax.</p>
            </div>
            <Button variant="outline" @click="refreshTable">
                <RefreshCw :size="18" class="mr-2" /> Refresh Table
            </Button>
        </div>

        <!-- Add New Cron Job Form -->
        <Card class="rounded-xl">
            <CardHeader class="bg-muted/30">
                <CardTitle>Add New Cron Job</CardTitle>
            </CardHeader>
            <CardContent class="p-6 space-y-6">
                <!-- Preset Dropdown -->
                <div class="max-w-md">
                    <label class="flex flex-col gap-2">
                        <span class="text-foreground text-sm font-semibold">Common Settings</span>
                        <select v-model="cronPreset" @change="onPresetChange" class="block w-full rounded-lg border-border bg-background text-foreground focus:border-primary focus:ring-primary/20 h-11 text-sm">
                            <option v-for="p in presets" :key="p.value" :value="p.value">{{ p.label }}</option>
                        </select>
                    </label>
                </div>

                <!-- Cron Syntax Grid -->
                <div class="grid grid-cols-5 gap-4">
                    <div>
                        <label class="block text-foreground text-sm font-semibold mb-2">Minute</label>
                        <Input v-model="cronMinute" class="h-11 text-center font-mono" placeholder="0-59" />
                        <span class="text-[10px] text-muted-foreground uppercase mt-1 block">0-59</span>
                    </div>
                    <div>
                        <label class="block text-foreground text-sm font-semibold mb-2">Hour</label>
                        <Input v-model="cronHour" class="h-11 text-center font-mono" placeholder="0-23" />
                        <span class="text-[10px] text-muted-foreground uppercase mt-1 block">0-23</span>
                    </div>
                    <div>
                        <label class="block text-foreground text-sm font-semibold mb-2">Day</label>
                        <Input v-model="cronDay" class="h-11 text-center font-mono" placeholder="1-31" />
                        <span class="text-[10px] text-muted-foreground uppercase mt-1 block">1-31</span>
                    </div>
                    <div>
                        <label class="block text-foreground text-sm font-semibold mb-2">Month</label>
                        <Input v-model="cronMonth" class="h-11 text-center font-mono" placeholder="1-12" />
                        <span class="text-[10px] text-muted-foreground uppercase mt-1 block">1-12</span>
                    </div>
                    <div>
                        <label class="block text-foreground text-sm font-semibold mb-2">Weekday</label>
                        <Input v-model="cronWeekday" class="h-11 text-center font-mono" placeholder="0-6" />
                        <span class="text-[10px] text-muted-foreground uppercase mt-1 block">0-6 (Sun-Sat)</span>
                    </div>
                </div>

                <!-- Command Field -->
                <div>
                    <label class="block text-foreground text-sm font-semibold mb-2">Command to Run</label>
                    <div class="relative">
                        <Terminal :size="18" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                        <Input v-model="cronCommand" class="pl-10 h-11 font-mono text-sm" placeholder="/usr/bin/php /var/www/html/artisan schedule:run" />
                    </div>
                </div>

                <div class="flex justify-end pt-2">
                    <Button @click="createCronJob">
                        <Plus :size="18" class="mr-2" /> Add Cron Job
                    </Button>
                </div>
            </CardContent>
        </Card>

        <!-- Current Cron Jobs Table -->
        <Card class="rounded-xl">
            <CardHeader class="bg-muted/30 flex-row items-center justify-between">
                <CardTitle>Current Cron Jobs</CardTitle>
                <Badge variant="default">{{ cronJobs.length }} Tasks Active</Badge>
            </CardHeader>

            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-muted-foreground">Loading cron jobs...</p>
            </div>

            <div v-else-if="cronJobs.length === 0" class="p-12 text-center">
                <Clock :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                <p class="text-muted-foreground">No cron jobs configured</p>
            </div>

            <div v-else class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-muted/30">
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Schedule</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border">Command</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border text-center">Status</th>
                            <th class="px-6 py-3 text-muted-foreground text-xs font-bold uppercase tracking-wider border-b border-border text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-border">
                        <tr v-for="cron in cronJobs" :key="cron.id" class="hover:bg-muted/30 transition-colors">
                            <td class="px-6 py-4">
                                <div class="flex flex-col">
                                    <span class="font-mono text-primary text-sm font-semibold">{{ cron.schedule }}</span>
                                    <span class="text-[11px] text-muted-foreground">{{ getScheduleDescription(cron.schedule) }}</span>
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <code class="bg-muted px-2 py-1 rounded text-xs text-foreground break-all">{{ cron.command }}</code>
                            </td>
                            <td class="px-6 py-4 text-center">
                                <Badge :variant="cron.is_active !== false ? 'success' : 'secondary'">
                                    {{ cron.is_active !== false ? 'Active' : 'Paused' }}
                                </Badge>
                            </td>
                            <td class="px-6 py-4 text-right">
                                <div class="flex justify-end gap-2">
                                    <Button variant="ghost" size="icon" @click="viewLogs(cron.id)" title="View Logs">
                                        <FileText :size="18" />
                                    </Button>
                                    <Button variant="ghost" size="icon" @click="editCronJob(cron)" title="Edit">
                                        <Edit :size="18" />
                                    </Button>
                                    <Button variant="ghost" size="icon" @click="deleteCronJob(cron.id)" class="text-destructive hover:text-destructive" title="Delete">
                                        <Trash2 :size="18" />
                                    </Button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </Card>

        <!-- Quick Cron Guide -->
        <Card class="rounded-xl bg-primary/5 border-primary/20">
            <CardContent class="p-6 flex gap-4 items-start">
                <Info :size="20" class="text-primary mt-0.5" />
                <div class="text-sm">
                    <p class="text-foreground font-bold mb-1">Quick Cron Guide</p>
                    <p class="text-muted-foreground leading-relaxed">
                        Cron uses a five-field format: <code class="bg-primary/10 text-primary px-1 font-bold">Min Hour Day Month Weekday</code>. 
                        Use <code class="font-bold">*</code> for "every", <code class="font-bold">*/n</code> for "every n intervals", and <code class="font-bold">x,y</code> for specific values.
                    </p>
                </div>
            </CardContent>
        </Card>
    </div>

    <!-- Delete Confirmation Modal -->
    <BaseModal :isOpen="showDeleteModal" @close="showDeleteModal = false" title="Confirm Deletion" width="sm">
        <div class="flex items-start gap-4 py-2">
            <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center shrink-0">
                <AlertTriangle class="w-5 h-5 text-destructive" />
            </div>
            <div>
                <p class="text-sm font-medium text-foreground">Delete Cron Job</p>
                <p class="text-sm text-muted-foreground mt-1">
                    Are you sure you want to delete this cron job? This action cannot be undone.
                </p>
            </div>
        </div>
        <template #footer>
            <Button variant="ghost" @click="showDeleteModal = false">Cancel</Button>
            <Button variant="destructive" @click="confirmDelete" :disabled="isLoading">
                {{ isLoading ? 'Deleting...' : 'Delete' }}
            </Button>
        </template>
    </BaseModal>

    <!-- Log Viewer Modal -->
    <BaseModal :isOpen="showLogModal" @close="showLogModal = false" title="Cron Job Logs" width="lg">
        <div class="space-y-4">
            <div class="bg-muted rounded-lg p-4 font-mono text-xs overflow-x-auto whitespace-pre-wrap max-h-[60vh] min-h-[200px] border border-border">
                <div v-if="logsLoading && !logsContent" class="flex items-center justify-center py-8">
                    <div class="animate-spin w-6 h-6 border-2 border-primary border-t-transparent rounded-full"></div>
                </div>
                <div v-else>{{ logsContent || 'No logs available.' }}</div>
            </div>
        </div>
        <template #footer>
            <Button variant="outline" @click="fetchLogs" :disabled="logsLoading">
                <RotateCcw :size="16" class="mr-2" /> Refresh
            </Button>
            <Button variant="destructive" @click="clearLogs" :disabled="logsLoading || !logsContent">
                <Eraser :size="16" class="mr-2" /> Clear Logs
            </Button>
            <Button variant="ghost" @click="showLogModal = false">Close</Button>
        </template>
    </BaseModal>

    <!-- Edit Modal -->
    <BaseModal :isOpen="showEditModal" @close="showEditModal = false" title="Edit Cron Job" width="md">
        <div class="space-y-4">
            <div class="space-y-2">
                <label class="text-sm font-medium">Schedule</label>
                <Input v-model="editSchedule" placeholder="* * * * *" />
                <p class="text-xs text-muted-foreground">Format: min hour day month weekday</p>
            </div>
            <div class="space-y-2">
                <label class="text-sm font-medium">Command</label>
                <Input v-model="editCommand" placeholder="php /path/to/script.php" />
            </div>
            <div class="space-y-2">
                <label class="text-sm font-medium">Description</label>
                <Input v-model="editDescription" placeholder="Description (optional)" />
            </div>
        </div>
        <template #footer>
            <Button variant="ghost" @click="showEditModal = false">Cancel</Button>
            <Button @click="updateCronJob" :disabled="isLoading">Save Changes</Button>
        </template>
    </BaseModal>
</MainLayout>
</template>
