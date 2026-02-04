<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { 
  Database, Plus, Trash2, Key, 
  ExternalLink,
  User, Shield, Layers, HardDrive,
  Check, Search, 
  RefreshCw
} from 'lucide-vue-next'
import { databaseService } from '@/services'
import { useAuthStore } from '@/stores/auth'
import { useToastStore } from '@/stores/toast'
import type { ManagedDatabase, DatabaseUser } from '@/types'
import BaseModal from '@/components/ui/BaseModal.vue'

const authStore = useAuthStore()
const toast = useToastStore()

// Derived state
const dbPrefix = computed(() => authStore.user?.username ? `${authStore.user.username}_` : '')

// ==========================================
// STATE
// ==========================================
const databases = ref<ManagedDatabase[]>([])
const dbUsers = ref<DatabaseUser[]>([])
const isLoading = ref(true)
const isRefreshing = ref(false)
const error = ref<string | null>(null)
// successMsg removed in favor of toast
const activeTab = ref<'databases' | 'users' | 'assign'>('databases')
const searchQuery = ref('')

// Modal states
const showCreateDbModal = ref(false)
const showCreateUserModal = ref(false)
const showDeleteDbModal = ref(false)
const showDeleteUserModal = ref(false)
const showChangePasswordModal = ref(false)
const showPrivilegesModal = ref(false)
const showAssignModal = ref(false)

// Form data - Database
const newDbName = ref('')
const newDbCharset = ref('utf8mb4')
const newDbCollation = ref('utf8mb4_unicode_ci')

// Form data - User
const newUsername = ref('')
const newPassword = ref('')
const newConfirmPassword = ref('')
const newHost = ref('localhost')
const createWithDb = ref(false)
const selectedDbForCreateUser = ref('')

// Form data - Assign/Privileges
const selectedUserForAssign = ref('')
const selectedDbForAssign = ref('')
const selectedPrivileges = ref<string[]>(['ALL'])

// Form data - Change password
const changePasswordUser = ref<DatabaseUser | null>(null)
const newUserPassword = ref('')
const confirmUserPassword = ref('')

// Form data - Privileges edit
const editPrivilegesUser = ref<DatabaseUser | null>(null)

// Selected items for delete
const selectedDb = ref<ManagedDatabase | null>(null)
const selectedUser = ref<DatabaseUser | null>(null)

// Available privileges
const availablePrivileges = [
  { value: 'ALL', label: 'ALL PRIVILEGES', desc: 'Full access to all tables' },
  { value: 'SELECT', label: 'SELECT', desc: 'Read data from tables' },
  { value: 'INSERT', label: 'INSERT', desc: 'Insert new data' },
  { value: 'UPDATE', label: 'UPDATE', desc: 'Modify existing data' },
  { value: 'DELETE', label: 'DELETE', desc: 'Remove data' },
  { value: 'CREATE', label: 'CREATE', desc: 'Create new tables' },
  { value: 'DROP', label: 'DROP', desc: 'Delete tables' },
  { value: 'ALTER', label: 'ALTER', desc: 'Modify table structure' },
  { value: 'INDEX', label: 'INDEX', desc: 'Manage indexes' },
]

// Charset options
const charsetOptions = [
  { value: 'utf8mb4', label: 'utf8mb4 (Recommended)' },
  { value: 'utf8', label: 'utf8' },
  { value: 'latin1', label: 'latin1' },
]

// Collation options
const collationOptions = computed(() => {
  if (newDbCharset.value === 'utf8mb4') {
    return ['utf8mb4_unicode_ci', 'utf8mb4_general_ci', 'utf8mb4_bin']
  } else if (newDbCharset.value === 'utf8') {
    return ['utf8_unicode_ci', 'utf8_general_ci', 'utf8_bin']
  }
  return ['latin1_swedish_ci', 'latin1_general_ci', 'latin1_bin']
})

const filteredDatabases = computed(() => {
  if (!searchQuery.value) return databases.value
  const query = searchQuery.value.toLowerCase()
  return databases.value.filter(db => 
    db.db_name.toLowerCase().includes(query) || 
    db.id.toLowerCase().includes(query)
  )
})

const filteredUsers = computed(() => {
  if (!searchQuery.value) return dbUsers.value
  const query = searchQuery.value.toLowerCase()
  return dbUsers.value.filter(user => 
    user.db_username.toLowerCase().includes(query) ||
    (user.db_name && user.db_name.toLowerCase().includes(query))
  )
})

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Fetch data dari API
 */
const fetchData = async (background = false): Promise<void> => {
  try {
    if (!background) isLoading.value = true
    isRefreshing.value = true
    error.value = null
    const [dbRes, userRes] = await Promise.all([
      databaseService.listDatabases(),
      databaseService.listDatabaseUsers()
    ])
    databases.value = dbRes.data.data || []
    dbUsers.value = userRes.data.data || []
  } catch (err: any) {
    if (!background) error.value = err.response?.data?.message || 'Failed to load database data'
    else toast.error('Failed to refresh data')
  } finally {
    isLoading.value = false
    isRefreshing.value = false
  }
}

const showSuccess = (msg: string): void => {
  toast.success(msg)
}

const createDatabase = async (): Promise<void> => {
  if (!newDbName.value.trim()) return
  try {
    isRefreshing.value = true
    await databaseService.createDatabase({
      name: newDbName.value.trim(),
      charset: newDbCharset.value,
      collation: newDbCollation.value
    })
    showCreateDbModal.value = false
    newDbName.value = ''
    showSuccess('Database created successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to create database'
  } finally {
    isRefreshing.value = false
  }
}

const deleteDatabase = async (): Promise<void> => {
  if (!selectedDb.value) return
  try {
    isRefreshing.value = true
    await databaseService.deleteDatabase(selectedDb.value.id)
    showDeleteDbModal.value = false
    selectedDb.value = null
    showSuccess('Database deleted successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to delete database'
  } finally {
    isRefreshing.value = false
  }
}

const createDatabaseUser = async (): Promise<void> => {
  if (!newUsername.value.trim() || !newPassword.value) {
    error.value = 'Username and password are required'
    return
  }
  if (newPassword.value !== newConfirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  try {
    isRefreshing.value = true
    await databaseService.createDatabaseUser({
      username: newUsername.value.trim(),
      password: newPassword.value,
      database_id: createWithDb.value ? selectedDbForCreateUser.value : '',
      host: newHost.value
    })
    showCreateUserModal.value = false
    newUsername.value = ''
    newPassword.value = ''
    newConfirmPassword.value = ''
    createWithDb.value = false
    selectedDbForCreateUser.value = ''
    showSuccess('User created successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to create user'
  } finally {
    isRefreshing.value = false
  }
}

const deleteDatabaseUser = async (): Promise<void> => {
  if (!selectedUser.value) return
  try {
    isRefreshing.value = true
    await databaseService.deleteDatabaseUser(selectedUser.value.id)
    showDeleteUserModal.value = false
    selectedUser.value = null
    showSuccess('User deleted successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to delete user'
  } finally {
    isRefreshing.value = false
  }
}

const changePassword = async (): Promise<void> => {
  if (!changePasswordUser.value) return
  if (newUserPassword.value !== confirmUserPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  try {
    isRefreshing.value = true
    await databaseService.updateDatabaseUser(changePasswordUser.value.id, {
      password: newUserPassword.value
    })
    showChangePasswordModal.value = false
    changePasswordUser.value = null
    newUserPassword.value = ''
    confirmUserPassword.value = ''
    showSuccess('Password updated successfully')
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to update password'
  } finally {
    isRefreshing.value = false
  }
}

const assignUserToDatabase = async (): Promise<void> => {
  if (!selectedUserForAssign.value || !selectedDbForAssign.value) {
    error.value = 'Please select both user and database'
    return
  }
  try {
    isRefreshing.value = true
    await databaseService.updateDatabaseUser(selectedUserForAssign.value, {
      database_id: selectedDbForAssign.value,
      privileges: selectedPrivileges.value.join(',')
    })
    showAssignModal.value = false
    selectedUserForAssign.value = ''
    selectedDbForAssign.value = ''
    selectedPrivileges.value = ['ALL']
    showSuccess('User assigned to database successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to assign user'
  } finally {
    isRefreshing.value = false
  }
}

const updatePrivileges = async (): Promise<void> => {
  if (!editPrivilegesUser.value) return
  try {
    isRefreshing.value = true
    await databaseService.updateDatabaseUser(editPrivilegesUser.value.id, {
      privileges: selectedPrivileges.value.join(',')
    })
    showPrivilegesModal.value = false
    editPrivilegesUser.value = null
    showSuccess('Privileges updated successfully')
    await fetchData(true)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to update privileges'
  } finally {
    isRefreshing.value = false
  }
}

// ==========================================
// HELPERS & MODAL OPENERS
// ==========================================

const togglePrivilege = (priv: string): void => {
  if (priv === 'ALL') {
    selectedPrivileges.value = selectedPrivileges.value.includes('ALL') ? [] : ['ALL']
  } else {
    selectedPrivileges.value = selectedPrivileges.value.filter(p => p !== 'ALL')
    const idx = selectedPrivileges.value.indexOf(priv)
    if (idx > -1) selectedPrivileges.value.splice(idx, 1)
    else selectedPrivileges.value.push(priv)
  }
}

const openDeleteDb = (db: ManagedDatabase): void => {
  selectedDb.value = db
  showDeleteDbModal.value = true
}

const openDeleteUser = (user: DatabaseUser): void => {
  selectedUser.value = user
  showDeleteUserModal.value = true
}

const openChangePassword = (user: DatabaseUser): void => {
  changePasswordUser.value = user
  newUserPassword.value = ''
  confirmUserPassword.value = ''
  showChangePasswordModal.value = true
}

const openPrivileges = (user: DatabaseUser): void => {
  editPrivilegesUser.value = user
  selectedPrivileges.value = user.privileges?.split(',') || ['ALL']
  showPrivilegesModal.value = true
}

const openAssignUser = (preSelectedDbId?: string): void => {
  selectedUserForAssign.value = ''
  selectedDbForAssign.value = preSelectedDbId || ''
  selectedPrivileges.value = ['ALL']
  showAssignModal.value = true
}

const openAssignForUser = (user: DatabaseUser): void => {
  selectedUserForAssign.value = user.id
  selectedDbForAssign.value = ''
  selectedPrivileges.value = ['ALL']
  activeTab.value = 'assign'
  showAssignModal.value = true
}

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const openPhpMyAdmin = async (target?: string | ManagedDatabase): Promise<void> => {
  let dbUserId: string | undefined

  if (target && typeof target === 'object' && 'db_name' in target) {
      // Auto-detect user for this database
      // Prioritize user with same name as DB, otherwise take first available
      const relatedUsers = dbUsers.value.filter(u => u.db_name === target.db_name)
      if (relatedUsers.length > 0) {
          // Try to find exact match first (often dbname == username)
          const exactMatch = relatedUsers.find(u => u.db_username === target.db_name)
          dbUserId = exactMatch ? exactMatch.id : relatedUsers[0]!.id
      }
  } else if (typeof target === 'string') {
      dbUserId = target
  }

  if (!dbUserId) {
    const pmaUrl = import.meta.env.VITE_PHPMYADMIN_URL || '/phpmyadmin'
    window.open(pmaUrl, '_blank')
    return
  }
  try {
    const response = await databaseService.generateSignonToken(dbUserId)
    const signonUrl = response.data?.data?.signon_url
    if (signonUrl) {
      window.open(signonUrl, '_blank')
    } else {
      showSuccess('Opening phpMyAdmin...')
    }
  } catch (err: any) {
    error.value = 'Failed to generate signon token'
  }
}

const generatePassword = (): void => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%'
  let pass = ''
  for (let i = 0; i < 16; i++) pass += chars.charAt(Math.floor(Math.random() * chars.length))
  newPassword.value = pass
  newConfirmPassword.value = pass
}

const generateNewPassword = (): void => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%'
  let pass = ''
  for (let i = 0; i < 16; i++) pass += chars.charAt(Math.floor(Math.random() * chars.length))
  newUserPassword.value = pass
  confirmUserPassword.value = pass
}

onMounted(() => {
  fetchData()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-7xl mx-auto space-y-8 animate-in pb-20">
      


      <!-- Page Header -->
      <div class="mb-8">
        <h1 class="text-3xl font-black text-foreground tracking-tight mb-2">MySQL Databases & Users</h1>
        <p class="text-muted-foreground max-w-2xl">Create and manage your database instances, user permissions, and remote access configurations from a single dashboard.</p>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="flex flex-col items-center justify-center py-20 text-muted-foreground">
        <div class="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin mb-4" />
        <p class="text-sm font-medium">Loading data...</p>
      </div>

      <!-- Main Content -->
      <div v-else class="space-y-10">
        
        <!-- Section 1: Databases -->
        <section>
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
            
            <!-- Create Database Form -->
            <div class="lg:col-span-1 bg-card border border-border p-6 rounded-xl shadow-sm">
              <h4 class="text-lg font-bold text-foreground mb-4 flex items-center gap-2">
                <Database class="w-5 h-5 text-primary" />
                Create New Database
              </h4>
              <div class="space-y-4">
                <div>
                  <label class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">Database Name</label>
                  <div class="flex items-center gap-0">
                    <span class="bg-muted px-3 py-2.5 border border-r-0 border-border rounded-l-lg text-muted-foreground text-sm font-mono select-none">{{ dbPrefix }}</span>
                    <input 
                      v-model="newDbName" 
                      type="text" 
                      placeholder="dbname" 
                      class="flex-1 px-3 py-2.5 bg-background border border-border rounded-r-lg text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all"
                    />
                  </div>
                </div>
                <div class="grid grid-cols-2 gap-3">
                  <div>
                    <label class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">Charset</label>
                    <select v-model="newDbCharset" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary">
                      <option v-for="opt in charsetOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
                    </select>
                  </div>
                  <div>
                    <label class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">Collation</label>
                    <select v-model="newDbCollation" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary">
                      <option v-for="opt in collationOptions" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                  </div>
                </div>
                <button 
                  @click="createDatabase" 
                  :disabled="!newDbName.trim() || isRefreshing"
                  class="w-full py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground font-bold rounded-lg transition-all flex items-center justify-center gap-2 shadow-lg shadow-primary/20 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Plus class="w-4 h-4" />
                  Create Database
                </button>
              </div>
            </div>

            <!-- Current Databases Table -->
            <div class="lg:col-span-2 bg-card border border-border rounded-xl shadow-sm overflow-hidden flex flex-col">
              <div class="px-6 py-4 border-b border-border flex items-center justify-between bg-muted/30">
                <h4 class="font-bold text-foreground flex items-center gap-2">
                  <Layers class="w-4 h-4 text-primary" />
                  Current Databases
                  <span class="ml-2 text-xs font-medium text-muted-foreground bg-muted px-2 py-0.5 rounded-full">{{ databases.length }}</span>
                </h4>
                <div class="flex items-center gap-3">
                  <div class="relative">
                    <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                    <input 
                      v-model="searchQuery"
                      type="text" 
                      placeholder="Filter..."
                      class="pl-8 pr-3 py-1.5 bg-background border border-border rounded-lg text-xs focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none w-40"
                    />
                  </div>
                  <button 
                    @click="fetchData(true)" 
                    class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-all"
                    title="Refresh"
                  >
                    <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': isRefreshing }" />
                  </button>
                </div>
              </div>
              
              <div v-if="filteredDatabases.length === 0" class="p-12 text-center">
                <HardDrive class="w-12 h-12 text-muted-foreground/20 mx-auto mb-4" />
                <h3 class="text-foreground font-semibold">No databases found</h3>
                <p class="text-muted-foreground text-sm">Create your first database to get started</p>
              </div>

              <div v-else class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr class="text-xs font-semibold uppercase tracking-wider text-muted-foreground bg-muted/30 border-b border-border">
                      <th class="px-6 py-3">Database Name</th>
                      <th class="px-6 py-3">Size</th>
                      <th class="px-6 py-3">Users</th>
                      <th class="px-6 py-3 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-border">
                    <tr v-for="db in filteredDatabases" :key="db.id" class="hover:bg-muted/30 transition-colors group">
                      <td class="px-6 py-4">
                        <span class="font-mono text-sm text-primary font-medium">{{ db.db_name }}</span>
                      </td>
                      <td class="px-6 py-4 text-sm text-muted-foreground font-mono">{{ formatSize(db.size_bytes) }}</td>
                      <td class="px-6 py-4">
                        <div class="flex flex-wrap gap-1">
                          <span 
                            v-if="db.users_count > 0"
                            class="px-2 py-0.5 bg-muted text-muted-foreground rounded text-[10px] font-bold border border-border"
                          >
                            {{ db.users_count }} assigned
                          </span>
                          <span v-else class="text-xs text-muted-foreground">No users</span>
                        </div>
                      </td>
                      <td class="px-6 py-4 text-right">
                        <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button 
                            @click="openPhpMyAdmin(db)" 
                            class="p-1.5 text-primary bg-primary/10 hover:bg-primary/20 rounded-lg transition-colors" 
                            title="phpMyAdmin"
                          >
                            <ExternalLink class="w-4 h-4" />
                          </button>
                          <button 
                            @click="openAssignUser(db.id)"
                            class="p-1.5 text-emerald-600 bg-emerald-500/10 hover:bg-emerald-500/20 rounded-lg transition-colors" 
                            title="Add User"
                          >
                            <Plus class="w-4 h-4" />
                          </button>
                          <button 
                            @click="openDeleteDb(db)" 
                            class="p-1.5 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors" 
                            title="Delete"
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
        </section>

        <!-- Section 2: MySQL User Management -->
        <section>
          <h2 class="text-xl font-bold text-foreground border-b border-border pb-3 mb-8 flex items-center gap-2">
            <User class="w-5 h-5 text-emerald-500" />
            MySQL User Management
          </h2>
          
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
            
            <!-- Create User Form -->
            <div class="lg:col-span-1 bg-card border border-border p-6 rounded-xl shadow-sm">
              <h4 class="text-lg font-bold text-foreground mb-4">Create New User</h4>
              <div class="space-y-4">
                <div>
                  <label class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">Username</label>
                  <div class="flex items-center gap-0">
                    <span class="bg-muted px-3 py-2.5 border border-r-0 border-border rounded-l-lg text-muted-foreground text-sm font-mono select-none">{{ dbPrefix }}</span>
                    <input 
                      v-model="newUsername" 
                      type="text" 
                      placeholder="username" 
                      class="flex-1 px-3 py-2.5 bg-background border border-border rounded-r-lg text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    />
                  </div>
                </div>
                <div>
                  <div class="flex justify-between mb-1.5">
                    <label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Password</label>
                    <button @click="generatePassword" class="text-xs text-primary font-semibold hover:underline flex items-center gap-1">
                      <RefreshCw class="w-3 h-3" /> Generate
                    </button>
                  </div>
                  <input 
                    v-model="newPassword" 
                    type="password" 
                    placeholder="••••••••••••"
                    class="w-full px-3 py-2.5 bg-background border border-border rounded-lg text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none font-mono"
                  />
                </div>
                <div>
                  <label class="block text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5">Confirm Password</label>
                  <input 
                    v-model="newConfirmPassword" 
                    type="password" 
                    placeholder="••••••••••••"
                    class="w-full px-3 py-2.5 bg-background border border-border rounded-lg text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none font-mono"
                  />
                </div>
                
                <!-- Password Strength -->
                <div v-if="newPassword" class="bg-primary/5 border border-primary/20 p-3 rounded-lg">
                  <p class="text-[10px] text-primary font-bold uppercase mb-1">Security Score</p>
                  <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                    <div 
                      class="h-full bg-primary transition-all" 
                      :style="{ width: Math.min(100, newPassword.length * 6.25) + '%' }"
                    />
                  </div>
                  <p class="text-[10px] text-muted-foreground mt-2 font-medium">
                    Password strength: {{ newPassword.length >= 16 ? 'Very Strong' : newPassword.length >= 12 ? 'Strong' : newPassword.length >= 8 ? 'Medium' : 'Weak' }}
                  </p>
                </div>

                <button 
                  @click="createDatabaseUser" 
                  :disabled="!newUsername.trim() || !newPassword || isRefreshing"
                  class="w-full py-2.5 bg-emerald-600 hover:bg-emerald-700 text-white font-bold rounded-lg transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Plus class="w-4 h-4" />
                  Create User
                </button>
              </div>
            </div>

            <!-- Current Users Table -->
            <div class="lg:col-span-2 bg-card border border-border rounded-xl shadow-sm overflow-hidden flex flex-col">
              <div class="px-6 py-4 border-b border-border flex items-center justify-between bg-muted/30">
                <h4 class="font-bold text-foreground">Current Users</h4>
                <span class="text-xs font-medium text-muted-foreground bg-muted px-2 py-0.5 rounded-full">{{ dbUsers.length }}</span>
              </div>
              
              <div v-if="dbUsers.length === 0" class="p-12 text-center">
                <User class="w-12 h-12 text-muted-foreground/20 mx-auto mb-4" />
                <h3 class="text-foreground font-semibold">No users found</h3>
                <p class="text-muted-foreground text-sm">Create database users to grant access</p>
              </div>

              <div v-else class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                  <thead>
                    <tr class="text-xs font-semibold uppercase tracking-wider text-muted-foreground bg-muted/30 border-b border-border">
                      <th class="px-6 py-3">Username</th>
                      <th class="px-6 py-3">Host</th>
                      <th class="px-6 py-3">Database Access</th>
                      <th class="px-6 py-3 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-border">
                    <tr v-for="user in filteredUsers" :key="user.id" class="hover:bg-muted/30 transition-colors group">
                      <td class="px-6 py-4">
                        <div class="flex items-center gap-2">
                          <div class="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center">
                            <User class="w-3 h-3" />
                          </div>
                          <span class="font-mono text-sm font-medium">{{ user.db_username }}</span>
                        </div>
                      </td>
                      <td class="px-6 py-4 text-sm text-muted-foreground">{{ user.host }}</td>
                      <td class="px-6 py-4">
                        <div class="flex items-center gap-1.5">
                          <button 
                            v-if="!user.db_name"
                            @click="openAssignForUser(user)"
                            class="flex items-center gap-1 px-2.5 py-1 bg-primary text-primary-foreground text-[10px] font-bold rounded-lg hover:bg-primary/90 transition-all"
                          >
                            <Plus class="w-3 h-3" /> Add to DB
                          </button>
                          <span v-else class="px-2 py-0.5 bg-muted text-muted-foreground rounded text-[10px] font-bold border border-border">{{ user.db_name }}</span>
                        </div>
                      </td>
                      <td class="px-6 py-4 text-right">
                        <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button 
                            @click="openChangePassword(user)" 
                            class="p-1.5 text-muted-foreground hover:bg-muted rounded-lg transition-colors" 
                            title="Change Password"
                          >
                            <Key class="w-4 h-4" />
                          </button>
                          <button 
                            v-if="user.db_name"
                            @click="openPhpMyAdmin(user.id)" 
                            class="p-1.5 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-colors" 
                            title="SSO Login"
                          >
                            <ExternalLink class="w-4 h-4" />
                          </button>
                          <button 
                            @click="openPrivileges(user)" 
                            class="p-1.5 text-muted-foreground hover:bg-muted rounded-lg transition-colors" 
                            title="Permissions"
                          >
                            <Shield class="w-4 h-4" />
                          </button>
                          <button 
                            @click="openDeleteUser(user)" 
                            class="p-1.5 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors" 
                            title="Delete User"
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
        </section>

      </div>
    </div>

    <!-- ==================== MODALS ==================== -->
    
    <!-- Shared Modal Layout Component could go here, but keeping inline for simplicity -->
    
    <!-- Create Database -->
    <!-- ==================== MODALS ==================== -->
    
    <!-- Create Database -->
    <BaseModal 
      :isOpen="showCreateDbModal" 
      title="Create New Database" 
      width="md"
      @close="showCreateDbModal = false"
    >
        <div class="space-y-4">
          <div>
            <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Database Name</label>
            <div class="flex items-center gap-2">
              <div class="bg-muted px-3 py-2.5 rounded-lg border border-border text-sm text-muted-foreground font-mono select-none">
                  {{ dbPrefix }}
              </div>
              <input v-model="newDbName" type="text" placeholder="name" class="flex-1 bg-background border border-border rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" autofocus />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Charset</label>
                <select v-model="newDbCharset" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none">
                  <option v-for="opt in charsetOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
                </select>
              </div>
              <div>
                <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Collation</label>
                <select v-model="newDbCollation" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none">
                  <option v-for="opt in collationOptions" :key="opt" :value="opt">{{ opt }}</option>
                </select>
              </div>
          </div>
        </div>
        <template #footer>
          <button @click="showCreateDbModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="createDatabase" class="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-all shadow-sm">Create Database</button>
        </template>
    </BaseModal>

    <!-- Create User -->
    <BaseModal
      :isOpen="showCreateUserModal"
      title="Create New User"
      width="md"
      @close="showCreateUserModal = false"
    >
        <div class="space-y-4">
          <!-- Create with DB info -->
          <div v-if="createWithDb" class="bg-primary/5 text-primary text-xs px-3 py-2 rounded-lg mb-4 flex items-center gap-2">
            <Database class="w-4 h-4" />
            Creating user for database: <span class="font-mono font-bold">{{ selectedDbForCreateUser }}</span>
          </div>

          <div>
            <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Username</label>
            <div class="flex items-center gap-2">
              <div class="bg-muted px-3 py-2.5 rounded-lg border border-border text-sm text-muted-foreground font-mono select-none">
                  {{ dbPrefix }}
              </div>
              <input v-model="newUsername" type="text" placeholder="username" class="flex-1 bg-background border border-border rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" autofocus />
            </div>
          </div>
          <div>
            <div class="flex justify-between mb-1.5">
              <label class="text-xs font-bold text-muted-foreground uppercase">Password</label>
              <button @click="generatePassword" class="text-xs text-primary font-bold hover:underline flex items-center gap-1">
                <RefreshCw class="w-3 h-3" /> Generate
              </button>
            </div>
            <input v-model="newPassword" type="password" placeholder="••••••••" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm font-mono outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary" />
          </div>
          <div>
            <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Confirm Password</label>
            <input v-model="newConfirmPassword" type="password" placeholder="••••••••" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm font-mono outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary" />
          </div>
        </div>
        <template #footer>
          <button @click="showCreateUserModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="createDatabaseUser" class="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-all shadow-sm">Create User</button>
        </template>
    </BaseModal>

    <!-- Delete Database -->
    <BaseModal
      :isOpen="showDeleteDbModal"
      title="Delete Database"
      width="sm"
      @close="showDeleteDbModal = false"
    >
        <div class="text-center py-4">
          <div class="w-16 h-16 bg-red-100 dark:bg-red-900/30 text-red-600 rounded-full flex items-center justify-center mx-auto mb-4">
            <AlertCircle class="w-8 h-8" />
          </div>
          <h3 class="text-lg font-bold text-foreground mb-2">Are you sure?</h3>
          <p class="text-muted-foreground text-sm">
            You are about to delete database <span class="font-mono font-bold">{{ selectedDb?.db_name }}</span>. This action cannot be undone.
          </p>
        </div>
        <template #footer>
          <button @click="showDeleteDbModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="deleteDatabase" class="px-6 py-2 text-sm font-bold bg-red-600 hover:bg-red-700 text-white rounded-lg transition-all shadow-sm">Delete Database</button>
        </template>
    </BaseModal>

    <!-- Delete User -->
    <BaseModal
      :isOpen="showDeleteUserModal"
      title="Delete User"
      width="sm"
      @close="showDeleteUserModal = false"
    >
        <div class="text-center py-4">
          <div class="w-16 h-16 bg-red-100 dark:bg-red-900/30 text-red-600 rounded-full flex items-center justify-center mx-auto mb-4">
            <AlertCircle class="w-8 h-8" />
          </div>
          <h3 class="text-lg font-bold text-foreground mb-2">Are you sure?</h3>
          <p class="text-muted-foreground text-sm">
            You are about to delete user <span class="font-mono font-bold">{{ selectedUser?.db_username }}</span>. This action cannot be undone.
          </p>
        </div>
        <template #footer>
          <button @click="showDeleteUserModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="deleteDatabaseUser" class="px-6 py-2 text-sm font-bold bg-red-600 hover:bg-red-700 text-white rounded-lg transition-all shadow-sm">Delete User</button>
        </template>
    </BaseModal>

    <!-- Change Password -->
    <BaseModal
      :isOpen="showChangePasswordModal"
      title="Change User Password"
      width="md"
      @close="showChangePasswordModal = false"
    >
        <div class="space-y-4">
          <div class="bg-muted p-3 rounded-lg flex items-center gap-3 mb-4">
            <User class="w-5 h-5 text-primary" />
            <div>
              <p class="text-xs text-muted-foreground font-bold uppercase">Target User</p>
              <p class="font-mono font-bold">{{ changePasswordUser?.db_username }}</p>
            </div>
          </div>
          <div>
            <div class="flex justify-between mb-1.5">
              <label class="text-xs font-bold text-muted-foreground uppercase">New Password</label>
              <button @click="generateNewPassword" class="text-xs text-primary font-bold hover:underline flex items-center gap-1">
                <RefreshCw class="w-3 h-3" /> Generate
              </button>
            </div>
            <input v-model="newUserPassword" type="password" placeholder="••••••••" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm font-mono outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary" />
          </div>
          <div>
            <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Confirm Password</label>
            <input v-model="confirmUserPassword" type="password" placeholder="••••••••" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm font-mono outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary" />
          </div>
        </div>
        <template #footer>
          <button @click="showChangePasswordModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="changePassword" class="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-all shadow-sm">Update Password</button>
        </template>
    </BaseModal>

    <!-- Privileges Modal -->
    <BaseModal
      :isOpen="showPrivilegesModal"
      title="Manage Privileges"
      width="lg"
      @close="showPrivilegesModal = false"
    >
        <div class="space-y-6">
          <div class="flex items-center gap-4 bg-muted p-4 rounded-xl">
             <div>
               <p class="text-xs font-bold text-muted-foreground uppercase">User</p>
               <p class="font-mono font-bold">{{ editPrivilegesUser?.db_username }}</p>
             </div>
             <div class="h-8 w-px bg-border"></div>
             <div>
               <p class="text-xs font-bold text-muted-foreground uppercase">Database</p>
               <p class="font-mono font-bold">{{ editPrivilegesUser?.db_name || 'Global' }}</p>
             </div>
          </div>

          <div>
            <h4 class="text-sm font-bold uppercase text-muted-foreground mb-3">Select Privileges</h4>
            <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
              <button 
                v-for="priv in availablePrivileges" 
                :key="priv.value"
                @click="togglePrivilege(priv.value)"
                :class="[
                  'p-3 rounded-lg border text-left transition-all relative overflow-hidden group',
                  selectedPrivileges.includes(priv.value) 
                    ? 'bg-primary/5 border-primary text-primary' 
                    : 'bg-background border-border hover:border-primary/50'
                ]"
              >
                <div class="flex items-center justify-between mb-1">
                  <span class="font-mono font-bold text-sm">{{ priv.label }}</span>
                  <Check v-if="selectedPrivileges.includes(priv.value)" class="w-4 h-4 text-primary" />
                </div>
                <p class="text-[10px] text-muted-foreground">{{ priv.desc }}</p>
              </button>
            </div>
          </div>
        </div>
        <template #footer>
          <button @click="showPrivilegesModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="updatePrivileges" class="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-all shadow-sm">Save Changes</button>
        </template>
    </BaseModal>

    <!-- Assign User Modal -->
    <BaseModal
      :isOpen="showAssignModal"
      title="Assign User to Database"
      width="md"
      @close="showAssignModal = false"
    >
        <div class="space-y-4">
           <!-- Tabs for selection mode -->
           <div class="flex p-1 bg-muted rounded-lg mb-4">
             <button 
               @click="activeTab = 'databases'" 
               :class="['flex-1 py-1.5 text-xs font-bold rounded-md transition-all', activeTab === 'databases' ? 'bg-background shadow-sm text-foreground' : 'text-muted-foreground hover:text-foreground']"
             >
               Select Database
             </button>
             <button 
               @click="activeTab = 'users'" 
               :class="['flex-1 py-1.5 text-xs font-bold rounded-md transition-all', activeTab === 'users' ? 'bg-background shadow-sm text-foreground' : 'text-muted-foreground hover:text-foreground']"
             >
               Select User
             </button>
           </div>
           
           <div v-if="activeTab === 'users' || selectedDbForAssign" class="space-y-4">
               <div>
                  <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">User</label>
                  <select v-model="selectedUserForAssign" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none">
                    <option value="" disabled>Select User</option>
                    <option v-for="user in dbUsers" :key="user.id" :value="user.id">{{ user.db_username }}</option>
                  </select>
               </div>
               <div>
                  <label class="block text-xs font-bold text-muted-foreground uppercase mb-1.5">Database</label>
                  <select v-model="selectedDbForAssign" class="w-full bg-background border border-border rounded-lg px-3 py-2.5 text-sm outline-none">
                    <option value="" disabled>Select Database</option>
                    <option v-for="db in databases" :key="db.id" :value="db.id">{{ db.db_name }}</option>
                  </select>
               </div>
               
               <div class="pt-4 border-t border-border mt-4">
                 <label class="block text-xs font-bold text-muted-foreground uppercase mb-2">Default Privileges</label>
                 <div class="p-3 bg-muted/50 rounded-lg border border-border flex items-center justify-between">
                    <span class="text-sm font-mono">ALL PRIVILEGES</span>
                    <button @click="selectedPrivileges = ['ALL']" class="text-xs text-primary font-bold hover:underline">Reset</button>
                 </div>
               </div>
           </div>
        </div>
        <template #footer>
          <button @click="showAssignModal = false" class="px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors">Cancel</button>
          <button @click="assignUserToDatabase" class="px-6 py-2 text-sm font-bold bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-all shadow-sm">Assign User</button>
        </template>
    </BaseModal>
  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeUp 0.5s ease-out forwards; }
@keyframes fadeUp { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
</style>
