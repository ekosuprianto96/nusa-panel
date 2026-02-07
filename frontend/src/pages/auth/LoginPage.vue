<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { Button, Input } from '@/components/ui'
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from '@/components/ui/card'
import Label from '@/components/ui/Label.vue'
import { Server, Lock, User, ArrowRight, AlertCircle } from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()

const isLoading = ref(false)
const errorMessage = ref('')
const requiresTwoFa = ref(false)
const form = ref({
  username_or_email: '',
  password: '',
  two_fa_code: '',
})

const handleLogin = async () => {
  isLoading.value = true
  errorMessage.value = ''
  
  try {
    const success = await authStore.login({
      username_or_email: form.value.username_or_email,
      password: form.value.password,
      two_fa_code: requiresTwoFa.value ? form.value.two_fa_code : undefined,
    })
    
    if (success) {
      requiresTwoFa.value = false
      const userRole = authStore.user?.role || 'user'
      if (userRole === 'admin' || userRole === 'reseller') {
        router.push('/admin')
      } else {
        router.push('/dashboard')
      }
    }
  } catch (error: any) {
    console.error('Login error:', error)
    const errorCode = error.response?.data?.error_code
    if (errorCode === 'TWO_FACTOR_REQUIRED') {
      requiresTwoFa.value = true
      errorMessage.value = 'Masukkan kode 2FA untuk melanjutkan'
    } else if (errorCode === 'TWO_FACTOR_INVALID') {
      requiresTwoFa.value = true
      errorMessage.value = 'Kode 2FA tidak valid'
    } else {
      errorMessage.value = error.response?.data?.message || 'Invalid username or password'
    }
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-background p-4 overflow-hidden relative font-sans">
    <!-- Background Decor -->
    <div class="absolute inset-0 z-0">
      <div class="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary/5 rounded-full blur-[120px]"></div>
      <div class="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-primary/5 rounded-full blur-[120px]"></div>
    </div>

    <div class="w-full max-w-[420px] relative z-10 animate-in fade-in slide-in-from-bottom-4 duration-1000">
      <!-- Title/Logo Area -->
      <div class="text-center mb-10">
        <div class="inline-flex items-center justify-center p-4 bg-primary rounded-3xl shadow-2xl shadow-primary/20 mb-6">
          <Server class="h-10 w-10 text-primary-foreground" />
        </div>
        <h1 class="text-4xl font-bold text-foreground tracking-tight">NusaPanel</h1>
        <p class="text-muted-foreground font-bold uppercase tracking-[0.2em] text-[10px] mt-2">Server Management v1.0</p>
      </div>

      <!-- Login Card -->
      <Card class="rounded-[2.5rem] shadow-2xl shadow-primary/5 overflow-hidden border-border">
        <CardHeader class="p-10 pb-6 text-center">
          <CardTitle class="text-2xl">Welcome Back</CardTitle>
          <CardDescription class="mt-2 font-medium">Please enter your details to sign in.</CardDescription>
        </CardHeader>

        <CardContent class="px-10 pb-6">
          <!-- Error Alert -->
          <div v-if="errorMessage" class="mb-8 p-4 bg-destructive/10 border border-destructive/20 rounded-2xl flex items-center gap-3 text-destructive text-sm font-semibold animate-in fade-in slide-in-from-top-2">
            <AlertCircle class="h-5 w-5 shrink-0" />
            {{ errorMessage }}
          </div>

          <form @submit.prevent="handleLogin" class="space-y-6">
            <div class="space-y-2">
              <Label for="username" class="text-[11px] uppercase tracking-widest font-bold text-muted-foreground ml-1">Username / Email</Label>
              <div class="relative group">
                <User class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input 
                  id="username" 
                  v-model="form.username_or_email" 
                  placeholder="admin_root" 
                  class="bg-muted/50 border-border h-14 pl-12 rounded-2xl text-foreground focus:ring-primary/10 focus:border-primary focus:bg-background transition-all font-semibold"
                  required 
                />
              </div>
            </div>
            
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <Label for="password" class="text-[11px] uppercase tracking-widest font-bold text-muted-foreground ml-1">Password</Label>
                <a href="#" class="text-[10px] font-bold uppercase tracking-widest text-primary hover:text-primary/80 transition-colors">Forgot?</a>
              </div>
              <div class="relative group">
                <Lock class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input 
                  id="password" 
                  type="password" 
                  v-model="form.password" 
                  placeholder="••••••••" 
                  class="bg-muted/50 border-border h-14 pl-12 rounded-2xl text-foreground focus:ring-primary/10 focus:border-primary focus:bg-background transition-all font-semibold"
                  required 
                />
              </div>
            </div>

            <div v-if="requiresTwoFa" class="space-y-2">
              <Label for="twofa" class="text-[11px] uppercase tracking-widest font-bold text-muted-foreground ml-1">2FA Code</Label>
              <div class="relative group">
                <Lock class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input
                  id="twofa"
                  v-model="form.two_fa_code"
                  placeholder="6-digit code"
                  maxlength="6"
                  class="bg-muted/50 border-border h-14 pl-12 rounded-2xl text-foreground focus:ring-primary/10 focus:border-primary focus:bg-background transition-all font-semibold tracking-widest"
                  required
                />
              </div>
            </div>
            
            <Button 
              type="submit" 
              size="lg"
              class="w-full h-14 rounded-2xl shadow-xl shadow-primary/20 transition-all group mt-4 active:scale-95" 
              :disabled="isLoading"
            >
              <span v-if="isLoading">Processing...</span>
              <span v-else class="flex items-center justify-center gap-2">
                Sign in to Panel
                <ArrowRight class="h-5 w-5 group-hover:translate-x-1 transition-all" />
              </span>
            </Button>
          </form>
        </CardContent>

        <CardFooter class="p-8 bg-muted/30 text-center border-t border-border justify-center">
          <p class="text-[10px] font-bold text-muted-foreground tracking-wider">SECURE 256-BIT ENCRYPTION ENABLED</p>
        </CardFooter>
      </Card>
      
      <p class="text-center mt-10 text-muted-foreground text-sm font-medium">
        Don't have an account? <a href="#" class="text-primary font-bold hover:underline ml-1">Contact Administrator</a>
      </p>
    </div>
  </div>
</template>

<style scoped>
/* Custom animations if needed */
</style>
