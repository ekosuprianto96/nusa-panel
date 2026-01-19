<script setup lang="ts">
/**
 * UploadPage - Halaman Upload File dengan Progress Tracking
 *
 * Features:
 * - Drag & drop area
 * - Multiple file selection
 * - Progress bar per file
 * - Cancel upload capability
 */
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import FileManagerLayout from '@/layouts/FileManagerLayout.vue';
import { fileService } from '@/services/file.service';
import {
    Upload,
    File,
    CheckCircle,
    AlertCircle,
    ArrowLeft,
    FolderUp,
    Trash2,
} from 'lucide-vue-next';

/** Interface untuk upload item */
interface UploadItem {
    id: string;
    file: File | undefined;
    progress: number;
    status: 'pending' | 'uploading' | 'complete' | 'error';
    error?: string;
}

const route = useRoute();
const router = useRouter();

/** Target path dari query */
const targetPath = computed(() => (route.query.path as string) || '');

/** Files untuk upload */
const uploadItems = ref<UploadItem[]>([]);

/** Drag state */
const isDragging = ref(false);

/** Upload semua files */
const uploadAll = async (): Promise<void> => {
    const pendingItems = uploadItems.value.filter(
        (item) => item.status === 'pending',
    );

    for (const item of pendingItems) {
        item.status = 'uploading';
        try {
            await fileService.uploadWithProgress(
                item?.file || null,
                targetPath.value,
                (percent) => {
                    item.progress = percent;
                },
            );
            item.status = 'complete';
            item.progress = 100;
        } catch (err: any) {
            item.status = 'error';
            item.error = err.message || 'Upload gagal';
        }
    }
};

/** Add files ke list */
const addFiles = (files: FileList | null): void => {
    if (!files) return;

    for (let i = 0; i < files.length; i++) {
        const file = files[i];
        // Check 100MB limit
        if ((file?.size || 0) > 100 * 1024 * 1024) {
            uploadItems.value.push({
                id: crypto.randomUUID(),
                file: file || undefined,
                progress: 0,
                status: 'error',
                error: 'File terlalu besar (max 100MB)',
            });
        } else {
            uploadItems.value.push({
                id: crypto.randomUUID(),
                file: file || undefined,
                progress: 0,
                status: 'pending',
            });
        }
    }
};

/** Handle file input change */
const handleFileSelect = (event: Event): void => {
    const target = event.target as HTMLInputElement;
    addFiles(target.files);
    target.value = '';
};

/** Handle drag events */
const handleDragOver = (event: DragEvent): void => {
    event.preventDefault();
    isDragging.value = true;
};

const handleDragLeave = (): void => {
    isDragging.value = false;
};

const handleDrop = (event: DragEvent): void => {
    event.preventDefault();
    isDragging.value = false;
    addFiles(event.dataTransfer?.files || null);
};

/** Remove item from list */
const removeItem = (id: string): void => {
    uploadItems.value = uploadItems.value.filter((item) => item.id !== id);
};

/** Clear all completed */
const clearCompleted = (): void => {
    uploadItems.value = uploadItems.value.filter(
        (item) => item.status !== 'complete',
    );
};

/** Format file size */
const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

/** Go back to file manager */
const goBack = (): void => {
    router.push({
        path: '/dashboard/files',
        query: targetPath.value ? { path: targetPath.value } : undefined,
    });
};

/** Computed stats */
const stats = computed(() => {
    const total = uploadItems.value.length;
    const completed = uploadItems.value.filter(
        (i) => i.status === 'complete',
    ).length;
    const errors = uploadItems.value.filter((i) => i.status === 'error').length;
    const pending = uploadItems.value.filter(
        (i) => i.status === 'pending' || i.status === 'uploading',
    ).length;
    return { total, completed, errors, pending };
});
</script>

<template>
    <FileManagerLayout>
        <div class="flex flex-col h-full bg-background">
            <!-- Header -->
            <header
                class="h-14 border-b border-border bg-card px-6 flex items-center justify-between"
            >
                <div class="flex items-center gap-4">
                    <button
                        @click="goBack"
                        class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                    >
                        <ArrowLeft class="w-5 h-5" />
                    </button>
                    <div>
                        <h1 class="text-lg font-semibold text-foreground">
                            Upload Files
                        </h1>
                        <p class="text-sm text-muted-foreground">
                            Target: {{ targetPath || 'Home' }}
                        </p>
                    </div>
                </div>

                <div class="flex items-center gap-3">
                    <button
                        v-if="stats.completed > 0"
                        @click="clearCompleted"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Clear Completed
                    </button>
                    <button
                        v-if="stats.pending > 0"
                        @click="uploadAll"
                        class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors flex items-center gap-2"
                    >
                        <Upload class="w-4 h-4" />
                        Upload All ({{ stats.pending }})
                    </button>
                </div>
            </header>

            <!-- Content -->
            <div class="flex-1 p-6 overflow-auto">
                <!-- Drop Zone -->
                <div
                    @dragover="handleDragOver"
                    @dragleave="handleDragLeave"
                    @drop="handleDrop"
                    :class="[
                        'relative border-2 border-dashed rounded-xl p-12 text-center transition-all',
                        isDragging
                            ? 'border-primary bg-primary/5'
                            : 'border-border hover:border-primary/50 hover:bg-muted/30',
                    ]"
                >
                    <input
                        type="file"
                        multiple
                        @change="handleFileSelect"
                        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                    />
                    <FolderUp
                        :class="[
                            'w-16 h-16 mx-auto mb-4',
                            isDragging
                                ? 'text-primary'
                                : 'text-muted-foreground/30',
                        ]"
                    />
                    <h3 class="text-lg font-semibold text-foreground mb-2">
                        Drag & drop files here
                    </h3>
                    <p class="text-sm text-muted-foreground mb-4">
                        or click to browse files
                    </p>
                    <p class="text-xs text-muted-foreground">
                        Maximum file size: 100MB per file
                    </p>
                </div>

                <!-- File List -->
                <div v-if="uploadItems.length > 0" class="mt-6 space-y-3">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-sm font-semibold text-foreground">
                            Files ({{ stats.total }})
                        </h3>
                        <div
                            class="flex items-center gap-4 text-xs text-muted-foreground"
                        >
                            <span class="flex items-center gap-1">
                                <CheckCircle class="w-3 h-3 text-green-500" />
                                {{ stats.completed }} completed
                            </span>
                            <span
                                v-if="stats.errors > 0"
                                class="flex items-center gap-1"
                            >
                                <AlertCircle class="w-3 h-3 text-destructive" />
                                {{ stats.errors }} errors
                            </span>
                        </div>
                    </div>

                    <div
                        v-for="item in uploadItems"
                        :key="item.id"
                        class="bg-card border border-border rounded-lg p-4 flex items-center gap-4"
                    >
                        <!-- Icon -->
                        <div
                            :class="[
                                'w-10 h-10 rounded-lg flex items-center justify-center',
                                item.status === 'complete'
                                    ? 'bg-green-500/10 text-green-500'
                                    : item.status === 'error'
                                      ? 'bg-destructive/10 text-destructive'
                                      : 'bg-muted text-muted-foreground',
                            ]"
                        >
                            <component
                                :is="
                                    item.status === 'complete'
                                        ? CheckCircle
                                        : item.status === 'error'
                                          ? AlertCircle
                                          : File
                                "
                                class="w-5 h-5"
                            />
                        </div>

                        <!-- Info -->
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2">
                                <span
                                    class="text-sm font-medium text-foreground truncate"
                                >
                                    {{ item?.file?.name || 'Unknown' }}
                                </span>
                                <span class="text-xs text-muted-foreground">
                                    {{ formatSize(item?.file?.size || 0) }}
                                </span>
                            </div>

                            <!-- Progress Bar -->
                            <div
                                v-if="
                                    item.status === 'uploading' ||
                                    item.status === 'pending'
                                "
                                class="mt-2"
                            >
                                <div
                                    class="h-1.5 bg-muted rounded-full overflow-hidden"
                                >
                                    <div
                                        class="h-full bg-primary transition-all duration-300"
                                        :style="{ width: `${item.progress}%` }"
                                    />
                                </div>
                                <span
                                    class="text-xs text-muted-foreground mt-1"
                                >
                                    {{
                                        item.status === 'uploading'
                                            ? `${item.progress}%`
                                            : 'Waiting...'
                                    }}
                                </span>
                            </div>

                            <!-- Error -->
                            <p
                                v-if="item.error"
                                class="text-xs text-destructive mt-1"
                            >
                                {{ item.error }}
                            </p>
                        </div>

                        <!-- Remove Button -->
                        <button
                            v-if="item.status !== 'uploading'"
                            @click="removeItem(item.id)"
                            class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
                        >
                            <Trash2 class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </FileManagerLayout>
</template>
