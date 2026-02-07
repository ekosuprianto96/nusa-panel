<script setup lang="ts">
/**
 * TwoFactorAuthPage - 2FA Security Setup
 */
import { ref, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui'
import { securityService } from '@/services'
import { Info, Copy, Download, Printer, QrCode, CheckCircle, Lock, Shield } from 'lucide-vue-next'

const router = useRouter()
const isLoading = ref(true)
const twoFaEnabled = ref(false)
const qrCodeUrl = ref('')
const secretKey = ref('')
const verificationCode = ref(['', '', '', '', '', ''])
const isVerifying = ref(false)
const backupCodes = ref<string[]>([])
const codeInputs = ref<HTMLInputElement[]>([])

import { useToastStore } from '@/stores/toast'
const toast = useToastStore()

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await securityService.get2faStatus()
        twoFaEnabled.value = res.data.data?.enabled || false
        if (!twoFaEnabled.value) {
            const setupRes = await securityService.setup2fa()
            qrCodeUrl.value = setupRes.data.data?.qr_code || ''
            secretKey.value = setupRes.data.data?.secret || ''
            backupCodes.value = setupRes.data.data?.backup_codes || []
        }
    } catch (e) { console.error('Failed to load 2FA status:', e) }
    finally { isLoading.value = false }
}

const handleCodeInput = (index: number, event: Event) => {
    const input = event.target as HTMLInputElement
    const value = input.value.replace(/\D/g, '')
    verificationCode.value[index] = value.slice(0, 1)
    if (value && index < 5) nextTick(() => { codeInputs.value[index + 1]?.focus() })
}

const handleCodeKeydown = (index: number, event: KeyboardEvent) => {
    if (event.key === 'Backspace' && !verificationCode.value[index] && index > 0)
        nextTick(() => { codeInputs.value[index - 1]?.focus() })
}

const handlePaste = (event: ClipboardEvent) => {
    event.preventDefault()
    const paste = event.clipboardData?.getData('text')?.replace(/\D/g, '').slice(0, 6)
    if (paste) { paste.split('').forEach((char, i) => { if (i < 6) verificationCode.value[i] = char }); codeInputs.value[Math.min(paste.length, 5)]?.focus() }
}

const enable2fa = async () => {
    const code = verificationCode.value.join('')
    if (code.length !== 6) { toast.error('Please enter a 6-digit code'); return }
    isVerifying.value = true
    try { await securityService.verify2fa({ code }); twoFaEnabled.value = true; toast.success('2FA enabled successfully!') }
    catch (e) { toast.error('Invalid verification code') }
    finally { isVerifying.value = false }
}

const disable2fa = async () => {
    try {
        await securityService.disable2fa()
        twoFaEnabled.value = false
        await fetchData()
        toast.success('2FA disabled')
    } catch (e) {
        toast.error('Failed to disable 2FA')
    }
}

const copySecretKey = async () => {
    try { await navigator.clipboard.writeText(secretKey.value.replace(/\s/g, '')); toast.success('Secret key copied') }
    catch (e) { toast.error('Failed to copy') }
}

const downloadBackupCodes = () => {
    const blob = new Blob([backupCodes.value.join('\n')], { type: 'text/plain' })
    const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'backup-codes.txt'; a.click()
    toast.success('Backup codes downloaded')
}

const printBackupCodes = () => {
    const w = window.open('', '_blank')
    if (w) { w.document.write(`<html><head><title>2FA Backup Codes</title></head><body style="font-family:monospace;padding:20px;"><h2>Backup Codes</h2><pre style="font-size:16px;line-height:2;">${backupCodes.value.join('\n')}</pre></body></html>`); w.document.close(); w.print() }
}

onMounted(fetchData)
</script>

<template>
<MainLayout>
    <div class="space-y-8">
        <!-- Header -->
        <div>
            <AppBreadcrumb
                class="mb-4"
                :items="[
                    { label: 'Security Center', icon: Shield, onClick: () => router.push('/dashboard/security') },
                    { label: '2FA Setup', current: true }
                ]"
            />
            <h2 class="text-3xl font-black text-foreground">2FA Security Setup</h2>
            <p class="text-muted-foreground text-sm mt-1">Add an extra layer of security to your account.</p>
        </div>

        <!-- Loading -->
        <div v-if="isLoading" class="flex justify-center py-20">
            <div class="animate-spin w-10 h-10 border-3 border-primary border-t-transparent rounded-full"></div>
        </div>

        <!-- 2FA Enabled State -->
        <Card v-else-if="twoFaEnabled" class="rounded-2xl p-12 text-center">
            <CardContent>
                <div class="w-20 h-20 bg-emerald-500/10 rounded-full flex items-center justify-center mx-auto mb-6">
                    <CheckCircle :size="48" class="text-emerald-500" />
                </div>
                <h3 class="text-2xl font-bold mb-2 text-foreground">Two-Factor Authentication is Active</h3>
                <p class="text-muted-foreground mb-6">Your account is protected with an extra layer of security.</p>
                <Button variant="destructive" @click="disable2fa">Disable 2FA</Button>
            </CardContent>
        </Card>

        <!-- 2FA Setup Flow -->
        <template v-else>
            <Card class="rounded-2xl overflow-hidden">
                <CardContent class="p-8 flex flex-col lg:flex-row gap-12">
                    <!-- Step 1: Scan QR Code -->
                    <div class="flex-1 space-y-6">
                        <div class="flex items-center gap-4">
                            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary text-primary-foreground font-bold text-sm">1</span>
                            <h3 class="text-lg font-bold text-foreground">Scan the QR Code</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">Open your authenticator app and scan the QR code below.</p>
                        <div class="bg-white p-4 inline-block border-2 border-border rounded-xl">
                            <div class="w-48 h-48 bg-muted flex items-center justify-center border border-border rounded-lg">
                                <img v-if="qrCodeUrl" :src="qrCodeUrl" alt="2FA QR Code" class="w-full h-full object-contain" />
                                <QrCode v-else :size="96" class="text-muted-foreground/50" />
                            </div>
                        </div>
                        <div class="space-y-2">
                            <p class="text-xs font-semibold text-muted-foreground uppercase">Can't scan?</p>
                            <div class="flex items-center gap-2">
                                <code class="bg-muted px-3 py-1.5 rounded text-xs font-mono text-primary select-all">{{ secretKey }}</code>
                                <button @click="copySecretKey" class="p-1.5 text-muted-foreground hover:text-primary"><Copy :size="16" /></button>
                            </div>
                        </div>
                    </div>

                    <!-- Step 2: Verify -->
                    <div class="flex-1 space-y-6">
                        <div class="flex items-center gap-4">
                            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary text-primary-foreground font-bold text-sm">2</span>
                            <h3 class="text-lg font-bold text-foreground">Verify Your Device</h3>
                        </div>
                        <p class="text-sm text-muted-foreground">Enter the 6-digit code from your authenticator app.</p>
                        <div class="grid grid-cols-6 gap-2" @paste="handlePaste">
                            <input v-for="(_, i) in 6" :key="i" :ref="el => { if (el) codeInputs[i] = el as HTMLInputElement }" v-model="verificationCode[i]" 
                                type="text" inputmode="numeric" maxlength="1" @input="handleCodeInput(i, $event)" @keydown="handleCodeKeydown(i, $event)" 
                                class="w-full h-12 text-center text-xl font-bold rounded-lg border-border bg-muted focus:ring-primary focus:border-primary" placeholder="·" />
                        </div>
                        <Button class="w-full" @click="enable2fa" :disabled="isVerifying || verificationCode.join('').length !== 6">
                            <Lock :size="18" class="mr-2" />{{ isVerifying ? 'Verifying...' : 'Enable 2FA' }}
                        </Button>
                        <div class="pt-4 border-t border-border flex items-center gap-2 text-amber-500">
                            <Info :size="16" />
                            <p class="text-[11px] font-medium">Complete both steps to secure your account.</p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <!-- Backup Codes -->
            <section class="space-y-4">
                <div class="flex items-center justify-between">
                    <h3 class="text-lg font-bold text-foreground">Backup Recovery Codes</h3>
                    <div class="flex gap-2">
                        <Button variant="outline" size="sm" @click="downloadBackupCodes"><Download :size="14" class="mr-2" />Download</Button>
                        <Button variant="outline" size="sm" @click="printBackupCodes"><Printer :size="14" class="mr-2" />Print</Button>
                    </div>
                </div>
                <Card class="rounded-2xl p-6">
                    <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800/50 p-4 rounded-xl mb-6">
                        <p class="text-xs text-blue-700 dark:text-blue-300"><strong>Note:</strong> Store these codes safely. Each code can only be used once.</p>
                    </div>
                    <div v-if="backupCodes.length" class="grid grid-cols-2 md:grid-cols-4 gap-3">
                        <div v-for="code in backupCodes" :key="code" class="font-mono bg-muted border border-border px-3 py-2 rounded text-center text-sm text-foreground">{{ code }}</div>
                    </div>
                    <p v-else class="text-xs text-muted-foreground">Backup codes will appear after setup.</p>
                </Card>
            </section>
        </template>
    </div>
</MainLayout>
</template>
