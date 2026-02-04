<script setup lang="ts">
/**
 * AppInstallerPage - One-Click App Installer
 * Modern UI with hero section, category tabs, app grid, and installations table
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { appService, domainService } from '@/services'

// ==========================================
// STATE
// ==========================================
const isLoadingDomains = ref(true)
const isInstalling = ref(false)
const domains = ref<any[]>([])
const installations = ref<any[]>([])
const error = ref<string | null>(null)
const searchQuery = ref('')
const selectedCategory = ref('all')
const showInstallModal = ref(false)
const selectedApp = ref<any>(null)

// Toast
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([])
let toastId = 0
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'success') => {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 4000)
}

// Categories
const categories = [
    { id: 'all', name: 'All Applications' },
    { id: 'blogs', name: 'Blogs' },
    { id: 'cms', name: 'Portals / CMS' },
    { id: 'ecommerce', name: 'E-Commerce' },
    { id: 'forums', name: 'Forums' },
    { id: 'frameworks', name: 'Frameworks' },
]

// Apps Catalog
const apps = [
    { id: 'wordpress', name: 'WordPress', icon: 'web', iconColor: 'text-blue-600', bgColor: 'bg-blue-50 dark:bg-blue-900/30', version: '6.4.2', rating: '4.9', category: 'cms', description: "The world's most popular tool for creating any type of website." },
    { id: 'joomla', name: 'Joomla', icon: 'view_quilt', iconColor: 'text-red-600', bgColor: 'bg-red-50 dark:bg-red-900/30', version: '5.0.1', rating: '4.5', category: 'cms', description: 'A powerful, flexible open-source content management system.' },
    { id: 'laravel', name: 'Laravel', icon: 'code_blocks', iconColor: 'text-orange-600', bgColor: 'bg-orange-50 dark:bg-orange-900/30', version: '10.x', rating: '5.0', category: 'frameworks', description: 'The PHP framework for Web Artisans. Simple and elegant.' },
    { id: 'magento', name: 'Magento', icon: 'shopping_bag', iconColor: 'text-orange-500', bgColor: 'bg-orange-50 dark:bg-orange-900/30', version: '2.4.6', rating: '4.2', category: 'ecommerce', description: 'Modern eCommerce platform with infinite flexibility.' },
    { id: 'prestashop', name: 'PrestaShop', icon: 'shopping_cart', iconColor: 'text-pink-600', bgColor: 'bg-pink-50 dark:bg-pink-900/30', version: '8.1', rating: '4.3', category: 'ecommerce', description: 'A leading open-source e-commerce solution.' },
    { id: 'drupal', name: 'Drupal', icon: 'water_drop', iconColor: 'text-blue-500', bgColor: 'bg-blue-50 dark:bg-blue-900/30', version: '10.2', rating: '4.4', category: 'cms', description: 'Ambitious digital experiences for the web.' },
    { id: 'ghost', name: 'Ghost', icon: 'rocket_launch', iconColor: 'text-purple-600', bgColor: 'bg-purple-50 dark:bg-purple-900/30', version: '5.x', rating: '4.8', category: 'blogs', description: 'A powerful publishing platform for creators.' },
    { id: 'phpbb', name: 'phpBB', icon: 'forum', iconColor: 'text-indigo-600', bgColor: 'bg-indigo-50 dark:bg-indigo-900/30', version: '3.3', rating: '4.1', category: 'forums', description: 'The most widely used Open Source forum solution.' },
]

// Featured Apps
const featuredApps = [
    { id: 'nodejs', name: 'Node.js', icon: 'terminal', iconColor: 'text-primary', subtitle: 'Runtime Env' },
    { id: 'prestashop', name: 'PrestaShop', icon: 'shopping_cart', iconColor: 'text-orange-600', subtitle: 'E-Commerce' },
    { id: 'ghost', name: 'Ghost', icon: 'rocket_launch', iconColor: 'text-purple-600', subtitle: 'Modern CMS' },
]

// Form Data
const formData = ref({
    domain_id: '',
    site_title: 'My Awesome Site',
    admin_username: 'admin',
    admin_password: '',
    admin_email: '',
})

// ==========================================
// COMPUTED
// ==========================================
const filteredApps = computed(() => {
    let result = apps
    if (selectedCategory.value !== 'all') {
        result = result.filter(a => a.category === selectedCategory.value)
    }
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase()
        result = result.filter(a => a.name.toLowerCase().includes(q) || a.description.toLowerCase().includes(q))
    }
    return result
})

// ==========================================
// ACTIONS
// ==========================================
const fetchDomains = async () => {
    try {
        isLoadingDomains.value = true
        const res = await domainService.listDomains(1, 100)
        domains.value = res.data.data || []
        if (domains.value.length > 0) {
            formData.value.domain_id = domains.value[0].id
            formData.value.admin_email = `admin@${domains.value[0].domain_name}`
        }
    } catch (err) {
        console.error('Failed to load domains', err)
    } finally {
        isLoadingDomains.value = false
    }
}

const fetchInstallations = async () => {
    try {
        const res = await appService.listInstallations()
        installations.value = res.data.data || []
    } catch (err) {
        console.error('Failed to load installations', err)
    }
}

const openInstallModal = (app: any) => {
    selectedApp.value = app
    showInstallModal.value = true
    generatePassword()
}

const installApp = async () => {
    if (!formData.value.domain_id || !formData.value.admin_password || !selectedApp.value) return

    try {
        isInstalling.value = true
        error.value = null
        await appService.installApp({
            app_type: selectedApp.value.id,
            ...formData.value,
        })
        showToast(`${selectedApp.value.name} installed successfully!`, 'success')
        showInstallModal.value = false
        fetchInstallations()
    } catch (err: any) {
        showToast(err.response?.data?.message || 'Installation failed.', 'error')
    } finally {
        isInstalling.value = false
    }
}

const generatePassword = () => {
    formData.value.admin_password =
        Math.random().toString(36).slice(-12) +
        Math.random().toString(36).slice(-4).toUpperCase() + '!'
}

onMounted(() => {
    fetchDomains()
    fetchInstallations()
})
</script>

<template>
<MainLayout>
    <div class="font-display text-[#0d131b] dark:text-slate-200">

        <!-- =========================================================================
             HERO SECTION
             ========================================================================= -->
        <div class="bg-gradient-to-br from-primary to-blue-700 py-12 px-6 lg:px-10 -mx-6 -mt-6 mb-10 rounded-b-3xl">
            <div class="max-w-[1200px] mx-auto text-center">
                <h2 class="text-white text-3xl md:text-4xl font-black tracking-tight mb-4">One-Click App Installer</h2>
                <p class="text-blue-100 text-lg mb-8 max-w-2xl mx-auto">Deploy your favorite applications in seconds with pre-configured settings and automated security.</p>
                
                <!-- Search Bar -->
                <div class="relative max-w-2xl mx-auto mb-10">
                    <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                        <span class="material-symbols-outlined text-[#4c6c9a] text-2xl">search</span>
                    </div>
                    <input v-model="searchQuery" class="block w-full pl-12 pr-4 py-4 border-none bg-white dark:bg-slate-800 rounded-xl text-base focus:ring-4 focus:ring-primary/20 shadow-xl dark:text-white placeholder:text-[#4c6c9a]" placeholder="Search for apps like WordPress, Joomla, Laravel..." type="text"/>
                </div>

                <!-- Featured Apps -->
                <div class="flex flex-col gap-4">
                    <p class="text-white/80 text-sm font-bold uppercase tracking-widest">Featured Apps</p>
                    <div class="flex gap-4 overflow-x-auto hide-scrollbar pb-4 -mx-4 px-4 justify-center">
                        <div v-for="app in featuredApps" :key="app.id" 
                            @click="openInstallModal(apps.find(a => a.id === app.id) || app)"
                            class="flex-shrink-0 bg-white/10 backdrop-blur-md border border-white/20 p-4 rounded-xl flex items-center gap-4 w-64 text-left hover:bg-white/20 transition-all cursor-pointer">
                            <div class="size-12 bg-white rounded-lg flex items-center justify-center">
                                <span class="material-symbols-outlined text-3xl" :class="app.iconColor">{{ app.icon }}</span>
                            </div>
                            <div>
                                <p class="text-white font-bold">{{ app.name }}</p>
                                <p class="text-blue-100 text-xs">{{ app.subtitle }}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- =========================================================================
             CATEGORY TABS
             ========================================================================= -->
        <div class="max-w-[1200px] mx-auto">
            <div class="flex border-b border-[#cfd9e7] dark:border-slate-800 mb-8 overflow-x-auto hide-scrollbar">
                <button v-for="cat in categories" :key="cat.id"
                    @click="selectedCategory = cat.id"
                    :class="[
                        'px-6 py-4 font-semibold text-sm whitespace-nowrap transition-colors',
                        selectedCategory === cat.id 
                            ? 'text-primary border-b-2 border-primary font-bold' 
                            : 'text-[#4c6c9a] hover:text-primary'
                    ]">
                    {{ cat.name }}
                </button>
            </div>

            <!-- =========================================================================
                 APP GRID
                 ========================================================================= -->
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-12">
                <div v-for="app in filteredApps" :key="app.id" 
                    class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl p-6 shadow-sm hover:shadow-md transition-all flex flex-col">
                    <div class="flex justify-between items-start mb-4">
                        <div class="size-14 rounded-xl flex items-center justify-center" :class="app.bgColor">
                            <span class="material-symbols-outlined text-4xl" :class="app.iconColor">{{ app.icon }}</span>
                        </div>
                        <span class="bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 text-[10px] font-bold px-2 py-0.5 rounded uppercase">v{{ app.version }}</span>
                    </div>
                    <h3 class="text-[#0d131b] dark:text-white font-bold text-lg mb-2">{{ app.name }}</h3>
                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm line-clamp-2 mb-6">{{ app.description }}</p>
                    <div class="mt-auto flex items-center justify-between">
                        <span class="text-xs text-[#4c6c9a] flex items-center gap-1">
                            <span class="material-symbols-outlined text-sm">star</span> {{ app.rating }}/5
                        </span>
                        <button @click="openInstallModal(app)" class="bg-primary text-white px-4 py-2 rounded-lg text-sm font-bold hover:bg-blue-700 transition-colors">Install</button>
                    </div>
                </div>

                <!-- Empty State -->
                <div v-if="filteredApps.length === 0" class="col-span-full text-center py-12">
                    <span class="material-symbols-outlined text-6xl text-[#cfd9e7] dark:text-slate-700 mb-4">search_off</span>
                    <p class="text-[#4c6c9a] dark:text-slate-400">No apps found matching your search.</p>
                </div>
            </div>

            <!-- =========================================================================
                 RECENT INSTALLATIONS TABLE
                 ========================================================================= -->
            <div class="bg-white dark:bg-slate-800 border border-[#cfd9e7] dark:border-slate-700 rounded-xl shadow-sm overflow-hidden mb-10">
                <div class="px-6 py-5 border-b border-[#cfd9e7] dark:border-slate-700 flex items-center justify-between">
                    <div>
                        <h2 class="text-[#0d131b] dark:text-white text-lg font-bold">Recent Installations</h2>
                        <p class="text-[#4c6c9a] dark:text-slate-400 text-xs">Manage your currently active applications</p>
                    </div>
                    <button class="text-primary text-sm font-bold hover:underline">View All</button>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-slate-50 dark:bg-slate-900/50">
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Application</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">URL</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Version</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700">Status</th>
                                <th class="px-6 py-4 text-[#4c6c9a] dark:text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-[#cfd9e7] dark:border-slate-700 text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-[#cfd9e7] dark:divide-slate-700">
                            <tr v-for="inst in installations" :key="inst.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="flex items-center gap-3">
                                        <div class="size-8 rounded bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                                            <span class="material-symbols-outlined text-blue-600 text-lg">web</span>
                                        </div>
                                        <span class="text-sm font-semibold text-[#0d131b] dark:text-white">{{ inst.app_type }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <a class="text-primary hover:underline text-sm font-medium" :href="'https://' + inst.domain_name" target="_blank">{{ inst.domain_name }}</a>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-[#4c6c9a] dark:text-slate-400">{{ inst.version || '-' }}</td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase"
                                        :class="inst.status === 'active' ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'">
                                        {{ inst.status || 'Active' }}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-right space-x-2">
                                    <button class="text-primary hover:text-blue-700 text-xs font-bold px-3 py-1 bg-primary/10 rounded">Manage</button>
                                    <button class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white p-1 align-middle">
                                        <span class="material-symbols-outlined text-lg">settings</span>
                                    </button>
                                </td>
                            </tr>
                            <!-- Empty State -->
                            <tr v-if="installations.length === 0">
                                <td colspan="5" class="px-6 py-12 text-center">
                                    <span class="material-symbols-outlined text-5xl text-[#cfd9e7] dark:text-slate-700 mb-2">apps</span>
                                    <p class="text-[#4c6c9a] dark:text-slate-400 text-sm">No applications installed yet. Click "Install" on any app above to get started.</p>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <!-- =========================================================================
             INSTALL MODAL
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
        <div v-if="showInstallModal" @click="showInstallModal = false"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 animate-in fade-in duration-200" />

        <!-- Install Modal -->
        <div v-if="showInstallModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div @click.stop class="bg-white dark:bg-slate-800 rounded-xl shadow-2xl w-full max-w-lg animate-in zoom-in-95 duration-200">
                <div class="px-6 py-4 border-b border-[#cfd9e7] dark:border-slate-700 flex justify-between items-center">
                    <div class="flex items-center gap-3">
                        <div class="size-10 rounded-lg flex items-center justify-center" :class="selectedApp?.bgColor">
                            <span class="material-symbols-outlined text-2xl" :class="selectedApp?.iconColor">{{ selectedApp?.icon }}</span>
                        </div>
                        <div>
                            <h3 class="text-lg font-bold text-[#0d131b] dark:text-white">Install {{ selectedApp?.name }}</h3>
                            <p class="text-xs text-[#4c6c9a]">Version {{ selectedApp?.version }}</p>
                        </div>
                    </div>
                    <button @click="showInstallModal = false" class="text-[#4c6c9a] hover:text-[#0d131b] dark:hover:text-white">
                        <span class="material-symbols-outlined">close</span>
                    </button>
                </div>
                <div class="p-6 space-y-4">
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Domain *</label>
                        <select v-model="formData.domain_id" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white">
                            <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
                        </select>
                        <p v-if="domains.length === 0 && !isLoadingDomains" class="text-xs text-red-500 mt-1">No domains found. Please add a domain first.</p>
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Site Title</label>
                        <input v-model="formData.site_title" type="text" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Admin Username</label>
                            <input v-model="formData.admin_username" type="text" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                        </div>
                        <div>
                            <div class="flex justify-between mb-2">
                                <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider">Admin Password</label>
                                <button type="button" @click="generatePassword" class="text-xs text-primary hover:underline font-medium">Generate</button>
                            </div>
                            <input v-model="formData.admin_password" type="text" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white font-mono" />
                        </div>
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-[#4c6c9a] uppercase tracking-wider mb-2">Admin Email</label>
                        <input v-model="formData.admin_email" type="email" class="w-full px-4 py-2 border border-[#cfd9e7] dark:border-slate-700 bg-white dark:bg-slate-900 rounded-lg text-sm focus:ring-2 focus:ring-primary dark:text-white" />
                    </div>
                </div>
                <div class="px-6 py-4 border-t border-[#cfd9e7] dark:border-slate-700 flex justify-end gap-3">
                    <button @click="showInstallModal = false" class="px-4 py-2 text-sm font-medium text-[#4c6c9a] hover:text-[#0d131b]">Cancel</button>
                    <button @click="installApp" :disabled="isInstalling || !formData.domain_id" class="flex items-center gap-2 px-5 py-2 bg-primary text-white rounded-lg text-sm font-bold hover:bg-blue-700 disabled:opacity-50 transition-all">
                        <span v-if="isInstalling" class="material-symbols-outlined text-lg animate-spin">progress_activity</span>
                        {{ isInstalling ? 'Installing...' : 'Install Now' }}
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

.font-display {
    font-family: 'Inter', sans-serif;
}
.material-symbols-outlined {
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
}
.hide-scrollbar::-webkit-scrollbar { display: none; }
.hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
.line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
</style>
