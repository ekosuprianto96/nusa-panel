<script setup lang="ts">
/**
 * CronJobsPage - Cron Job Management
 * 
 * Features:
 * - Add cron job form with presets
 * - Cron syntax grid (minute, hour, day, month, weekday)
 * - Current cron jobs table
 * - Quick cron guide
 */
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { systemService } from '@/services'
import { RefreshCw, Clock, Terminal, Edit, Trash2, Info } from 'lucide-vue-next'

const isLoading = ref(true)
const cronJobs = ref<any[]>([])


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

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

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
        showToast('Failed to load cron jobs', 'error')
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
        showToast('Command is required', 'error')
        return
    }
    const schedule = `${cronMinute.value} ${cronHour.value} ${cronDay.value} ${cronMonth.value} ${cronWeekday.value}`
    try {
        isLoading.value = true
        await systemService.createCronJob({ schedule, command: cronCommand.value.trim() })
        showToast('Cron job created successfully', 'success')
        resetForm()
        await fetchData()
    } catch (e: any) {
        showToast(e.response?.data?.message || 'Failed to create cron job', 'error')
    } finally {
        isLoading.value = false
    }
}

const deleteCronJob = async (cronId: string) => {
    if (!confirm('Are you sure you want to delete this cron job?')) return
    try {
        await systemService.deleteCronJob(cronId)
        showToast('Cron job deleted successfully', 'success')
        await fetchData()
    } catch (e: any) {
        showToast(e.response?.data?.message || 'Failed to delete cron job', 'error')
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
    showToast('Refreshing...', 'info')
    fetchData()
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <!-- Toast Notifications -->
    <div class="fixed top-4 right-4 z-50 space-y-2">
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-red-500 text-white' : 'bg-primary text-white']">{{ toast.message }}</div>
    </div>

    <div class="space-y-8">
        <!-- Breadcrumbs -->
        <div class="flex items-center gap-2">
            <router-link to="/dashboard/system" class="text-slate-500 dark:text-slate-400 text-sm font-medium hover:text-primary transition-colors">System Tools</router-link>
            <span class="text-slate-400 dark:text-slate-600">/</span>
            <span class="text-[#0d131b] dark:text-white text-sm font-semibold">Cron Jobs</span>
        </div>

        <!-- Page Header -->
        <div class="flex justify-between items-end">
            <div class="flex flex-col gap-2">
                <h1 class="text-[#0d131b] dark:text-white text-4xl font-black leading-tight tracking-tight">Cron Job Management</h1>
                <p class="text-slate-500 dark:text-slate-400 text-base max-w-2xl">Schedule and manage recurring system tasks using standard cron syntax. Presets are available for common intervals.</p>
            </div>
            <button @click="refreshTable" class="flex items-center gap-2 px-4 py-2 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors">
                <RefreshCw :size="18" /> Refresh Table
            </button>
        </div>

        <!-- Add New Cron Job Form -->
        <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/30">
                <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Add New Cron Job</h2>
            </div>
            <div class="p-6 space-y-6">
                <!-- Preset Dropdown -->
                <div class="max-w-md">
                    <label class="flex flex-col gap-2">
                        <span class="text-slate-700 dark:text-slate-300 text-sm font-semibold">Common Settings</span>
                        <select v-model="cronPreset" @change="onPresetChange" class="block w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-sm">
                            <option v-for="p in presets" :key="p.value" :value="p.value">{{ p.label }}</option>
                        </select>
                    </label>
                </div>

                <!-- Cron Syntax Grid -->
                <div class="grid grid-cols-5 gap-4">
                    <div>
                        <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Minute</label>
                        <input v-model="cronMinute" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-center font-mono" placeholder="0-59" type="text" />
                        <span class="text-[10px] text-slate-400 uppercase mt-1 block">0-59</span>
                    </div>
                    <div>
                        <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Hour</label>
                        <input v-model="cronHour" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-center font-mono" placeholder="0-23" type="text" />
                        <span class="text-[10px] text-slate-400 uppercase mt-1 block">0-23</span>
                    </div>
                    <div>
                        <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Day</label>
                        <input v-model="cronDay" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-center font-mono" placeholder="1-31" type="text" />
                        <span class="text-[10px] text-slate-400 uppercase mt-1 block">1-31</span>
                    </div>
                    <div>
                        <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Month</label>
                        <input v-model="cronMonth" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-center font-mono" placeholder="1-12" type="text" />
                        <span class="text-[10px] text-slate-400 uppercase mt-1 block">1-12</span>
                    </div>
                    <div>
                        <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Weekday</label>
                        <input v-model="cronWeekday" class="w-full rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 text-center font-mono" placeholder="0-6" type="text" />
                        <span class="text-[10px] text-slate-400 uppercase mt-1 block">0-6 (Sun-Sat)</span>
                    </div>
                </div>

                <!-- Command Field -->
                <div>
                    <label class="block text-slate-700 dark:text-slate-300 text-sm font-semibold mb-2">Command to Run</label>
                    <div class="relative">
                        <Terminal :size="18" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                        <input v-model="cronCommand" class="w-full pl-10 rounded-lg border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 focus:border-primary focus:ring-primary/20 h-11 font-mono text-sm" placeholder="/usr/bin/php /var/www/html/artisan schedule:run" type="text" />
                    </div>
                </div>

                <div class="flex justify-end pt-2">
                    <button @click="createCronJob" class="bg-primary hover:bg-primary/90 text-white font-bold py-3 px-8 rounded-lg transition-all shadow-md active:scale-[0.98]">
                        Add Cron Job
                    </button>
                </div>
            </div>
        </div>

        <!-- Current Cron Jobs Table -->
        <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-100 dark:border-slate-800 flex justify-between items-center bg-slate-50/50 dark:bg-slate-800/30">
                <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Current Cron Jobs</h2>
                <span class="bg-primary/10 text-primary text-[11px] font-bold px-2 py-0.5 rounded uppercase tracking-wider">{{ cronJobs.length }} Tasks Active</span>
            </div>

            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-slate-500">Loading cron jobs...</p>
            </div>

            <div v-else-if="cronJobs.length === 0" class="p-12 text-center">
                <Clock :size="48" class="mx-auto mb-4 text-slate-300" />
                <p class="text-slate-500">No cron jobs configured</p>
            </div>

            <div v-else class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-slate-50 dark:bg-slate-800/50">
                            <th class="px-6 py-3 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-slate-100 dark:border-slate-800">Schedule</th>
                            <th class="px-6 py-3 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-slate-100 dark:border-slate-800">Command</th>
                            <th class="px-6 py-3 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-slate-100 dark:border-slate-800 text-center">Status</th>
                            <th class="px-6 py-3 text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-slate-100 dark:border-slate-800 text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                        <tr v-for="cron in cronJobs" :key="cron.id" class="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-colors">
                            <td class="px-6 py-4">
                                <div class="flex flex-col">
                                    <span class="font-mono text-primary text-sm font-semibold">{{ cron.schedule }}</span>
                                    <span class="text-[11px] text-slate-400">{{ getScheduleDescription(cron.schedule) }}</span>
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <code class="bg-slate-100 dark:bg-slate-800 px-2 py-1 rounded text-xs text-slate-700 dark:text-slate-300 break-all">{{ cron.command }}</code>
                            </td>
                            <td class="px-6 py-4 text-center">
                                <span :class="['text-[10px] font-bold uppercase px-2 py-1 rounded', cron.is_active !== false ? 'bg-emerald-500/10 text-emerald-600' : 'bg-slate-100 text-slate-500']">
                                    {{ cron.is_active !== false ? 'Active' : 'Paused' }}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-right">
                                <div class="flex justify-end gap-2">
                                    <button class="p-2 text-slate-400 hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Edit">
                                        <Edit :size="18" />
                                    </button>
                                    <button @click="deleteCronJob(cron.id)" class="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-all" title="Delete">
                                        <Trash2 :size="18" />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Quick Cron Guide -->
        <div class="p-6 bg-primary/5 dark:bg-primary/10 border border-primary/20 rounded-xl flex gap-4 items-start">
            <Info :size="20" class="text-primary mt-0.5" />
            <div class="text-sm">
                <p class="text-[#0d131b] dark:text-white font-bold mb-1">Quick Cron Guide</p>
                <p class="text-slate-600 dark:text-slate-400 leading-relaxed">
                    Cron uses a five-field format: <code class="bg-primary/10 text-primary px-1 font-bold">Min Hour Day Month Weekday</code>. 
                    Use <code class="font-bold">*</code> for "every", <code class="font-bold">*/n</code> for "every n intervals", and <code class="font-bold">x,y</code> for specific values. 
                    The command field should contain the full path to your scripts or binaries.
                </p>
            </div>
        </div>
    </div>
</MainLayout>
</template>
