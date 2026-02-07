<script setup lang="ts">
/**
 * PhpManagerPage - PHP Version & Extensions Manager
 */
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent, CardTitle } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { systemService } from '@/services'
import { Search, Settings } from 'lucide-vue-next'

const isLoading = ref(true)
const currentVersion = ref('8.2')
const selectedVersion = ref('8.2')
const activeTab = ref<'extensions' | 'options' | 'fpm'>('extensions')
const searchQuery = ref('')
const router = useRouter()

const versions = ref<{ value: string; label: string }[]>([])

const extensions = ref([
    { id: 'bcmath', name: 'bcmath', desc: 'Arbitrary Precision', enabled: true },
    { id: 'curl', name: 'curl', desc: 'Client URL Library', enabled: true },
    { id: 'gd', name: 'gd', desc: 'Image Processing', enabled: true },
    { id: 'imagick', name: 'imagick', desc: 'ImageMagick', enabled: false },
    { id: 'intl', name: 'intl', desc: 'Internationalization', enabled: true },
    { id: 'mbstring', name: 'mbstring', desc: 'Multibyte String', enabled: true },
    { id: 'mysqli', name: 'mysqli', desc: 'MySQL Improved', enabled: true },
    { id: 'opcache', name: 'opcache', desc: 'Bytecode Cache', enabled: true },
    { id: 'pdo_mysql', name: 'pdo_mysql', desc: 'PDO MySQL Driver', enabled: true },
    { id: 'xml', name: 'xml', desc: 'XML Support', enabled: true },
    { id: 'zip', name: 'zip', desc: 'Zip Compression', enabled: true },
    { id: 'xdebug', name: 'xdebug', desc: 'Debugging Module', enabled: false }
])

const directives = ref([
    { name: 'memory_limit', desc: 'Maximum amount of memory a script may consume', value: '256M', type: 'select', options: ['128M', '256M', '512M', '1024M'] },
    { name: 'upload_max_filesize', desc: 'Maximum size of an uploaded file', value: '32', unit: 'MB', type: 'input' },
    { name: 'max_execution_time', desc: 'Maximum time a script is allowed to run', value: '300', unit: 'sec', type: 'input' }
])

const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

const filteredExtensions = computed(() => {
    if (!searchQuery.value) return extensions.value
    return extensions.value.filter(e => e.name.toLowerCase().includes(searchQuery.value.toLowerCase()) || e.desc.toLowerCase().includes(searchQuery.value.toLowerCase()))
})

const fetchData = async () => {
    isLoading.value = true
    try {
        const [currentRes, versionsRes] = await Promise.all([
            systemService.getCurrentPhpVersion(),
            systemService.getPhpVersions()
        ])

        currentVersion.value = currentRes.data.data || '8.2'
        selectedVersion.value = currentVersion.value
        const list = versionsRes.data.data || []
        versions.value = (list.length > 0 ? list : ['8.2', '8.1', '8.0']).map(v => ({
            value: v,
            label: v === currentVersion.value ? `PHP ${v} (Active)` : `PHP ${v}`
        }))
    } catch (e) { console.error(e) }
    finally { isLoading.value = false }
}

const applyVersion = async () => {
    try {
        isLoading.value = true
        await systemService.changePhpVersion(selectedVersion.value)
        currentVersion.value = selectedVersion.value
        showToast(`PHP version changed to ${selectedVersion.value}`, 'success')
    } catch (e: any) {
        showToast(e.response?.data?.message || 'Failed to change PHP version', 'error')
    } finally {
        isLoading.value = false
    }
}

const toggleExtension = (ext: any) => {
    ext.enabled = !ext.enabled
    showToast(`${ext.name} ${ext.enabled ? 'enabled' : 'disabled'}`, 'success')
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <div class="fixed top-4 right-4 z-50 space-y-2">
        <div v-for="toast in toasts" :key="toast.id" :class="['px-4 py-3 rounded-lg shadow-lg font-medium text-sm', toast.type === 'success' ? 'bg-emerald-500 text-white' : toast.type === 'error' ? 'bg-destructive text-destructive-foreground' : 'bg-primary text-primary-foreground']">{{ toast.message }}</div>
    </div>

    <div class="space-y-8">
        <!-- Breadcrumbs -->
        <AppBreadcrumb
            :items="[
                { label: 'System Tools', icon: Settings, onClick: () => router.push('/dashboard/system') },
                { label: 'PHP Manager', current: true }
            ]"
        />

        <!-- Page Heading -->
        <div class="flex flex-wrap justify-between gap-3">
            <div class="flex flex-col gap-2">
                <h1 class="text-foreground text-4xl font-black leading-tight tracking-[-0.033em]">PHP Manager</h1>
                <p class="text-muted-foreground text-base font-normal">Configure global PHP settings, switch versions, and manage extensions.</p>
            </div>
        </div>

        <!-- Version Selection Card -->
        <Card class="rounded-xl overflow-hidden">
            <div class="flex flex-col lg:flex-row items-stretch">
                <div class="w-full lg:w-1/3 bg-primary/10 flex items-center justify-center p-8">
                    <div class="text-primary text-center">
                        <Settings :size="64" class="mx-auto mb-2" />
                        <p class="font-bold text-xl">PHP {{ currentVersion }}</p>
                        <p class="text-xs uppercase tracking-wider opacity-70">Current Active</p>
                    </div>
                </div>
                <CardContent class="flex w-full grow flex-col gap-6 p-6">
                    <div>
                        <CardTitle class="mb-2">Global Version Selection</CardTitle>
                        <p class="text-muted-foreground text-sm">Changing the version will restart the PHP-FPM service for all domains using the default profile.</p>
                    </div>
                    <div class="flex flex-col sm:flex-row items-end gap-4">
                        <label class="flex flex-col flex-1 min-w-[200px]">
                            <p class="text-foreground text-sm font-medium pb-2">Select PHP Version</p>
                            <select v-model="selectedVersion" class="flex w-full rounded-lg text-foreground border border-border bg-muted h-11 px-4 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
                                <option v-for="v in versions" :key="v.value" :value="v.value">{{ v.label }}</option>
                            </select>
                        </label>
                        <Button @click="applyVersion">
                            Apply Version
                        </Button>
                    </div>
                </CardContent>
            </div>
        </Card>

        <!-- Tabs & Filters -->
        <div class="flex flex-col md:flex-row md:items-center justify-between border-b border-border gap-4">
            <div class="flex gap-8">
                <button @click="activeTab = 'extensions'" :class="['pb-4 text-sm font-medium', activeTab === 'extensions' ? 'border-b-2 border-primary text-primary font-bold' : 'text-muted-foreground hover:text-foreground']">PHP Extensions</button>
                <button @click="activeTab = 'options'" :class="['pb-4 text-sm font-medium', activeTab === 'options' ? 'border-b-2 border-primary text-primary font-bold' : 'text-muted-foreground hover:text-foreground']">PHP Options</button>
                <button @click="activeTab = 'fpm'" :class="['pb-4 text-sm font-medium', activeTab === 'fpm' ? 'border-b-2 border-primary text-primary font-bold' : 'text-muted-foreground hover:text-foreground']">FPM Settings</button>
            </div>
            <div class="pb-3 md:pb-0 flex items-center bg-muted px-3 py-1.5 rounded-lg border border-border">
                <Search :size="16" class="text-muted-foreground mr-2" />
                <input v-model="searchQuery" class="bg-transparent border-none text-sm p-0 focus:ring-0 text-foreground w-48 placeholder:text-muted-foreground" placeholder="Search modules..." type="text" />
            </div>
        </div>

        <!-- Extensions Grid -->
        <div v-if="activeTab === 'extensions'" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            <Card v-for="ext in filteredExtensions" :key="ext.id" class="rounded-xl">
                <CardContent class="p-4 flex items-center justify-between">
                    <div class="flex flex-col">
                        <span class="text-sm font-bold text-foreground">{{ ext.name }}</span>
                        <span class="text-[10px] text-muted-foreground uppercase tracking-tighter">{{ ext.desc }}</span>
                    </div>
                    <button @click="toggleExtension(ext)" :class="['relative inline-flex h-6 w-11 items-center rounded-full transition-colors', ext.enabled ? 'bg-primary' : 'bg-muted']">
                        <span :class="['inline-block h-5 w-5 transform rounded-full bg-background border border-border shadow transition-transform', ext.enabled ? 'translate-x-5' : 'translate-x-0.5']"></span>
                    </button>
                </CardContent>
            </Card>
        </div>

        <!-- Common Directives -->
        <div v-if="activeTab === 'options'" class="border-t border-border pt-8">
            <h3 class="text-foreground text-xl font-bold mb-6">Common Directives</h3>
            <Card class="rounded-xl overflow-hidden">
                <div class="divide-y divide-border">
                    <div v-for="dir in directives" :key="dir.name" class="flex flex-col sm:flex-row sm:items-center justify-between p-4 gap-4">
                        <div class="flex-1">
                            <p class="text-sm font-bold text-foreground">{{ dir.name }}</p>
                            <p class="text-xs text-muted-foreground">{{ dir.desc }}</p>
                        </div>
                        <div v-if="dir.type === 'select'" class="w-full sm:w-32">
                            <select v-model="dir.value" class="rounded-lg border border-border bg-muted px-3 py-1.5 text-sm w-full outline-none">
                                <option v-for="opt in dir.options" :key="opt" :value="opt">{{ opt }}</option>
                            </select>
                        </div>
                        <div v-else class="flex items-center gap-2 w-full sm:w-32">
                            <Input v-model="dir.value" class="text-sm" />
                            <span class="text-xs font-bold text-muted-foreground">{{ dir.unit }}</span>
                        </div>
                    </div>
                </div>
            </Card>
        </div>

        <!-- FPM Settings Placeholder -->
        <div v-if="activeTab === 'fpm'" class="border-t border-border pt-8 text-center py-20">
            <Settings :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
            <p class="text-muted-foreground">FPM Settings coming soon...</p>
        </div>
    </div>
</MainLayout>
</template>
