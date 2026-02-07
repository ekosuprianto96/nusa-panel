<script setup lang="ts">
/**
 * NpmManagerPage - NPM Package Manager with NVM & Env Vars
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { useToastStore } from '@/stores/toast'
import { nodejsService } from '@/services/nodejs.service'
import { useAuthStore } from '@/stores/auth'
import {
    Download, Play, Search, ArrowUp, Trash2, Terminal, 
    Plus, RefreshCw, Box, Eye, EyeOff,
    Cpu, Settings
} from 'lucide-vue-next'

const toast = useToastStore()
const authStore = useAuthStore()
const router = useRouter()

// ==========================================
// STATE
// ==========================================
const productionMode = ref(true)
const searchQuery = ref('')
const projectDirectory = ref('')
const activeNodeVersion = ref('Unknown')
const nvmInstalled = ref(false)
const isBootstrapping = ref(false)



const nodeVersions = ref<{ value: string; label: string }[]>([])

const packages = ref<any[]>([])
const isLoadingPackages = ref(false)
const installPackageName = ref('')
const installPackageVersion = ref('')
const installDevDependency = ref(false)
const showVersionModal = ref(false)
const availableVersions = ref<string[]>([])
const isLoadingVersions = ref(false)

const fetchPackages = async () => {
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    isLoadingPackages.value = true
    try {
        const res = await nodejsService.npmList(projectDirectory.value)
        // Transform backend data to match UI needs
        packages.value = res.data.data.map(p => ({
            name: p.name,
            current: p.version,
            latest: 'latest', // TODO: Fetch latest version if possible or use helper
            type: p.type,
            hasUpdate: false // Logic to check updates could be added later
        }))
    } catch (e) {
        toast.error('Failed to fetch packages')
    } finally {
        isLoadingPackages.value = false
    }
}

interface EnvVar {
    key: string;
    value: string;
    show: boolean;
}

const envVars = ref<EnvVar[]>([])
const isLoadingEnv = ref(false)
const showAddEnvModal = ref(false)
const newEnv = ref({ key: '', value: '' })

const consoleOutput = ref([
    { type: 'cmd', text: '$ nvm use v20.10.0' },
    { type: 'info', text: 'Now using node v20.10.0 (npm v10.2.3)' },
    { type: 'info', text: 'Ready to manage packages.' }
])

// ==========================================
// ACTIONS
// ==========================================

// --- Env Vars Integration ---
const fetchEnvVars = async () => {
    if (!projectDirectory.value) return
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    isLoadingEnv.value = true
    try {
        const res = await nodejsService.getEnvVars(projectDirectory.value)
        envVars.value = res.data.data.map(line => {
             const [key, ...rest] = line.split('=')
             return { key: (key || '').trim(), value: rest.join('=').trim(), show: false }
        })
    } catch (e) {
        toast.error('Failed to load env vars. Check path.')
        envVars.value = []
    } finally {
        isLoadingEnv.value = false
    }
}

const isValidEnvKey = (key: string) => /^[A-Z_][A-Z0-9_]*$/.test(key)
const isValidEnvValue = (value: string) => value.trim().length > 0

const saveEnvVar = async () => {
    if (!newEnv.value.key || !projectDirectory.value) return
    if (!isValidEnvKey(newEnv.value.key)) {
        toast.error('Invalid key. Use letters, numbers, and underscores only.')
        return
    }
    if (!isValidEnvValue(newEnv.value.value)) {
        toast.error('Value cannot be empty')
        return
    }
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    try {
        await nodejsService.saveEnvVar(projectDirectory.value, newEnv.value.key, newEnv.value.value)
        toast.success('Environment variable saved')
        showAddEnvModal.value = false
        newEnv.value = { key: '', value: '' }
        await fetchEnvVars()
    } catch (e) {
        toast.error('Failed to save variable')
    }
}

const deleteEnvVar = async (key: string) => {
    if (!confirm(`Delete variable ${key}?`)) return
    try {
        if (!isValidProjectPath(projectDirectory.value)) {
            toast.error('Invalid project path. Use /home/user_<username>/...')
            return
        }
        await nodejsService.deleteEnvVar(projectDirectory.value, key)
        toast.success('Variable deleted')
        await fetchEnvVars()
    } catch (e) {
        toast.error('Failed to delete variable')
    }
}
// ----------------------------

const normalizeVersion = (ver: string) => ver.replace(/^v/, '').trim()
const isValidNodeVersion = (ver: string) => /^\d+\.\d+\.\d+$/.test(ver)

const installDependencies = async () => {
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    toast.info('Installing dependencies...')
    consoleOutput.value.push({ type: 'cmd', text: '$ npm install' })
    try {
        await nodejsService.npmInstallAll(projectDirectory.value)
        consoleOutput.value.push({ type: 'success', text: 'All dependencies installed successfully!' })
        toast.success('Dependencies installed')
        await fetchPackages()
    } catch (e: any) {
        consoleOutput.value.push({ type: 'error', text: e.response?.data?.message || 'Install failed' })
        toast.error('Install failed')
    }
}

const npmStart = () => {
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    toast.info('Starting application...')
    consoleOutput.value.push({ type: 'cmd', text: '$ npm start' })
    nodejsService.npmRun('start', projectDirectory.value)
        .then(() => {
            consoleOutput.value.push({ type: 'success', text: 'npm start executed' })
            toast.success('Application started')
        })
        .catch((e: any) => {
            consoleOutput.value.push({ type: 'error', text: e.response?.data?.message || 'Start failed' })
            toast.error('Start failed')
        })
}

const updatePackage = async (pkg: any) => {
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    toast.info(`Updating ${pkg.name}...`)
    consoleOutput.value.push({ type: 'cmd', text: `$ npm install ${pkg.name}@latest` })
    try {
        await nodejsService.npmInstall(pkg.name, 'latest', pkg.type === 'dev-dep', projectDirectory.value)
        pkg.hasUpdate = false
        consoleOutput.value.push({ type: 'success', text: `${pkg.name} updated` })
        toast.success(`${pkg.name} updated`)
        await fetchPackages()
    } catch (e: any) {
        consoleOutput.value.push({ type: 'error', text: 'Update failed' })
        toast.error('Update failed')
    }
}

const uninstallPackage = async (pkg: any) => {
    if (!confirm(`Are you sure you want to uninstall ${pkg.name}?`)) return
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    consoleOutput.value.push({ type: 'cmd', text: `$ npm uninstall ${pkg.name}` })
    try {
        await nodejsService.npmUninstall(pkg.name, projectDirectory.value)
        packages.value = packages.value.filter(p => p.name !== pkg.name)
        toast.success(`${pkg.name} uninstalled`)
    } catch (e: any) {
         consoleOutput.value.push({ type: 'error', text: 'Uninstall failed' })
         toast.error('Uninstall failed')
    }
}

const clearLogs = () => {
    consoleOutput.value = []
    toast.info('Console cleared')
}

const installNodeVersion = async () => {
    if (!nvmInstalled.value) return
    showVersionModal.value = true
    availableVersions.value = []
    await fetchAvailableVersions()
}

const setDefaultVersion = async () => {
    const version = normalizeVersion(activeNodeVersion.value)
    if (!isValidNodeVersion(version)) {
        toast.error('Select a valid Node.js version first')
        return
    }
    try {
        await nodejsService.setDefault(version)
        toast.success(`Default Node.js set to v${version}`)
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to set default')
    }
}

const isValidProjectPath = (path: string) => {
    return path.startsWith('/home/user_') && !path.includes('..')
}

const installPackage = async () => {
    if (!installPackageName.value) {
        toast.error('Package name is required')
        return
    }
    if (!isValidProjectPath(projectDirectory.value)) {
        toast.error('Invalid project path. Use /home/user_<username>/...')
        return
    }
    const pkg = installPackageName.value.trim()
    const version = installPackageVersion.value.trim() || undefined
    try {
        toast.info(`Installing ${pkg}...`)
        consoleOutput.value.push({ type: 'cmd', text: `$ npm install ${pkg}${version ? '@' + version : ''}${installDevDependency.value ? ' --save-dev' : ''}` })
        await nodejsService.npmInstall(pkg, version, installDevDependency.value, projectDirectory.value)
        toast.success('Package installed')
        installPackageName.value = ''
        installPackageVersion.value = ''
        installDevDependency.value = false
        await fetchPackages()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Install failed')
    }
}

const fetchNodeStatus = async () => {
    try {
        const res = await nodejsService.getStatus()
        const status = res.data.data
        nvmInstalled.value = !!status?.nvm_installed
        if (status?.current_version) {
            activeNodeVersion.value = `v${status.current_version}`
        }

        const installed = (status?.installed_versions || []).map(v => ({
            value: `v${v}`,
            label: `v${v} (Installed)`
        }))

        const lts = nvmInstalled.value
            ? await nodejsService.getAvailableVersions().catch(() => null)
            : null
        const ltsList = (lts?.data?.data || []).slice(0, 6).map((v: string) => ({
            value: `v${v}`,
            label: `v${v} (LTS)`
        }))

        const combined = [...installed, ...ltsList].filter((v, i, arr) => arr.findIndex(x => x.value === v.value) === i)
        nodeVersions.value = combined.length > 0 ? combined : [
            { value: 'v20.10.0', label: 'v20.10.0 (Latest LTS)' },
            { value: 'v18.18.2', label: 'v18.18.2 (LTS)' }
        ]

        if (!status?.current_version) {
            activeNodeVersion.value = nodeVersions.value[0]?.value || 'v20.10.0'
        }
    } catch (e) {
        nvmInstalled.value = false
        nodeVersions.value = [
            { value: 'v20.10.0', label: 'v20.10.0 (Latest LTS)' },
            { value: 'v18.18.2', label: 'v18.18.2 (LTS)' }
        ]
        activeNodeVersion.value = 'v20.10.0'
    }
}

const fetchAvailableVersions = async () => {
    isLoadingVersions.value = true
    try {
        const res = await nodejsService.getAvailableVersions()
        availableVersions.value = res.data.data || []
    } catch (e) {
        availableVersions.value = []
        toast.error('Failed to load available versions')
    } finally {
        isLoadingVersions.value = false
    }
}

const installSelectedVersion = async (ver: string) => {
    const version = normalizeVersion(ver)
    if (!isValidNodeVersion(version)) {
        toast.error('Invalid Node.js version')
        return
    }
    showVersionModal.value = false
    toast.info(`Installing Node.js ${version}...`)
    consoleOutput.value.push({ type: 'cmd', text: `$ nvm install ${version}` })
    try {
        await nodejsService.installVersion(version)
        consoleOutput.value.push({ type: 'success', text: `Node.js ${version} installed` })
        toast.success('Node.js installed')
        await fetchNodeStatus()
    } catch (e: any) {
        consoleOutput.value.push({ type: 'error', text: e.response?.data?.message || 'Install failed' })
        toast.error('Install failed')
    }
}

const bootstrapNvm = async () => {
    isBootstrapping.value = true
    try {
        const version = normalizeVersion(activeNodeVersion.value)
        const safeVersion = isValidNodeVersion(version) ? version : undefined
        await nodejsService.bootstrap(safeVersion)
        toast.success(`NVM installed & Node.js v${version} ready`)
        await fetchNodeStatus()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to install NVM')
    } finally {
        isBootstrapping.value = false
    }
}

// ==========================================
// COMPUTED
// ==========================================
const filteredPackages = computed(() => {
    if (!searchQuery.value) return packages.value
    return packages.value.filter(p => p.name.toLowerCase().includes(searchQuery.value.toLowerCase()))
})

onMounted(async () => {
    if (!authStore.user) {
        await authStore.fetchMe()
    }
    const username = authStore.user?.username || 'user'
    projectDirectory.value = `/home/user_${username}`
    fetchEnvVars()
    fetchPackages()
    fetchNodeStatus()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-7xl mx-auto space-y-8 animate-in pb-20">
        
        <!-- Header -->
        <div class="mb-4">
            <AppBreadcrumb
                class="mb-2"
                :items="[
                    { label: 'System Tools', icon: Settings, onClick: () => router.push('/dashboard/system') },
                    { label: 'NPM Manager', current: true }
                ]"
            />
            <h2 class="text-3xl font-black text-foreground tracking-tight mb-2">NPM & Node Version Manager</h2>
            <p class="text-muted-foreground max-w-2xl text-base">
                Manage your Node.js application environments, dependencies, and run NPM scripts directly from your dashboard.
            </p>
        </div>

        <!-- Node Version & Env Panel -->
        <div class="bg-card border border-border p-6 rounded-xl shadow-sm flex flex-col lg:flex-row lg:items-center justify-between gap-6">
            <div class="flex items-center gap-4">
                <div class="bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 w-12 h-12 rounded-xl flex items-center justify-center shrink-0">
                    <Cpu class="w-7 h-7" />
                </div>
                <div>
                    <p class="text-xs font-bold text-muted-foreground uppercase tracking-wider mb-0.5">Active Node.js Version</p>
                    <h3 class="text-2xl font-black text-foreground flex items-center gap-2">
                        {{ activeNodeVersion }}
                        <span class="bg-emerald-500 w-2 h-2 rounded-full animate-pulse"></span>
                    </h3>
                </div>
            </div>
            <div class="flex flex-wrap items-center gap-3">
                <div class="relative min-w-[200px]">
                    <select 
                        v-model="activeNodeVersion"
                        :disabled="!nvmInstalled"
                        class="w-full h-11 pl-4 pr-10 rounded-lg border border-border bg-muted/50 text-sm font-semibold focus:ring-2 focus:ring-primary/20 focus:border-primary appearance-none outline-none transition-all"
                    >
                        <option v-for="ver in nodeVersions" :key="ver.value" :value="ver.value">{{ ver.label }}</option>
                    </select>
                    <RefreshCw class="absolute right-3 top-3.5 w-4 h-4 text-muted-foreground pointer-events-none" />
                </div>
                <button 
                    v-if="!nvmInstalled"
                    @click="bootstrapNvm"
                    :disabled="isBootstrapping"
                    class="bg-emerald-600 text-white font-bold py-2.5 px-4 rounded-lg text-sm hover:bg-emerald-700 transition-all"
                >
                    {{ isBootstrapping ? 'Installing...' : 'Install NVM' }}
                </button>
                <button 
                    @click="setDefaultVersion"
                    :disabled="!nvmInstalled"
                    class="bg-primary text-primary-foreground font-bold py-2.5 px-4 rounded-lg text-sm hover:bg-primary/90 transition-all"
                >
                    Set Default
                </button>
                <button 
                    @click="installNodeVersion"
                    :disabled="!nvmInstalled"
                    class="bg-foreground text-background font-bold py-2.5 px-5 rounded-lg text-sm flex items-center gap-2 hover:opacity-90 transition-all"
                >
                    <Plus class="w-4 h-4" />
                    Install New Version
                </button>
            </div>
        </div>

        <!-- Project Controls -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-1 bg-card border border-border p-6 rounded-xl shadow-sm">
                <label class="block text-sm font-bold text-foreground mb-2">Project Directory</label>
                <div class="relative">
                     <!-- Allow editing the path to find .env -->
                    <div class="flex gap-2">
                         <input v-model="projectDirectory" type="text" class="w-full h-11 px-4 rounded-lg border border-border bg-background text-sm font-medium focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" placeholder="/path/to/project" />
                         <button @click="fetchEnvVars" class="bg-muted hover:bg-muted/80 text-foreground px-3 rounded-lg"><RefreshCw class="w-4 h-4"/></button>
                    </div>
                </div>
            </div>
            <div class="lg:col-span-2 bg-card border border-border p-6 rounded-xl shadow-sm flex flex-col md:flex-row items-center justify-between gap-6">
                <div class="flex items-center gap-4 w-full md:w-auto">
                    <button @click="installDependencies" :disabled="!nvmInstalled" class="flex-1 md:flex-none bg-primary hover:bg-primary/90 text-primary-foreground font-bold py-3 px-6 rounded-lg flex items-center justify-center gap-2 transition-all shadow-lg shadow-primary/20 disabled:opacity-60">
                        <Download class="w-4 h-4" />
                        Install Dependencies
                    </button>
                    <button @click="npmStart" :disabled="!nvmInstalled" class="flex-1 md:flex-none bg-muted hover:bg-muted/80 text-foreground font-bold py-3 px-6 rounded-lg flex items-center justify-center gap-2 transition-all disabled:opacity-60">
                        <Play class="w-4 h-4" />
                        npm start
                    </button>
                </div>
                <div class="flex items-center gap-3 w-full md:w-auto justify-between md:justify-end">
                    <span class="text-sm font-semibold text-muted-foreground">Production Mode</span>
                    <button 
                        @click="productionMode = !productionMode" 
                        class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary/50"
                        :class="productionMode ? 'bg-primary' : 'bg-muted'"
                    >
                        <span class="sr-only">Toggle production mode</span>
                        <span 
                            class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                            :class="productionMode ? 'translate-x-6' : 'translate-x-1'"
                        ></span>
                    </button>
                </div>
            </div>
        </div>

        <!-- Package Management -->
        <div class="space-y-4">
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <h3 class="text-xl font-bold text-foreground tracking-tight">Manage Packages</h3>
                <div class="w-full md:w-96 relative group">
                    <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <input 
                        v-model="searchQuery" 
                        class="block w-full h-11 pl-10 pr-4 rounded-lg border border-border bg-card shadow-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none text-sm" 
                        placeholder="Search packages..." 
                        type="text" 
                    />
                </div>
            </div>

            <!-- Install Package Panel -->
            <div class="bg-card border border-border rounded-xl p-4 flex flex-col md:flex-row gap-3 items-stretch">
                <input
                    v-model="installPackageName"
                    type="text"
                    :disabled="!nvmInstalled"
                    class="h-11 px-3 rounded-lg border border-border bg-background text-sm font-medium focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    placeholder="package-name"
                />
                <input
                    v-model="installPackageVersion"
                    type="text"
                    :disabled="!nvmInstalled"
                    class="h-11 px-3 rounded-lg border border-border bg-background text-sm font-medium focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    placeholder="version (optional)"
                />
                <button
                    @click="installDevDependency = !installDevDependency"
                    :disabled="!nvmInstalled"
                    class="h-11 px-4 rounded-lg text-sm font-bold border transition-colors"
                    :class="installDevDependency ? 'bg-amber-100 text-amber-700 border-amber-200 dark:bg-amber-900/30 dark:text-amber-300 dark:border-amber-800' : 'bg-muted text-muted-foreground border-border'"
                >
                    Dev Dependency
                </button>
                <button
                    @click="installPackage"
                    :disabled="!nvmInstalled"
                    class="h-11 px-5 rounded-lg bg-primary text-primary-foreground font-bold text-sm hover:bg-primary/90 transition-colors"
                >
                    Install
                </button>
            </div>

            <div class="bg-card border border-border rounded-xl overflow-hidden shadow-sm">
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-muted/30 border-b border-border">
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Package Name</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Current</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Latest</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Type</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <tr v-for="pkg in filteredPackages" :key="pkg.name" class="hover:bg-muted/30 transition-colors group">
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <Box class="w-4 h-4 text-muted-foreground" />
                                        <span class="font-bold text-foreground">{{ pkg.name }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-sm text-muted-foreground font-mono">{{ pkg.current }}</td>
                                <td class="px-6 py-4 text-sm font-mono" :class="pkg.hasUpdate ? 'text-emerald-500 font-bold' : 'text-muted-foreground'">{{ pkg.latest }}</td>
                                <td class="px-6 py-4">
                                    <span 
                                        class="px-2 py-0.5 text-[10px] font-bold rounded-md uppercase border"
                                        :class="pkg.type === 'dependency' 
                                            ? 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-900' 
                                            : 'bg-muted text-muted-foreground border-border'"
                                    >
                                        {{ pkg.type }}
                                    </span>
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button 
                                            @click="updatePackage(pkg)" 
                                            class="p-2 rounded-lg transition-colors"
                                            :class="pkg.hasUpdate ? 'text-primary hover:bg-primary/10' : 'text-muted-foreground hover:bg-muted'" 
                                            title="Update"
                                        >
                                            <ArrowUp class="w-4 h-4" />
                                        </button>
                                        <button 
                                            @click="uninstallPackage(pkg)" 
                                            class="p-2 text-muted-foreground hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-colors" 
                                            title="Uninstall"
                                        >
                                            <Trash2 class="w-4 h-4" />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <!-- Environment Variables -->
        <div class="space-y-4">
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div class="flex flex-col">
                    <h3 class="text-xl font-bold text-foreground tracking-tight">Environment Variables</h3>
                    <p class="text-sm text-muted-foreground">Configure secrets and runtime settings for your application.</p>
                </div>
                <button @click="showAddEnvModal = true" class="bg-primary hover:bg-primary/90 text-primary-foreground font-bold py-2 px-4 rounded-lg text-sm flex items-center gap-2 transition-all shadow-sm">
                    <Plus class="w-4 h-4" />
                    Add Variable
                </button>
            </div>

            <div class="bg-card border border-border rounded-xl overflow-hidden shadow-sm">
                 <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-muted/30 border-b border-border">
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Key</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Value</th>
                                <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                             <tr v-if="isLoadingEnv"><td colspan="3" class="px-6 py-4 text-center">Loading...</td></tr>
                             <tr v-else-if="envVars.length === 0"><td colspan="3" class="px-6 py-4 text-center text-muted-foreground">No environment variables found or path invalid.</td></tr>
                            <tr v-else v-for="env in envVars" :key="env.key" class="hover:bg-muted/30 transition-colors group">
                                <td class="px-6 py-4">
                                    <code class="text-xs font-bold text-foreground bg-muted px-1.5 py-0.5 rounded border border-border">{{ env.key }}</code>
                                </td>
                                <td class="px-6 py-4">
                                    <div class="flex items-center gap-2">
                                        <span class="text-sm font-mono text-muted-foreground">{{ env.show ? env.value : '••••••••••••••••' }}</span>
                                        <button @click="env.show = !env.show" class="text-muted-foreground hover:text-primary transition-colors">
                                            <component :is="env.show ? EyeOff : Eye" class="w-4 h-4" />
                                        </button>
                                    </div>
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <div class="flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <button @click="deleteEnvVar(env.key)" class="p-2 text-muted-foreground hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-colors" title="Delete">
                                            <Trash2 class="w-4 h-4" />
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                 </div>
            </div>
        </div>

        <!-- CLI Console -->
        <div class="space-y-4">
            <div class="flex items-center justify-between">
                <h3 class="text-xl font-bold text-foreground tracking-tight">NPM Output Console</h3>
                <button @click="clearLogs" class="text-xs font-bold text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors">
                    <Terminal class="w-3.5 h-3.5" /> Clear Logs
                </button>
            </div>
            
            <div class="bg-[#0f172a] rounded-xl border border-border/50 shadow-2xl overflow-hidden flex flex-col h-80">
                <div class="flex items-center justify-between px-4 py-2 bg-[#1e293b] border-b border-border/10">
                    <div class="flex gap-1.5">
                        <div class="w-2.5 h-2.5 rounded-full bg-red-500/50"></div>
                        <div class="w-2.5 h-2.5 rounded-full bg-amber-500/50"></div>
                        <div class="w-2.5 h-2.5 rounded-full bg-emerald-500/50"></div>
                    </div>
                    <span class="text-[10px] font-mono text-slate-500 uppercase tracking-widest">bash — npm</span>
                </div>
                <div class="flex-1 p-4 font-mono text-xs leading-relaxed overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
                     <p v-for="(line, index) in consoleOutput" :key="index" :class="[
                        'mb-1', 
                        line.type === 'cmd' ? 'text-slate-500' : 
                        line.type === 'success' ? 'text-emerald-400 font-bold' : 
                        line.type === 'info' ? 'text-blue-400' :
                        'text-slate-300'
                    ]">
                        {{ line.text }}
                    </p>
                    <p class="text-white mt-4 flex items-center gap-2 animate-pulse">
                        <span class="bg-emerald-500 w-1.5 h-1.5 rounded-full"></span>
                        Ready to serve...
                    </p>
                </div>
            </div>
        </div>

    </div>

    <!-- Install Node Version Modal -->
    <div v-if="showVersionModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
        <div class="bg-card rounded-xl shadow-2xl w-full max-w-md p-6 border border-border">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-bold text-foreground">Select Node.js Version</h3>
                <button @click="showVersionModal = false" class="text-muted-foreground hover:text-foreground">Close</button>
            </div>
            <div v-if="isLoadingVersions" class="py-6 text-center text-sm text-muted-foreground">Loading versions...</div>
            <div v-else class="max-h-72 overflow-y-auto custom-scrollbar space-y-2">
                <button
                    v-for="ver in availableVersions"
                    :key="ver"
                    @click="installSelectedVersion(ver)"
                    class="w-full text-left px-3 py-2 rounded-lg border border-border hover:bg-muted/50 text-sm font-semibold"
                >
                    v{{ ver }} (LTS)
                </button>
                <div v-if="availableVersions.length === 0" class="text-sm text-muted-foreground text-center py-4">
                    No versions available.
                </div>
            </div>
        </div>
    </div>

    <!-- Add Env Modal -->
    <div v-if="showAddEnvModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
        <div class="bg-card rounded-xl shadow-2xl w-full max-w-md p-6 border border-border">
            <h3 class="text-lg font-bold mb-4 text-foreground">Add Environment Variable</h3>
            <div class="space-y-4">
                <div>
                    <label class="block text-xs font-bold text-muted-foreground mb-1">Key</label>
                    <input v-model="newEnv.key" type="text" class="w-full rounded-lg border border-border bg-background h-11 px-3 text-sm font-medium focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" placeholder="API_KEY">
                </div>
                <div>
                    <label class="block text-xs font-bold text-muted-foreground mb-1">Value</label>
                    <input v-model="newEnv.value" type="text" class="w-full rounded-lg border border-border bg-background h-11 px-3 text-sm font-medium focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" placeholder="secret_value">
                </div>
                <div class="flex justify-end gap-2 mt-6">
                    <button @click="showAddEnvModal = false" class="px-4 py-2 text-muted-foreground hover:text-foreground font-bold text-sm">Cancel</button>
                    <button @click="saveEnvVar" class="px-4 py-2 bg-primary text-primary-foreground rounded-lg font-bold text-sm">Save</button>
                </div>
            </div>
        </div>
    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeUp 0.5s ease-out forwards; }
@keyframes fadeUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

/* Custom scrollbar for console */
.scrollbar-thin::-webkit-scrollbar {
    width: 6px;
}
.scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
}
.scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: #334155;
    border-radius: 20px;
}
</style>
