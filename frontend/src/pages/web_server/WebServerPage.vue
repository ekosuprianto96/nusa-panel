<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { webserverService } from '@/services'
import { 
  Globe, Server, Settings, ShieldCheck, 
  RefreshCw, FileJson, Check, AlertCircle,
  X
} from 'lucide-vue-next'

const isLoading = ref(true)
const vhosts = ref<any[]>([])
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)

// Modal States
const showConfigModal = ref(false)
const selectedVHost = ref<any>(null)
const customConfig = ref('')

// PHP Versions
const phpVersions = ['7.4', '8.0', '8.1', '8.2', '8.3']

const showSuccess = (msg: string) => {
  successMsg.value = msg
  setTimeout(() => successMsg.value = null, 3000)
}

const fetchVHosts = async () => {
  try {
    isLoading.value = true
    const res = await webserverService.listVHosts()
    vhosts.value = res.data.data || []
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to load Virtual Hosts'
  } finally {
    isLoading.value = false
  }
}

const updatePhpVersion = async (vhost: any, version: string) => {
  try {
    await webserverService.updateVHost(vhost.id, { php_version: version })
    showSuccess(`PHP version updated to ${version} for ${vhost.domain_name}`)
    vhost.php_version = version // Optimistic update
  } catch (err: any) {
    error.value = 'Failed to update PHP version'
  }
}

const restartService = async (service: 'nginx' | 'php') => {
  // Mock restart
  showSuccess(`Restarting ${service} service...`)
  // Actual API call would go here
}

const openConfig = (vhost: any) => {
  selectedVHost.value = vhost
  customConfig.value = vhost.custom_config || '# Custom Nginx Configuration\nlocation /custom {\n  deny all;\n}'
  showConfigModal.value = true
}

const saveConfig = async () => {
  if (!selectedVHost.value) return
  try {
    await webserverService.updateVHost(selectedVHost.value.id, { custom_config: customConfig.value })
    showSuccess('Configuration saved successfully')
    showConfigModal.value = false
  } catch (err: any) {
    error.value = 'Failed to save configuration'
  }
}

onMounted(() => {
  fetchVHosts()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-6xl mx-auto space-y-6 animate-in">
      <!-- Success Message -->
      <div v-if="successMsg" class="fixed top-4 right-4 bg-emerald-500 text-white px-6 py-3 rounded-xl shadow-lg z-50 flex items-center gap-2 animate-in">
        <Check class="w-5 h-5" />
        {{ successMsg }}
      </div>

      <!-- Header -->
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
           <h1 class="text-2xl font-bold text-foreground tracking-tight flex items-center gap-3">
            <div class="p-2 bg-green-500/10 rounded-xl">
              <Server class="w-5 h-5 text-green-600" />
            </div>
            Web Server Manager
          </h1>
          <p class="text-muted-foreground mt-1 text-sm">Manage Nginx Virtual Hosts and PHP versions.</p>
        </div>
        
        <div class="flex gap-2">
           <button @click="restartService('nginx')" class="btn-secondary">
             <RefreshCw class="w-4 h-4 mr-2" /> Restart Nginx
           </button>
           <button @click="restartService('php')" class="btn-secondary">
             <RefreshCw class="w-4 h-4 mr-2" /> Restart PHP-FPM
           </button>
        </div>
      </div>

      <!-- Content -->
      <div class="bg-card border border-border rounded-2xl overflow-hidden min-h-[400px]">
        <div v-if="isLoading" class="p-20 text-center">
           <div class="w-10 h-10 border-4 border-muted border-t-green-500 rounded-full animate-spin mx-auto" />
           <p class="text-muted-foreground mt-4 text-sm">Loading Virtual Hosts...</p>
        </div>

        <div v-else-if="vhosts.length === 0" class="p-20 text-center">
           <Globe class="w-16 h-16 text-muted-foreground/30 mx-auto mb-4" />
           <h3 class="text-lg font-bold">No Virtual Hosts</h3>
           <p class="text-muted-foreground mt-2">Add a domain in the Domains menu to create a Virtual Host.</p>
           <router-link to="/dashboard/domains" class="btn-primary mt-4 inline-flex">Go to Domains</router-link>
        </div>

        <table v-else class="w-full text-left">
           <thead>
             <tr class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
               <th class="px-6 py-4">Domain</th>
               <th class="px-6 py-4">PHP Version</th>
               <th class="px-6 py-4">SSL Status</th>
               <th class="px-6 py-4 text-right">Actions</th>
             </tr>
           </thead>
           <tbody class="divide-y divide-border">
             <tr v-for="vhost in vhosts" :key="vhost.id" class="group hover:bg-muted/50 transition-colors">
               <td class="px-6 py-4">
                 <div class="flex items-center gap-3">
                   <div class="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-600">
                     <Globe class="w-4 h-4" />
                   </div>
                   <div>
                     <div class="font-medium text-foreground">{{ vhost.domain_name }}</div>
                     <div class="text-[10px] text-muted-foreground font-mono">{{ vhost.document_root }}</div>
                   </div>
                 </div>
               </td>
               <td class="px-6 py-4">
                 <select 
                   @change="updatePhpVersion(vhost, ($event.target as HTMLSelectElement).value)"
                   class="bg-muted border border-border rounded-lg px-2 py-1 text-xs font-medium focus:ring-2 focus:ring-green-500/20 outline-none"
                  >
                   <option v-for="ver in phpVersions" :key="ver" :value="ver" :selected="vhost.php_version === ver">PHP {{ ver }}</option>
                 </select>
               </td>
               <td class="px-6 py-4">
                 <div class="flex items-center gap-1.5">
                   <ShieldCheck v-if="vhost.ssl_enabled" class="w-4 h-4 text-emerald-500" />
                   <AlertCircle v-else class="w-4 h-4 text-amber-500" />
                   <span :class="['text-xs font-medium', vhost.ssl_enabled ? 'text-emerald-600' : 'text-amber-600']">
                     {{ vhost.ssl_enabled ? 'Secured' : 'Not Secured' }}
                   </span>
                 </div>
               </td>
               <td class="px-6 py-4 text-right">
                 <button @click="openConfig(vhost)" class="p-2 text-muted-foreground hover:text-primary hover:bg-primary/10 rounded-lg transition-all" title="Edit Nginx Config">
                   <FileJson class="w-4 h-4" />
                 </button>
               </td>
             </tr>
           </tbody>
        </table>
      </div>
    </div>

    <!-- Config Modal -->
    <div v-if="showConfigModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showConfigModal = false">
      <div class="bg-card border border-border rounded-xl shadow-xl w-full max-w-2xl animate-in flex flex-col max-h-[90vh]">
        <div class="p-6 border-b border-border flex items-center justify-between">
           <h3 class="text-lg font-semibold flex items-center gap-2">
             <Settings class="w-5 h-5" />
             Nginx Config: {{ selectedVHost?.domain_name }}
           </h3>
           <button @click="showConfigModal = false"><X class="w-5 h-5 text-muted-foreground" /></button>
        </div>
        <div class="p-6 flex-1 overflow-hidden flex flex-col">
           <p class="text-sm text-muted-foreground mb-4">Add custom Nginx directives for this virtual host. Be careful, invalid config can crash Nginx.</p>
           <textarea 
             v-model="customConfig"
             class="w-full flex-1 bg-muted font-mono text-sm border border-border rounded-lg p-4 outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary resize-none"
             spellcheck="false"
           ></textarea>
        </div>
        <div class="p-6 border-t border-border flex justify-end gap-3">
          <button @click="showConfigModal = false" class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground">Cancel</button>
          <button @click="saveConfig" class="px-4 py-2 bg-primary text-primary-foreground text-sm font-medium rounded-lg hover:bg-primary/90">Save & Reload</button>
        </div>
      </div>
    </div>

  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeIn 0.3s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

.btn-primary { @apply px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg font-medium text-sm transition-all; }
.btn-secondary { @apply px-4 py-2 bg-muted hover:bg-muted/80 text-foreground rounded-lg font-medium text-sm transition-all flex items-center; }
</style>
