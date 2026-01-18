<script setup lang="ts">
/**
 * SecurityPage - Halaman keamanan (cPanel-like)
 *
 * Features:
 * - SSH Keys management
 * - IP Access Manager (block/whitelist)
 * - SSL/TLS Status per domain
 * - 2FA Setup
 * - Access logs
 */
import { ref, onMounted, computed } from 'vue';
import MainLayout from '@/layouts/MainLayout.vue';
import {
    Shield,
    Key,
    Lock,
    AlertCircle,
    Trash2,
    X,
    Check,
    Cpu,
    HardDrive,
    Activity,
    List,
    ShieldAlert,
    ShieldCheck,
    Plus,
    UserCheck,
    Smartphone,
    QrCode,
} from 'lucide-vue-next';
import { securityService, domainService } from '@/services';

// ==========================================
// STATE
// ==========================================
const sshKeys = ref<any[]>([]);
const blockedIps = ref<any[]>([]);
const accessLogs = ref<any[]>([]);
const stats = ref<any>(null);
const domains = ref<any[]>([]);
const isLoading = ref(true);
const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const activeTab = ref<'ssh' | 'ips' | 'ssl' | '2fa' | 'logs'>('ssh');

// Modal states
const showAddKeyModal = ref(false);
const showDeleteKeyModal = ref(false);
const showBlockIpModal = ref(false);
const showSetup2faModal = ref(false);

// Form data
const newKeyLabel = ref('');
const newKeyPublic = ref('');
const newBlockIp = ref('');
const newBlockReason = ref('');
const blockType = ref<'block' | 'whitelist'>('block');

// 2FA
const twoFaEnabled = ref(false);
const twoFaSecret = ref('');
const twoFaCode = ref('');

// Selected items
const selectedKey = ref<any>(null);

// ==========================================
// COMPUTED
// ==========================================
const sslStatuses = computed(() => {
    return domains.value.map((d) => ({
        ...d,
        ssl_status: d.ssl_enabled ? 'Active' : 'None',
        ssl_expiry: d.ssl_enabled ? 'In 90 days' : 'N/A',
        ssl_provider: d.ssl_enabled ? "Let's Encrypt" : 'N/A',
    }));
});

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Fetch data dari API
 */
const fetchData = async (): Promise<void> => {
    try {
        isLoading.value = true;
        error.value = null;
        const [sshRes, statsRes, logsRes, domRes] = await Promise.all([
            securityService.listSshKeys(),
            securityService.getResourceStats(),
            securityService.getAccessLogs(),
            domainService.listDomains(1, 50),
        ]);
        sshKeys.value = sshRes.data.data || [];
        stats.value = statsRes.data.data || null;
        accessLogs.value = logsRes.data.data || [];
        domains.value = domRes.data.data || [];

        try {
            const ipsRes = await securityService.listBlockedIps();
            blockedIps.value = ipsRes.data.data || [];
        } catch {
            // Not permitted for non-admin
        }
    } catch (err: any) {
        error.value =
            err.response?.data?.message || 'Gagal memuat data keamanan';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Show success message
 */
const showSuccess = (msg: string): void => {
    successMsg.value = msg;
    setTimeout(() => (successMsg.value = null), 3000);
};

/**
 * Add SSH Key
 */
const addSshKey = async (): Promise<void> => {
    if (!newKeyLabel.value.trim() || !newKeyPublic.value.trim()) return;
    try {
        isLoading.value = true;
        await securityService.addSshKey({
            label: newKeyLabel.value.trim(),
            public_key: newKeyPublic.value.trim(),
        });
        showAddKeyModal.value = false;
        newKeyLabel.value = '';
        newKeyPublic.value = '';
        showSuccess('SSH key berhasil ditambahkan');
        await fetchData();
    } catch (err: any) {
        error.value =
            err.response?.data?.message || 'Gagal menambahkan SSH key';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Delete SSH Key
 */
const deleteSshKey = async (): Promise<void> => {
    if (!selectedKey.value) return;
    try {
        isLoading.value = true;
        await securityService.deleteSshKey(selectedKey.value.id);
        showDeleteKeyModal.value = false;
        selectedKey.value = null;
        showSuccess('SSH key berhasil dihapus');
        await fetchData();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal menghapus SSH key';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Block/Whitelist IP
 */
const blockIp = async (): Promise<void> => {
    if (!newBlockIp.value.trim()) return;
    try {
        isLoading.value = true;
        await securityService.blockIp({
            ip_address: newBlockIp.value.trim(),
            reason: newBlockReason.value.trim() || `Manual ${blockType.value}`,
        });
        showBlockIpModal.value = false;
        newBlockIp.value = '';
        newBlockReason.value = '';
        showSuccess(`IP berhasil di-${blockType.value}`);
        await fetchData();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal memproses IP';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Unblock IP
 */
const unblockIp = async (ip: any): Promise<void> => {
    try {
        isLoading.value = true;
        await securityService.unblockIp(ip.id);
        showSuccess('IP berhasil di-unblock');
        await fetchData();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal unblock IP';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Setup 2FA (mock)
 */
const setup2fa = async (): Promise<void> => {
    if (!twoFaCode.value || twoFaCode.value.length !== 6) {
        error.value = 'Masukkan kode 6 digit';
        return;
    }
    try {
        isLoading.value = true;
        // Mock API call
        await new Promise((r) => setTimeout(r, 1000));
        twoFaEnabled.value = true;
        showSetup2faModal.value = false;
        twoFaCode.value = '';
        showSuccess('2FA berhasil diaktifkan');
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal mengaktifkan 2FA';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Generate 2FA secret (mock)
 */
const generate2faSecret = (): void => {
    // Mock secret generation
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let secret = '';
    for (let i = 0; i < 16; i++) {
        secret += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    twoFaSecret.value = secret;
    showSetup2faModal.value = true;
};

// ==========================================
// MODAL OPENERS
// ==========================================

const openDeleteKey = (key: any): void => {
    selectedKey.value = key;
    showDeleteKeyModal.value = true;
};

const openBlockIp = (type: 'block' | 'whitelist'): void => {
    blockType.value = type;
    newBlockIp.value = '';
    newBlockReason.value = '';
    showBlockIpModal.value = true;
};

onMounted(() => {
    fetchData();
});
</script>

<template>
    <MainLayout>
        <div class="max-w-7xl mx-auto space-y-6 animate-in">
            <!-- Success Message -->
            <div
                v-if="successMsg"
                class="fixed top-4 right-4 bg-emerald-500 text-white px-6 py-3 rounded-xl shadow-lg z-50 flex items-center gap-2 animate-in"
            >
                <Check class="w-5 h-5" />
                {{ successMsg }}
            </div>

            <!-- Header -->
            <div
                class="flex flex-col md:flex-row md:items-center justify-between gap-4"
            >
                <div>
                    <h1
                        class="text-2xl font-bold text-foreground tracking-tight flex items-center gap-3"
                    >
                        <div
                            class="p-2 bg-emerald-500/10 dark:bg-emerald-500/20 rounded-xl"
                        >
                            <Shield
                                class="w-5 h-5 text-emerald-600 dark:text-emerald-400"
                            />
                        </div>
                        Security Center
                    </h1>
                    <p class="text-muted-foreground mt-1 text-sm">
                        Manage SSH keys, firewall, 2FA, and security settings.
                    </p>
                </div>

                <div class="flex gap-2">
                    <button
                        v-if="activeTab === 'ssh'"
                        @click="showAddKeyModal = true"
                        class="flex items-center gap-2 px-5 py-2.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl font-medium transition-all shadow-lg shadow-emerald-600/20 active:scale-[0.98] text-sm"
                    >
                        <Plus class="w-4 h-4" />
                        Add SSH Key
                    </button>
                    <button
                        v-if="activeTab === 'ips'"
                        @click="openBlockIp('block')"
                        class="flex items-center gap-2 px-5 py-2.5 bg-destructive hover:bg-destructive/90 text-destructive-foreground rounded-xl font-medium transition-all shadow-lg shadow-destructive/20 active:scale-[0.98] text-sm"
                    >
                        <ShieldAlert class="w-4 h-4" />
                        Block IP
                    </button>
                    <button
                        v-if="activeTab === '2fa' && !twoFaEnabled"
                        @click="generate2faSecret"
                        class="flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl font-medium transition-all shadow-lg shadow-primary/20 active:scale-[0.98] text-sm"
                    >
                        <Smartphone class="w-4 h-4" />
                        Setup 2FA
                    </button>
                </div>
            </div>

            <!-- Tabs -->
            <div
                class="flex flex-wrap gap-1 bg-card border border-border p-1 rounded-xl"
            >
                <button
                    v-for="tab in [
                        { id: 'ssh', label: 'SSH Keys', icon: Key },
                        { id: 'ips', label: 'IP Access', icon: ShieldAlert },
                        { id: 'ssl', label: 'SSL Status', icon: Lock },
                        { id: '2fa', label: '2FA', icon: Smartphone },
                        { id: 'logs', label: 'Access Logs', icon: List },
                    ]"
                    :key="tab.id"
                    @click="activeTab = tab.id as any"
                    :class="[
                        'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all',
                        activeTab === tab.id
                            ? 'bg-primary/10 text-primary'
                            : 'text-muted-foreground hover:text-foreground hover:bg-muted',
                    ]"
                >
                    <component :is="tab.icon" class="w-4 h-4" />
                    {{ tab.label }}
                </button>
            </div>

            <!-- Stats -->
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
                <div
                    v-for="stat in [
                        {
                            label: 'Security',
                            val: 'Good',
                            icon: ShieldCheck,
                            color: 'text-emerald-500',
                        },
                        {
                            label: 'Blocked IPs',
                            val: blockedIps.length,
                            icon: ShieldAlert,
                            color: 'text-destructive',
                        },
                        {
                            label: 'SSH Keys',
                            val: sshKeys.length,
                            icon: Key,
                            color: 'text-blue-500',
                        },
                        {
                            label: '2FA',
                            val: twoFaEnabled ? 'Enabled' : 'Disabled',
                            icon: Smartphone,
                            color: twoFaEnabled
                                ? 'text-emerald-500'
                                : 'text-amber-500',
                        },
                        {
                            label: 'Firewall',
                            val: 'Active',
                            icon: Activity,
                            color: 'text-purple-500',
                        },
                    ]"
                    :key="stat.label"
                    class="bg-card border border-border p-4 rounded-xl"
                >
                    <div class="flex items-center gap-2 mb-2">
                        <component
                            :is="stat.icon"
                            class="w-4 h-4"
                            :class="stat.color"
                        />
                        <p
                            class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider"
                        >
                            {{ stat.label }}
                        </p>
                    </div>
                    <p class="text-xl font-bold text-foreground">
                        {{ stat.val }}
                    </p>
                </div>
            </div>

            <!-- Content -->
            <div
                class="bg-card border border-border rounded-2xl overflow-hidden min-h-[400px]"
            >
                <!-- Loading -->
                <div
                    v-if="isLoading"
                    class="p-20 flex flex-col items-center justify-center space-y-4"
                >
                    <div
                        class="w-10 h-10 border-4 border-border border-t-emerald-600 rounded-full animate-spin"
                    />
                    <p
                        class="text-muted-foreground font-medium text-xs uppercase tracking-wider"
                    >
                        Loading...
                    </p>
                </div>

                <!-- Error -->
                <div
                    v-else-if="error && !sshKeys.length"
                    class="p-20 text-center"
                >
                    <AlertCircle
                        class="w-16 h-16 text-destructive/30 mx-auto mb-6"
                    />
                    <h3 class="text-lg font-bold text-foreground">
                        Security Check Failed
                    </h3>
                    <p class="text-muted-foreground mt-2 text-sm">
                        {{ error }}
                    </p>
                    <button
                        @click="fetchData"
                        class="mt-6 px-6 py-2.5 bg-foreground text-background rounded-lg font-medium"
                    >
                        Retry
                    </button>
                </div>

                <div v-else>
                    <!-- SSH Keys Tab -->
                    <div v-if="activeTab === 'ssh'">
                        <div
                            class="p-4 border-b border-border flex items-center justify-between"
                        >
                            <span class="text-sm font-semibold text-foreground"
                                >SSH Keys</span
                            >
                            <button
                                @click="showAddKeyModal = true"
                                class="text-xs text-primary hover:underline"
                            >
                                + Add New
                            </button>
                        </div>
                        <div
                            v-if="sshKeys.length === 0"
                            class="p-20 text-center"
                        >
                            <Lock
                                class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6"
                            />
                            <h3 class="text-lg font-bold text-foreground">
                                No SSH keys
                            </h3>
                            <p class="text-muted-foreground mt-2 text-sm">
                                Add SSH keys for secure server access.
                            </p>
                        </div>
                        <div
                            v-else
                            class="p-6 grid grid-cols-1 md:grid-cols-2 gap-4"
                        >
                            <div
                                v-for="key in sshKeys"
                                :key="key.id"
                                class="bg-muted/50 border border-border p-5 rounded-xl group hover:border-primary/30 transition-all"
                            >
                                <div class="flex items-start justify-between">
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="w-10 h-10 bg-emerald-500/10 rounded-xl flex items-center justify-center text-emerald-600"
                                        >
                                            <Key class="w-5 h-5" />
                                        </div>
                                        <div>
                                            <h4
                                                class="text-sm font-semibold text-foreground"
                                            >
                                                {{ key.label }}
                                            </h4>
                                            <p
                                                class="text-[10px] text-muted-foreground mt-0.5"
                                            >
                                                Added:
                                                {{
                                                    key.created_at?.split(
                                                        'T',
                                                    )[0]
                                                }}
                                            </p>
                                        </div>
                                    </div>
                                    <button
                                        @click="openDeleteKey(key)"
                                        class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </button>
                                </div>
                                <div class="mt-4">
                                    <p
                                        class="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider mb-1"
                                    >
                                        Fingerprint
                                    </p>
                                    <div
                                        class="bg-card border border-border p-3 rounded-lg"
                                    >
                                        <p
                                            class="text-[10px] font-mono text-muted-foreground break-all"
                                        >
                                            {{
                                                key.fingerprint ||
                                                'SHA256:xxxxxxxxxxxxxxxxxxxxxxxxxxx'
                                            }}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- IP Access Tab -->
                    <div v-else-if="activeTab === 'ips'">
                        <div
                            class="p-4 border-b border-border flex items-center justify-between"
                        >
                            <span class="text-sm font-semibold text-foreground"
                                >IP Access Manager</span
                            >
                            <div class="flex gap-2">
                                <button
                                    @click="openBlockIp('whitelist')"
                                    class="text-xs text-emerald-600 hover:underline"
                                >
                                    + Whitelist
                                </button>
                                <button
                                    @click="openBlockIp('block')"
                                    class="text-xs text-destructive hover:underline"
                                >
                                    + Block
                                </button>
                            </div>
                        </div>
                        <div
                            v-if="blockedIps.length === 0"
                            class="p-20 text-center"
                        >
                            <ShieldCheck
                                class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6"
                            />
                            <h3 class="text-lg font-bold text-foreground">
                                No IP rules
                            </h3>
                            <p class="text-muted-foreground mt-2 text-sm">
                                Add IPs to whitelist or blocklist.
                            </p>
                        </div>
                        <table v-else class="w-full text-left">
                            <thead>
                                <tr
                                    class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider"
                                >
                                    <th class="px-6 py-4">IP Address</th>
                                    <th class="px-6 py-4">Type</th>
                                    <th class="px-6 py-4">Reason</th>
                                    <th class="px-6 py-4">Date</th>
                                    <th class="px-6 py-4 text-right">
                                        Actions
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border">
                                <tr
                                    v-for="ip in blockedIps"
                                    :key="ip.id"
                                    class="group hover:bg-muted/50 transition-all"
                                >
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-2">
                                            <div
                                                :class="[
                                                    'w-2 h-2 rounded-full',
                                                    ip.type === 'whitelist'
                                                        ? 'bg-emerald-500'
                                                        : 'bg-destructive',
                                                ]"
                                            />
                                            <span
                                                class="text-sm font-mono font-semibold text-foreground"
                                                >{{ ip.ip_address }}</span
                                            >
                                        </div>
                                    </td>
                                    <td class="px-6 py-4">
                                        <span
                                            :class="[
                                                'text-[10px] font-semibold uppercase px-2 py-1 rounded',
                                                ip.type === 'whitelist'
                                                    ? 'bg-emerald-500/10 text-emerald-600'
                                                    : 'bg-destructive/10 text-destructive',
                                            ]"
                                        >
                                            {{ ip.type || 'Block' }}
                                        </span>
                                    </td>
                                    <td
                                        class="px-6 py-4 text-xs text-muted-foreground"
                                    >
                                        {{ ip.reason }}
                                    </td>
                                    <td
                                        class="px-6 py-4 text-xs text-muted-foreground"
                                    >
                                        {{ ip.blocked_at?.split('T')[0] }}
                                    </td>
                                    <td class="px-6 py-4 text-right">
                                        <button
                                            @click="unblockIp(ip)"
                                            class="px-3 py-1.5 bg-muted hover:bg-muted/80 text-foreground rounded-lg text-xs font-medium transition-all opacity-0 group-hover:opacity-100 border border-border"
                                        >
                                            Remove
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <!-- SSL Status Tab -->
                    <div v-else-if="activeTab === 'ssl'">
                        <div class="p-4 border-b border-border">
                            <span class="text-sm font-semibold text-foreground"
                                >SSL/TLS Status by Domain</span
                            >
                        </div>
                        <div
                            v-if="domains.length === 0"
                            class="p-20 text-center"
                        >
                            <Lock
                                class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6"
                            />
                            <h3 class="text-lg font-bold text-foreground">
                                No domains
                            </h3>
                            <p class="text-muted-foreground mt-2 text-sm">
                                Add domains to manage SSL certificates.
                            </p>
                        </div>
                        <table v-else class="w-full text-left">
                            <thead>
                                <tr
                                    class="bg-muted/50 border-b border-border text-[10px] font-semibold text-muted-foreground uppercase tracking-wider"
                                >
                                    <th class="px-6 py-4">Domain</th>
                                    <th class="px-6 py-4">Status</th>
                                    <th class="px-6 py-4">Provider</th>
                                    <th class="px-6 py-4">Expires</th>
                                    <th class="px-6 py-4 text-right">
                                        Actions
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border">
                                <tr
                                    v-for="domain in sslStatuses"
                                    :key="domain.id"
                                    class="group hover:bg-muted/50 transition-all"
                                >
                                    <td class="px-6 py-4">
                                        <div class="flex items-center gap-3">
                                            <div
                                                :class="[
                                                    'w-10 h-10 rounded-xl flex items-center justify-center',
                                                    domain.ssl_enabled
                                                        ? 'bg-emerald-500/10 text-emerald-600'
                                                        : 'bg-amber-500/10 text-amber-600',
                                                ]"
                                            >
                                                <Lock class="w-5 h-5" />
                                            </div>
                                            <div>
                                                <p
                                                    class="text-sm font-semibold text-foreground"
                                                >
                                                    {{ domain.domain_name }}
                                                </p>
                                                <p
                                                    class="text-[10px] text-muted-foreground mt-0.5"
                                                >
                                                    {{ domain.document_root }}
                                                </p>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4">
                                        <span
                                            :class="[
                                                'text-[10px] font-semibold uppercase px-2 py-1 rounded border',
                                                domain.ssl_enabled
                                                    ? 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20'
                                                    : 'bg-amber-500/10 text-amber-600 border-amber-500/20',
                                            ]"
                                        >
                                            {{ domain.ssl_status }}
                                        </span>
                                    </td>
                                    <td
                                        class="px-6 py-4 text-sm text-muted-foreground"
                                    >
                                        {{ domain.ssl_provider }}
                                    </td>
                                    <td
                                        class="px-6 py-4 text-sm text-muted-foreground"
                                    >
                                        {{ domain.ssl_expiry }}
                                    </td>
                                    <td class="px-6 py-4 text-right">
                                        <button
                                            class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium transition-all opacity-0 group-hover:opacity-100"
                                        >
                                            {{
                                                domain.ssl_enabled
                                                    ? 'Renew'
                                                    : 'Install'
                                            }}
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <!-- 2FA Tab -->
                    <div v-else-if="activeTab === '2fa'">
                        <div class="p-4 border-b border-border">
                            <span class="text-sm font-semibold text-foreground"
                                >Two-Factor Authentication</span
                            >
                        </div>
                        <div class="p-6">
                            <div
                                class="bg-muted/50 border border-border rounded-xl p-6"
                            >
                                <div class="flex items-start gap-4">
                                    <div
                                        :class="[
                                            'w-12 h-12 rounded-xl flex items-center justify-center',
                                            twoFaEnabled
                                                ? 'bg-emerald-500/10 text-emerald-600'
                                                : 'bg-amber-500/10 text-amber-600',
                                        ]"
                                    >
                                        <Smartphone class="w-6 h-6" />
                                    </div>
                                    <div class="flex-1">
                                        <h3
                                            class="text-lg font-semibold text-foreground"
                                        >
                                            Two-Factor Authentication
                                        </h3>
                                        <p
                                            class="text-sm text-muted-foreground mt-1"
                                        >
                                            {{
                                                twoFaEnabled
                                                    ? '2FA is currently enabled on your account.'
                                                    : 'Add an extra layer of security to your account by enabling 2FA.'
                                            }}
                                        </p>
                                        <div class="mt-4">
                                            <span
                                                :class="[
                                                    'px-3 py-1.5 rounded-lg text-sm font-semibold',
                                                    twoFaEnabled
                                                        ? 'bg-emerald-500/10 text-emerald-600 border border-emerald-500/20'
                                                        : 'bg-amber-500/10 text-amber-600 border border-amber-500/20',
                                                ]"
                                            >
                                                {{
                                                    twoFaEnabled
                                                        ? 'Enabled'
                                                        : 'Disabled'
                                                }}
                                            </span>
                                        </div>
                                    </div>
                                    <button
                                        v-if="!twoFaEnabled"
                                        @click="generate2faSecret"
                                        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-all"
                                    >
                                        Enable 2FA
                                    </button>
                                    <button
                                        v-else
                                        class="px-4 py-2 bg-destructive text-destructive-foreground rounded-lg text-sm font-medium hover:bg-destructive/90 transition-all"
                                    >
                                        Disable 2FA
                                    </button>
                                </div>
                            </div>
                            <div
                                class="mt-6 bg-primary/5 border border-primary/20 p-4 rounded-xl"
                            >
                                <div class="flex items-start gap-3">
                                    <UserCheck
                                        class="w-5 h-5 text-primary mt-0.5"
                                    />
                                    <div>
                                        <p
                                            class="text-sm text-foreground font-medium"
                                        >
                                            Recommended: Use an authenticator
                                            app
                                        </p>
                                        <p
                                            class="text-xs text-muted-foreground mt-1"
                                        >
                                            Google Authenticator, Authy,
                                            1Password, or similar apps.
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Access Logs Tab -->
                    <div v-else>
                        <div class="p-4 border-b border-border">
                            <span class="text-sm font-semibold text-foreground"
                                >Recent Access Logs</span
                            >
                        </div>
                        <div
                            v-if="accessLogs.length === 0"
                            class="p-20 text-center"
                        >
                            <List
                                class="w-16 h-16 text-muted-foreground/30 mx-auto mb-6"
                            />
                            <h3 class="text-lg font-bold text-foreground">
                                No logs
                            </h3>
                            <p class="text-muted-foreground mt-2 text-sm">
                                No access attempts recorded.
                            </p>
                        </div>
                        <div v-else class="divide-y divide-border">
                            <div
                                v-for="(log, idx) in accessLogs.slice(0, 50)"
                                :key="idx"
                                class="px-6 py-4 flex items-center justify-between text-xs group hover:bg-muted/50 transition-colors"
                            >
                                <div class="flex items-center gap-4">
                                    <span
                                        class="text-muted-foreground font-mono"
                                        >[{{
                                            log.timestamp
                                                ?.split('T')[1]
                                                ?.split('.')[0]
                                        }}]</span
                                    >
                                    <span
                                        :class="[
                                            'px-2 py-0.5 rounded text-[9px] font-semibold uppercase tracking-wider border',
                                            log.event_type === 'login_success'
                                                ? 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20'
                                                : 'bg-destructive/10 text-destructive border-destructive/20',
                                        ]"
                                        >{{ log.event_type }}</span
                                    >
                                    <span
                                        class="text-foreground font-semibold font-mono"
                                        >{{ log.ip_address }}</span
                                    >
                                </div>
                                <span
                                    class="text-muted-foreground hidden md:block truncate max-w-[250px]"
                                    >{{ log.user_agent }}</span
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Resource Monitoring -->
            <div v-if="stats" class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="bg-card border border-border p-6 rounded-2xl">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                            <div class="p-2 bg-blue-500/10 rounded-lg">
                                <Cpu class="w-4 h-4 text-blue-500" />
                            </div>
                            <span class="text-sm font-semibold text-foreground"
                                >CPU Load</span
                            >
                        </div>
                        <span class="text-lg font-bold text-blue-500"
                            >{{ stats.cpu_percent }}%</span
                        >
                    </div>
                    <div
                        class="h-2 w-full bg-muted rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full bg-blue-500 rounded-full transition-all"
                            :style="{ width: stats.cpu_percent + '%' }"
                        />
                    </div>
                </div>
                <div class="bg-card border border-border p-6 rounded-2xl">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                            <div class="p-2 bg-emerald-500/10 rounded-lg">
                                <HardDrive class="w-4 h-4 text-emerald-500" />
                            </div>
                            <span class="text-sm font-semibold text-foreground"
                                >Memory</span
                            >
                        </div>
                        <span class="text-lg font-bold text-emerald-500"
                            >{{ stats.memory_percent }}%</span
                        >
                    </div>
                    <div
                        class="h-2 w-full bg-muted rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full bg-emerald-500 rounded-full transition-all"
                            :style="{ width: stats.memory_percent + '%' }"
                        />
                    </div>
                </div>
            </div>
        </div>

        <!-- ==================== MODALS ==================== -->

        <!-- Add SSH Key Modal -->
        <div
            v-if="showAddKeyModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showAddKeyModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <div class="flex items-center justify-between mb-6">
                    <h3 class="text-lg font-semibold text-foreground">
                        Add SSH Key
                    </h3>
                    <button
                        @click="showAddKeyModal = false"
                        class="p-1 rounded hover:bg-muted transition-colors"
                    >
                        <X class="w-5 h-5 text-muted-foreground" />
                    </button>
                </div>
                <div class="space-y-4">
                    <div>
                        <label
                            class="block text-sm font-medium text-foreground mb-2"
                            >Label</label
                        >
                        <input
                            v-model="newKeyLabel"
                            type="text"
                            placeholder="My Laptop"
                            class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                        />
                    </div>
                    <div>
                        <label
                            class="block text-sm font-medium text-foreground mb-2"
                            >Public Key</label
                        >
                        <textarea
                            v-model="newKeyPublic"
                            rows="4"
                            placeholder="ssh-rsa AAAAB3..."
                            class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none resize-none"
                        />
                    </div>
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showAddKeyModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="addSshKey"
                        class="px-4 py-2 text-sm font-medium bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-colors"
                    >
                        Add Key
                    </button>
                </div>
            </div>
        </div>

        <!-- Block IP Modal -->
        <div
            v-if="showBlockIpModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showBlockIpModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <div class="flex items-center justify-between mb-6">
                    <h3 class="text-lg font-semibold text-foreground">
                        {{ blockType === 'block' ? 'Block' : 'Whitelist' }} IP
                        Address
                    </h3>
                    <button
                        @click="showBlockIpModal = false"
                        class="p-1 rounded hover:bg-muted transition-colors"
                    >
                        <X class="w-5 h-5 text-muted-foreground" />
                    </button>
                </div>
                <div class="space-y-4">
                    <div>
                        <label
                            class="block text-sm font-medium text-foreground mb-2"
                            >IP Address (or CIDR)</label
                        >
                        <input
                            v-model="newBlockIp"
                            type="text"
                            placeholder="192.168.1.100 or 10.0.0.0/24"
                            class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                        />
                    </div>
                    <div>
                        <label
                            class="block text-sm font-medium text-foreground mb-2"
                            >Reason (optional)</label
                        >
                        <input
                            v-model="newBlockReason"
                            type="text"
                            placeholder="Suspicious activity"
                            class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                        />
                    </div>
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showBlockIpModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="blockIp"
                        :class="[
                            'px-4 py-2 text-sm font-medium text-white rounded-lg transition-colors',
                            blockType === 'block'
                                ? 'bg-destructive hover:bg-destructive/90'
                                : 'bg-emerald-600 hover:bg-emerald-700',
                        ]"
                    >
                        {{
                            blockType === 'block' ? 'Block IP' : 'Whitelist IP'
                        }}
                    </button>
                </div>
            </div>
        </div>

        <!-- 2FA Setup Modal -->
        <div
            v-if="showSetup2faModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showSetup2faModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <div class="flex items-center justify-between mb-6">
                    <h3 class="text-lg font-semibold text-foreground">
                        Setup 2FA
                    </h3>
                    <button
                        @click="showSetup2faModal = false"
                        class="p-1 rounded hover:bg-muted transition-colors"
                    >
                        <X class="w-5 h-5 text-muted-foreground" />
                    </button>
                </div>
                <div
                    v-if="error"
                    class="mb-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex items-center gap-2"
                >
                    <AlertCircle class="w-4 h-4" />{{ error }}
                </div>
                <div class="text-center mb-6">
                    <div
                        class="w-32 h-32 bg-muted border border-border rounded-xl mx-auto flex items-center justify-center"
                    >
                        <QrCode class="w-16 h-16 text-muted-foreground" />
                    </div>
                    <p class="text-xs text-muted-foreground mt-3">
                        Scan with your authenticator app
                    </p>
                </div>
                <div class="bg-muted p-3 rounded-lg mb-4">
                    <p
                        class="text-[10px] text-muted-foreground uppercase tracking-wider mb-1"
                    >
                        Manual entry key
                    </p>
                    <p class="text-sm font-mono text-foreground break-all">
                        {{ twoFaSecret }}
                    </p>
                </div>
                <div>
                    <label
                        class="block text-sm font-medium text-foreground mb-2"
                        >Verification Code</label
                    >
                    <input
                        v-model="twoFaCode"
                        type="text"
                        maxlength="6"
                        placeholder="000000"
                        class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm text-center font-mono text-lg tracking-widest focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    />
                </div>
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="
                            showSetup2faModal = false;
                            error = null;
                        "
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="setup2fa"
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Verify & Enable
                    </button>
                </div>
            </div>
        </div>

        <!-- Delete Key Modal -->
        <div
            v-if="showDeleteKeyModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showDeleteKeyModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-2">
                    Delete SSH Key
                </h3>
                <p class="text-sm text-muted-foreground mb-6">
                    Delete
                    <span class="font-semibold text-foreground">{{
                        selectedKey?.label
                    }}</span
                    >?
                </p>
                <div class="flex justify-end gap-3">
                    <button
                        @click="showDeleteKeyModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="deleteSshKey"
                        class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors"
                    >
                        Delete
                    </button>
                </div>
            </div>
        </div>
    </MainLayout>
</template>

<style scoped>
.animate-in {
    animation: animate-in 0.4s ease-out;
}
@keyframes animate-in {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
