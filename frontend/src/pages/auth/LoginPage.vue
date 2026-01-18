<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Label from '@/components/ui/Label.vue'
import { Server, Lock, User, ArrowRight, AlertCircle } from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const errorMessage = ref('')
const form = ref({
  username_or_email: '',
  password: '',
})

const handleLogin = async () => {
  isLoading.value = true
  errorMessage.value = ''
  
  try {
    const success = await authStore.login({
      username_or_email: form.value.username_or_email,
      password: form.value.password
    })
    
    if (success) {
      router.push('/dashboard')
    }
  } catch (error: any) {
    console.error('Login error:', error)
    errorMessage.value = error.response?.data?.message || 'Invalid username or password'
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-[#F4F7FA] p-4 overflow-hidden relative font-sans">
    <!-- Background Decor -->
    <div class="absolute inset-0 z-0">
        <div class="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-blue-600/5 rounded-full blur-[120px]"></div>
        <div class="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-indigo-600/5 rounded-full blur-[120px]"></div>
    </div>

    <div class="w-full max-w-[420px] relative z-10 animate-in fade-in slide-in-from-bottom-4 duration-1000">
      <!-- Title/Logo Area -->
      <div class="text-center mb-10">
        <div class="inline-flex items-center justify-center p-4 bg-blue-600 rounded-3xl shadow-2xl shadow-blue-600/20 mb-6">
            <Server class="h-10 w-10 text-white" />
        </div>
        <h1 class="text-4xl font-bold text-slate-900 tracking-tight">NusaPanel</h1>
        <p class="text-slate-400 font-bold uppercase tracking-[0.2em] text-[10px] mt-2">Server Management v1.0</p>
      </div>

      <!-- Login Card -->
      <div class="bg-white border border-slate-200 rounded-[2.5rem] shadow-2xl shadow-blue-900/5 overflow-hidden">
        <div class="p-10 pb-6 text-center">
            <h2 class="text-2xl font-bold text-slate-900">Welcome Back</h2>
            <p class="text-slate-500 mt-2 font-medium">Please enter your details to sign in.</p>
        </div>

        <div class="p-10 pt-4">
          <!-- Error Alert -->
          <div v-if="errorMessage" class="mb-8 p-4 bg-red-50 border border-red-100 rounded-2xl flex items-center gap-3 text-red-600 text-sm font-semibold animate-in fade-in slide-in-from-top-2">
            <AlertCircle class="h-5 w-5 shrink-0" />
            {{ errorMessage }}
          </div>

          <form @submit.prevent="handleLogin" class="space-y-6">
            <div class="space-y-2">
              <Label for="username" class="text-[11px] uppercase tracking-widest font-bold text-slate-400 ml-1">Username / Email</Label>
              <div class="relative group">
                <User class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400 group-focus-within:text-blue-600 transition-colors" />
                <Input 
                  id="username" 
                  v-model="form.username_or_email" 
                  placeholder="admin_root" 
                  class="bg-slate-50 border-slate-200 h-14 pl-12 rounded-2xl text-slate-900 focus:ring-blue-600/10 focus:border-blue-600 focus:bg-white transition-all font-semibold"
                  required 
                />
              </div>
            </div>
            
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <Label for="password" class="text-[11px] uppercase tracking-widest font-bold text-slate-400 ml-1">Password</Label>
                <a href="#" class="text-[10px] font-bold uppercase tracking-widest text-blue-600 hover:text-blue-700 transition-colors">Forgot?</a>
              </div>
              <div class="relative group">
                <Lock class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400 group-focus-within:text-blue-600 transition-colors" />
                <Input 
                  id="password" 
                  type="password" 
                  v-model="form.password" 
                  placeholder="••••••••" 
                  class="bg-slate-50 border-slate-200 h-14 pl-12 rounded-2xl text-slate-900 focus:ring-blue-600/10 focus:border-blue-600 focus:bg-white transition-all font-semibold"
                  required 
                />
              </div>
            </div>
            
            <Button type="submit" class="w-full h-14 bg-blue-600 hover:bg-blue-700 text-white font-bold text-base rounded-2xl shadow-xl shadow-blue-600/20 transition-all group mt-4 active:scale-95" :disabled="isLoading">
              <span v-if="isLoading">Processing...</span>
              <span v-else class="flex items-center justify-center gap-2">
                Sign in to Panel
                <ArrowRight class="h-5 w-5 group-hover:translate-x-1 transition-all" />
              </span>
            </Button>
          </form>
        </div>

        <div class="p-8 bg-slate-50/50 text-center border-t border-slate-100">
            <p class="text-[10px] font-bold text-slate-400 tracking-wider">SECURE 256-BIT ENCRYPTION ENABLED</p>
        </div>
      </div>
      
      <p class="text-center mt-10 text-slate-500 text-sm font-medium">
        Don't have an account? <a href="#" class="text-blue-600 font-bold hover:underline ml-1">Contact Administrator</a>
      </p>
    </div>
  </div>
</template>


<style scoped>
/* Custom animations if needed */
</style>
