<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { userService, securityService, systemService } from '@/services'
import { useToastStore } from '@/stores/toast'
import type { AdminUserStats, UserResponse } from '@/types'
import { Activity, Users, Shield, Server, HardDrive, Cpu, MemoryStick } from 'lucide-vue-next'

const toast = useToastStore()

const users = ref<UserResponse[]>([])
const totalUsers = ref(0)
const isLoadingUsers = ref(true)
const accessLogs = ref<any[]>([])
const resourceUsage = ref<{ cpu: number; memory: number; disk: number } | null>(null)
const adminStats = ref<AdminUserStats | null>(null)

const latestUsers = computed(() => users.value.slice(0, 6))

const formatDate = (date: string) => {
  return new Date(date).toLocaleDateString('id-ID', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 GB'
  const gb = bytes / 1024 / 1024 / 1024
  return `${gb.toFixed(2)} GB`
}

const statsCards = computed(() => {
  const usage = resourceUsage.value
  const stats = adminStats.value
  return [
    {
      label: 'CPU Usage',
      value: usage ? `${usage.cpu.toFixed(1)}%` : '0%',
      icon: Cpu,
      color: 'text-blue-600',
      bg: 'bg-blue-500/10'
    },
    {
      label: 'Memory Used',
      value: usage ? formatBytes(usage.memory) : '0 GB',
      icon: MemoryStick,
      color: 'text-emerald-600',
      bg: 'bg-emerald-500/10'
    },
    {
      label: 'Disk Used',
      value: usage ? formatBytes(usage.disk) : '0 GB',
      icon: HardDrive,
      color: 'text-amber-600',
      bg: 'bg-amber-500/10'
    },
    {
      label: 'Total Users',
      value: (stats?.total_users ?? totalUsers.value).toString(),
      icon: Users,
      color: 'text-indigo-600',
      bg: 'bg-indigo-500/10'
    },
    {
      label: 'Active Users',
      value: (stats?.active_users ?? 0).toString(),
      icon: Users,
      color: 'text-emerald-600',
      bg: 'bg-emerald-500/10'
    },
    {
      label: 'Blocked Users',
      value: (stats?.blocked_users ?? 0).toString(),
      icon: Users,
      color: 'text-red-600',
      bg: 'bg-red-500/10'
    },
    {
      label: 'New Signups (7d)',
      value: (stats?.new_signups_7d ?? 0).toString(),
      icon: Users,
      color: 'text-purple-600',
      bg: 'bg-purple-500/10'
    }
  ]
})

const fetchUsers = async () => {
  isLoadingUsers.value = true
  try {
    const res = await userService.listUsers(1, 10)
    users.value = res.data.data || []
    totalUsers.value = res.data.pagination?.total || 0
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Failed to load users')
  } finally {
    isLoadingUsers.value = false
  }
}

const fetchAdminStats = async () => {
  try {
    const res = await userService.getAdminStats()
    adminStats.value = res.data.data || null
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Failed to load admin stats')
  }
}

const fetchAccessLogs = async () => {
  try {
    const res = await securityService.getAccessLogs()
    accessLogs.value = res.data.data || []
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Failed to load access logs')
  }
}

const fetchResourceUsage = async () => {
  try {
    const res = await systemService.getResourceUsage()
    resourceUsage.value = res.data.data || null
  } catch (e: any) {
    toast.error(e.response?.data?.message || 'Failed to load resource usage')
  }
}

onMounted(() => {
  fetchUsers()
  fetchAdminStats()
  fetchAccessLogs()
  fetchResourceUsage()
})
</script>

<template>
  <MainLayout>
    <div class="space-y-8 pb-10">
      <!-- Header -->
      <div class="flex flex-wrap justify-between items-end gap-4">
        <div>
          <h1 class="text-3xl font-black text-[#0d131b] dark:text-white">Admin Dashboard</h1>
          <p class="text-slate-500 text-sm mt-2">Server health, monitoring, and user activity overview.</p>
        </div>
        <div class="flex items-center gap-2 text-xs font-semibold text-slate-500">
          <Server class="w-4 h-4" />
          <span>System Control Center</span>
        </div>
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        <div
          v-for="card in statsCards"
          :key="card.label"
          class="bg-card border border-border rounded-2xl p-5 flex items-center gap-4 hover:border-primary/30 transition-colors"
        >
          <div :class="['p-3 rounded-xl', card.bg, card.color]">
            <component :is="card.icon" class="w-5 h-5" />
          </div>
          <div>
            <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">{{ card.label }}</p>
            <p class="text-2xl font-bold text-foreground">{{ card.value }}</p>
          </div>
        </div>
      </div>

      <!-- Main Panels -->
      <div class="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <!-- Latest Users -->
        <div class="xl:col-span-2 bg-card border border-border rounded-2xl p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
              <Users class="w-4 h-4 text-primary" />
              <h3 class="text-lg font-bold text-foreground">New Users</h3>
            </div>
            <router-link to="/dashboard/users" class="text-xs font-semibold text-primary hover:underline">View all</router-link>
          </div>
          <div v-if="isLoadingUsers" class="text-sm text-muted-foreground">Loading users...</div>
          <div v-else class="space-y-3">
            <div
              v-for="user in latestUsers"
              :key="user.id"
              class="flex items-center justify-between bg-background border border-border rounded-xl px-4 py-3"
            >
              <div>
                <p class="text-sm font-semibold text-foreground">{{ user.full_name || user.username }}</p>
                <p class="text-xs text-muted-foreground">{{ user.email }}</p>
              </div>
              <div class="text-right">
                <p class="text-xs font-semibold text-muted-foreground">{{ formatDate(user.created_at) }}</p>
                <span class="text-[10px] uppercase tracking-wide text-slate-500">{{ user.role }}</span>
              </div>
            </div>
            <div v-if="latestUsers.length === 0" class="text-sm text-muted-foreground">No users found.</div>
          </div>
        </div>

        <!-- Monitoring -->
        <div class="bg-card border border-border rounded-2xl p-6">
          <div class="flex items-center gap-2 mb-4">
            <Activity class="w-4 h-4 text-primary" />
            <h3 class="text-lg font-bold text-foreground">Monitoring</h3>
          </div>
          <div class="space-y-4">
            <div class="p-4 rounded-xl bg-background border border-border">
              <p class="text-xs uppercase tracking-wide text-muted-foreground font-semibold mb-1">Security Events</p>
              <p class="text-2xl font-bold text-foreground">{{ accessLogs.length }}</p>
              <p class="text-xs text-muted-foreground">Latest 100 events</p>
            </div>
            <div class="p-4 rounded-xl bg-background border border-border">
              <p class="text-xs uppercase tracking-wide text-muted-foreground font-semibold mb-1">Service Health</p>
              <p class="text-sm text-foreground">Monitoring data is updated automatically from the server.</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Recent Security Logs -->
      <div class="bg-card border border-border rounded-2xl p-6">
        <div class="flex items-center gap-2 mb-4">
          <Shield class="w-4 h-4 text-primary" />
          <h3 class="text-lg font-bold text-foreground">Recent Security Logs</h3>
        </div>
        <div class="space-y-3">
          <div
            v-for="log in accessLogs.slice(0, 6)"
            :key="log.id"
            class="flex items-center justify-between bg-background border border-border rounded-xl px-4 py-3"
          >
            <div>
              <p class="text-sm font-semibold text-foreground">{{ log.event_type }}</p>
              <p class="text-xs text-muted-foreground">{{ log.ip_address || 'unknown' }}</p>
            </div>
            <div class="text-right text-xs text-muted-foreground">
              {{ log.timestamp ? formatDate(log.timestamp) : '—' }}
            </div>
          </div>
          <div v-if="accessLogs.length === 0" class="text-sm text-muted-foreground">No security logs available.</div>
        </div>
      </div>
    </div>
  </MainLayout>
</template>
