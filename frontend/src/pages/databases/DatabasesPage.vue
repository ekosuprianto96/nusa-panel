<script setup lang="ts">
/**
 * DatabasesPage - Halaman manajemen database MySQL (cPanel-like)
 * 
 * Features:
 * - Create/Delete Database
 * - Create Database User
 * - Assign User ke Database dengan Privileges
 * - Change Password
 * - Manage Privileges
 */
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { 
  Database, Plus, Trash2, Key, 
  ExternalLink, Info, AlertCircle,
  User, Shield, Layers, HardDrive, Settings, X,
  Link, Check, CheckSquare, Square
} from 'lucide-vue-next'
import { databaseService } from '@/services'
import type { ManagedDatabase, DatabaseUser } from '@/types'

// ==========================================
// STATE
// ==========================================
const databases = ref<ManagedDatabase[]>([])
const dbUsers = ref<DatabaseUser[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)
const activeTab = ref<'databases' | 'users' | 'assign'>('databases')

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
  { value: 'ALL', label: 'ALL PRIVILEGES', desc: 'Full access' },
  { value: 'SELECT', label: 'SELECT', desc: 'Read data' },
  { value: 'INSERT', label: 'INSERT', desc: 'Add data' },
  { value: 'UPDATE', label: 'UPDATE', desc: 'Modify data' },
  { value: 'DELETE', label: 'DELETE', desc: 'Remove data' },
  { value: 'CREATE', label: 'CREATE', desc: 'Create tables' },
  { value: 'DROP', label: 'DROP', desc: 'Delete tables' },
  { value: 'ALTER', label: 'ALTER', desc: 'Modify tables' },
  { value: 'INDEX', label: 'INDEX', desc: 'Manage indexes' },
  { value: 'REFERENCES', label: 'REFERENCES', desc: 'Create references' },
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
    const [dbRes, userRes] = await Promise.all([
      databaseService.listDatabases(),
      databaseService.listDatabaseUsers()
    ])
    databases.value = dbRes.data.data || []
    dbUsers.value = userRes.data.data || []
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat data database'
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
 * Create new database
 */
const createDatabase = async (): Promise<void> => {
  if (!newDbName.value.trim()) return
  try {
    isLoading.value = true
    await databaseService.createDatabase({
      name: newDbName.value.trim(),
      charset: newDbCharset.value,
      collation: newDbCollation.value
    })
    showCreateDbModal.value = false
    newDbName.value = ''
    showSuccess('Database berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat database'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete database
 */
const deleteDatabase = async (): Promise<void> => {
  if (!selectedDb.value) return
  try {
    isLoading.value = true
    await databaseService.deleteDatabase(selectedDb.value.id)
    showDeleteDbModal.value = false
    selectedDb.value = null
    showSuccess('Database berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus database'
  } finally {
    isLoading.value = false
  }
}

/**
 * Create database user
 */
const createDatabaseUser = async (): Promise<void> => {
  if (!newUsername.value.trim() || !newPassword.value) {
    error.value = 'Username dan password wajib diisi'
    return
  }
  if (newPassword.value !== newConfirmPassword.value) {
    error.value = 'Password tidak cocok'
    return
  }
  if (newPassword.value.length < 8) {
    error.value = 'Password minimal 8 karakter'
    return
  }
  try {
    isLoading.value = true
    error.value = null
    await databaseService.createDatabaseUser({
      username: newUsername.value.trim(),
      password: newPassword.value,
      database_id: '', // Will be assigned later
      host: newHost.value
    })
    showCreateUserModal.value = false
    newUsername.value = ''
    newPassword.value = ''
    newConfirmPassword.value = ''
    newHost.value = 'localhost'
    showSuccess('User berhasil dibuat')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal membuat user'
  } finally {
    isLoading.value = false
  }
}

/**
 * Delete database user
 */
const deleteDatabaseUser = async (): Promise<void> => {
  if (!selectedUser.value) return
  try {
    isLoading.value = true
    await databaseService.deleteDatabaseUser(selectedUser.value.id)
    showDeleteUserModal.value = false
    selectedUser.value = null
    showSuccess('User berhasil dihapus')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menghapus user'
  } finally {
    isLoading.value = false
  }
}

/**
 * Change user password
 */
const changePassword = async (): Promise<void> => {
  if (!changePasswordUser.value) return
  if (newUserPassword.value !== confirmUserPassword.value) {
    error.value = 'Password tidak cocok'
    return
  }
  if (newUserPassword.value.length < 8) {
    error.value = 'Password minimal 8 karakter'
    return
  }
  try {
    isLoading.value = true
    error.value = null
    await databaseService.updateDatabaseUser(changePasswordUser.value.id, {
      password: newUserPassword.value
    })
    showChangePasswordModal.value = false
    changePasswordUser.value = null
    newUserPassword.value = ''
    confirmUserPassword.value = ''
    showSuccess('Password berhasil diubah')
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal mengubah password'
  } finally {
    isLoading.value = false
  }
}

/**
 * Assign user to database with privileges
 */
const assignUserToDatabase = async (): Promise<void> => {
  if (!selectedUserForAssign.value || !selectedDbForAssign.value) {
    error.value = 'Pilih user dan database'
    return
  }
  try {
    isLoading.value = true
    error.value = null
    await databaseService.updateDatabaseUser(selectedUserForAssign.value, {
      database_id: selectedDbForAssign.value,
      privileges: selectedPrivileges.value.join(',')
    })
    showAssignModal.value = false
    selectedUserForAssign.value = ''
    selectedDbForAssign.value = ''
    selectedPrivileges.value = ['ALL']
    showSuccess('User berhasil di-assign ke database')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal assign user'
  } finally {
    isLoading.value = false
  }
}

/**
 * Update user privileges
 */
const updatePrivileges = async (): Promise<void> => {
  if (!editPrivilegesUser.value) return
  try {
    isLoading.value = true
    await databaseService.updateDatabaseUser(editPrivilegesUser.value.id, {
      privileges: selectedPrivileges.value.join(',')
    })
    showPrivilegesModal.value = false
    editPrivilegesUser.value = null
    showSuccess('Privileges berhasil diupdate')
    await fetchData()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal update privileges'
  } finally {
    isLoading.value = false
  }
}

/**
 * Toggle privilege selection
 */
const togglePrivilege = (priv: string): void => {
  if (priv === 'ALL') {
    selectedPrivileges.value = selectedPrivileges.value.includes('ALL') ? [] : ['ALL']
  } else {
    // Remove ALL if selecting individual
    selectedPrivileges.value = selectedPrivileges.value.filter(p => p !== 'ALL')
    
    const idx = selectedPrivileges.value.indexOf(priv)
    if (idx > -1) {
      selectedPrivileges.value.splice(idx, 1)
    } else {
      selectedPrivileges.value.push(priv)
    }
  }
}

// ==========================================
// MODAL OPENERS
// ==========================================

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

const openAssignUser = (): void => {
  selectedUserForAssign.value = ''
  selectedDbForAssign.value = ''
  selectedPrivileges.value = ['ALL']
  showAssignModal.value = true
}

// ==========================================
// HELPERS
// ==========================================

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const openPhpMyAdmin = (): void => {
  // Open phpMyAdmin in new tab
  // Default URL, change if production uses different path
  window.open('http://localhost/phpmyadmin', '_blank')
}

const generatePassword = (): void => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%'
  let pass = ''
  for (let i = 0; i < 16; i++) {
    pass += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  newPassword.value = pass
  newConfirmPassword.value = pass
}

const generateNewPassword = (): void => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%'
  let pass = ''
  for (let i = 0; i < 16; i++) {
    pass += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  newUserPassword.value = pass
  confirmUserPassword.value = pass
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
              <Database class="w-5 h-5 text-primary" />
            </div>
            MySQL Databases
          </h1>
          <p class="text-muted-foreground mt-1 text-sm">Manage databases, users, and privileges.</p>
        </div>
        
        <div class="flex gap-2">
          <button 
            v-if="activeTab === 'databases'"
            @click="showCreateDbModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-all shadow-lg shadow-primary/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Create Database
          </button>
          <button 
            v-if="activeTab === 'users'"
            @click="showCreateUserModal = true"
            class="flex items-center gap-2 px-5 py-2.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-emerald-600/20 active:scale-[0.98] text-sm"
          >
            <Plus class="w-4 h-4" />
            Create User
          </button>
          <button 
            v-if="activeTab === 'assign'"
            @click="openAssignUser"
            class="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-indigo-600/20 active:scale-[0.98] text-sm"
          >
            <Link class="w-4 h-4" />
            Assign User to Database
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex gap-1 bg-card border border-border p-1 w-fit rounded-xl">
        <button 
          @click="activeTab = 'databases'"
          :class="['flex items-center gap-2 px-5 py-2 rounded-lg text-sm font-medium transition-all', activeTab === 'databases' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted']"
        >
          <Layers class="w-4 h-4" />
          Databases
        </button>
        <button 
          @click="activeTab = 'users'"
          :class="['flex items-center gap-2 px-5 py-2 rounded-lg text-sm font-medium transition-all', activeTab === 'users' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted']"
        >
          <User class="w-4 h-4" />
          Users
        </button>
        <button 
          @click="activeTab = 'assign'"
          :class="['flex items-center gap-2 px-5 py-2 rounded-lg text-sm font-medium transition-all', activeTab === 'assign' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-muted']"
        >
          <Link class="w-4 h-4" />
          User-Database Assignment
        </button>
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div v-for="stat in [
          { label: 'Total DB', val: databases.length, icon: Database, color: 'text-primary' },
          { label: 'Total Users', val: dbUsers.length, icon: User, color: 'text-emerald-500' },
          { label: 'Used Space', val: formatSize(databases.reduce((acc, db) => acc + db.size_bytes, 0)), icon: HardDrive, color: 'text-blue-500' },
          { label: 'Quota', val: `${databases.length} / 20`, icon: Info, color: 'text-amber-500' }
        ]" :key="stat.label" class="bg-card border border-border p-4 rounded-xl transition-colors duration-300">
          <div class="flex items-center gap-2 mb-2">
            <component :is="stat.icon" class="w-4 h-4" :class="stat.color" />
            <p class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">{{ stat.label }}</p>
          </div>
          <p class="text-xl font-bold text-foreground">{{ stat.val }}</p>
        </div>
      </div>

      <!-- Content -->
      <div class="bg-card border border-border rounded-2xl overflow-hidden min-h-[400px] transition-colors duration-300">
        <!-- Loading -->
        <div v-if="isLoading" class="p-20 flex flex-col items-center justify-center space-y-4">
          <div class="w-10 h-10 border-4 border-border border-t-primary rounded-full animate-spin" />
          <p class="text-muted-foreground font-medium text-xs uppercase tracking-wider">Loading...</p>
        </div>

        <!-- Error -->
        <div v-else-if="error && !databases.length" class="p-20 text-center">
          <div class="inline-flex p-5 bg-destructive/10 rounded-full mb-5">
            <AlertCircle class="w-10 h-10 text-destructive" />
          </div>
          <h3 class="text-lg font-bold text-foreground">Connection Failed</h3>
          <p class="text-muted-foreground mt-2 text-sm max-w-xs mx-auto">{{ error }}</p>
          <button @click="fetchData" class="mt-6 px-6 py-2.5 bg-foreground text-background rounded-lg font-medium transition-all hover:opacity-90">Retry</button>
        </div>

        <!-- Databases Tab -->
        <div v-else-if="activeTab === 'databases'">
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">MySQL Databases</span>
            <button @click="showCreateDbModal = true" class="text-xs text-primary hover:underline">+ Create New</button>
          </div>
          <div v-if="databases.length === 0" class="p-20 text-center">
            <HardDrive class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
            <h3 class="text-lg font-bold text-foreground">No databases</h3>
            <p class="text-muted-foreground mt-2 text-sm">Create your first database to get started.</p>
          </div>
          <table v-else class="w-full text-left">
            <thead>
              <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                <th class="px-6 py-4">Database</th>
                <th class="px-6 py-4">Size</th>
                <th class="px-6 py-4">Charset</th>
                <th class="px-6 py-4">Users</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr v-for="db in databases" :key="db.id" class="group hover:bg-muted/50 transition-all">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-primary/10 rounded-xl flex items-center justify-center text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-all">
                      <Layers class="w-5 h-5" />
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-foreground">{{ db.db_name }}</p>
                      <p class="text-[10px] text-muted-foreground font-mono mt-0.5">{{ db.id?.slice(0,8) }}</p>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <span class="text-xs font-medium text-foreground font-mono bg-muted px-2 py-1 rounded border border-border">{{ formatSize(db.size_bytes) }}</span>
                </td>
                <td class="px-6 py-4">
                  <span class="text-xs text-muted-foreground">{{ db.charset }} / {{ db.collation }}</span>
                </td>
                <td class="px-6 py-4">
                  <div class="flex items-center gap-1.5 px-2 py-1 bg-muted border border-border rounded-lg w-fit">
                    <User class="w-3.5 h-3.5 text-primary" />
                    <span class="text-[10px] font-semibold text-foreground">{{ db.users_count }} Users</span>
                  </div>
                </td>
                <td class="px-6 py-4 text-right">
                  <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-all">
                    <button @click="openPhpMyAdmin" class="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-all" title="phpMyAdmin">
                      <ExternalLink class="w-4 h-4" />
                    </button>
                    <button @click="openDeleteDb(db)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all" title="Delete">
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Users Tab -->
        <div v-else-if="activeTab === 'users'">
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">Database Users</span>
            <button @click="showCreateUserModal = true" class="text-xs text-primary hover:underline">+ Create New</button>
          </div>
          <div v-if="dbUsers.length === 0" class="p-20 text-center">
            <Shield class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
            <h3 class="text-lg font-bold text-foreground">No users</h3>
            <p class="text-muted-foreground mt-2 text-sm">Create database users for secure access.</p>
          </div>
          <table v-else class="w-full text-left">
            <thead>
              <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                <th class="px-6 py-4">Username</th>
                <th class="px-6 py-4">Host</th>
                <th class="px-6 py-4">Database</th>
                <th class="px-6 py-4">Privileges</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border">
              <tr v-for="dbUser in dbUsers" :key="dbUser.id" class="group hover:bg-muted/50 transition-all">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-emerald-500/10 rounded-xl flex items-center justify-center text-emerald-600 group-hover:bg-emerald-500 group-hover:text-white transition-all">
                      <User class="w-5 h-5" />
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-foreground">{{ dbUser.db_username }}</p>
                      <div class="flex items-center gap-1.5 mt-0.5">
                        <span class="w-1.5 h-1.5 rounded-full" :class="dbUser.is_active ? 'bg-emerald-500' : 'bg-destructive'" />
                        <span class="text-[9px] text-muted-foreground uppercase font-semibold">{{ dbUser.is_active ? 'Active' : 'Inactive' }}</span>
                      </div>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <p class="text-xs font-mono text-foreground bg-muted p-1.5 rounded-lg border border-border w-fit">{{ dbUser.host }}</p>
                </td>
                <td class="px-6 py-4">
                  <div v-if="dbUser.db_name" class="inline-flex items-center gap-2 px-2 py-1 bg-primary/10 border border-primary/20 rounded-lg">
                    <Database class="w-3.5 h-3.5 text-primary" />
                    <span class="text-[10px] font-semibold text-foreground">{{ dbUser.db_name }}</span>
                  </div>
                  <span v-else class="text-xs text-muted-foreground italic">Not assigned</span>
                </td>
                <td class="px-6 py-4">
                  <span class="text-[10px] font-mono text-foreground bg-muted px-2 py-1 rounded border border-border">
                    {{ dbUser.privileges || 'N/A' }}
                  </span>
                </td>
                <td class="px-6 py-4 text-right">
                  <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-all">
                    <button @click="openChangePassword(dbUser)" class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Change Password">
                      <Key class="w-4 h-4" />
                    </button>
                    <button @click="openPrivileges(dbUser)" class="p-2 text-muted-foreground hover:text-emerald-600 hover:bg-emerald-500/10 rounded-lg transition-all" title="Manage Privileges">
                      <Shield class="w-4 h-4" />
                    </button>
                    <button @click="openDeleteUser(dbUser)" class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all" title="Delete">
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Assign Tab -->
        <div v-else>
          <div class="p-4 border-b border-border flex items-center justify-between">
            <span class="text-sm font-semibold text-foreground">User-Database Assignments</span>
            <button @click="openAssignUser" class="text-xs text-primary hover:underline">+ Add Assignment</button>
          </div>
          <div class="p-6">
            <div v-if="dbUsers.filter(u => u.db_name).length === 0" class="py-16 text-center">
              <Link class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6" />
              <h3 class="text-lg font-bold text-foreground">No assignments</h3>
              <p class="text-muted-foreground mt-2 text-sm">Assign users to databases to grant access.</p>
            </div>
            <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div v-for="user in dbUsers.filter(u => u.db_name)" :key="user.id" class="bg-muted/50 border border-border p-5 rounded-xl group hover:border-primary/30 transition-all">
                <div class="flex items-center justify-between mb-4">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 bg-indigo-500/10 rounded-xl flex items-center justify-center text-indigo-600">
                      <User class="w-5 h-5" />
                    </div>
                    <div>
                      <p class="text-sm font-semibold text-foreground">{{ user.db_username }}</p>
                      <p class="text-[10px] text-muted-foreground">{{ user.host }}</p>
                    </div>
                  </div>
                  <button @click="openPrivileges(user)" class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all">
                    <Settings class="w-4 h-4" />
                  </button>
                </div>
                <div class="flex items-center gap-2 mb-3">
                  <Link class="w-4 h-4 text-muted-foreground" />
                  <div class="flex items-center gap-2 px-3 py-1.5 bg-primary/10 border border-primary/20 rounded-lg">
                    <Database class="w-4 h-4 text-primary" />
                    <span class="text-sm font-semibold text-foreground">{{ user.db_name }}</span>
                  </div>
                </div>
                <div class="flex flex-wrap gap-1">
                  <span v-for="priv in (user.privileges || 'ALL').split(',')" :key="priv" class="px-2 py-0.5 bg-emerald-500/10 text-emerald-600 text-[9px] font-semibold uppercase rounded">
                    {{ priv }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Info Panel -->
      <div class="bg-primary/5 border border-primary/20 p-6 rounded-2xl flex items-start gap-4">
        <div class="p-2.5 bg-card border border-border rounded-xl">
          <Info class="w-5 h-5 text-primary shrink-0" />
        </div>
        <div>
          <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-1">phpMyAdmin Access</p>
          <p class="text-sm text-foreground/80">Use your database credentials to access phpMyAdmin. For security, always use strong passwords and limit privileges.</p>
        </div>
      </div>
    </div>

    <!-- ==================== MODALS ==================== -->

    <!-- Create Database Modal -->
    <div v-if="showCreateDbModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showCreateDbModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Create Database</h3>
          <button @click="showCreateDbModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Database Name</label>
            <input v-model="newDbName" type="text" placeholder="my_database" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
            <p class="text-[10px] text-muted-foreground mt-1">Only alphanumeric and underscores allowed</p>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-foreground mb-2">Charset</label>
              <select v-model="newDbCharset" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
                <option v-for="opt in charsetOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-foreground mb-2">Collation</label>
              <select v-model="newDbCollation" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
                <option v-for="coll in collationOptions" :key="coll" :value="coll">{{ coll }}</option>
              </select>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showCreateDbModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createDatabase" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Create</button>
        </div>
      </div>
    </div>

    <!-- Create User Modal -->
    <div v-if="showCreateUserModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showCreateUserModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Create Database User</h3>
          <button @click="showCreateUserModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <!-- Error in modal -->
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4 flex-shrink-0" />
          {{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Username</label>
            <input v-model="newUsername" type="text" placeholder="db_user" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Host</label>
            <select v-model="newHost" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="localhost">localhost (Recommended)</option>
              <option value="%">% (All hosts - less secure)</option>
              <option value="127.0.0.1">127.0.0.1</option>
            </select>
          </div>
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-foreground">Password</label>
              <button @click="generatePassword" type="button" class="text-xs text-primary hover:underline">Generate</button>
            </div>
            <input v-model="newPassword" type="password" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Confirm Password</label>
            <input v-model="newConfirmPassword" type="password" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showCreateUserModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="createDatabaseUser" class="px-4 py-2 text-sm font-medium bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors">Create User</button>
        </div>
      </div>
    </div>

    <!-- Assign User Modal -->
    <div v-if="showAssignModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showAssignModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-lg p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Assign User to Database</h3>
          <button @click="showAssignModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4 flex-shrink-0" />
          {{ error }}
        </div>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Select User</label>
            <select v-model="selectedUserForAssign" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="">-- Select User --</option>
              <option v-for="user in dbUsers" :key="user.id" :value="user.id">{{ user.db_username }} ({{ user.host }})</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Select Database</label>
            <select v-model="selectedDbForAssign" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option value="">-- Select Database --</option>
              <option v-for="db in databases" :key="db.id" :value="db.id">{{ db.db_name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-3">Privileges</label>
            <div class="grid grid-cols-2 gap-2">
              <button 
                v-for="priv in availablePrivileges" 
                :key="priv.value"
                @click="togglePrivilege(priv.value)"
                :class="[
                  'flex items-center gap-2 p-3 border rounded-lg text-left transition-all',
                  selectedPrivileges.includes(priv.value) 
                    ? 'border-primary bg-primary/10 text-primary' 
                    : 'border-border bg-muted text-muted-foreground hover:border-primary/30'
                ]"
              >
                <component :is="selectedPrivileges.includes(priv.value) ? CheckSquare : Square" class="w-4 h-4 flex-shrink-0" />
                <div>
                  <p class="text-xs font-semibold">{{ priv.label }}</p>
                  <p class="text-[10px] opacity-70">{{ priv.desc }}</p>
                </div>
              </button>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showAssignModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="assignUserToDatabase" class="px-4 py-2 text-sm font-medium bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors">Assign</button>
        </div>
      </div>
    </div>

    <!-- Change Password Modal -->
    <div v-if="showChangePasswordModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showChangePasswordModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Change Password</h3>
          <button @click="showChangePasswordModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <div v-if="error" class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2">
          <AlertCircle class="w-4 h-4 flex-shrink-0" />
          {{ error }}
        </div>
        <p class="text-sm text-muted-foreground mb-4">
          Change password for user <span class="font-semibold text-foreground">{{ changePasswordUser?.db_username }}</span>
        </p>
        <div class="space-y-4">
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="text-sm font-medium text-foreground">New Password</label>
              <button @click="generateNewPassword" type="button" class="text-xs text-primary hover:underline">Generate</button>
            </div>
            <input v-model="newUserPassword" type="password" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
          <div>
            <label class="block text-sm font-medium text-foreground mb-2">Confirm Password</label>
            <input v-model="confirmUserPassword" type="password" placeholder="••••••••" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
          </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showChangePasswordModal = false; error = null" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="changePassword" class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors">Change Password</button>
        </div>
      </div>
    </div>

    <!-- Privileges Modal -->
    <div v-if="showPrivilegesModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showPrivilegesModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-lg p-6 animate-in">
        <div class="flex items-center justify-between mb-6">
          <h3 class="text-lg font-semibold text-foreground">Manage Privileges</h3>
          <button @click="showPrivilegesModal = false" class="p-1 rounded hover:bg-muted transition-colors">
            <X class="w-5 h-5 text-muted-foreground" />
          </button>
        </div>
        <p class="text-sm text-muted-foreground mb-4">
          Update privileges for <span class="font-semibold text-foreground">{{ editPrivilegesUser?.db_username }}</span>
          <span v-if="editPrivilegesUser?.db_name"> on <span class="font-semibold text-primary">{{ editPrivilegesUser?.db_name }}</span></span>
        </p>
        <div class="grid grid-cols-2 gap-2">
          <button 
            v-for="priv in availablePrivileges" 
            :key="priv.value"
            @click="togglePrivilege(priv.value)"
            :class="[
              'flex items-center gap-2 p-3 border rounded-lg text-left transition-all',
              selectedPrivileges.includes(priv.value) 
                ? 'border-primary bg-primary/10 text-primary' 
                : 'border-border bg-muted text-muted-foreground hover:border-primary/30'
            ]"
          >
            <component :is="selectedPrivileges.includes(priv.value) ? CheckSquare : Square" class="w-4 h-4 flex-shrink-0" />
            <div>
              <p class="text-xs font-semibold">{{ priv.label }}</p>
              <p class="text-[10px] opacity-70">{{ priv.desc }}</p>
            </div>
          </button>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <button @click="showPrivilegesModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="updatePrivileges" class="px-4 py-2 text-sm font-medium bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors">Save Privileges</button>
        </div>
      </div>
    </div>

    <!-- Delete Database Modal -->
    <div v-if="showDeleteDbModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteDbModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete Database</h3>
        <p class="text-sm text-muted-foreground mb-6">
          Are you sure you want to delete <span class="font-semibold text-foreground">{{ selectedDb?.db_name }}</span>? This will delete all data and remove all user access.
        </p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteDbModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteDatabase" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
        </div>
      </div>
    </div>

    <!-- Delete User Modal -->
    <div v-if="showDeleteUserModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showDeleteUserModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in">
        <h3 class="text-lg font-semibold text-foreground mb-2">Delete User</h3>
        <p class="text-sm text-muted-foreground mb-6">
          Are you sure you want to delete user <span class="font-semibold text-foreground">{{ selectedUser?.db_username }}</span>?
        </p>
        <div class="flex justify-end gap-3">
          <button @click="showDeleteUserModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
          <button @click="deleteDatabaseUser" class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors">Delete</button>
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
