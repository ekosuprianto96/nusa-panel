<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { appService, domainService } from '@/services'
import { 
  AppWindow, Check, Package, Loader2,
  AlertCircle 
} from 'lucide-vue-next'

const isLoadingDomains = ref(true)
const isInstalling = ref(false)
const domains = ref<any[]>([])
const error = ref<string | null>(null)
const successMsg = ref<string | null>(null)

// Apps
const apps = [
  { id: 'wordpress', name: 'WordPress', icon: '/wordpress.svg', description: 'World\'s most popular CMS.', color: 'bg-blue-500' },
  { id: 'laravel', name: 'Laravel', icon: '/laravel.svg', description: 'The PHP Framework for Web Artisans.', color: 'bg-red-500' }
]
const selectedApp = ref(apps[0]!)

// Form
const formData = ref({
  domain_id: '',
  site_title: 'My Awesome Site',
  admin_username: 'admin',
  admin_password: '',
  admin_email: '' 
})

const fetchDomains = async () => {
  try {
    isLoadingDomains.value = true
    const res = await domainService.listDomains(1, 100)
    domains.value = res.data.data.items || []
    if (domains.value.length > 0) {
      formData.value.domain_id = domains.value[0].id
      formData.value.admin_email = `admin@${domains.value[0].domain_name}`
    }
  } catch (err) {
    console.error('Failed to load domains', err)
    error.value = 'Failed to load domains. Please ensure you have added a domain first.'
  } finally {
    isLoadingDomains.value = false
  }
}

const installApp = async () => {
  if (!formData.value.domain_id || !formData.value.admin_password) return
  
  try {
    isInstalling.value = true
    error.value = null
    await appService.installApp({
      app_type: selectedApp.value.id as any,
      ...formData.value
    })
    successMsg.value = `${selectedApp.value.name} installed successfully!`
    // Reset or redirect could happen here
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Installation failed. Please check logs.'
  } finally {
    isInstalling.value = false
  }
}

const generatePassword = () => {
  formData.value.admin_password = Math.random().toString(36).slice(-12) + Math.random().toString(36).slice(-4).toUpperCase() + '!'
}

onMounted(() => {
  fetchDomains()
  generatePassword()
})
</script>

<template>
  <MainLayout>
    <div class="max-w-5xl mx-auto space-y-8 animate-in">
       <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-foreground tracking-tight flex items-center gap-3">
          <div class="p-2 bg-purple-500/10 rounded-xl">
            <Package class="w-5 h-5 text-purple-600" />
          </div>
          One-Click App Installer
        </h1>
        <p class="text-muted-foreground mt-1 text-sm">Deploy popular applications to your domains in seconds.</p>
      </div>

       <!-- Success Message -->
      <div v-if="successMsg" class="bg-emerald-50 text-emerald-600 border border-emerald-200 p-4 rounded-xl flex items-center gap-3 animate-in">
        <div class="p-2 bg-emerald-100/50 rounded-full">
          <Check class="w-5 h-5" />
        </div>
        <div>
           <h3 class="font-semibold">Installation Successful!</h3>
           <p class="text-sm opacity-90">{{ successMsg }} Check your email for details.</p>
        </div>
      </div>

      <!-- App Selection -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div v-for="app in apps" :key="app.id" 
          @click="selectedApp = app"
          :class="['cursor-pointer border-2 rounded-2xl p-6 transition-all hover:scale-[1.02] relative overflow-hidden', selectedApp.id === app.id ? 'border-primary bg-primary/5' : 'border-border bg-card hover:border-primary/50']"
        >
          <div :class="['w-12 h-12 rounded-xl flex items-center justify-center text-white font-bold text-xl mb-4', app.color]">
            {{ app.name[0] }}
          </div>
          <h3 class="text-lg font-bold text-foreground">{{ app.name }}</h3>
          <p class="text-sm text-muted-foreground mt-1">{{ app.description }}</p>

          <div v-if="selectedApp.id === app.id" class="absolute top-4 right-4 text-primary">
            <div class="w-6 h-6 bg-primary rounded-full flex items-center justify-center text-primary-foreground">
              <Check class="w-4 h-4" />
            </div>
          </div>
        </div>
      </div>

      <!-- Installation Form -->
      <div class="bg-card border border-border rounded-2xl p-8" v-if="!successMsg">
        <h3 class="text-lg font-bold flex items-center gap-2 mb-6">
          <AppWindow class="w-5 h-5 text-muted-foreground" />
          Configure {{ selectedApp.name }}
        </h3>

        <div class="grid gap-6 md:grid-cols-2">
          <!-- Domain Selection -->
          <div class="space-y-4">
            <label class="block text-sm font-medium text-foreground">Installation Destination</label>
            <div v-if="isLoadingDomains" class="flex items-center gap-2 text-muted-foreground text-sm">
               <Loader2 class="w-4 h-4 animate-spin" /> Loading domains...
            </div>
            <select v-else v-model="formData.domain_id" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none">
              <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.domain_name }}</option>
            </select>
            <p v-if="domains.length === 0 && !isLoadingDomains" class="text-xs text-red-500">
               No domains found. Please add a domain first.
            </p>
          </div>

          <!-- Site Title -->
           <div class="space-y-4">
            <label class="block text-sm font-medium text-foreground">Site Settings</label>
            <input v-model="formData.site_title" type="text" placeholder="Site Title" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
           </div>

           <!-- Admin User -->
           <div class="space-y-4">
             <label class="block text-sm font-medium text-foreground">Admin Username</label>
             <input v-model="formData.admin_username" type="text" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
           </div>

           <!-- Admin Password -->
           <div class="space-y-4">
             <div class="flex justify-between">
                <label class="block text-sm font-medium text-foreground">Admin Password</label>
                <button type="button" @click="generatePassword" class="text-xs text-primary hover:underline font-medium">Generate</button>
             </div>
             <input v-model="formData.admin_password" type="text" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
           </div>

           <!-- Admin Email -->
            <div class="space-y-4 md:col-span-2">
             <label class="block text-sm font-medium text-foreground">Admin Email</label>
             <input v-model="formData.admin_email" type="email" class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none" />
           </div>
        </div>

        <div v-if="error" class="mt-6 p-4 bg-destructive/10 text-destructive rounded-lg flex items-center gap-2 text-sm">
           <AlertCircle class="w-4 h-4" /> {{ error }}
        </div>

        <div class="mt-8 flex justify-end">
          <button 
            @click="installApp" 
            :disabled="isInstalling || !formData.domain_id"
            class="px-8 py-3 bg-primary text-primary-foreground font-semibold rounded-xl hover:bg-primary/90 transition-all shadow-lg shadow-primary/20 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <Loader2 v-if="isInstalling" class="w-5 h-5 animate-spin" />
            {{ isInstalling ? 'Installing...' : 'Install Now' }}
          </button>
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<style scoped>
.animate-in { animation: fadeIn 0.4s ease-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
</style>
