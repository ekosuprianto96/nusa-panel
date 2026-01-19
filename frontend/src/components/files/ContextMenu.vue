<script setup lang="ts">
/**
 * ContextMenu - Komponen Context Menu untuk File Manager
 *
 * Features:
 * - Posisi dinamis berdasarkan klik
 * - Menu items sesuai tipe file
 * - Keyboard navigation
 */
import { computed, ref, onMounted, onBeforeUnmount } from 'vue';
import type { FileInfo } from '@/types';
import {
    FolderOpen,
    Edit3,
    Type,
    Copy,
    Scissors,
    Clipboard,
    Archive,
    PackageOpen,
    Download,
    Lock,
    Trash2,
} from 'lucide-vue-next';

/** Props */
const props = defineProps<{
    visible: boolean;
    x: number;
    y: number;
    file: FileInfo | null;
    hasClipboard: boolean;
}>();

/** Emits */
const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'action', action: string): void;
}>();

/** Menu ref untuk positioning */
const menuRef = ref<HTMLElement | null>(null);

/** Adjusted position */
const position = computed(() => {
    let posX = props.x;
    let posY = props.y;

    // Adjust if menu goes off screen
    const menuWidth = 200;
    const menuHeight = 400;

    if (posX + menuWidth > window.innerWidth) {
        posX = window.innerWidth - menuWidth - 10;
    }
    if (posY + menuHeight > window.innerHeight) {
        posY = window.innerHeight - menuHeight - 10;
    }

    return { x: posX, y: posY };
});

/** Check if file is zip */
const isZipFile = computed(() => {
    if (!props.file) return false;
    return props.file.name.endsWith('.zip');
});

/** Check if file is editable */
const isEditable = computed(() => {
    if (!props.file || props.file.file_type === 'directory') return false;
    const ext = props.file.name.split('.').pop()?.toLowerCase();
    const editableExts = [
        'txt',
        'md',
        'html',
        'htm',
        'css',
        'js',
        'ts',
        'json',
        'xml',
        'yaml',
        'yml',
        'toml',
        'ini',
        'cfg',
        'conf',
        'log',
        'sh',
        'bash',
        'py',
        'rb',
        'php',
        'rs',
        'go',
        'java',
        'c',
        'cpp',
        'h',
        'vue',
        'jsx',
        'tsx',
        'env',
        'htaccess',
    ];
    return editableExts.includes(ext || '');
});

/** Menu items */
const menuItems = computed(() => {
    const items: Array<{
        id: string;
        label: string;
        icon: any;
        divider?: boolean;
        disabled?: boolean;
        danger?: boolean;
    }> = [];

    if (props.file?.file_type === 'directory') {
        items.push({ id: 'open', label: 'Open', icon: FolderOpen });
    } else if (isEditable.value) {
        items.push({ id: 'edit', label: 'Edit in Editor', icon: Edit3 });
    }

    items.push({ id: 'rename', label: 'Rename', icon: Type });
    items.push({ id: 'copy', label: 'Copy', icon: Copy });
    items.push({ id: 'cut', label: 'Cut', icon: Scissors });

    if (props.hasClipboard) {
        items.push({ id: 'paste', label: 'Paste', icon: Clipboard });
    }

    items.push({ id: 'divider1', label: '', icon: null, divider: true });

    items.push({ id: 'compress', label: 'Compress to Zip', icon: Archive });

    if (isZipFile.value) {
        items.push({ id: 'extract', label: 'Extract Here', icon: PackageOpen });
    }

    items.push({ id: 'divider2', label: '', icon: null, divider: true });

    if (props.file?.file_type !== 'directory') {
        items.push({ id: 'download', label: 'Download', icon: Download });
    }

    items.push({ id: 'permissions', label: 'Permissions', icon: Lock });

    items.push({ id: 'divider3', label: '', icon: null, divider: true });

    items.push({ id: 'delete', label: 'Delete', icon: Trash2, danger: true });

    return items;
});

/** Handle click outside */
const handleClickOutside = (event: MouseEvent) => {
    if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
        emit('close');
    }
};

/** Handle escape key */
const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
        emit('close');
    }
};

/** Handle menu item click */
const handleAction = (id: string) => {
    if (id.startsWith('divider')) return;
    emit('action', id);
    emit('close');
};

onMounted(() => {
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
});

onBeforeUnmount(() => {
    document.removeEventListener('mousedown', handleClickOutside);
    document.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
    <Teleport to="body">
        <div
            v-if="visible"
            ref="menuRef"
            class="fixed z-50 min-w-[180px] bg-card border border-border rounded-lg shadow-xl py-1 animate-in"
            :style="{ left: `${position.x}px`, top: `${position.y}px` }"
        >
            <template v-for="item in menuItems" :key="item.id">
                <!-- Divider -->
                <div v-if="item.divider" class="h-px bg-border my-1" />

                <!-- Menu Item -->
                <button
                    v-else
                    @click="handleAction(item.id)"
                    :disabled="item.disabled"
                    :class="[
                        'w-full px-3 py-2 text-sm text-left flex items-center gap-3 transition-colors',
                        item.danger
                            ? 'text-destructive hover:bg-destructive/10'
                            : 'text-foreground hover:bg-muted',
                        item.disabled && 'opacity-50 cursor-not-allowed',
                    ]"
                >
                    <component :is="item.icon" class="w-4 h-4" />
                    {{ item.label }}
                </button>
            </template>
        </div>
    </Teleport>
</template>

<style scoped>
.animate-in {
    animation: scaleIn 0.15s ease-out;
}

@keyframes scaleIn {
    from {
        opacity: 0;
        transform: scale(0.95);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
</style>
