<script setup lang="ts">
/**
 * SshAccessPage - SSH Access Configuration
 */
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button, Input } from '@/components/ui'
import { Badge } from '@/components/ui/badge'
import { securityService } from '@/services'
import { Terminal, Key, Plus, Upload, Eye, Trash2, Copy, Edit3, Info, Circle, Shield } from 'lucide-vue-next'

const router = useRouter()
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
        <!-- Header -->
        <div class="flex flex-wrap justify-between items-center gap-4">
            <div>
                <AppBreadcrumb
                    class="mb-4"
                    :items="[
                        { label: 'Security Center', icon: Shield, onClick: () => router.push('/dashboard/security') },
                        { label: 'SSH Access', current: true }
                    ]"
                />
                <h2 class="text-3xl font-black text-foreground flex items-center gap-3"><Terminal :size="32" class="text-primary" />SSH Access Configuration</h2>
            </div>
            <Card class="flex items-center gap-4 p-4">
                <div class="flex flex-col">
                    <span class="text-[10px] font-bold text-muted-foreground uppercase">SSH Status</span>
                    <span :class="['text-sm font-bold', sshEnabled ? 'text-emerald-500' : 'text-destructive']">{{ sshEnabled ? 'Active' : 'Disabled' }}</span>
                </div>
                <button @click="sshEnabled = !sshEnabled" :class="['relative inline-flex h-6 w-11 items-center rounded-full', sshEnabled ? 'bg-primary' : 'bg-muted']">
                    <span :class="['inline-block h-4 w-4 transform rounded-full bg-white', sshEnabled ? 'translate-x-6' : 'translate-x-1']" />
                </button>
            </Card>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <Card class="lg:col-span-2 rounded-xl">
                <CardHeader>
                    <CardTitle class="text-sm font-bold text-muted-foreground uppercase">Connection Details</CardTitle>
                </CardHeader>
                <CardContent>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="bg-muted p-4 rounded-lg">
                            <p class="text-xs text-muted-foreground mb-1">Host</p>
                            <div class="flex justify-between">
                                <code class="text-sm font-mono font-bold text-primary">{{ sshHost }}</code>
                                <button @click="copyToClipboard(sshHost)" class="text-muted-foreground hover:text-primary"><Copy :size="16" /></button>
                            </div>
                        </div>
                        <div class="bg-muted p-4 rounded-lg">
                            <p class="text-xs text-muted-foreground mb-1">Port</p>
                            <div class="flex justify-between">
                                <code class="text-sm font-mono font-bold text-primary">{{ sshPort }}</code>
                                <Edit3 :size="16" class="text-muted-foreground" />
                            </div>
                        </div>
                    </div>
                    <div class="mt-4 flex items-start gap-3 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800 rounded-lg">
                        <Info :size="18" class="text-primary flex-shrink-0" />
                        <p class="text-xs text-blue-800 dark:text-blue-300">Use a custom SSH port (1024-65535) for better security.</p>
                    </div>
                </CardContent>
            </Card>
            <Card class="bg-primary text-primary-foreground rounded-xl">
                <CardContent class="p-6">
                    <h3 class="text-sm font-bold uppercase opacity-80">Quick Command</h3>
                    <code class="block bg-black/20 p-3 rounded-lg text-xs font-mono my-4">{{ sshCommand }}</code>
                    <Button variant="secondary" class="w-full" @click="copyToClipboard(sshCommand)">
                        <Terminal :size="16" class="mr-2" />Copy
                    </Button>
                </CardContent>
            </Card>
        </div>

        <!-- SSH Keys Section -->
        <section>
            <div class="flex justify-between mb-6">
                <h3 class="text-xl font-bold text-foreground">Manage SSH Keys</h3>
                <div class="flex gap-2">
                    <Button variant="outline"><Upload :size="14" class="mr-2" />Import</Button>
                    <Button @click="showAddKeyModal = true"><Plus :size="14" class="mr-2" />Add Key</Button>
                </div>
            </div>
            
            <div v-if="isLoading" class="flex justify-center py-12">
                <div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full"></div>
            </div>
            
            <Card v-else-if="sshKeys.length === 0" class="text-center py-12 rounded-xl">
                <CardContent>
                    <Key :size="48" class="mx-auto mb-4 text-muted-foreground/50" />
                    <p class="text-muted-foreground mb-4">No SSH keys found</p>
                    <Button @click="showAddKeyModal = true">Add Key</Button>
                </CardContent>
            </Card>
            
            <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Card v-for="key in sshKeys" :key="key.id" class="p-5 rounded-xl hover:border-primary/50 transition-colors">
                    <div class="flex justify-between items-start mb-4">
                        <div class="flex items-center gap-3">
                            <div class="w-10 h-10 bg-muted rounded-lg flex items-center justify-center">
                                <Key :size="20" class="text-muted-foreground" />
                            </div>
                            <div>
                                <h4 class="font-bold text-sm text-foreground">{{ key.label || 'SSH Key' }}</h4>
                                <p class="text-[10px] text-muted-foreground font-mono">{{ getKeyFingerprint(key) }}</p>
                            </div>
                        </div>
                        <Badge variant="success">Active</Badge>
                    </div>
                    <div class="flex justify-between border-t border-border pt-4">
                        <span class="text-xs text-muted-foreground">Added: {{ key.created_at?.split('T')[0] || 'Unknown' }}</span>
                        <div class="flex gap-2">
                            <button @click="viewKey(key)" class="p-1.5 text-muted-foreground hover:text-primary"><Eye :size="16" /></button>
                            <button @click="confirmDelete(key)" class="p-1.5 text-muted-foreground hover:text-destructive"><Trash2 :size="16" /></button>
                        </div>
                    </div>
                </Card>
            </div>
        </section>

        <!-- Active Sessions -->
        <section>
            <div class="flex justify-between mb-6">
                <h3 class="text-xl font-bold text-foreground">Active Sessions</h3>
                <button v-if="sessions.length" class="text-xs text-destructive font-bold">Terminate All</button>
            </div>
            <Card class="rounded-xl overflow-hidden">
                <table class="w-full text-left">
                    <thead>
                        <tr class="bg-muted/50 border-b border-border">
                            <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase">User</th>
                            <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase">From</th>
                            <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase">Since</th>
                            <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase">Idle</th>
                            <th class="px-6 py-4 text-xs font-bold text-muted-foreground uppercase text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="s in sessions" :key="s.id" class="hover:bg-muted/30">
                            <td class="px-6 py-4">
                                <div class="flex items-center gap-2">
                                    <Circle :size="8" class="fill-emerald-500 text-emerald-500" />
                                    <span class="text-sm text-foreground">{{ s.user }}</span>
                                </div>
                            </td>
                            <td class="px-6 py-4 text-sm text-muted-foreground">{{ s.ip }}{{ s.isYou ? ' (You)' : '' }}</td>
                            <td class="px-6 py-4 text-sm text-foreground">{{ s.since?.split('T')[0] }}</td>
                            <td class="px-6 py-4 text-sm text-muted-foreground">{{ s.idle }}</td>
                            <td class="px-6 py-4 text-right">
                                <Button variant="ghost" size="sm" class="text-destructive" @click="killSession(s.id)">Kill</Button>
                            </td>
                        </tr>
                        <tr v-if="!sessions.length">
                            <td colspan="5" class="px-6 py-12 text-center text-muted-foreground">No active sessions</td>
                        </tr>
                    </tbody>
                </table>
            </Card>
        </section>
    </div>

    <!-- Add Key Modal -->
    <Teleport to="body">
        <div v-if="showAddKeyModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showAddKeyModal = false">
            <Card class="w-full max-w-lg p-6 rounded-2xl">
                <h3 class="text-xl font-bold mb-4 text-foreground">Add SSH Key</h3>
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium mb-2 text-foreground">Label</label>
                        <Input v-model="newKeyLabel" placeholder="e.g., MacBook" />
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-2 text-foreground">Public Key</label>
                        <textarea v-model="newKeyPublic" rows="4" placeholder="ssh-rsa AAAA..." class="w-full px-4 py-3 bg-muted border border-border rounded-lg font-mono text-sm resize-none"></textarea>
                    </div>
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <Button variant="ghost" @click="showAddKeyModal = false">Cancel</Button>
                    <Button @click="addSshKey" :disabled="isSubmitting">{{ isSubmitting ? 'Adding...' : 'Add Key' }}</Button>
                </div>
            </Card>
        </div>
    </Teleport>

    <!-- View Key Modal -->
    <Teleport to="body">
        <div v-if="showViewKeyModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showViewKeyModal = false">
            <Card class="w-full max-w-lg p-6 rounded-2xl">
                <h3 class="text-xl font-bold mb-4 text-foreground">{{ selectedKey?.label }}</h3>
                <div class="bg-muted p-4 rounded-lg border border-border">
                    <code class="text-xs font-mono break-all text-foreground">{{ selectedKey?.public_key }}</code>
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <Button variant="ghost" @click="copyToClipboard(selectedKey?.public_key)"><Copy :size="14" class="mr-2" />Copy</Button>
                    <Button variant="secondary" @click="showViewKeyModal = false">Close</Button>
                </div>
            </Card>
        </div>
    </Teleport>

    <!-- Delete Modal -->
    <Teleport to="body">
        <div v-if="showDeleteModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showDeleteModal = false">
            <Card class="w-full max-w-md p-6 text-center rounded-2xl">
                <div class="w-16 h-16 bg-destructive/10 rounded-full flex items-center justify-center mx-auto mb-4">
                    <Trash2 :size="32" class="text-destructive" />
                </div>
                <h3 class="text-xl font-bold mb-2 text-foreground">Delete SSH Key?</h3>
                <p class="text-muted-foreground">Delete "{{ selectedKey?.label }}"?</p>
                <div class="flex justify-center gap-3 mt-6">
                    <Button variant="ghost" @click="showDeleteModal = false">Cancel</Button>
                    <Button variant="destructive" @click="deleteKey">Delete</Button>
                </div>
            </Card>
        </div>
    </Teleport>
</MainLayout>
</template>
