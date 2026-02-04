<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import { userService, packageService } from '@/services'
import type { UserResponse, Package, CreateUserByAdminRequest } from '@/types'
import { Users, Search, Trash2, Shield, Package as PackageIcon, UserPlus, Eye, EyeOff, ExternalLink } from 'lucide-vue-next'
import { useToastStore } from '@/stores/toast'
import BaseModal from '@/components/ui/BaseModal.vue'

const toast = useToastStore()
const router = useRouter()

// State
const users = ref<UserResponse[]>([])
const packages = ref<Package[]>([])
const isLoading = ref(true)
const searchQuery = ref('')
const currentPage = ref(1)
const totalPages = ref(1)
const totalUsers = ref(0)
const perPage = ref(10)

// Modal state
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const selectedUser = ref<UserResponse | null>(null)
const isSubmitting = ref(false)
const showPassword = ref(false)

// Form data
const formData = ref<CreateUserByAdminRequest>({
    username: '',
    email: '',
    password: '',
    first_name: '',
    last_name: '',
    role: 'user',
    package_id: '',
    company: '',
    phone: '',
    status: 'active'
})

// Computed
const filteredUsers = computed(() => {
    if (!searchQuery.value) return users.value
    const q = searchQuery.value.toLowerCase()
    return users.value.filter(u => 
        u.username.toLowerCase().includes(q) ||
        u.email.toLowerCase().includes(q) ||
        u.full_name.toLowerCase().includes(q)
    )
})

const roleColor = (role: string) => {
    switch (role) {
        case 'admin': return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
        case 'reseller': return 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
        default: return 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
    }
}

const statusColor = (status: string) => {
    switch (status) {
        case 'active': return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
        case 'inactive': return 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
        case 'blocked': return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
        default: return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
    }
}

// Methods
const fetchUsers = async (page = 1) => {
    isLoading.value = true
    try {
        const res = await userService.listUsers(page, perPage.value)
        users.value = res.data.data || []
        totalPages.value = res.data.pagination?.total_pages || 1
        totalUsers.value = res.data.pagination?.total || 0
        currentPage.value = page
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to load users')
    } finally {
        isLoading.value = false
    }
}

const fetchPackages = async () => {
    try {
        const res = await packageService.listPackages()
        packages.value = res.data.data || []
    } catch (e) {
        console.error('Failed to load packages', e)
    }
}

const openCreateModal = () => {
    formData.value = {
        username: '',
        email: '',
        password: '',
        first_name: '',
        last_name: '',
        role: 'user',
        package_id: packages.value.find(p => p.is_default)?.id || '',
        company: '',
        phone: '',
        status: 'active'
    }
    showCreateModal.value = true
}

const createUser = async () => {
    if (!formData.value.username || !formData.value.email || !formData.value.password) {
        toast.error('Username, email and password are required')
        return
    }
    isSubmitting.value = true
    try {
        await userService.createUser(formData.value)
        toast.success('User created successfully')
        showCreateModal.value = false
        fetchUsers(currentPage.value)
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to create user')
    } finally {
        isSubmitting.value = false
    }
}

const confirmDelete = (user: UserResponse) => {
    selectedUser.value = user
    showDeleteModal.value = true
}

const viewUser = (user: UserResponse) => {
    router.push(`/dashboard/users/${user.id}`)
}

const deleteUser = async () => {
    if (!selectedUser.value) return
    isSubmitting.value = true
    try {
        await userService.deleteUser(selectedUser.value.id)
        toast.success('User deleted successfully')
        showDeleteModal.value = false
        selectedUser.value = null
        fetchUsers(currentPage.value)
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to delete user')
    } finally {
        isSubmitting.value = false
    }
}

const toggleStatus = async (user: UserResponse) => {
    const newStatus = user.status === 'active' ? 'blocked' : 'active'
    try {
        await userService.updateStatus(user.id, newStatus)
        toast.success(`User ${newStatus === 'active' ? 'activated' : 'blocked'}`)
        fetchUsers(currentPage.value)
    } catch (e: any) {
        toast.error(e.response?.data?.message || 'Failed to update status')
    }
}

const formatDate = (date: string) => {
    return new Date(date).toLocaleDateString('id-ID', { 
        year: 'numeric', month: 'short', day: 'numeric' 
    })
}

const getPackageName = (packageId: string | null) => {
    if (!packageId) return '-'
    const pkg = packages.value.find(p => p.id === packageId)
    return pkg?.name || '-'
}

onMounted(() => {
    fetchUsers()
    fetchPackages()
})
</script>

<template>
<MainLayout>


    <div class="space-y-6">
        <!-- Header -->
        <div class="flex flex-wrap justify-between items-end gap-4">
            <div>
                <div class="flex items-center gap-3">
                    <h2 class="text-3xl font-black text-[#0d131b] dark:text-white">User Management</h2>
                    <span class="bg-primary/10 text-primary text-xs font-bold px-2.5 py-1 rounded-full">
                        {{ totalUsers }} users
                    </span>
                </div>
                <p class="text-slate-500 text-sm mt-2">Manage users, assign packages, and control access.</p>
            </div>
            <button @click="openCreateModal" 
                class="flex items-center gap-2 px-5 py-2.5 bg-primary text-white hover:bg-primary/90 rounded-lg text-sm font-bold shadow-lg shadow-primary/20 transition-all">
                <UserPlus :size="18" />
                Add User
            </button>
        </div>

        <!-- Stats Cards -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                        <Users :size="20" class="text-blue-600" />
                    </div>
                    <div>
                        <p class="text-2xl font-black">{{ totalUsers }}</p>
                        <p class="text-xs text-slate-500">Total Users</p>
                    </div>
                </div>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
                        <Shield :size="20" class="text-green-600" />
                    </div>
                    <div>
                        <p class="text-2xl font-black">{{ users.filter(u => u.status === 'active').length }}</p>
                        <p class="text-xs text-slate-500">Active</p>
                    </div>
                </div>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center">
                        <Users :size="20" class="text-purple-600" />
                    </div>
                    <div>
                        <p class="text-2xl font-black">{{ users.filter(u => u.role === 'reseller').length }}</p>
                        <p class="text-xs text-slate-500">Resellers</p>
                    </div>
                </div>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-orange-100 dark:bg-orange-900/30 flex items-center justify-center">
                        <PackageIcon :size="20" class="text-orange-600" />
                    </div>
                    <div>
                        <p class="text-2xl font-black">{{ packages.length }}</p>
                        <p class="text-xs text-slate-500">Packages</p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Search & Filters -->
        <div class="flex flex-wrap gap-4 items-center">
            <div class="relative flex-1 min-w-[250px]">
                <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                <input v-model="searchQuery" type="text" placeholder="Search users by name, email, username..."
                    class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg text-sm" />
            </div>
        </div>

        <!-- Users Table -->
        <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden">
            <div v-if="isLoading" class="p-12 text-center">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
                <p class="text-slate-500">Loading users...</p>
            </div>
            <div v-else-if="filteredUsers.length === 0" class="p-12 text-center">
                <Users :size="48" class="mx-auto mb-4 text-slate-300" />
                <p class="text-slate-500 font-medium">No users found</p>
                <p class="text-slate-400 text-sm mt-1">Try adjusting your search or add a new user</p>
            </div>
            <table v-else class="w-full text-left">
                <thead>
                    <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">User</th>
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Role</th>
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Package</th>
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Status</th>
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Created</th>
                        <th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase text-right">Actions</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                    <tr v-for="user in filteredUsers" :key="user.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/30">
                        <td class="px-6 py-4">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-full bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center text-white font-bold text-sm">
                                    {{ user.username.charAt(0).toUpperCase() }}
                                </div>
                                <div>
                                    <p class="font-semibold text-sm">{{ user.full_name || user.username }}</p>
                                    <p class="text-xs text-slate-500">{{ user.email }}</p>
                                </div>
                            </div>
                        </td>
                        <td class="px-6 py-4">
                            <span :class="['text-xs font-bold px-2.5 py-1 rounded-full capitalize', roleColor(user.role)]">
                                {{ user.role }}
                            </span>
                        </td>
                        <td class="px-6 py-4">
                            <span class="text-sm">{{ getPackageName(user.package_id) }}</span>
                        </td>
                        <td class="px-6 py-4">
                            <span :class="['text-xs font-bold px-2.5 py-1 rounded-full capitalize', statusColor(user.status)]">
                                {{ user.status }}
                            </span>
                        </td>
                        <td class="px-6 py-4 text-sm text-slate-500">{{ formatDate(user.created_at) }}</td>
                        <td class="px-6 py-4 text-right">
                            <div class="flex items-center justify-end gap-2">
                                <button @click="toggleStatus(user)" 
                                    :class="['p-2 rounded-lg transition-colors', user.status === 'active' ? 'hover:bg-red-50 text-red-500' : 'hover:bg-green-50 text-green-500']"
                                    :title="user.status === 'active' ? 'Block user' : 'Activate user'">
                                    <Shield :size="16" />
                                </button>
                                <button @click="viewUser(user)"
                                    class="p-2 hover:bg-slate-100 text-slate-600 rounded-lg transition-colors"
                                    title="View details">
                                    <ExternalLink :size="16" />
                                </button>
                                <button v-if="user.role !== 'admin'" @click="confirmDelete(user)" 
                                    class="p-2 hover:bg-red-50 text-red-500 rounded-lg transition-colors" title="Delete user">
                                    <Trash2 :size="16" />
                                </button>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
            
            <!-- Pagination -->
            <div v-if="totalPages > 1" class="flex items-center justify-between px-6 py-4 border-t border-slate-200 dark:border-slate-800">
                <p class="text-sm text-slate-500">
                    Showing page {{ currentPage }} of {{ totalPages }}
                </p>
                <div class="flex gap-2">
                    <button @click="fetchUsers(currentPage - 1)" :disabled="currentPage === 1"
                        class="px-4 py-2 text-sm font-medium bg-slate-100 dark:bg-slate-800 rounded-lg disabled:opacity-50">
                        Previous
                    </button>
                    <button @click="fetchUsers(currentPage + 1)" :disabled="currentPage === totalPages"
                        class="px-4 py-2 text-sm font-medium bg-primary text-white rounded-lg disabled:opacity-50">
                        Next
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Create User Modal -->
    <BaseModal
        :isOpen="showCreateModal"
        title="Create New User"
        width="lg"
        @close="showCreateModal = false"
    >
        <div class="space-y-4">
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium mb-1.5">Username *</label>
                    <input v-model="formData.username" type="text" required
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="johndoe" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1.5">Email *</label>
                    <input v-model="formData.email" type="email" required
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="john@example.com" />
                </div>
            </div>
            <div>
                <label class="block text-sm font-medium mb-1.5">Password *</label>
                <div class="relative">
                    <input v-model="formData.password" :type="showPassword ? 'text' : 'password'" required
                        class="w-full px-4 py-2.5 pr-12 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="Min 8 characters" />
                    <button type="button" @click="showPassword = !showPassword" 
                        class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600">
                        <Eye v-if="!showPassword" :size="18" />
                        <EyeOff v-else :size="18" />
                    </button>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium mb-1.5">First Name</label>
                    <input v-model="formData.first_name" type="text"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="John" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1.5">Last Name</label>
                    <input v-model="formData.last_name" type="text"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="Doe" />
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium mb-1.5">Role</label>
                    <select v-model="formData.role"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none">
                        <option value="user">User</option>
                        <option value="reseller">Reseller</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1.5">Package</label>
                    <select v-model="formData.package_id"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none">
                        <option value="">No Package</option>
                        <option v-for="pkg in packages" :key="pkg.id" :value="pkg.id">
                            {{ pkg.name }}
                        </option>
                    </select>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium mb-1.5">Company</label>
                    <input v-model="formData.company" type="text"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="Company name" />
                </div>
                <div>
                    <label class="block text-sm font-medium mb-1.5">Phone</label>
                    <input v-model="formData.phone" type="tel"
                        class="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-sm transition-colors focus:border-primary/50 focus:ring-2 focus:ring-primary/10 outline-none"
                        placeholder="+62 xxx" />
                </div>
            </div>
        </div>
        <template #footer>
            <button @click="showCreateModal = false"
                class="px-5 py-2.5 text-sm font-medium hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors">
                Cancel
            </button>
            <button @click="createUser" :disabled="isSubmitting"
                class="px-5 py-2.5 text-sm font-bold bg-primary text-white rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors shadow-lg shadow-primary/20">
                {{ isSubmitting ? 'Creating...' : 'Create User' }}
            </button>
        </template>
    </BaseModal>

    <!-- Delete Confirmation Modal -->
    <BaseModal
        :isOpen="showDeleteModal"
        title="Delete User"
        width="sm"
        @close="showDeleteModal = false"
    >
        <div class="text-center py-4">
            <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <Trash2 :size="32" class="text-red-500" />
            </div>
            <h3 class="text-xl font-bold mb-2">Delete User?</h3>
            <p class="text-slate-500 mb-6 text-sm">
                Are you sure you want to delete <strong>{{ selectedUser?.username }}</strong>? This action cannot be undone.
            </p>
        </div>
        <template #footer>
            <button @click="showDeleteModal = false"
                class="px-5 py-2.5 text-sm font-medium hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors">
                Cancel
            </button>
            <button @click="deleteUser" :disabled="isSubmitting"
                class="px-5 py-2.5 text-sm font-bold bg-red-500 text-white rounded-lg hover:bg-red-600 disabled:opacity-50 transition-colors shadow-lg shadow-red-500/20">
                {{ isSubmitting ? 'Deleting...' : 'Delete User' }}
            </button>
        </template>
    </BaseModal>
</MainLayout>
</template>
