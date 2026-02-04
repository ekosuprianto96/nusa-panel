<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import { securityService } from '@/services'
import { Terminal, ChevronRight, Key, Plus, Upload, Eye, Trash2, Copy, Edit3, Info, Circle } from 'lucide-vue-next'

const sshKeys = ref<any[]>([])
const sessions = ref<any[]>([])
const isLoading = ref(true)
const sshEnabled = ref(true)
const showAddKeyModal = ref(false)
const showViewKeyModal = ref(false)
const showDeleteModal = ref(false)
const selectedKey = ref<any>(null)
const newKeyLabel = ref('')
const newKeyPublic = ref('')
const isSubmitting = ref(false)
const sshHost = ref('ssh.example-server.com')
const sshPort = ref('22')

import { useToastStore } from '@/stores/toast'

const toast = useToastStore()

const sshCommand = computed(() => `ssh root@${sshHost.value} -p ${sshPort.value}`)

const fetchData = async () => {
    isLoading.value = true
    try {
        const res = await securityService.getSshKeys()
        sshKeys.value = res.data.data || []
        sessions.value = [{ id: 1, user: 'root', ip: '192.168.1.145', since: new Date().toISOString(), idle: '0s', isYou: true }]
    } catch (e) {
        toast.error('Failed to load SSH data')
    } finally {
        isLoading.value = false
    }
}

const copyToClipboard = async (text: string) => {
    try { await navigator.clipboard.writeText(text); toast.success('Copied to clipboard') }
    catch (e) { toast.error('Failed to copy') }
}

const addSshKey = async () => {
    if (!newKeyLabel.value || !newKeyPublic.value) { toast.error('Please fill in all fields'); return }
    isSubmitting.value = true
    try {
        await securityService.addSshKey({ label: newKeyLabel.value, public_key: newKeyPublic.value })
        toast.success('SSH key added successfully')
        showAddKeyModal.value = false
        newKeyLabel.value = ''; newKeyPublic.value = ''
        await fetchData()
    } catch (e) { toast.error('Failed to add SSH key') }
    finally { isSubmitting.value = false }
}

const deleteKey = async () => {
    if (!selectedKey.value) return
    try { await securityService.deleteSshKey(selectedKey.value.id); toast.success('SSH key deleted'); showDeleteModal.value = false; await fetchData() }
    catch (e) { toast.error('Failed to delete SSH key') }
}

const viewKey = (key: any) => { selectedKey.value = key; showViewKeyModal.value = true }
const confirmDelete = (key: any) => { selectedKey.value = key; showDeleteModal.value = true }
const killSession = (sessionId: number) => { toast.success('Session terminated'); sessions.value = sessions.value.filter(s => s.id !== sessionId) }
const getKeyFingerprint = (key: any) => { const h = key.public_key?.slice(-20)?.replace(/[^a-zA-Z0-9]/g, '') || ''; return `SHA256: ${h.slice(0, 4)}...${h.slice(-4)}` }

onMounted(fetchData)
</script>

<template>
<MainLayout>


    <div class="space-y-8">
        <div class="flex flex-wrap justify-between items-center gap-4">
            <div class="flex flex-col gap-2">
                <nav class="flex items-center gap-2 text-xs font-medium text-slate-400 mb-1">
                    <router-link to="/dashboard/security" class="hover:text-primary transition-colors">Security Center</router-link>
                    <ChevronRight :size="12" /><span class="text-slate-600 dark:text-slate-300">SSH Access</span>
                </nav>
                <h2 class="text-3xl font-black text-[#0d131b] dark:text-white flex items-center gap-3"><Terminal :size="32" class="text-primary" />SSH Access Configuration</h2>
            </div>
            <div class="flex items-center gap-4 bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl">
                <div class="flex flex-col"><span class="text-[10px] font-bold text-slate-500 uppercase">SSH Status</span><span :class="['text-sm font-bold', sshEnabled ? 'text-success' : 'text-error']">{{ sshEnabled ? 'Active' : 'Disabled' }}</span></div>
                <button @click="sshEnabled = !sshEnabled" :class="['relative inline-flex h-6 w-11 items-center rounded-full', sshEnabled ? 'bg-primary' : 'bg-slate-300']"><span :class="['inline-block h-4 w-4 transform rounded-full bg-white', sshEnabled ? 'translate-x-6' : 'translate-x-1']" /></button>
            </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <h3 class="text-sm font-bold text-slate-500 uppercase mb-4">Connection Details</h3>
                <div class="grid grid-cols-2 gap-4">
                    <div class="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg"><p class="text-xs text-slate-500 mb-1">Host</p><div class="flex justify-between"><code class="text-sm font-mono font-bold text-primary">{{ sshHost }}</code><button @click="copyToClipboard(sshHost)" class="text-slate-400 hover:text-primary"><Copy :size="16" /></button></div></div>
                    <div class="bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg"><p class="text-xs text-slate-500 mb-1">Port</p><div class="flex justify-between"><code class="text-sm font-mono font-bold text-primary">{{ sshPort }}</code><Edit3 :size="16" class="text-slate-400" /></div></div>
                </div>
                <div class="mt-4 flex items-start gap-3 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800 rounded-lg"><Info :size="18" class="text-primary flex-shrink-0" /><p class="text-xs text-blue-800 dark:text-blue-300">Use a custom SSH port (1024-65535) for better security.</p></div>
            </div>
            <div class="bg-primary text-white rounded-xl p-6 shadow-xl shadow-primary/20">
                <h3 class="text-sm font-bold uppercase opacity-80">Quick Command</h3><code class="block bg-black/20 p-3 rounded-lg text-xs font-mono my-4">{{ sshCommand }}</code>
                <button @click="copyToClipboard(sshCommand)" class="w-full py-2.5 bg-white text-primary font-bold rounded-lg text-sm flex items-center justify-center gap-2"><Terminal :size="16" />Copy</button>
            </div>
        </div>

        <section>
            <div class="flex justify-between mb-6"><h3 class="text-xl font-bold">Manage SSH Keys</h3><div class="flex gap-2"><button class="flex items-center gap-2 px-4 py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-xs font-bold"><Upload :size="14" />Import</button><button @click="showAddKeyModal = true" class="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg text-xs font-bold"><Plus :size="14" />Add Key</button></div></div>
            <div v-if="isLoading" class="flex justify-center py-12"><div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full"></div></div>
            <div v-else-if="sshKeys.length === 0" class="text-center py-12 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl"><Key :size="48" class="mx-auto mb-4 text-slate-300" /><p class="text-slate-500 mb-4">No SSH keys found</p><button @click="showAddKeyModal = true" class="px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold">Add Key</button></div>
            <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div v-for="key in sshKeys" :key="key.id" class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-5 rounded-xl hover:border-primary/50">
                    <div class="flex justify-between items-start mb-4"><div class="flex items-center gap-3"><div class="w-10 h-10 bg-slate-100 dark:bg-slate-800 rounded-lg flex items-center justify-center"><Key :size="20" class="text-slate-500" /></div><div><h4 class="font-bold text-sm">{{ key.label || 'SSH Key' }}</h4><p class="text-[10px] text-slate-400 font-mono">{{ getKeyFingerprint(key) }}</p></div></div><span class="px-2 py-0.5 bg-success/10 text-success text-[10px] font-bold rounded uppercase">Active</span></div>
                    <div class="flex justify-between border-t border-slate-100 dark:border-slate-800 pt-4"><span class="text-xs text-slate-500">Added: {{ key.created_at?.split('T')[0] || 'Unknown' }}</span><div class="flex gap-2"><button @click="viewKey(key)" class="p-1.5 text-slate-400 hover:text-primary"><Eye :size="16" /></button><button @click="confirmDelete(key)" class="p-1.5 text-slate-400 hover:text-error"><Trash2 :size="16" /></button></div></div>
                </div>
            </div>
        </section>

        <section>
            <div class="flex justify-between mb-6"><h3 class="text-xl font-bold">Active Sessions</h3><button v-if="sessions.length" class="text-xs text-error font-bold">Terminate All</button></div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden">
                <table class="w-full text-left"><thead><tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800"><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">User</th><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">From</th><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Since</th><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase">Idle</th><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase text-right">Actions</th></tr></thead>
                <tbody><tr v-for="s in sessions" :key="s.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/30"><td class="px-6 py-4"><div class="flex items-center gap-2"><Circle :size="8" class="fill-success text-success" /><span class="text-sm">{{ s.user }}</span></div></td><td class="px-6 py-4 text-sm text-slate-500">{{ s.ip }}{{ s.isYou ? ' (You)' : '' }}</td><td class="px-6 py-4 text-sm">{{ s.since?.split('T')[0] }}</td><td class="px-6 py-4 text-sm text-slate-400">{{ s.idle }}</td><td class="px-6 py-4 text-right"><button @click="killSession(s.id)" class="text-xs font-bold text-error">Kill</button></td></tr><tr v-if="!sessions.length"><td colspan="5" class="px-6 py-12 text-center text-slate-500">No active sessions</td></tr></tbody></table>
            </div>
        </section>
    </div>

    <Teleport to="body"><div v-if="showAddKeyModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showAddKeyModal = false"><div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-lg p-6"><h3 class="text-xl font-bold mb-4">Add SSH Key</h3><div class="space-y-4"><div><label class="block text-sm font-medium mb-2">Label</label><input v-model="newKeyLabel" type="text" placeholder="e.g., MacBook" class="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg" /></div><div><label class="block text-sm font-medium mb-2">Public Key</label><textarea v-model="newKeyPublic" rows="4" placeholder="ssh-rsa AAAA..." class="w-full px-4 py-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg font-mono text-sm resize-none"></textarea></div></div><div class="flex justify-end gap-3 mt-6"><button @click="showAddKeyModal = false" class="px-4 py-2 text-sm text-slate-600 hover:bg-slate-100 rounded-lg">Cancel</button><button @click="addSshKey" :disabled="isSubmitting" class="px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold disabled:opacity-50">{{ isSubmitting ? 'Adding...' : 'Add Key' }}</button></div></div></div></Teleport>
    <Teleport to="body"><div v-if="showViewKeyModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showViewKeyModal = false"><div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-lg p-6"><h3 class="text-xl font-bold mb-4">{{ selectedKey?.label }}</h3><div class="bg-slate-50 dark:bg-slate-800 p-4 rounded-lg border border-slate-200"><code class="text-xs font-mono break-all">{{ selectedKey?.public_key }}</code></div><div class="flex justify-end gap-3 mt-6"><button @click="copyToClipboard(selectedKey?.public_key)" class="px-4 py-2 text-sm text-primary hover:bg-primary/10 rounded-lg flex items-center gap-2"><Copy :size="14" />Copy</button><button @click="showViewKeyModal = false" class="px-4 py-2 bg-slate-100 text-slate-600 rounded-lg text-sm font-bold">Close</button></div></div></div></Teleport>
    <Teleport to="body"><div v-if="showDeleteModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showDeleteModal = false"><div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-md p-6 text-center"><div class="w-16 h-16 bg-error/10 rounded-full flex items-center justify-center mx-auto mb-4"><Trash2 :size="32" class="text-error" /></div><h3 class="text-xl font-bold mb-2">Delete SSH Key?</h3><p class="text-slate-500">Delete "{{ selectedKey?.label }}"?</p><div class="flex justify-center gap-3 mt-6"><button @click="showDeleteModal = false" class="px-6 py-2 text-sm text-slate-600 hover:bg-slate-100 rounded-lg">Cancel</button><button @click="deleteKey" class="px-6 py-2 bg-error text-white rounded-lg text-sm font-bold">Delete</button></div></div></div></Teleport>
</MainLayout>
</template>
