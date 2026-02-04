<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { packageService } from '@/services'
import type { Package, CreatePackageRequest, UpdatePackageRequest } from '@/types'
import { Package as PackageIcon, Plus, Edit, Trash2, Database, Globe, Mail, HardDrive, Wifi, Star } from 'lucide-vue-next'
import { useToastStore } from '@/stores/toast'
import BaseModal from '@/components/ui/BaseModal.vue'

const toast = useToastStore()

// State
const packages = ref<Package[]>([])
const isLoading = ref(true)
const showFormModal = ref(false)
const showDeleteModal = ref(false)
const selectedPackage = ref<Package | null>(null)
const isEditing = ref(false)
const isSubmitting = ref(false)

// Form
const formData = ref<CreatePackageRequest>({
    name: '',
    description: '',
    disk_quota_mb: 1024,
    bandwidth_mb: 10240,
    max_domains: 1,
    max_subdomains: 5,
    max_databases: 1,
    max_email_accounts: 5,
    max_ftp_accounts: 2,
    max_cron_jobs: 3,
    price_monthly: 0,
    price_yearly: 0,
    is_active: true,
    is_default: false,
    sort_order: 0
})

const resetForm = () => {
    formData.value = {
        name: '',
        description: '',
        disk_quota_mb: 1024,
        bandwidth_mb: 10240,
        max_domains: 1,
        max_subdomains: 5,
        max_databases: 1,
        max_email_accounts: 5,
        max_ftp_accounts: 2,
        max_cron_jobs: 3,
        price_monthly: 0,
        price_yearly: 0,
        is_active: true,
        is_default: false,
        sort_order: 0
    }
}

// Methods
const fetchPackages = async () => {
    isLoading.value = true
    try {
        const res = await packageService.listPackages()
        packages.value = res.data.data || []
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to load packages')
    } finally {
        isLoading.value = false
    }
}

const openCreateModal = () => {
    resetForm()
    isEditing.value = false
    selectedPackage.value = null
    showFormModal.value = true
}

const openEditModal = (pkg: Package) => {
    selectedPackage.value = pkg
    isEditing.value = true
    formData.value = {
        name: pkg.name,
        description: pkg.description || '',
        disk_quota_mb: pkg.disk_quota_mb,
        bandwidth_mb: pkg.bandwidth_mb,
        max_domains: pkg.max_domains,
        max_subdomains: pkg.max_subdomains,
        max_databases: pkg.max_databases,
        max_email_accounts: pkg.max_email_accounts,
        max_ftp_accounts: pkg.max_ftp_accounts,
        max_cron_jobs: pkg.max_cron_jobs,
        price_monthly: pkg.price_monthly,
        price_yearly: pkg.price_yearly,
        is_active: pkg.is_active,
        is_default: pkg.is_default,
        sort_order: pkg.sort_order
    }
    showFormModal.value = true
}

const savePackage = async () => {
    if (!formData.value.name) {
        toast.error('Package name is required')
        return
    }
    isSubmitting.value = true
    try {
        if (isEditing.value && selectedPackage.value) {
            await packageService.updatePackage(selectedPackage.value.id, formData.value as UpdatePackageRequest)
            toast.success('Package updated successfully')
        } else {
            await packageService.createPackage(formData.value)
            toast.success('Package created successfully')
        }
        showFormModal.value = false
        fetchPackages()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to save package')
    } finally {
        isSubmitting.value = false
    }
}

const confirmDelete = (pkg: Package) => {
    selectedPackage.value = pkg
    showDeleteModal.value = true
}

const deletePackage = async () => {
    if (!selectedPackage.value) return
    isSubmitting.value = true
    try {
        await packageService.deletePackage(selectedPackage.value.id)
        toast.success('Package deleted successfully')
        showDeleteModal.value = false
        fetchPackages()
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to delete package')
    } finally {
        isSubmitting.value = false
    }
}

const formatStorage = (mb: number) => {
    if (mb === 0) return 'Unlimited'
    if (mb >= 1024) return `${(mb / 1024).toFixed(0)} GB`
    return `${mb} MB`
}

const formatPrice = (price: number) => {
    return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0 }).format(price)
}

const formatLimit = (val: number) => val === 0 ? 'Unlimited' : val.toString()

onMounted(fetchPackages)
</script>

<template>
<MainLayout>


    <div class="space-y-6">
        <!-- Header -->
        <div class="flex flex-wrap justify-between items-end gap-4">
            <div>
                <h2 class="text-3xl font-black text-[#0d131b] dark:text-white flex items-center gap-3">
                    <PackageIcon :size="32" class="text-primary" />
                    Package Management
                </h2>
                <p class="text-slate-500 text-sm mt-2">Define hosting packages with resource limits and pricing.</p>
            </div>
            <button @click="openCreateModal"
                class="flex items-center gap-2 px-5 py-2.5 bg-primary text-white hover:bg-primary/90 rounded-lg text-sm font-bold shadow-lg shadow-primary/20">
                <Plus :size="18" />
                New Package
            </button>
        </div>

        <!-- Loading State -->
        <div v-if="isLoading" class="flex justify-center py-12">
            <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full"></div>
        </div>

        <!-- Empty State -->
        <div v-else-if="packages.length === 0" class="text-center py-16 bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800">
            <PackageIcon :size="48" class="mx-auto mb-4 text-slate-300" />
            <p class="text-slate-500 font-medium">No packages defined yet</p>
            <p class="text-slate-400 text-sm mt-1">Create your first hosting package to get started</p>
        </div>

        <!-- Package Cards -->
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
            <div v-for="pkg in packages" :key="pkg.id" 
                :class="['relative bg-white dark:bg-slate-900 border rounded-xl overflow-hidden transition-all hover:shadow-lg',
                    pkg.is_default ? 'border-primary shadow-primary/10 ring-2 ring-primary/20' : 'border-slate-200 dark:border-slate-800']">
                
                <!-- Default Badge -->
                <div v-if="pkg.is_default" class="absolute top-3 right-3">
                    <span class="flex items-center gap-1 bg-primary text-white text-xs font-bold px-2 py-1 rounded-full">
                        <Star :size="12" /> Default
                    </span>
                </div>

                <!-- Status Badge -->
                <div v-if="!pkg.is_active" class="absolute top-3 left-3">
                    <span class="bg-red-100 text-red-600 text-xs font-bold px-2 py-1 rounded-full">Inactive</span>
                </div>

                <!-- Header -->
                <div class="p-5 pb-3 border-b border-slate-100 dark:border-slate-800">
                    <h3 class="text-lg font-bold">{{ pkg.name }}</h3>
                    <p class="text-sm text-slate-500 line-clamp-2 min-h-[40px]">{{ pkg.description || 'No description' }}</p>
                </div>

                <!-- Pricing -->
                <div class="p-5 pt-4 bg-slate-50 dark:bg-slate-800/50">
                    <p class="text-3xl font-black text-primary">{{ formatPrice(pkg.price_monthly) }}</p>
                    <p class="text-xs text-slate-500">/month</p>
                    <p v-if="pkg.price_yearly > 0" class="text-xs text-slate-400 mt-1">
                        {{ formatPrice(pkg.price_yearly) }}/year
                    </p>
                </div>

                <!-- Limits -->
                <div class="p-5 space-y-2.5 text-sm">
                    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
                        <HardDrive :size="14" class="text-slate-400" />
                        <span>{{ formatStorage(pkg.disk_quota_mb) }} Storage</span>
                    </div>
                    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
                        <Wifi :size="14" class="text-slate-400" />
                        <span>{{ formatStorage(pkg.bandwidth_mb) }} Bandwidth</span>
                    </div>
                    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
                        <Globe :size="14" class="text-slate-400" />
                        <span>{{ formatLimit(pkg.max_domains) }} Domains</span>
                    </div>
                    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
                        <Database :size="14" class="text-slate-400" />
                        <span>{{ formatLimit(pkg.max_databases) }} Databases</span>
                    </div>
                    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
                        <Mail :size="14" class="text-slate-400" />
                        <span>{{ formatLimit(pkg.max_email_accounts) }} Email Accounts</span>
                    </div>
                </div>

                <!-- Actions -->
                <div class="flex border-t border-slate-100 dark:border-slate-800">
                    <button @click="openEditModal(pkg)" class="flex-1 py-3 text-sm font-medium text-center hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors flex items-center justify-center gap-2">
                        <Edit :size="14" /> Edit
                    </button>
                    <button @click="confirmDelete(pkg)" class="flex-1 py-3 text-sm font-medium text-center text-red-500 hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors border-l border-slate-100 dark:border-slate-800 flex items-center justify-center gap-2">
                        <Trash2 :size="14" /> Delete
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Form Modal -->
    <BaseModal
        :isOpen="showFormModal"
        :title="isEditing ? 'Edit Package' : 'Create Package'"
        width="lg"
        @close="showFormModal = false"
    >
        <div class="space-y-5">
            <!-- Basic Info -->
            <div class="grid grid-cols-2 gap-4">
                <div class="col-span-2">
                    <label class="block text-sm font-medium mb-1.5">Package Name *</label>
                    <input v-model="formData.name" required
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="e.g. Basic, Pro, Enterprise" />
                </div>
                <div class="col-span-2">
                    <label class="block text-sm font-medium mb-1.5">Description</label>
                    <textarea v-model="formData.description" rows="2"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="Brief description of this package"></textarea>
                </div>
            </div>

            <!-- Resources -->
            <div>
                <h4 class="font-semibold text-sm mb-3 text-slate-600">Resource Limits</h4>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Disk Quota (MB)</label>
                        <input v-model.number="formData.disk_quota_mb" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                        <span class="text-xs text-slate-400">0 = Unlimited</span>
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Bandwidth (MB)</label>
                        <input v-model.number="formData.bandwidth_mb" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max Domains</label>
                        <input v-model.number="formData.max_domains" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max Subdomains</label>
                        <input v-model.number="formData.max_subdomains" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max Databases</label>
                        <input v-model.number="formData.max_databases" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max Email Accounts</label>
                        <input v-model.number="formData.max_email_accounts" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max FTP Accounts</label>
                        <input v-model.number="formData.max_ftp_accounts" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Max Cron Jobs</label>
                        <input v-model.number="formData.max_cron_jobs" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                </div>
            </div>

            <!-- Pricing -->
            <div>
                <h4 class="font-semibold text-sm mb-3 text-slate-600">Pricing</h4>
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Monthly Price (IDR)</label>
                        <input v-model.number="formData.price_monthly" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                    <div>
                        <label class="block text-xs text-slate-500 mb-1">Yearly Price (IDR)</label>
                        <input v-model.number="formData.price_yearly" type="number" min="0"
                            class="w-full px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm" />
                    </div>
                </div>
            </div>

            <!-- Options -->
            <div class="flex items-center gap-6">
                <label class="flex items-center gap-2 cursor-pointer">
                    <input v-model="formData.is_active" type="checkbox" class="w-4 h-4 rounded border-slate-300 text-primary focus:ring-primary">
                    <span class="text-sm">Active</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input v-model="formData.is_default" type="checkbox" class="w-4 h-4 rounded border-slate-300 text-primary focus:ring-primary">
                    <span class="text-sm">Set as Default</span>
                </label>
            </div>
        </div>
        <template #footer>
            <button @click="showFormModal = false"
                class="px-5 py-2.5 text-sm font-medium hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors">
                Cancel
            </button>
            <button @click="savePackage" :disabled="isSubmitting"
                class="px-5 py-2.5 text-sm font-bold bg-primary text-white rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors shadow-lg shadow-primary/20">
                {{ isSubmitting ? 'Saving...' : (isEditing ? 'Update Package' : 'Create Package') }}
            </button>
        </template>
    </BaseModal>

    <!-- Delete Modal -->
    <BaseModal
        :isOpen="showDeleteModal"
        title="Delete Package"
        width="sm"
        @close="showDeleteModal = false"
    >
        <div class="text-center py-4">
            <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <Trash2 :size="32" class="text-red-500" />
            </div>
            <h3 class="text-xl font-bold mb-2">Delete Package</h3>
            <p class="text-slate-500 mb-6 text-sm">
                Are you sure you want to delete <strong>{{ selectedPackage?.name }}</strong>? 
                This will fail if users are assigned to this package.
            </p>
        </div>
        <template #footer>
            <button @click="showDeleteModal = false"
                class="px-5 py-2.5 text-sm font-medium hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors">
                Cancel
            </button>
            <button @click="deletePackage" :disabled="isSubmitting"
                class="px-5 py-2.5 text-sm font-bold bg-red-500 text-white rounded-lg hover:bg-red-600 disabled:opacity-50 transition-colors shadow-lg shadow-red-500/20">
                {{ isSubmitting ? 'Deleting...' : 'Delete' }}
            </button>
        </template>
    </BaseModal>
</MainLayout>
</template>
