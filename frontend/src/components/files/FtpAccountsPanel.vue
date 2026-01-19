<script setup lang="ts">
/**
 * FtpAccountsPanel - Panel FTP Account Management
 *
 * Features:
 * - List FTP accounts
 * - Create/Edit/Delete accounts
 * - Change password
 * - Show server info
 */
import { ref, onMounted } from 'vue';
import { ftpService } from '@/services/ftp.service';
import {
    Server,
    Plus,
    Trash2,
    Key,
    ToggleLeft,
    ToggleRight,
    X,
    Loader2,
    AlertCircle,
    Copy,
    Check,
} from 'lucide-vue-next';

/** Props */
defineProps<{
    visible: boolean;
}>();

/** Emits */
const emit = defineEmits<{
    (e: 'close'): void;
}>();

/** FTP accounts list */
interface FtpAccount {
    id: string;
    username: string;
    home_directory: string;
    is_active: boolean;
    quota_mb: number | null;
    used_bytes: number;
    created_at: string;
    last_login: string | null;
}

const accounts = ref<FtpAccount[]>([]);
const serverInfo = ref<{ host: string; port: number } | null>(null);
const isLoading = ref(false);
const error = ref('');

/** Modal states */
const showCreateModal = ref(false);
const showPasswordModal = ref(false);
const selectedAccount = ref<FtpAccount | null>(null);

/** Form data */
const formData = ref({
    username: '',
    password: '',
    home_directory: '',
    quota_mb: 0,
});

/** Copied indicator */
const copiedField = ref('');

/** Load accounts */
const loadAccounts = async (): Promise<void> => {
    try {
        isLoading.value = true;
        error.value = '';
        const response = await ftpService.listAccounts();
        accounts.value = response.data.data || [];

        const infoResponse = await ftpService.getServerInfo();
        serverInfo.value = infoResponse.data.data;
    } catch (err: any) {
        error.value =
            err.response?.data?.message || 'Gagal memuat FTP accounts';
    } finally {
        isLoading.value = false;
    }
};

/** Create account */
const createAccount = async (): Promise<void> => {
    try {
        isLoading.value = true;
        await ftpService.createAccount({
            username: formData.value.username,
            password: formData.value.password,
            home_directory: formData.value.home_directory || undefined,
            quota_mb: formData.value.quota_mb || undefined,
        });
        showCreateModal.value = false;
        formData.value = {
            username: '',
            password: '',
            home_directory: '',
            quota_mb: 0,
        };
        await loadAccounts();
    } catch (err: any) {
        error.value =
            err.response?.data?.message || 'Gagal membuat FTP account';
    } finally {
        isLoading.value = false;
    }
};

/** Toggle account status */
const toggleStatus = async (account: FtpAccount): Promise<void> => {
    try {
        await ftpService.toggleStatus(account.id);
        await loadAccounts();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal mengubah status';
    }
};

/** Change password */
const changePassword = async (): Promise<void> => {
    if (!selectedAccount.value) return;
    try {
        isLoading.value = true;
        await ftpService.changePassword(selectedAccount.value.id, {
            new_password: formData.value.password,
        });
        showPasswordModal.value = false;
        formData.value.password = '';
        selectedAccount.value = null;
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal mengubah password';
    } finally {
        isLoading.value = false;
    }
};

/** Delete account */
const deleteAccount = async (account: FtpAccount): Promise<void> => {
    if (!confirm(`Hapus FTP account "${account.username}"?`)) return;
    try {
        await ftpService.deleteAccount(account.id);
        await loadAccounts();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal menghapus account';
    }
};

/** Copy to clipboard */
const copyToClipboard = async (text: string, field: string): Promise<void> => {
    try {
        await navigator.clipboard.writeText(text);
        copiedField.value = field;
        setTimeout(() => (copiedField.value = ''), 2000);
    } catch {
        // Fallback
    }
};

/** Format bytes */
const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

onMounted(() => {
    loadAccounts();
});
</script>

<template>
    <Teleport to="body">
        <div
            v-if="visible"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="emit('close')"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-2xl w-full max-w-3xl max-h-[80vh] flex flex-col animate-in"
            >
                <!-- Header -->
                <div
                    class="h-14 border-b border-border px-6 flex items-center justify-between flex-shrink-0"
                >
                    <div class="flex items-center gap-3">
                        <Server class="w-5 h-5 text-primary" />
                        <h2 class="text-lg font-semibold text-foreground">
                            FTP Accounts
                        </h2>
                    </div>
                    <button
                        @click="emit('close')"
                        class="p-2 hover:bg-muted rounded-lg transition-colors"
                    >
                        <X class="w-5 h-5 text-muted-foreground" />
                    </button>
                </div>

                <!-- Server Info -->
                <div
                    v-if="serverInfo"
                    class="px-6 py-4 border-b border-border bg-muted/30"
                >
                    <div class="flex items-center gap-6 text-sm">
                        <div class="flex items-center gap-2">
                            <span class="text-muted-foreground">Host:</span>
                            <code
                                class="bg-muted px-2 py-0.5 rounded text-foreground"
                                >{{ serverInfo.host }}</code
                            >
                            <button
                                @click="
                                    copyToClipboard(serverInfo.host, 'host')
                                "
                                class="p-1 hover:bg-muted rounded"
                            >
                                <component
                                    :is="copiedField === 'host' ? Check : Copy"
                                    class="w-3 h-3"
                                />
                            </button>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-muted-foreground">Port:</span>
                            <code
                                class="bg-muted px-2 py-0.5 rounded text-foreground"
                                >{{ serverInfo.port }}</code
                            >
                        </div>
                    </div>
                </div>

                <!-- Actions -->
                <div
                    class="px-6 py-3 border-b border-border flex items-center justify-between"
                >
                    <span class="text-sm text-muted-foreground"
                        >{{ accounts.length }} accounts</span
                    >
                    <button
                        @click="showCreateModal = true"
                        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors flex items-center gap-2"
                    >
                        <Plus class="w-4 h-4" />
                        New Account
                    </button>
                </div>

                <!-- Error -->
                <div
                    v-if="error"
                    class="mx-6 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg flex items-center gap-2 text-destructive text-sm"
                >
                    <AlertCircle class="w-4 h-4" />
                    {{ error }}
                </div>

                <!-- Accounts List -->
                <div class="flex-1 overflow-auto p-6">
                    <div
                        v-if="isLoading"
                        class="flex items-center justify-center py-12"
                    >
                        <Loader2 class="w-8 h-8 text-primary animate-spin" />
                    </div>

                    <div
                        v-else-if="accounts.length === 0"
                        class="text-center py-12 text-muted-foreground"
                    >
                        No FTP accounts yet
                    </div>

                    <div v-else class="space-y-3">
                        <div
                            v-for="account in accounts"
                            :key="account.id"
                            class="bg-muted/30 border border-border rounded-lg p-4"
                        >
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <div
                                        :class="[
                                            'w-2 h-2 rounded-full',
                                            account.is_active
                                                ? 'bg-green-500'
                                                : 'bg-muted-foreground',
                                        ]"
                                    />
                                    <div>
                                        <div
                                            class="font-medium text-foreground"
                                        >
                                            {{ account.username }}
                                        </div>
                                        <div
                                            class="text-xs text-muted-foreground"
                                        >
                                            {{ account.home_directory }}
                                        </div>
                                    </div>
                                </div>

                                <div class="flex items-center gap-2">
                                    <button
                                        @click="toggleStatus(account)"
                                        class="p-2 hover:bg-muted rounded-lg transition-colors"
                                        :title="
                                            account.is_active
                                                ? 'Disable'
                                                : 'Enable'
                                        "
                                    >
                                        <component
                                            :is="
                                                account.is_active
                                                    ? ToggleRight
                                                    : ToggleLeft
                                            "
                                            :class="[
                                                'w-5 h-5',
                                                account.is_active
                                                    ? 'text-green-500'
                                                    : 'text-muted-foreground',
                                            ]"
                                        />
                                    </button>
                                    <button
                                        @click="
                                            selectedAccount = account;
                                            showPasswordModal = true;
                                        "
                                        class="p-2 hover:bg-muted rounded-lg transition-colors text-muted-foreground"
                                        title="Change Password"
                                    >
                                        <Key class="w-4 h-4" />
                                    </button>
                                    <button
                                        @click="deleteAccount(account)"
                                        class="p-2 hover:bg-destructive/10 rounded-lg transition-colors text-destructive"
                                        title="Delete"
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </button>
                                </div>
                            </div>

                            <div
                                class="mt-3 flex items-center gap-4 text-xs text-muted-foreground"
                            >
                                <span
                                    >Quota:
                                    {{
                                        account.quota_mb
                                            ? `${account.quota_mb} MB`
                                            : 'Unlimited'
                                    }}</span
                                >
                                <span
                                    >Used:
                                    {{ formatBytes(account.used_bytes) }}</span
                                >
                                <span v-if="account.last_login"
                                    >Last login:
                                    {{
                                        new Date(
                                            account.last_login,
                                        ).toLocaleDateString()
                                    }}</span
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Create Modal -->
                <div
                    v-if="showCreateModal"
                    class="absolute inset-0 bg-black/50 flex items-center justify-center"
                    @click.self="showCreateModal = false"
                >
                    <div
                        class="bg-card border border-border rounded-xl p-6 w-full max-w-md"
                    >
                        <h3 class="text-lg font-semibold text-foreground mb-4">
                            Create FTP Account
                        </h3>
                        <div class="space-y-4">
                            <div>
                                <label
                                    class="block text-sm font-medium text-muted-foreground mb-1"
                                    >Username</label
                                >
                                <input
                                    v-model="formData.username"
                                    type="text"
                                    class="w-full bg-muted border border-border rounded-lg px-4 py-2 text-sm"
                                />
                            </div>
                            <div>
                                <label
                                    class="block text-sm font-medium text-muted-foreground mb-1"
                                    >Password</label
                                >
                                <input
                                    v-model="formData.password"
                                    type="password"
                                    class="w-full bg-muted border border-border rounded-lg px-4 py-2 text-sm"
                                />
                            </div>
                            <div>
                                <label
                                    class="block text-sm font-medium text-muted-foreground mb-1"
                                    >Home Directory (optional)</label
                                >
                                <input
                                    v-model="formData.home_directory"
                                    type="text"
                                    placeholder="/public_html"
                                    class="w-full bg-muted border border-border rounded-lg px-4 py-2 text-sm"
                                />
                            </div>
                            <div>
                                <label
                                    class="block text-sm font-medium text-muted-foreground mb-1"
                                    >Quota (MB, 0 = unlimited)</label
                                >
                                <input
                                    v-model.number="formData.quota_mb"
                                    type="number"
                                    class="w-full bg-muted border border-border rounded-lg px-4 py-2 text-sm"
                                />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3 mt-6">
                            <button
                                @click="showCreateModal = false"
                                class="px-4 py-2 text-sm text-muted-foreground hover:text-foreground"
                            >
                                Cancel
                            </button>
                            <button
                                @click="createAccount"
                                class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90"
                            >
                                Create
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Password Modal -->
                <div
                    v-if="showPasswordModal"
                    class="absolute inset-0 bg-black/50 flex items-center justify-center"
                    @click.self="showPasswordModal = false"
                >
                    <div
                        class="bg-card border border-border rounded-xl p-6 w-full max-w-md"
                    >
                        <h3 class="text-lg font-semibold text-foreground mb-4">
                            Change Password
                        </h3>
                        <p class="text-sm text-muted-foreground mb-4">
                            Account: {{ selectedAccount?.username }}
                        </p>
                        <div>
                            <label
                                class="block text-sm font-medium text-muted-foreground mb-1"
                                >New Password</label
                            >
                            <input
                                v-model="formData.password"
                                type="password"
                                class="w-full bg-muted border border-border rounded-lg px-4 py-2 text-sm"
                            />
                        </div>
                        <div class="flex justify-end gap-3 mt-6">
                            <button
                                @click="showPasswordModal = false"
                                class="px-4 py-2 text-sm text-muted-foreground hover:text-foreground"
                            >
                                Cancel
                            </button>
                            <button
                                @click="changePassword"
                                class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90"
                            >
                                Change
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<style scoped>
.animate-in {
    animation: slideUp 0.2s ease-out;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
