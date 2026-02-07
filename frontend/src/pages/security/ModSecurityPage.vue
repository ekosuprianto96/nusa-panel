<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import MainLayout from '@/layouts/MainLayout.vue'
import AppBreadcrumb from '@/components/AppBreadcrumb.vue'
import { domainService, securityService } from '@/services'
import { Search, RefreshCw, FileText, Globe, Code, Database, FolderOpen, Bug, TrendingUp, Settings, Shield } from 'lucide-vue-next'

const domains = ref<any[]>([])
const router = useRouter()
const isLoading = ref(true)
const searchQuery = ref('')
const rulesTriggered = ref(0)
const trendPercentage = ref(0)
const globalSettings = ref({ mainEngine: true, paranoiaLevel: 2, anomalyThreshold: 5 })
const ruleSets = ref<any[]>([])
const chartData = ref<number[]>([])
const chartDays = ref<string[]>([])
const showSettingsModal = ref(false)
const settingsForm = ref({ mainEngine: true, paranoiaLevel: 2, anomalyThreshold: 5 })
const showCustomRulesModal = ref(false)
const showAuditLogsModal = ref(false)
const customRules = ref<any[]>([])
const auditLogs = ref<any[]>([])
const customRuleForm = ref({
    id: '',
    name: '',
    description: '',
    rule_content: '',
    enabled: true,
})

import { useToastStore } from '@/stores/toast'

const toast = useToastStore()

const filteredDomains = computed(() => {
    if (!searchQuery.value) return domains.value
    return domains.value.filter(d => d.domain_name?.toLowerCase().includes(searchQuery.value.toLowerCase()))
})

const fetchData = async () => {
    isLoading.value = true
    try {
        const [domainRes, overviewRes, customRulesRes] = await Promise.all([
            domainService.listDomains(),
            securityService.getModSecurityOverview(),
            securityService.listCustomRules(),
        ])
        domains.value = domainRes.data.data || []
        const overview = overviewRes.data.data || {}
        rulesTriggered.value = overview.rules_triggered || 0
        trendPercentage.value = overview.trend_percent || 0
        chartData.value = overview.chart_data?.length ? overview.chart_data : [40, 30, 55, 45, 80, 70, 90]
        chartDays.value = overview.chart_labels?.length ? overview.chart_labels : ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
        ruleSets.value = (overview.rules || []).map((r: any) => ({
            ...r,
            icon: r.id === 'sqli' ? Database : r.id === 'xss' ? Code : r.id === 'lfi' ? FolderOpen : Bug,
        }))
        if (overview.settings) {
            globalSettings.value = {
                mainEngine: !!overview.settings.main_engine,
                paranoiaLevel: overview.settings.paranoia_level ?? 2,
                anomalyThreshold: overview.settings.anomaly_threshold ?? 5,
            }
        }
        customRules.value = customRulesRes.data.data || []
    } catch (e) { toast.error('Failed to load domains') }
    finally { isLoading.value = false }
}

const refreshRules = async () => {
    try {
        toast.info('Refreshing ModSecurity rules...')
        await fetchData()
        toast.success('Rules refreshed successfully')
    } catch {
        toast.error('Failed to refresh rules')
    }
}

const toggleDomainModSecurity = async (domain: any) => {
    const current = domain.modsecurity_enabled ?? true
    const nextValue = !current
    try {
        await securityService.updateModSecurityDomain(domain.id, { enabled: nextValue })
        domain.modsecurity_enabled = nextValue
        toast.success(`ModSecurity ${nextValue ? 'enabled' : 'disabled'} for ${domain.domain_name}`)
    } catch {
        toast.error('Failed to update domain ModSecurity')
    }
}

const toggleRule = async (rule: any) => {
    const nextValue = !rule.enabled
    try {
        await securityService.updateModSecurityRule(rule.id, { enabled: nextValue })
        rule.enabled = nextValue
        toast.success(`${rule.name} ${nextValue ? 'enabled' : 'disabled'}`)
    } catch {
        toast.error('Failed to update rule')
    }
}

const openSettings = () => {
    settingsForm.value = { ...globalSettings.value }
    showSettingsModal.value = true
}

const saveSettings = async () => {
    try {
        const res = await securityService.updateModSecuritySettings({
            main_engine: settingsForm.value.mainEngine,
            paranoia_level: settingsForm.value.paranoiaLevel,
            anomaly_threshold: settingsForm.value.anomalyThreshold,
        })
        if (res.data.data) {
            globalSettings.value = {
                mainEngine: res.data.data.main_engine ?? settingsForm.value.mainEngine,
                paranoiaLevel: res.data.data.paranoia_level ?? settingsForm.value.paranoiaLevel,
                anomalyThreshold: res.data.data.anomaly_threshold ?? settingsForm.value.anomalyThreshold,
            }
        }
        showSettingsModal.value = false
        toast.success('Settings updated')
    } catch {
        toast.error('Failed to update settings')
    }
}

const openCustomRules = () => {
    showCustomRulesModal.value = true
}

const openAuditLogs = async () => {
    showAuditLogsModal.value = true
    try {
        const res = await securityService.getModSecurityAuditLogs({ limit: 100 })
        auditLogs.value = res.data.data || []
    } catch {
        toast.error('Failed to load audit logs')
    }
}

const resetCustomRuleForm = () => {
    customRuleForm.value = {
        id: '',
        name: '',
        description: '',
        rule_content: '',
        enabled: true,
    }
}

const editCustomRule = (rule: any) => {
    customRuleForm.value = {
        id: rule.id,
        name: rule.name,
        description: rule.description || '',
        rule_content: rule.rule_content,
        enabled: rule.enabled,
    }
}

const saveCustomRule = async () => {
    if (!customRuleForm.value.name || !customRuleForm.value.rule_content) {
        toast.error('Name and rule content are required')
        return
    }
    try {
        if (customRuleForm.value.id) {
            const res = await securityService.updateCustomRule(customRuleForm.value.id, {
                name: customRuleForm.value.name,
                description: customRuleForm.value.description,
                rule_content: customRuleForm.value.rule_content,
                enabled: customRuleForm.value.enabled,
            })
            const updated = res.data.data
            const idx = customRules.value.findIndex((r) => r.id === updated.id)
            if (idx >= 0) customRules.value[idx] = updated
            toast.success('Custom rule updated')
        } else {
            const res = await securityService.createCustomRule({
                name: customRuleForm.value.name,
                description: customRuleForm.value.description,
                rule_content: customRuleForm.value.rule_content,
                enabled: customRuleForm.value.enabled,
            })
            customRules.value.unshift(res.data.data)
            toast.success('Custom rule created')
        }
        resetCustomRuleForm()
    } catch {
        toast.error('Failed to save custom rule')
    }
}

const deleteCustomRule = async (ruleId: string) => {
    try {
        await securityService.deleteCustomRule(ruleId)
        customRules.value = customRules.value.filter((r) => r.id !== ruleId)
        toast.success('Custom rule deleted')
    } catch {
        toast.error('Failed to delete custom rule')
    }
}

onMounted(fetchData)
</script>

<template>
<MainLayout>


    <div class="space-y-8">
        <div class="flex flex-wrap justify-between items-end gap-4">
            <div>
                <AppBreadcrumb
                    class="mb-4"
                    :items="[
                        { label: 'Security Center', icon: Shield, onClick: () => router.push('/dashboard/security') },
                        { label: 'ModSecurity', current: true }
                    ]"
                />
                <div class="flex items-center gap-3"><h2 class="text-3xl font-black text-[#0d131b] dark:text-white">ModSecurity</h2><span class="bg-success text-white text-[10px] font-black uppercase px-2 py-0.5 rounded-full">Active</span></div>
                <p class="text-slate-500 text-sm mt-2 max-w-xl">Web Application Firewall (WAF) to protect against SQL injection, XSS, and more.</p>
            </div>
            <div class="flex gap-3">
                <button @click="openAuditLogs" class="flex items-center gap-2 px-5 py-2.5 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-lg text-sm font-bold border border-slate-200 dark:border-slate-700"><FileText :size="18" />Audit Logs</button>
                <button @click="refreshRules" class="flex items-center gap-2 px-5 py-2.5 bg-primary text-white hover:bg-primary/90 rounded-lg text-sm font-bold shadow-lg shadow-primary/20"><RefreshCw :size="18" />Refresh Rules</button>
            </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6">
                <div class="flex justify-between items-center mb-6"><div><h3 class="text-sm font-bold text-slate-500 uppercase">Rules Triggered</h3><p class="text-xs text-slate-400">Last 7 days</p></div><div class="text-right"><span class="text-2xl font-black">{{ rulesTriggered.toLocaleString() }}</span><span class="text-xs text-error font-bold flex items-center justify-end gap-0.5"><TrendingUp :size="12" /> {{ trendPercentage }}%</span></div></div>
                <div class="h-32 w-full flex items-end gap-4">
                    <div v-for="(value, index) in chartData" :key="index" class="flex-1 flex flex-col items-center gap-2 group cursor-pointer"><div class="w-full bg-primary/20 rounded-t relative overflow-hidden" :style="{ height: `${value}%` }"><div class="absolute inset-0 bg-primary opacity-80 group-hover:opacity-100 transition-opacity"></div></div><span class="text-[10px] text-slate-400">{{ chartDays[index] }}</span></div>
                </div>
            </div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6 flex flex-col gap-6">
                <h3 class="text-sm font-bold text-slate-500 uppercase">Global Status</h3>
                <div class="space-y-4"><div class="flex justify-between"><span class="text-sm">Main Engine</span><span class="text-sm font-bold text-success">{{ globalSettings.mainEngine ? 'On' : 'Off' }}</span></div><div class="flex justify-between"><span class="text-sm">Paranoia Level</span><span class="text-sm font-bold">Level {{ globalSettings.paranoiaLevel }}</span></div><div class="flex justify-between"><span class="text-sm">Anomaly Threshold</span><span class="text-sm font-bold">{{ globalSettings.anomalyThreshold }} pts</span></div></div>
                <button @click="openSettings" class="w-full mt-auto py-2.5 bg-slate-50 dark:bg-slate-800 text-xs font-bold rounded-lg border border-slate-200 dark:border-slate-700 hover:bg-slate-100 flex items-center justify-center gap-2"><Settings :size="14" />Edit Settings</button>
            </div>
        </div>

        <section>
            <div class="flex justify-between mb-6"><h3 class="text-[22px] font-bold">Domain Protection</h3><div class="relative"><Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" /><input v-model="searchQuery" type="text" placeholder="Search..." class="pl-10 pr-4 py-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg text-xs w-64" /></div></div>
            <div class="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden">
                <div v-if="isLoading" class="p-12 text-center"><div class="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div><p class="text-slate-500">Loading...</p></div>
                <div v-else-if="filteredDomains.length === 0" class="p-12 text-center"><Globe :size="48" class="mx-auto mb-4 text-slate-300" /><p class="text-slate-500">No domains found</p></div>
                <table v-else class="w-full text-left"><thead><tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800"><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase w-2/3">Domain</th><th class="px-6 py-4 text-xs font-bold text-slate-500 uppercase text-right">ModSecurity</th></tr></thead>
                <tbody class="divide-y divide-slate-100 dark:divide-slate-800"><tr v-for="d in filteredDomains" :key="d.id" class="hover:bg-slate-50 dark:hover:bg-slate-800/30"><td class="px-6 py-4"><div class="flex items-center gap-2"><Globe :size="18" class="text-slate-400" /><span class="text-sm font-medium">{{ d.domain_name }}</span></div></td><td class="px-6 py-4 text-right"><button @click="toggleDomainModSecurity(d)" :class="['relative inline-flex h-6 w-11 items-center rounded-full', d.modsecurity_enabled ? 'bg-primary' : 'bg-slate-200 dark:bg-slate-700']"><span :class="['inline-block h-5 w-5 transform rounded-full bg-white border border-slate-300 shadow', d.modsecurity_enabled ? 'translate-x-5' : 'translate-x-0.5']" /></button></td></tr></tbody></table>
            </div>
        </section>

        <section>
            <div class="flex justify-between mb-6"><div><h3 class="text-[22px] font-bold">Rule Management</h3><p class="text-sm text-slate-500">Configure OWASP ModSecurity Core Rule Set.</p></div><button @click="openCustomRules" class="px-4 py-2 border border-slate-200 dark:border-slate-800 rounded-lg text-sm font-bold">Custom Rules</button></div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div v-for="rule in ruleSets" :key="rule.id" :class="['bg-white dark:bg-slate-900 border p-6 rounded-xl flex items-start justify-between', rule.enabled ? 'border-slate-200 dark:border-slate-800' : 'border-dashed border-slate-200 dark:border-slate-800']">
                    <div class="flex gap-4"><div class="w-10 h-10 rounded-lg bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-slate-500"><component :is="rule.icon" :size="20" /></div><div><h4 class="text-sm font-bold mb-1">{{ rule.name }}</h4><p class="text-xs text-slate-500 line-clamp-2">{{ rule.description }}</p></div></div>
                    <button @click="toggleRule(rule)" :class="['relative inline-flex h-5 w-9 items-center rounded-full ml-4 flex-shrink-0', rule.enabled ? 'bg-primary' : 'bg-slate-200 dark:bg-slate-700']"><span :class="['inline-block h-4 w-4 transform rounded-full bg-white border border-slate-300 shadow', rule.enabled ? 'translate-x-4' : 'translate-x-0.5']" /></button>
                </div>
            </div>
        </section>
    </div>

    <Teleport to="body">
        <div v-if="showSettingsModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showSettingsModal = false">
            <div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-lg p-6">
                <h3 class="text-xl font-bold mb-4">ModSecurity Settings</h3>
                <div class="space-y-4">
                    <label class="flex items-center justify-between text-sm">
                        <span>Main Engine</span>
                        <button @click="settingsForm.mainEngine = !settingsForm.mainEngine" :class="['relative inline-flex h-6 w-11 items-center rounded-full', settingsForm.mainEngine ? 'bg-primary' : 'bg-slate-200 dark:bg-slate-700']">
                            <span :class="['inline-block h-5 w-5 transform rounded-full bg-white border border-slate-300 shadow', settingsForm.mainEngine ? 'translate-x-5' : 'translate-x-0.5']" />
                        </button>
                    </label>
                    <div>
                        <label class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Paranoia Level</label>
                        <input v-model.number="settingsForm.paranoiaLevel" type="number" min="1" max="4" class="w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-sm bg-slate-50 dark:bg-slate-800" />
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Anomaly Threshold</label>
                        <input v-model.number="settingsForm.anomalyThreshold" type="number" min="1" max="20" class="w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-sm bg-slate-50 dark:bg-slate-800" />
                    </div>
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <button @click="showSettingsModal = false" class="px-4 py-2 text-sm text-slate-600 hover:bg-slate-100 rounded-lg">Cancel</button>
                    <button @click="saveSettings" class="px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold">Save</button>
                </div>
            </div>
        </div>
    </Teleport>

    <Teleport to="body">
        <div v-if="showCustomRulesModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showCustomRulesModal = false">
            <div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-3xl p-6">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-xl font-bold">Custom Rules</h3>
                    <button @click="showCustomRulesModal = false" class="text-slate-500 hover:text-slate-700">Close</button>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="text-sm font-bold mb-3">Create / Edit Rule</h4>
                        <div class="space-y-3">
                            <input v-model="customRuleForm.name" type="text" placeholder="Rule name" class="w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-sm bg-slate-50 dark:bg-slate-800" />
                            <input v-model="customRuleForm.description" type="text" placeholder="Description (optional)" class="w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-sm bg-slate-50 dark:bg-slate-800" />
                            <textarea v-model="customRuleForm.rule_content" rows="6" placeholder="SecRule REQUEST_URI ... " class="w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-xs bg-slate-50 dark:bg-slate-800 font-mono"></textarea>
                            <label class="flex items-center gap-2 text-sm">
                                <input v-model="customRuleForm.enabled" type="checkbox" class="rounded border-slate-300" />
                                Enabled
                            </label>
                        </div>
                        <div class="flex gap-2 mt-4">
                            <button @click="saveCustomRule" class="px-4 py-2 bg-primary text-white rounded-lg text-sm font-bold">Save</button>
                            <button @click="resetCustomRuleForm" class="px-4 py-2 border border-slate-200 rounded-lg text-sm">Reset</button>
                        </div>
                    </div>
                    <div>
                        <h4 class="text-sm font-bold mb-3">Existing Rules</h4>
                        <div class="space-y-3 max-h-[380px] overflow-auto pr-2">
                            <div v-for="rule in customRules" :key="rule.id" class="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-sm font-semibold">{{ rule.name }}</p>
                                        <p class="text-xs text-slate-500">{{ rule.description || 'No description' }}</p>
                                    </div>
                                    <span :class="['text-[10px] font-bold px-2 py-0.5 rounded-full', rule.enabled ? 'bg-success/10 text-success' : 'bg-slate-100 text-slate-500']">{{ rule.enabled ? 'Enabled' : 'Disabled' }}</span>
                                </div>
                                <div class="mt-2 text-[11px] font-mono bg-slate-50 dark:bg-slate-800 p-2 rounded border border-slate-200 dark:border-slate-700 max-h-24 overflow-auto">
                                    {{ rule.rule_content }}
                                </div>
                                <div class="flex gap-2 mt-3">
                                    <button @click="editCustomRule(rule)" class="px-3 py-1.5 text-xs border border-slate-200 rounded">Edit</button>
                                    <button @click="deleteCustomRule(rule.id)" class="px-3 py-1.5 text-xs border border-red-200 text-red-600 rounded">Delete</button>
                                </div>
                            </div>
                            <p v-if="customRules.length === 0" class="text-xs text-slate-500">No custom rules yet.</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </Teleport>

    <Teleport to="body">
        <div v-if="showAuditLogsModal" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50" @click.self="showAuditLogsModal = false">
            <div class="bg-white dark:bg-slate-900 rounded-2xl shadow-2xl w-full max-w-4xl p-6">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-xl font-bold">ModSecurity Audit Logs</h3>
                    <button @click="showAuditLogsModal = false" class="text-slate-500 hover:text-slate-700">Close</button>
                </div>
                <div class="overflow-auto max-h-[520px] border border-slate-200 dark:border-slate-800 rounded-lg">
                    <table class="w-full text-left">
                        <thead>
                            <tr class="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
                                <th class="px-4 py-3 text-xs font-bold text-slate-500 uppercase">Time</th>
                                <th class="px-4 py-3 text-xs font-bold text-slate-500 uppercase">Severity</th>
                                <th class="px-4 py-3 text-xs font-bold text-slate-500 uppercase">Message</th>
                                <th class="px-4 py-3 text-xs font-bold text-slate-500 uppercase">IP</th>
                                <th class="px-4 py-3 text-xs font-bold text-slate-500 uppercase">URI</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-slate-800">
                            <tr v-for="log in auditLogs" :key="log.id">
                                <td class="px-4 py-3 text-xs">{{ log.created_at?.split('T')[0] }} {{ log.created_at?.split('T')[1]?.split('.')[0] }}</td>
                                <td class="px-4 py-3 text-xs font-bold">{{ log.severity }}</td>
                                <td class="px-4 py-3 text-xs">{{ log.message }}</td>
                                <td class="px-4 py-3 text-xs font-mono">{{ log.ip_address || '-' }}</td>
                                <td class="px-4 py-3 text-xs font-mono">{{ log.uri || '-' }}</td>
                            </tr>
                            <tr v-if="auditLogs.length === 0">
                                <td colspan="5" class="px-4 py-8 text-center text-xs text-slate-500">No audit logs found.</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </Teleport>
</MainLayout>
</template>
