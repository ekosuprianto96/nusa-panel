<script setup lang="ts">
/**
 * DirectoryPickerModal - Modal untuk memilih directory tujuan
 *
 * Digunakan untuk operasi Move/Copy files
 */
import { ref, computed, onMounted, watch } from 'vue';
import { fileService } from '@/services/file.service';
import type { FileInfo } from '@/types';
import {
    Folder,
    ChevronRight,
    Home,
    X,
    Loader2,
    FolderPlus,
    Check,
    ArrowLeft,
} from 'lucide-vue-next';

/** Props */
const props = defineProps<{
    visible: boolean;
    action: 'move' | 'copy';
    fileCount: number;
    excludePaths?: string[]; // Paths yang tidak bisa dipilih (source files)
}>();

/** Emits */
const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'select', path: string): void;
}>();

/** State */
const currentPath = ref('');
const directories = ref<FileInfo[]>([]);
const isLoading = ref(false);
const error = ref('');
const showNewFolderInput = ref(false);
const newFolderName = ref('');

/** Breadcrumbs */
const breadcrumbs = computed(() => {
    if (!currentPath.value) return [];
    const parts = currentPath.value.split('/').filter(Boolean);
    let path = '';
    return parts.map((part) => {
        path += '/' + part;
        return { name: part, path: path.substring(1) };
    });
});

/** Load directories */
const loadDirectories = async (): Promise<void> => {
    try {
        isLoading.value = true;
        error.value = '';
        const response = await fileService.listFiles(currentPath.value, false);
        // Filter hanya directories
        directories.value = (response.data.data.items || []).filter(
            (f) => f.file_type === 'directory',
        );
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal memuat folder';
    } finally {
        isLoading.value = false;
    }
};

/** Navigate to folder */
const navigateTo = (path: string): void => {
    currentPath.value = path;
};

/** Go back */
const goBack = (): void => {
    const parts = currentPath.value.split('/').filter(Boolean);
    parts.pop();
    currentPath.value = parts.join('/');
};

/** Check if path is excluded */
const isExcluded = (path: string): boolean => {
    if (!props.excludePaths) return false;
    return props.excludePaths.some(
        (p) => path === p || path.startsWith(p + '/'),
    );
};

/** Create new folder */
const createFolder = async (): Promise<void> => {
    if (!newFolderName.value.trim()) return;
    try {
        await fileService.create({
            path: currentPath.value || '',
            name: newFolderName.value,
            file_type: 'directory',
        });
        newFolderName.value = '';
        showNewFolderInput.value = false;
        await loadDirectories();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal membuat folder';
    }
};

/** Select current directory */
const selectCurrentDirectory = (): void => {
    emit('select', currentPath.value);
};

/** Watch visibility */
watch(
    () => props.visible,
    (visible) => {
        if (visible) {
            currentPath.value = '';
            loadDirectories();
        }
    },
);

onMounted(() => {
    if (props.visible) {
        loadDirectories();
    }
});

/** Watch path changes */
watch(currentPath, () => {
    loadDirectories();
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
                class="bg-card border border-border rounded-xl shadow-2xl w-full max-w-lg max-h-[70vh] flex flex-col animate-in"
            >
                <!-- Header -->
                <div
                    class="h-14 border-b border-border px-4 flex items-center justify-between flex-shrink-0"
                >
                    <div class="flex items-center gap-2">
                        <Folder class="w-5 h-5 text-primary" />
                        <h2 class="text-lg font-semibold text-foreground">
                            {{ action === 'move' ? 'Move' : 'Copy' }}
                            {{ fileCount }} item{{ fileCount > 1 ? 's' : '' }}
                        </h2>
                    </div>
                    <button
                        @click="emit('close')"
                        class="p-2 hover:bg-muted rounded-lg transition-colors"
                    >
                        <X class="w-5 h-5 text-muted-foreground" />
                    </button>
                </div>

                <!-- Breadcrumb -->
                <div
                    class="h-10 border-b border-border px-4 flex items-center gap-1 overflow-x-auto flex-shrink-0 bg-muted/30"
                >
                    <button
                        @click="navigateTo('')"
                        class="p-1 hover:bg-muted rounded transition-colors text-muted-foreground hover:text-primary"
                    >
                        <Home class="w-4 h-4" />
                    </button>
                    <template v-for="crumb in breadcrumbs" :key="crumb.path">
                        <ChevronRight
                            class="w-4 h-4 text-muted-foreground/50 flex-shrink-0"
                        />
                        <button
                            @click="navigateTo(crumb.path)"
                            class="text-sm font-medium text-foreground hover:text-primary transition-colors truncate"
                        >
                            {{ crumb.name }}
                        </button>
                    </template>
                </div>

                <!-- Error -->
                <div
                    v-if="error"
                    class="mx-4 mt-3 p-2 bg-destructive/10 border border-destructive/20 rounded text-destructive text-sm"
                >
                    {{ error }}
                </div>

                <!-- Directory List -->
                <div class="flex-1 overflow-auto p-2">
                    <!-- Loading -->
                    <div
                        v-if="isLoading"
                        class="flex items-center justify-center py-8"
                    >
                        <Loader2 class="w-6 h-6 text-primary animate-spin" />
                    </div>

                    <!-- Back button -->
                    <button
                        v-if="currentPath && !isLoading"
                        @click="goBack"
                        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-muted rounded-lg transition-colors text-left"
                    >
                        <ArrowLeft class="w-5 h-5 text-muted-foreground" />
                        <span class="text-sm text-muted-foreground">..</span>
                    </button>

                    <!-- Folders -->
                    <template v-if="!isLoading">
                        <button
                            v-for="dir in directories"
                            :key="dir.path"
                            @click="navigateTo(dir.path)"
                            :disabled="isExcluded(dir.path)"
                            :class="[
                                'w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-left',
                                isExcluded(dir.path)
                                    ? 'opacity-50 cursor-not-allowed'
                                    : 'hover:bg-muted',
                            ]"
                        >
                            <Folder class="w-5 h-5 text-primary" />
                            <span class="text-sm text-foreground truncate">{{
                                dir.name
                            }}</span>
                        </button>

                        <!-- Empty state -->
                        <div
                            v-if="directories.length === 0"
                            class="text-center py-8 text-muted-foreground text-sm"
                        >
                            No subfolders
                        </div>
                    </template>
                </div>

                <!-- New Folder Input -->
                <div
                    v-if="showNewFolderInput"
                    class="px-4 py-2 border-t border-border"
                >
                    <div class="flex gap-2">
                        <input
                            v-model="newFolderName"
                            type="text"
                            placeholder="New folder name"
                            class="flex-1 bg-muted border border-border rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                            @keyup.enter="createFolder"
                            autofocus
                        />
                        <button
                            @click="createFolder"
                            class="px-3 py-2 bg-primary text-primary-foreground rounded-lg text-sm hover:bg-primary/90"
                        >
                            <Check class="w-4 h-4" />
                        </button>
                        <button
                            @click="
                                showNewFolderInput = false;
                                newFolderName = '';
                            "
                            class="px-3 py-2 text-muted-foreground hover:text-foreground"
                        >
                            <X class="w-4 h-4" />
                        </button>
                    </div>
                </div>

                <!-- Footer -->
                <div
                    class="h-16 border-t border-border px-4 flex items-center justify-between flex-shrink-0"
                >
                    <button
                        @click="showNewFolderInput = true"
                        class="flex items-center gap-2 px-3 py-2 text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors"
                    >
                        <FolderPlus class="w-4 h-4" />
                        New Folder
                    </button>

                    <div class="flex items-center gap-2">
                        <button
                            @click="emit('close')"
                            class="px-4 py-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            @click="selectCurrentDirectory"
                            class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
                        >
                            {{ action === 'move' ? 'Move Here' : 'Copy Here' }}
                        </button>
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
