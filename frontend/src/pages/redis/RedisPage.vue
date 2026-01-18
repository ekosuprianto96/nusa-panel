<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { redisService } from '@/services'
import { 
  Database, Power, Activity, Server, 
  Copy, Check, RefreshCw 
} from 'lucide-vue-next'

// State
const isLoading = ref(true)
const isProcessing = ref(false)
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)
const redisStatus = ref<any>(null)

// Form
const maxMemory = ref('128mb')

// Helper
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    showSuccess('Copied to clipboard')
  } catch (err) {
    console.error('Failed to copy', err)
  }
}

const showSuccess = (msg: string) => {
  successMsg.value = msg
  setTimeout(() => successMsg.value = null, 3000)
}

// API
const fetchStatus = async () => {
  try {
    isLoading.value = true
    error.value = null
    const res = await redisService.getStatus()
    redisStatus.value = res.data.data
  } catch (err: any) {
    if (err.response?.status === 404) {
      redisStatus.value = null // Not found = inactive
    } else {
      error.value = err.response?.data?.message || 'Failed to fetch Redis status'
    }
  } finally {
    isLoading.value = false
  }
}

const enableRedis = async () => {
  try {
    isProcessing.value = true
    await redisService.enableRedis({ max_memory: maxMemory.value })
    showSuccess('Redis instance activated successfully')
    await fetchStatus()
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to activate Redis'
  } finally {
    isProcessing.value = false
  }
}

const disableRedis = async () => {
  if (!confirm('Are you sure you want to disable Redis? All data will be lost.')) return
  try {
    isProcessing.value = true
    await redisService.disableRedis()
    showSuccess('Redis instance disabled')
    redisStatus.value = null
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to disable Redis'
  } finally {
    isProcessing.value = false
  }
}

onMounted(() => {
  fetchStatus()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-4xl mx-auto space-y-6 animate-in">
      <!-- Success Message -->
      <div v-if="successMsg" class="fixed top-4 right-4 bg-emerald-500 text-white px-6 py-3 rounded-xl shadow-lg z-50 flex items-center gap-2 animate-in">
        <Check class="w-5 h-5" />
        {{ successMsg }}
      </div>

      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-foreground tracking-tight flex items-center gap-3">
          <div class="p-2 bg-red-500/10 rounded-xl">
            <Database class="w-5 h-5 text-red-600" />
          </div>
          Redis Manager
        </h1>
        <p class="text-muted-foreground mt-1 text-sm">Manage your private Redis instance for caching and queues.</p>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="p-20 text-center">
        <div class="w-10 h-10 border-4 border-muted border-t-red-500 rounded-full animate-spin mx-auto" />
        <p class="text-muted-foreground mt-4 text-sm">Loading Redis status...</p>
      </div>

      <!-- Active State -->
      <div v-else-if="redisStatus && redisStatus.is_active !== false" class="bg-card border border-border rounded-2xl overflow-hidden shadow-sm">
        <div class="p-6 border-b border-border flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 bg-emerald-500 rounded-full animate-pulse" />
            <h3 class="font-semibold text-lg">Redis is Active</h3>
          </div>
          <button @click="fetchStatus" class="p-2 text-muted-foreground hover:text-primary rounded-lg hover:bg-muted transition-colors">
            <RefreshCw class="w-4 h-4" />
          </button>
        </div>

        <div class="p-6 grid gap-6 md:grid-cols-2">
           <!-- Connection Info -->
           <div class="space-y-4">
              <h4 class="text-sm font-medium text-muted-foreground uppercase tracking-wider">Connection Details</h4>
              
              <div class="space-y-3">
                <div class="bg-muted/50 p-3 rounded-xl border border-border">
                  <label class="text-xs text-muted-foreground block mb-1">Host</label>
                  <div class="flex items-center justify-between">
                    <code class="font-mono text-sm font-semibold text-foreground">127.0.0.1</code>
                    <button @click="copyToClipboard('127.0.0.1')" class="text-muted-foreground hover:text-primary"><Copy class="w-4 h-4" /></button>
                  </div>
                </div>

                <div class="bg-muted/50 p-3 rounded-xl border border-border">
                  <label class="text-xs text-muted-foreground block mb-1">Port</label>
                  <div class="flex items-center justify-between">
                    <code class="font-mono text-sm font-semibold text-foreground">{{ redisStatus.port || 6379 }}</code>
                    <button @click="copyToClipboard(String(redisStatus.port || 6379))" class="text-muted-foreground hover:text-primary"><Copy class="w-4 h-4" /></button>
                  </div>
                </div>

                <div class="bg-muted/50 p-3 rounded-xl border border-border">
                  <label class="text-xs text-muted-foreground block mb-1">Password</label>
                  <div class="flex items-center justify-between">
                    <code class="font-mono text-sm font-semibold text-foreground">{{ redisStatus.password || '••••••••' }}</code>
                     <button @click="copyToClipboard(redisStatus.password)" class="text-muted-foreground hover:text-primary"><Copy class="w-4 h-4" /></button>
                  </div>
                </div>
              </div>
           </div>

           <!-- Stats & Actions -->
           <div class="flex flex-col justify-between">
              <div>
                 <h4 class="text-sm font-medium text-muted-foreground uppercase tracking-wider mb-4">Instance Status</h4>
                 <div class="flex items-center gap-4 mb-6">
                    <div class="flex-1 bg-blue-500/10 border border-blue-500/20 p-4 rounded-xl text-center">
                       <Activity class="w-6 h-6 text-blue-600 mx-auto mb-2" />
                       <div class="text-xl font-bold text-foreground">Using {{ redisStatus.memory_used || '0MB' }}</div>
                       <div class="text-xs text-muted-foreground">Memory Used</div>
                    </div>
                    <div class="flex-1 bg-purple-500/10 border border-purple-500/20 p-4 rounded-xl text-center">
                       <Server class="w-6 h-6 text-purple-600 mx-auto mb-2" />
                       <div class="text-xl font-bold text-foreground">{{ redisStatus.max_memory || '128mb' }}</div>
                       <div class="text-xs text-muted-foreground">Max Memory</div>
                    </div>
                 </div>
              </div>

              <div class="border-t border-border pt-6">
                 <button 
                  @click="disableRedis" 
                  :disabled="isProcessing"
                  class="w-full py-3 px-4 bg-red-50 text-red-600 border border-red-200 hover:bg-red-100 hover:border-red-300 rounded-xl font-medium transition-all flex items-center justify-center gap-2"
                >
                  <Power class="w-4 h-4" />
                  {{ isProcessing ? 'Disabling...' : 'Disable Redis Instance' }}
                 </button>
              </div>
           </div>
        </div>
      </div>

      <!-- Inactive State -->
      <div v-else class="bg-card border border-border rounded-2xl overflow-hidden shadow-sm p-12 text-center">
         <div class="w-16 h-16 bg-muted rounded-full flex items-center justify-center mx-auto mb-6">
            <Database class="w-8 h-8 text-muted-foreground" />
         </div>
         <h3 class="text-xl font-bold text-foreground mb-2">Redis is Not Enabled</h3>
         <p class="text-muted-foreground max-w-md mx-auto mb-8">
           Enable a private Redis instance for your account to speed up your applications with in-memory caching.
         </p>

         <div class="max-w-xs mx-auto space-y-4">
            <div>
               <label class="text-sm font-medium text-foreground block mb-2 text-left">Max Memory Limit</label>
               <select v-model="maxMemory" class="w-full bg-muted border border-border rounded-lg px-4 py-2.5 text-sm outline-none focus:ring-2 focus:ring-red-500/20 focus:border-red-500">
                  <option value="64mb">64 MB</option>
                  <option value="128mb">128 MB (Recommended)</option>
                  <option value="256mb">256 MB</option>
                  <option value="512mb">512 MB</option>
               </select>
            </div>
            <button 
              @click="enableRedis"
              :disabled="isProcessing"
              class="w-full py-2.5 px-4 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-all shadow-lg shadow-red-600/20 active:scale-[0.98] flex items-center justify-center gap-2"
            >
              <Power class="w-4 h-4" />
              {{ isProcessing ? 'Activating...' : 'Enable Redis' }}
            </button>
         </div>
      </div>
    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeIn 0.3s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
</style>
