<script setup lang="ts">
/**
 * FileManagerPage - Halaman File Manager standalone dengan semua fitur lengkap
 *
 * Features:
 * - Breadcrumb navigation
 * - Toolbar (New File, New Folder, Upload, Download, Delete, Permissions)
 * - File table dengan sorting
 * - Detail panel dengan file info dan actions
 * - Code editor untuk file text
 * - Context menu (right-click)
 * - Zip/Extract support
 */
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import FileManagerLayout from '@/layouts/FileManagerLayout.vue';
import AppBreadcrumb from '@/components/AppBreadcrumb.vue';
import ContextMenu from '@/components/files/ContextMenu.vue';
import DirectoryPickerModal from '@/components/files/DirectoryPickerModal.vue';
import { fileService } from '@/services';
import type { FileInfo } from '@/types';
import {
    Home,
    FilePlus,
    FolderPlus,
    Upload,
    Download,
    Trash2,
    Lock,
    Search,
    File,
    Folder,
    Image,
    FileCode,
    FileText,
    X,
    Edit3,
    RefreshCw,
    Eye,
    EyeOff,
    Clipboard,
    AlertCircle,
    Archive,
    PackageOpen,
    CheckSquare,
    Square,
    Move,
    Copy
} from 'lucide-vue-next';

// ==========================================
// STATE MANAGEMENT
// ==========================================

const route = useRoute();
const router = useRouter();

/** Current directory path */
const currentPath = ref<string>('');

/** List of files in current directory */
const files = ref<FileInfo[]>([]);

/** Currently selected file */
const selectedFile = ref<FileInfo | null>(null);

/** Loading state */
const isLoading = ref<boolean>(false);

/** Error message */
const error = ref<string>('');

/** Search query */
const searchQuery = ref<string>('');

/** Search results from API */
const searchResults = ref<FileInfo[]>([]);
const isSearching = ref(false);
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

/** Show hidden files */
const showHidden = ref<boolean>(false);

/** Modal states */
const showNewFileModal = ref<boolean>(false);
const showNewFolderModal = ref<boolean>(false);
const showRenameModal = ref<boolean>(false);
const showDeleteModal = ref<boolean>(false);
const showEditorModal = ref<boolean>(false);
const showPermissionsModal = ref<boolean>(false);

/** Form data */
const newFileName = ref<string>('');
const newFolderName = ref<string>('');
const renameValue = ref<string>('');
const editorContent = ref<string>('');
/** Permission value */
const permissionsValue = ref<string>('0644');

/** Context Menu state */
const showContextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuFile = ref<FileInfo | null>(null);

/** Zip modal state */
const showZipModal = ref(false);
const zipArchiveName = ref('');

/** Clipboard for copy/cut operations */
const clipboard = ref<{ file: FileInfo; action: 'copy' | 'cut' } | null>(null);

/** Multi-select state */
const selectedFiles = ref<FileInfo[]>([]);
const showMoveModal = ref(false);
const moveAction = ref<'move' | 'copy'>('move');

/** Drag & Drop state */
const draggedFile = ref<FileInfo | null>(null);
const isDragging = ref(false);
const dropTargetPath = ref<string | null>(null);

/** Sort config */
const sortBy = ref<'name' | 'size' | 'modified'>('name');
const sortDesc = ref<boolean>(false);

// ==========================================
// COMPUTED PROPERTIES
// ==========================================

/**
 * Breadcrumbs dari current path
 */
const breadcrumbs = computed(() => {
    if (!currentPath.value) return [];
    const parts = currentPath.value.split('/').filter(Boolean);
    let path = '';
    return parts.map((part) => {
        path += '/' + part;
        return { name: part, path };
    });
});

const breadcrumbItems = computed(() => {
    const items: { label: string; icon?: any; onClick: () => void; current?: boolean }[] = [
        { label: 'Home', icon: Home, onClick: () => navigateTo(''), current: breadcrumbs.value.length === 0 },
    ];
    breadcrumbs.value.forEach((crumb, idx) => {
        items.push({
            label: crumb.name,
            onClick: () => navigateTo(crumb.path),
            current: idx === breadcrumbs.value.length - 1,
        });
    });
    return items;
});

/**
 * Filtered dan sorted files
 * Jika ada search query, gunakan hasil dari API
 * Jika tidak, gunakan file list lokal
 */
const filteredFiles = computed(() => {
    // Jika ada hasil pencarian dari API, gunakan itu
    if (searchQuery.value && searchResults.value.length > 0) {
        return searchResults.value;
    }

    let result = [...files.value];

    // Filter lokal jika mengetik tapi belum ada hasil API
    if (
        searchQuery.value &&
        searchResults.value.length === 0 &&
        !isSearching.value
    ) {
        const query = searchQuery.value.toLowerCase();
        result = result.filter((f) => f.name.toLowerCase().includes(query));
    }

    // Sort
    result.sort((a, b) => {
        // Directories first
        if (a.file_type === 'directory' && b.file_type !== 'directory')
            return -1;
        if (a.file_type !== 'directory' && b.file_type === 'directory')
            return 1;

        let comparison = 0;
        switch (sortBy.value) {
            case 'name':
                comparison = a.name.localeCompare(b.name);
                break;
            case 'size':
                comparison = a.size - b.size;
                break;
            case 'modified':
                comparison =
                    new Date(a.modified_at).getTime() -
                    new Date(b.modified_at).getTime();
                break;
        }
        return sortDesc.value ? -comparison : comparison;
    });

    return result;
});

/**
 * Total size of all files
 */
const totalSize = computed(() => {
    const size = files.value.reduce((acc, f) => acc + f.size, 0);
    return formatSize(size);
});

// ==========================================
// API FUNCTIONS
// ==========================================

/**
 * Load files dari current directory
 */
const loadFiles = async (): Promise<void> => {
    try {
        isLoading.value = true;
        error.value = '';
        const response = await fileService.listFiles(
            currentPath.value,
            showHidden.value,
        );
        files.value = response.data.data.items || [];
        selectedFile.value = null;
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal memuat daftar file';
        files.value = [];
    } finally {
        isLoading.value = false;
    }
};

/**
 * Navigate ke folder
 */
const navigateTo = (path: string): void => {
    currentPath.value = path;
    router.replace({ query: { path: path || undefined } });
};

/**
 * Open file atau folder
 */
const openItem = (file: FileInfo): void => {
    if (file.file_type === 'directory') {
        navigateTo(file.path);
    } else {
        selectedFile.value = file;
    }
};

/**
 * Create new file
 */
const createFile = async (): Promise<void> => {
    if (!newFileName.value.trim()) return;
    try {
        isLoading.value = true;
        await fileService.create({
            path: currentPath.value,
            name: newFileName.value.trim(),
            file_type: 'file',
            content: '',
        });
        showNewFileModal.value = false;
        newFileName.value = '';
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal membuat file';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Create new folder
 */
const createFolder = async (): Promise<void> => {
    if (!newFolderName.value.trim()) return;
    try {
        isLoading.value = true;
        await fileService.create({
            path: currentPath.value,
            name: newFolderName.value.trim(),
            file_type: 'directory',
        });
        showNewFolderModal.value = false;
        newFolderName.value = '';
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal membuat folder';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Rename file/folder
 */
const renameItem = async (): Promise<void> => {
    if (!selectedFile.value || !renameValue.value.trim()) return;
    try {
        isLoading.value = true;
        await fileService.rename({
            path: selectedFile.value.path,
            new_name: renameValue.value.trim(),
        });
        showRenameModal.value = false;
        renameValue.value = '';
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal rename file';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Delete file/folder
 */
const deleteItem = async (): Promise<void> => {
    if (!selectedFile.value) return;
    try {
        isLoading.value = true;
        await fileService.delete({
            path: selectedFile.value.path,
            recursive: selectedFile.value.file_type === 'directory',
        });
        showDeleteModal.value = false;
        selectedFile.value = null;
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal menghapus file';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Open code editor - redirect to separate editor page
 */
const openEditor = (): void => {
    if (!selectedFile.value || selectedFile.value.file_type === 'directory')
        return;
    router.push({
        path: '/dashboard/files/editor',
        query: { path: selectedFile.value.path },
    });
};

/**
 * Save file content
 */
const saveFile = async (): Promise<void> => {
    if (!selectedFile.value) return;
    try {
        isLoading.value = true;
        await fileService.writeFile({
            path: selectedFile.value.path,
            content: editorContent.value,
        });
        showEditorModal.value = false;
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal menyimpan file';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Copy file - membuka modal untuk memilih folder tujuan
 */
const copyFile = (): void => {
    if (!selectedFile.value) return;
    // Tambahkan file ke selection jika belum ada
    if (!isFileSelected(selectedFile.value)) {
        selectedFiles.value = [selectedFile.value];
    }
    moveAction.value = 'copy';
    showMoveModal.value = true;
};

/**
 * Cut file - membuka modal untuk memilih folder tujuan
 */
const cutFile = (): void => {
    if (!selectedFile.value) return;
    // Tambahkan file ke selection jika belum ada
    if (!isFileSelected(selectedFile.value)) {
        selectedFiles.value = [selectedFile.value];
    }
    moveAction.value = 'move';
    showMoveModal.value = true;
};

/**
 * Paste from clipboard
 */
const pasteFile = async (): Promise<void> => {
    if (!clipboard.value) return;
    try {
        isLoading.value = true;
        const destination = currentPath.value
            ? `${currentPath.value}/${clipboard.value.file.name}`
            : clipboard.value.file.name;

        if (clipboard.value.action === 'copy') {
            await fileService.copy({
                source: clipboard.value.file.path,
                destination,
            });
        } else {
            await fileService.move({
                source: clipboard.value.file.path,
                destination,
            });
            clipboard.value = null;
        }
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal paste file';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Download file (simulasi - actual implementation needs backend endpoint)
 */
const downloadFile = (): void => {
    if (!selectedFile.value || selectedFile.value.file_type === 'directory')
        return;
    // For now, open in new tab - real implementation would use blob download
    window.open(
        `/api/files/download?path=${encodeURIComponent(selectedFile.value.path)}`,
        '_blank',
    );
};

/**
 * Open permissions modal
 */
const openPermissions = (): void => {
    if (!selectedFile.value) return;
    permissionsValue.value = selectedFile.value.permissions_octal || '0644';
    showPermissionsModal.value = true;
};

// ==========================================
// HELPER FUNCTIONS
// ==========================================

/**
 * Format file size
 */
const formatSize = (bytes: number): string => {
    if (bytes === 0) return '-';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

/**
 * Format date
 */
const formatDate = (date: string): string => {
    if (!date) return '-';
    return new Date(date).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: '2-digit',
    });
};

/**
 * Get file icon
 */
const getFileIcon = (file: FileInfo) => {
    if (file.file_type === 'directory') return Folder;
    const ext = file.name.split('.').pop()?.toLowerCase();
    if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp'].includes(ext || ''))
        return Image;
    if (
        [
            'js',
            'ts',
            'vue',
            'jsx',
            'tsx',
            'py',
            'php',
            'rb',
            'go',
            'rs',
        ].includes(ext || '')
    )
        return FileCode;
    if (['html', 'css', 'scss', 'json', 'xml', 'md', 'txt'].includes(ext || ''))
        return FileText;
    return File;
};

/**
 * Get file type label
 */
const getFileType = (file: FileInfo): string => {
    if (file.file_type === 'directory') return 'Folder';
    const ext = file.name.split('.').pop()?.toUpperCase();
    return ext ? `${ext} File` : 'File';
};

/**
 * Check if file is editable
 */
const isEditable = (file: FileInfo): boolean => {
    if (file.file_type === 'directory') return false;
    const ext = file.name.split('.').pop()?.toLowerCase();
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
        'zsh',
        'fish',
        'py',
        'rb',
        'php',
        'pl',
        'rs',
        'go',
        'java',
        'c',
        'cpp',
        'h',
        'hpp',
        'cs',
        'swift',
        'kt',
        'scala',
        'sql',
        'graphql',
        'vue',
        'jsx',
        'tsx',
        'svelte',
        'htaccess',
        'gitignore',
        'dockerfile',
        'makefile',
        'env',
    ];
    return editableExts.includes(ext || '') || file.name.startsWith('.');
};

/**
 * Toggle sort
 */
const toggleSort = (column: 'name' | 'size' | 'modified'): void => {
    if (sortBy.value === column) {
        sortDesc.value = !sortDesc.value;
    } else {
        sortBy.value = column;
        sortDesc.value = false;
    }
};

/**
 * Open rename modal
 */
const openRenameModal = (): void => {
    if (!selectedFile.value) return;
    renameValue.value = selectedFile.value.name;
    showRenameModal.value = true;
};

/**
 * Open delete modal
 */
const openDeleteModal = (): void => {
    if (!selectedFile.value) return;
    showDeleteModal.value = true;
};

// ==========================================
// ZIP & EXTRACT FUNCTIONS
// ==========================================

/**
 * Compress selected file to zip
 */
const compressFile = async (): Promise<void> => {
    if (!selectedFile.value) return;
    const archiveName = zipArchiveName.value || selectedFile.value.name;
    try {
        isLoading.value = true;
        await fileService.compress({
            paths: [selectedFile.value.path],
            archive_name: archiveName,
        });
        showZipModal.value = false;
        zipArchiveName.value = '';
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal membuat archive';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Extract zip file
 */
const extractFile = async (): Promise<void> => {
    if (!selectedFile.value || !selectedFile.value.name.endsWith('.zip'))
        return;
    try {
        isLoading.value = true;
        await fileService.extract({
            archive_path: selectedFile.value.path,
            destination: currentPath.value || '',
            overwrite: false,
        });
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal extract archive';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Open zip modal
 */
const openZipModal = (): void => {
    if (!selectedFile.value) return;
    zipArchiveName.value = selectedFile.value.name.replace(/\.[^/.]+$/, '');
    showZipModal.value = true;
};

// ==========================================
// MULTI-SELECT FUNCTIONS
// ==========================================

/**
 * Check if file is selected
 */
const isFileSelected = (file: FileInfo | null | undefined): boolean => {
    if(!file) return false;
    return selectedFiles.value.some(f => f.path === file.path);
};

/**
 * Toggle file selection dengan support Ctrl+Click dan Shift+Click
 * @param file File yang di-click
 * @param event MouseEvent untuk cek modifier keys
 */
const toggleFileSelection = (file: FileInfo, event?: MouseEvent): void => {
    const index = selectedFiles.value.findIndex(f => f.path === file.path);
    
    if (event?.shiftKey && selectedFiles.value.length > 0) {
        // Shift+Click: range select
        const lastSelected = selectedFiles.value[selectedFiles.value.length - 1];
        if (!lastSelected) return;
        const files = filteredFiles.value;
        const lastIndex = files.findIndex(f => f.path === lastSelected.path);
        const currentIndex = files.findIndex(f => f.path === file.path);
        const [start, end] = lastIndex < currentIndex ? [lastIndex, currentIndex] : [currentIndex, lastIndex];
        
        for (let i = start; i <= end; i++) {
            const fileToAdd = files[i];
            if (fileToAdd && !isFileSelected(fileToAdd)) {
                selectedFiles.value.push(fileToAdd);
            }
        }
    } else if (event?.ctrlKey || event?.metaKey) {
        // Ctrl+Click (atau Cmd+Click di Mac): toggle individual selection
        if (index === -1) {
            selectedFiles.value.push(file);
        } else {
            selectedFiles.value.splice(index, 1);
        }
    } else {
        // Regular click: toggle single file
        if (index === -1) {
            selectedFiles.value.push(file);
        } else {
            selectedFiles.value.splice(index, 1);
        }
    }
};

/**
 * Handle row click - select file dengan support Ctrl+Click untuk multi-select
 * @param file File yang di-click
 * @param event MouseEvent untuk cek modifier keys
 */
const handleRowClick = (file: FileInfo, event: MouseEvent): void => {
    if (event.ctrlKey || event.metaKey) {
        // Ctrl+Click: toggle selection
        toggleFileSelection(file, event);
    } else if (event.shiftKey && selectedFiles.value.length > 0) {
        // Shift+Click: range select
        toggleFileSelection(file, event);
    } else {
        // Regular click: just select this file (single selection)
        selectedFile.value = file;
    }
};

/**
 * Select all files
 */
const selectAllFiles = (): void => {
    if (selectedFiles.value.length === filteredFiles.value.length) {
        selectedFiles.value = [];
    } else {
        selectedFiles.value = [...filteredFiles.value];
    }
};

/**
 * Clear selection
 */
const clearSelection = (): void => {
    selectedFiles.value = [];
};

/**
 * Open move modal for batch move
 */
const openBatchMoveModal = (action: 'move' | 'copy'): void => {
    if (selectedFiles.value.length === 0) return;
    moveAction.value = action;
    showMoveModal.value = true;
};

/**
 * Handle batch move/copy
 */
const handleBatchMove = async (destinationPath: string): Promise<void> => {
    if (selectedFiles.value.length === 0) return;
    
    try {
        isLoading.value = true;
        error.value = '';
        
        for (const file of selectedFiles.value) {
            const newPath = destinationPath 
                ? `${destinationPath}/${file.name}`
                : file.name;
            
            if (moveAction.value === 'move') {
                await fileService.move({
                    source: file.path,
                    destination: newPath,
                });
            } else {
                await fileService.copy({
                    source: file.path,
                    destination: newPath,
                });
            }
        }
        
        showMoveModal.value = false;
        clearSelection();
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || `Gagal ${moveAction.value} files`;
    } finally {
        isLoading.value = false;
    }
};

/**
 * Batch delete selected files
 */
const batchDelete = async (): Promise<void> => {
    if (selectedFiles.value.length === 0) return;
    
    const confirmMsg = `Hapus ${selectedFiles.value.length} item? Aksi ini tidak bisa dibatalkan.`;
    if (!confirm(confirmMsg)) return;
    
    try {
        isLoading.value = true;
        error.value = '';
        
        for (const file of selectedFiles.value) {
            await fileService.delete({
                path: file.path,
                recursive: file.file_type === 'directory',
            });
        }
        
        clearSelection();
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal menghapus files';
    } finally {
        isLoading.value = false;
    }
};

/**
 * Batch zip selected files
 */
const batchZip = async (): Promise<void> => {
    if (selectedFiles.value.length === 0) return;
    
    const archiveName = prompt('Nama archive (tanpa .zip):', 'archive');
    if (!archiveName) return;
    
    try {
        isLoading.value = true;
        error.value = '';
        
        await fileService.compress({
            paths: selectedFiles.value.map(f => f.path),
            archive_name: archiveName,
        });
        
        clearSelection();
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal membuat archive';
    } finally {
        isLoading.value = false;
    }
};

// ==========================================
// DRAG & DROP FUNCTIONS
// ==========================================

/**
 * Handle drag start - mulai drag file/folder
 * @param event DragEvent
 * @param file File yang di-drag
 */
const handleDragStart = (event: DragEvent, file: FileInfo): void => {
    draggedFile.value = file;
    isDragging.value = true;
    
    // Jika file yang di-drag adalah bagian dari selection, drag semua yang terseleksi
    // Jika tidak, hanya drag file itu saja
    if (!isFileSelected(file)) {
        selectedFiles.value = [file];
    }
    
    // Set data untuk drag
    if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('text/plain', JSON.stringify(selectedFiles.value.map(f => f.path)));
    }
};

/**
 * Handle drag end - selesai drag
 */
const handleDragEnd = (): void => {
    draggedFile.value = null;
    isDragging.value = false;
    dropTargetPath.value = null;
};

/**
 * Handle drag over - file sedang di-drag di atas target
 * @param event DragEvent
 * @param targetFile Target folder
 */
const handleDragOver = (event: DragEvent, targetFile?: FileInfo): void => {
    event.preventDefault();
    
    // Hanya izinkan drop ke folder atau ke current directory (jika targetFile undefined)
    if (!targetFile || targetFile.file_type === 'directory') {
        if (event.dataTransfer) {
            event.dataTransfer.dropEffect = 'move';
        }
        dropTargetPath.value = targetFile?.path || currentPath.value || '';
    }
};

/**
 * Handle drag leave - file keluar dari area drop
 */
const handleDragLeave = (): void => {
    dropTargetPath.value = null;
};

/**
 * Handle drop - file di-drop ke folder
 * @param event DragEvent
 * @param targetFile Target folder (null untuk current directory)
 */
const handleDrop = async (event: DragEvent, targetFile?: FileInfo): Promise<void> => {
    event.preventDefault();
    
    // Pastikan drop ke folder atau parent area
    if (targetFile && targetFile.file_type !== 'directory') {
        handleDragEnd();
        return;
    }
    
    const targetPath = targetFile?.path || currentPath.value || '';
    
    // Jangan izinkan drop ke folder yang sama atau subfolder dari yang di-drag
    const filesToMove = selectedFiles.value;
    const invalidDrop = filesToMove.some(f => {
        // Tidak bisa drop ke diri sendiri
        if (f.path === targetPath) return true;
        // Tidak bisa drop folder ke dalam subfolder-nya sendiri
        if (targetPath.startsWith(f.path + '/')) return true;
        return false;
    });
    
    if (invalidDrop) {
        error.value = 'Tidak bisa memindahkan ke lokasi tersebut';
        handleDragEnd();
        return;
    }
    
    try {
        isLoading.value = true;
        error.value = '';
        
        for (const file of filesToMove) {
            const newPath = targetPath ? `${targetPath}/${file.name}` : file.name;
            
            // Skip jika source = destination
            if (file.path === newPath) continue;
            
            await fileService.move({
                source: file.path,
                destination: newPath,
            });
        }
        
        clearSelection();
        await loadFiles();
    } catch (err: any) {
        error.value = err.response?.data?.message || 'Gagal memindahkan file';
    } finally {
        isLoading.value = false;
        handleDragEnd();
    }
};

/**
 * Check if this folder is currently a drop target
 * @param file FileInfo to check
 */
const isDropTarget = (file?: FileInfo): boolean => {
    if (!isDragging.value) return false;
    const targetPath = file?.path || currentPath.value || '';
    return dropTargetPath.value === targetPath;
};

// ==========================================
// CONTEXT MENU FUNCTIONS
// ==========================================

/**
 * Handle right click on file
 */
const handleContextMenu = (event: MouseEvent, file: FileInfo): void => {
    event.preventDefault();
    selectedFile.value = file;
    contextMenuFile.value = file;
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenu.value = true;
};

/**
 * Handle context menu action
 */
const handleContextAction = (action: string): void => {
    switch (action) {
        case 'open':
            if (contextMenuFile.value) openItem(contextMenuFile.value);
            break;
        case 'edit':
            openEditor();
            break;
        case 'rename':
            openRenameModal();
            break;
        case 'copy':
            copyFile();
            break;
        case 'cut':
            cutFile();
            break;
        case 'paste':
            pasteFile();
            break;
        case 'compress':
            openZipModal();
            break;
        case 'extract':
            extractFile();
            break;
        case 'download':
            downloadFile();
            break;
        case 'permissions':
            openPermissions();
            break;
        case 'delete':
            openDeleteModal();
            break;
    }
};

// ==========================================
// LIFECYCLE
// ==========================================

onMounted(() => {
    // Get path from query
    const pathParam = route.query.path as string;
    if (pathParam) {
        currentPath.value = pathParam;
    }
    loadFiles();
});

// Watch path changes
watch(currentPath, () => {
    loadFiles();
    clearSelection();  // Clear selection saat pindah folder
});

// Watch hidden files toggle
watch(showHidden, () => {
    loadFiles();
});

// Watch search query dengan debounce
watch(searchQuery, (newQuery) => {
    // Clear timeout sebelumnya
    if (searchTimeout) {
        clearTimeout(searchTimeout);
    }

    // Jika query kosong, reset results
    if (!newQuery.trim()) {
        searchResults.value = [];
        isSearching.value = false;
        return;
    }

    // Debounce 300ms
    searchTimeout = setTimeout(async () => {
        try {
            isSearching.value = true;
            const response = await fileService.search(
                newQuery,
                currentPath.value || undefined,
                50,
            );
            searchResults.value = response.data.data || [];
        } catch (err: any) {
            console.error('Search error:', err);
            searchResults.value = [];
        } finally {
            isSearching.value = false;
        }
    }, 300);
});

// ==========================================
// UPLOAD HANDLING
// ==========================================

// const fileInput = ref<HTMLInputElement | null>(null)

/**
 * Redirect to upload page instead of simple upload
 */
const triggerUpload = (): void => {
    router.push({
        path: '/dashboard/files/upload',
        query: currentPath.value ? { path: currentPath.value } : undefined,
    });
};

const handleFileUpload = async (event: Event): Promise<void> => {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    // Limit 100MB (matches backend)
    if (file.size > 100 * 1024 * 1024) {
        error.value = 'File terlalu besar (maksimal 100MB)';
        target.value = '';
        return;
    }

    const reader = new FileReader();
    reader.onload = async (e) => {
        try {
            isLoading.value = true;
            const result = e.target?.result as string;
            // Extract base64 content (remove "data:mime/type;base64," prefix)
            const content = result.split(',')[1];

            await fileService.writeFile({
                path: currentPath.value
                    ? `${currentPath.value}/${file.name}`
                    : file.name,
                content: content,
                encoding: 'base64',
                create_if_not_exists: true,
            });

            await loadFiles();
            target.value = ''; // Reset input
        } catch (err: any) {
            error.value =
                err.response?.data?.message || 'Gagal mengupload file';
        } finally {
            isLoading.value = false;
        }
    };
    reader.onerror = () => {
        error.value = 'Gagal membaca file browser';
    };
    reader.readAsDataURL(file);
};
</script>

<template>
    <FileManagerLayout>
        <div class="flex h-full">
            <!-- Main File Browser -->
            <div class="flex-1 flex flex-col min-w-0">
                <!-- Hidden File Input -->
                <input
                    type="file"
                    ref="fileInput"
                    class="hidden"
                    @change="handleFileUpload"
                />

                <header
                    class="h-14 border-b border-border bg-card px-4 flex items-center justify-between gap-4 transition-colors"
                >
                    <!-- Breadcrumb -->
                    <AppBreadcrumb :items="breadcrumbItems" />

                    <!-- Search -->
                    <div class="relative w-64">
                        <Search
                            class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                        />
                        <input
                            v-model="searchQuery"
                            type="text"
                            placeholder="Search in files..."
                            class="w-full bg-muted border border-border rounded-lg pl-9 pr-4 py-2 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none transition-all"
                        />
                    </div>
                </header>

                <!-- Toolbar -->
                <div
                    class="h-12 border-b border-border bg-card px-4 flex items-center gap-2 transition-colors relative"
                >
                    <!-- <router-link
                        to="/dashboard/files-v2"
                        class="hidden md:flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-primary hover:bg-primary/10 rounded-lg transition-colors absolute right-4"
                    >
                        <Sparkles class="w-4 h-4" />
                        <span>Try New File Manager</span>
                    </router-link> -->
                    <button
                        @click="showNewFileModal = true"
                        class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                    >
                        <FilePlus class="w-4 h-4" />
                        <span class="hidden sm:inline">New File</span>
                    </button>
                    <button
                        @click="showNewFolderModal = true"
                        class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                    >
                        <FolderPlus class="w-4 h-4" />
                        <span class="hidden sm:inline">New Folder</span>
                    </button>

                    <div class="w-px h-6 bg-border mx-1" />

                    <button
                        @click="triggerUpload"
                        class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                        title="Upload Files"
                    >
                        <Upload class="w-4 h-4" />
                        <span class="hidden sm:inline">Upload</span>
                    </button>
                    <button
                        @click="downloadFile"
                        :disabled="
                            !selectedFile ||
                            selectedFile.file_type === 'directory'
                        "
                        :class="[
                            'flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                            selectedFile &&
                            selectedFile.file_type !== 'directory'
                                ? 'text-foreground hover:bg-muted'
                                : 'text-muted-foreground cursor-not-allowed',
                        ]"
                    >
                        <Download class="w-4 h-4" />
                        <span class="hidden sm:inline">Download</span>
                    </button>

                    <div class="w-px h-6 bg-border mx-1" />

                    <button
                        @click="openDeleteModal"
                        :disabled="!selectedFile"
                        :class="[
                            'flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                            selectedFile
                                ? 'text-destructive hover:bg-destructive/10'
                                : 'text-muted-foreground cursor-not-allowed',
                        ]"
                    >
                        <Trash2 class="w-4 h-4" />
                        <span class="hidden sm:inline">Delete</span>
                    </button>
                    <button
                        @click="openPermissions"
                        :disabled="!selectedFile"
                        :class="[
                            'flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                            selectedFile
                                ? 'text-foreground hover:bg-muted'
                                : 'text-muted-foreground cursor-not-allowed',
                        ]"
                    >
                        <Lock class="w-4 h-4" />
                        <span class="hidden sm:inline">Permissions</span>
                    </button>

                    <div class="w-px h-6 bg-border mx-1" />

                    <!-- Zip/Extract buttons -->
                    <button
                        @click="openZipModal"
                        :disabled="!selectedFile"
                        :class="[
                            'flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                            selectedFile
                                ? 'text-foreground hover:bg-muted'
                                : 'text-muted-foreground cursor-not-allowed',
                        ]"
                        title="Compress to Zip"
                    >
                        <Archive class="w-4 h-4" />
                        <span class="hidden sm:inline">Zip</span>
                    </button>
                    <button
                        @click="extractFile"
                        :disabled="
                            !selectedFile || !selectedFile.name.endsWith('.zip')
                        "
                        :class="[
                            'flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-lg transition-colors',
                            selectedFile && selectedFile.name.endsWith('.zip')
                                ? 'text-foreground hover:bg-muted'
                                : 'text-muted-foreground cursor-not-allowed',
                        ]"
                        title="Extract Archive"
                    >
                        <PackageOpen class="w-4 h-4" />
                        <span class="hidden sm:inline">Extract</span>
                    </button>

                    <div class="flex-1" />

                    <!-- Toggle hidden files -->
                    <button
                        @click="showHidden = !showHidden"
                        :class="[
                            'p-2 rounded-lg transition-colors',
                            showHidden
                                ? 'bg-primary/10 text-primary'
                                : 'text-muted-foreground hover:bg-muted',
                        ]"
                        :title="
                            showHidden
                                ? 'Hide hidden files'
                                : 'Show hidden files'
                        "
                    >
                        <component
                            :is="showHidden ? Eye : EyeOff"
                            class="w-4 h-4"
                        />
                    </button>

                    <!-- Refresh -->
                    <button
                        @click="loadFiles"
                        class="p-2 rounded-lg text-muted-foreground hover:bg-muted transition-colors"
                        title="Refresh"
                    >
                        <RefreshCw
                            class="w-4 h-4"
                            :class="{ 'animate-spin': isLoading }"
                        />
                    </button>
                </div>

                <!-- File Table -->
                <div class="flex-1 overflow-auto">
                    <!-- Error message -->
                    <div
                        v-if="error"
                        class="m-4 p-4 bg-destructive/10 border border-destructive/20 rounded-lg flex items-center gap-3 text-destructive"
                    >
                        <AlertCircle class="w-5 h-5 flex-shrink-0" />
                        <span class="text-sm">{{ error }}</span>
                        <button
                            @click="error = ''"
                            class="ml-auto p-1 hover:bg-destructive/10 rounded"
                        </button>
                    </div>

                    <!-- Batch Operations Toolbar -->
                    <div 
                        v-if="selectedFiles.length > 0"
                        class="mx-4 mt-3 p-3 bg-primary/5 border border-primary/20 rounded-lg flex items-center gap-3"
                    >
                        <span class="text-sm font-medium text-foreground">
                            {{ selectedFiles.length }} item{{ selectedFiles.length > 1 ? 's' : '' }} selected
                        </span>
                        <div class="flex-1" />
                        <button
                            @click="openBatchMoveModal('move')"
                            class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            <Move class="w-4 h-4" />
                            Move
                        </button>
                        <button
                            @click="openBatchMoveModal('copy')"
                            class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            <Copy class="w-4 h-4" />
                            Copy
                        </button>
                        <button
                            @click="batchZip"
                            class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            <Archive class="w-4 h-4" />
                            Zip
                        </button>
                        <button
                            @click="batchDelete"
                            class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
                        >
                            <Trash2 class="w-4 h-4" />
                            Delete
                        </button>
                        <button
                            @click="clearSelection"
                            class="p-1.5 text-muted-foreground hover:text-foreground rounded transition-colors"
                            title="Clear selection"
                        >
                            <X class="w-4 h-4" />
                        </button>
                    </div>

                    <!-- Table -->
                    <table class="w-full">
                        <thead
                            class="sticky top-0 bg-muted/50 backdrop-blur-sm z-10"
                        >
                            <tr
                                class="text-left text-xs text-muted-foreground font-medium uppercase tracking-wider"
                            >
                                <!-- Checkbox column -->
                                <th class="px-4 py-3 w-10">
                                    <button
                                        @click="selectAllFiles"
                                        class="p-1 hover:bg-muted rounded transition-colors"
                                    >
                                        <component 
                                            :is="selectedFiles.length === filteredFiles.length && filteredFiles.length > 0 ? CheckSquare : Square" 
                                            class="w-4 h-4"
                                            :class="selectedFiles.length > 0 ? 'text-primary' : 'text-muted-foreground'"
                                        />
                                    </button>
                                </th>
                                <th
                                    @click="toggleSort('name')"
                                    class="px-4 py-3 cursor-pointer hover:text-foreground transition-colors"
                                >
                                    <div class="flex items-center gap-2">
                                        Name
                                        <span
                                            v-if="sortBy === 'name'"
                                            class="text-primary"
                                            >{{ sortDesc ? '↓' : '↑' }}</span
                                        >
                                    </div>
                                </th>
                                <th
                                    @click="toggleSort('size')"
                                    class="px-4 py-3 cursor-pointer hover:text-foreground transition-colors w-24"
                                >
                                    <div class="flex items-center gap-2">
                                        Size
                                        <span
                                            v-if="sortBy === 'size'"
                                            class="text-primary"
                                            >{{ sortDesc ? '↓' : '↑' }}</span
                                        >
                                    </div>
                                </th>
                                <th
                                    @click="toggleSort('modified')"
                                    class="px-4 py-3 cursor-pointer hover:text-foreground transition-colors w-36"
                                >
                                    <div class="flex items-center gap-2">
                                        Last Modified
                                        <span
                                            v-if="sortBy === 'modified'"
                                            class="text-primary"
                                            >{{ sortDesc ? '↓' : '↑' }}</span
                                        >
                                    </div>
                                </th>
                                <th class="px-4 py-3 w-28">Permissions</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            <!-- Parent directory -->
                            <tr
                                v-if="currentPath"
                                @click="
                                    navigateTo(
                                        currentPath
                                            .split('/')
                                            .slice(0, -1)
                                            .join('/'),
                                    )
                                "
                                class="hover:bg-muted/50 cursor-pointer transition-colors"
                            >
                                <!-- Empty checkbox cell for alignment -->
                                <td class="px-4 py-3 w-10"></td>
                                <td class="px-4 py-3">
                                    <div class="flex items-center gap-3">
                                        <Folder class="w-5 h-5 text-primary" />
                                        <span
                                            class="text-sm font-medium text-foreground"
                                            >..</span
                                        >
                                    </div>
                                </td>
                                <td
                                    class="px-4 py-3 text-sm text-muted-foreground"
                                >
                                    -
                                </td>
                                <td
                                    class="px-4 py-3 text-sm text-muted-foreground"
                                >
                                    -
                                </td>
                                <td
                                    class="px-4 py-3 text-sm text-muted-foreground"
                                >
                                    -
                                </td>
                            </tr>

                            <!-- Files -->
                            <tr
                                v-for="file in filteredFiles"
                                :key="file.path"
                                draggable="true"
                                @click="handleRowClick(file, $event)"
                                @dblclick="openItem(file)"
                                @contextmenu="handleContextMenu($event, file)"
                                @dragstart="handleDragStart($event, file)"
                                @dragend="handleDragEnd"
                                @dragover="file.file_type === 'directory' ? handleDragOver($event, file) : undefined"
                                @dragleave="handleDragLeave"
                                @drop="file.file_type === 'directory' ? handleDrop($event, file) : undefined"
                                :class="[
                                    'cursor-pointer transition-all duration-150',
                                    isDropTarget(file) && file.file_type === 'directory'
                                        ? 'bg-primary/30 ring-2 ring-primary ring-inset'
                                        : isFileSelected(file)
                                        ? 'bg-primary/15'
                                        : selectedFile?.path === file.path
                                        ? 'bg-primary/10'
                                        : 'hover:bg-muted/50',
                                    isDragging && draggedFile?.path === file.path ? 'opacity-50' : '',
                                ]"
                            >
                                <!-- Checkbox -->
                                <td class="px-4 py-3 w-10">
                                    <button
                                        @click.stop="toggleFileSelection(file, $event)"
                                        class="p-1 hover:bg-muted rounded transition-colors"
                                    >
                                        <component 
                                            :is="isFileSelected(file) ? CheckSquare : Square" 
                                            class="w-4 h-4"
                                            :class="isFileSelected(file) ? 'text-primary' : 'text-muted-foreground'"
                                        />
                                    </button>
                                </td>
                                <td class="px-4 py-3">
                                    <div class="flex items-center gap-3">
                                        <component
                                            :is="getFileIcon(file)"
                                            :class="[
                                                'w-5 h-5',
                                                file.file_type === 'directory'
                                                    ? 'text-primary'
                                                    : 'text-muted-foreground',
                                            ]"
                                        />
                                        <span
                                            class="text-sm font-medium text-foreground"
                                            >{{ file.name }}</span
                                        >
                                    </div>
                                </td>
                                <td
                                    class="px-4 py-3 text-sm text-muted-foreground"
                                >
                                    {{
                                        file.file_type === 'directory'
                                            ? '-'
                                            : formatSize(file.size)
                                    }}
                                </td>
                                <td
                                    class="px-4 py-3 text-sm text-muted-foreground"
                                >
                                    {{ formatDate(file.modified_at) }}
                                </td>
                                <td class="px-4 py-3">
                                    <span
                                        class="text-xs font-mono text-primary bg-primary/10 px-2 py-0.5 rounded"
                                    >
                                        {{ file.permissions_octal || '0644' }}
                                    </span>
                                </td>
                            </tr>

                            <!-- Empty state -->
                            <tr
                                v-if="
                                    !isLoading &&
                                    filteredFiles.length === 0 &&
                                    !currentPath
                                "
                            >
                                <td colspan="4" class="px-4 py-12 text-center">
                                    <Folder
                                        class="w-12 h-12 text-muted-foreground/30 mx-auto mb-3"
                                    />
                                    <p class="text-muted-foreground">
                                        This folder is empty
                                    </p>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <!-- Footer -->
                <footer
                    class="h-10 border-t border-border bg-card px-4 flex items-center justify-between text-xs text-muted-foreground transition-colors"
                >
                    <span>{{ filteredFiles.length }} items shown</span>
                    <span>Total size: {{ totalSize }}</span>
                </footer>
            </div>

            <!-- Detail Panel -->
            <aside
                v-if="selectedFile"
                class="w-72 border-l border-border bg-card flex flex-col transition-colors"
            >
                <!-- Panel Header -->
                <div
                    class="h-14 border-b border-border px-4 flex items-center justify-between"
                >
                    <span class="text-sm font-semibold text-foreground"
                        >File Details</span
                    >
                    <button
                        @click="selectedFile = null"
                        class="p-1 rounded hover:bg-muted transition-colors"
                    >
                        <X class="w-4 h-4 text-muted-foreground" />
                    </button>
                </div>

                <!-- File Preview -->
                <div
                    class="p-6 flex flex-col items-center border-b border-border"
                >
                    <div
                        class="w-20 h-20 bg-muted rounded-xl flex items-center justify-center mb-4"
                    >
                        <component
                            :is="getFileIcon(selectedFile)"
                            :class="[
                                'w-10 h-10',
                                selectedFile.file_type === 'directory'
                                    ? 'text-primary'
                                    : 'text-muted-foreground',
                            ]"
                        />
                    </div>
                    <h3
                        class="text-sm font-semibold text-foreground text-center break-all"
                    >
                        {{ selectedFile.name }}
                    </h3>
                    <p class="text-xs text-muted-foreground mt-1">
                        {{ getFileType(selectedFile) }}
                    </p>
                </div>

                <!-- File Info -->
                <div class="flex-1 p-4 space-y-4 overflow-auto">
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Size</span>
                        <span class="text-foreground font-medium">{{
                            formatSize(selectedFile.size)
                        }}</span>
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Permissions</span>
                        <span class="font-mono text-primary">{{
                            selectedFile.permissions_octal || '0644'
                        }}</span>
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Full Path</span>
                        <span
                            class="text-foreground text-right text-xs break-all max-w-[150px]"
                            >{{ selectedFile.path }}</span
                        >
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Modified</span>
                        <span class="text-foreground">{{
                            formatDate(selectedFile.modified_at)
                        }}</span>
                    </div>
                </div>

                <!-- Actions -->
                <div class="p-4 border-t border-border space-y-2">
                    <button
                        v-if="isEditable(selectedFile)"
                        @click="openEditor"
                        class="w-full py-2.5 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors flex items-center justify-center gap-2"
                    >
                        <Edit3 class="w-4 h-4" />
                        Edit in Code Editor
                    </button>
                    <div class="flex gap-2">
                        <button
                            @click="downloadFile"
                            :disabled="selectedFile.file_type === 'directory'"
                            :class="[
                                'flex-1 py-2 px-3 border border-border rounded-lg text-sm font-medium transition-colors flex items-center justify-center gap-2',
                                selectedFile.file_type !== 'directory'
                                    ? 'text-foreground hover:bg-muted'
                                    : 'text-muted-foreground cursor-not-allowed',
                            ]"
                        >
                            <Download class="w-4 h-4" />
                            Download
                        </button>
                        <button
                            @click="openDeleteModal"
                            class="flex-1 py-2 px-3 border border-destructive/30 text-destructive rounded-lg text-sm font-medium hover:bg-destructive/10 transition-colors flex items-center justify-center gap-2"
                        >
                            <Trash2 class="w-4 h-4" />
                            Delete
                        </button>
                    </div>

                    <!-- More actions -->
                    <div class="flex gap-2 pt-2">
                        <button
                            @click="openRenameModal"
                            class="flex-1 py-2 px-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            Rename
                        </button>
                        <button
                            @click="copyFile"
                            class="flex-1 py-2 px-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            Copy
                        </button>
                        <button
                            @click="cutFile"
                            class="flex-1 py-2 px-3 text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors"
                        >
                            Cut
                        </button>
                    </div>
                </div>
            </aside>

            <!-- Empty state when no file selected -->
            <aside
                v-else
                class="w-72 border-l border-border bg-card flex flex-col items-center justify-center text-center p-6 transition-colors"
            >
                <File class="w-12 h-12 text-muted-foreground/30 mb-3" />
                <p class="text-sm text-muted-foreground">
                    Select a file to view details
                </p>
            </aside>
        </div>

        <!-- Clipboard indicator -->
        <div
            v-if="clipboard"
            class="fixed bottom-4 left-20 bg-card border border-border shadow-lg rounded-lg px-4 py-3 flex items-center gap-3"
        >
            <Clipboard class="w-4 h-4 text-primary" />
            <span class="text-sm text-foreground"
                >{{ clipboard.file.name }} ({{ clipboard.action }})</span
            >
            <button
                @click="pasteFile"
                class="px-3 py-1 text-sm font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors"
            >
                Paste Here
            </button>
            <button
                @click="clipboard = null"
                class="p-1 hover:bg-muted rounded transition-colors"
            >
                <X class="w-4 h-4 text-muted-foreground" />
            </button>
        </div>

        <!-- ==================== MODALS ==================== -->

        <!-- New File Modal -->
        <div
            v-if="showNewFileModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showNewFileModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-4">
                    Create New File
                </h3>
                <input
                    v-model="newFileName"
                    type="text"
                    placeholder="filename.txt"
                    class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    @keyup.enter="createFile"
                    autofocus
                />
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showNewFileModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="createFile"
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Create
                    </button>
                </div>
            </div>
        </div>

        <!-- New Folder Modal -->
        <div
            v-if="showNewFolderModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showNewFolderModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-4">
                    Create New Folder
                </h3>
                <input
                    v-model="newFolderName"
                    type="text"
                    placeholder="folder_name"
                    class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    @keyup.enter="createFolder"
                    autofocus
                />
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showNewFolderModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="createFolder"
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Create
                    </button>
                </div>
            </div>
        </div>

        <!-- Rename Modal -->
        <div
            v-if="showRenameModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showRenameModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-4">
                    Rename
                </h3>
                <input
                    v-model="renameValue"
                    type="text"
                    class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    @keyup.enter="renameItem"
                    autofocus
                />
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showRenameModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="renameItem"
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Rename
                    </button>
                </div>
            </div>
        </div>

        <!-- Delete Confirmation Modal -->
        <div
            v-if="showDeleteModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showDeleteModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-2">
                    Delete
                    {{
                        selectedFile?.file_type === 'directory'
                            ? 'Folder'
                            : 'File'
                    }}
                </h3>
                <p class="text-sm text-muted-foreground mb-6">
                    Are you sure you want to delete
                    <span class="font-semibold text-foreground">{{
                        selectedFile?.name
                    }}</span
                    >? This action cannot be undone.
                </p>
                <div class="flex justify-end gap-3">
                    <button
                        @click="showDeleteModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="deleteItem"
                        class="px-4 py-2 text-sm font-medium bg-destructive text-destructive-foreground rounded-lg hover:bg-destructive/90 transition-colors"
                    >
                        Delete
                    </button>
                </div>
            </div>
        </div>

        <!-- Code Editor Modal -->
        <div
            v-if="showEditorModal"
            class="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-5xl h-[80vh] flex flex-col animate-in"
            >
                <div
                    class="h-14 border-b border-border px-6 flex items-center justify-between"
                >
                    <div class="flex items-center gap-3">
                        <FileCode class="w-5 h-5 text-primary" />
                        <span class="font-semibold text-foreground">{{
                            selectedFile?.name
                        }}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            @click="saveFile"
                            class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors flex items-center gap-2"
                        >
                            <Check class="w-4 h-4" />
                            Save
                        </button>
                        <button
                            @click="showEditorModal = false"
                            class="p-2 rounded-lg hover:bg-muted transition-colors"
                        >
                            <X class="w-5 h-5 text-muted-foreground" />
                        </button>
                    </div>
                </div>
                <textarea
                    v-model="editorContent"
                    class="flex-1 w-full bg-background p-4 font-mono text-sm resize-none focus:outline-none"
                    spellcheck="false"
                />
            </div>
        </div>

        <!-- Permissions Modal -->
        <div
            v-if="showPermissionsModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showPermissionsModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-4">
                    Change Permissions
                </h3>
                <p class="text-sm text-muted-foreground mb-4">
                    File:
                    <span class="font-medium text-foreground">{{
                        selectedFile?.name
                    }}</span>
                </p>
                <div class="flex items-center gap-4 mb-6">
                    <input
                        v-model="permissionsValue"
                        type="text"
                        placeholder="0644"
                        maxlength="4"
                        class="w-24 bg-muted border border-border rounded-lg px-4 py-3 text-sm font-mono text-center focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    />
                    <div class="text-xs text-muted-foreground">
                        <p>Common values:</p>
                        <p>0644 (files), 0755 (executable/dirs)</p>
                    </div>
                </div>
                <div class="flex justify-end gap-3">
                    <button
                        @click="showPermissionsModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Apply
                    </button>
                </div>
            </div>
        </div>

        <!-- Zip Modal -->
        <div
            v-if="showZipModal"
            class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
            @click.self="showZipModal = false"
        >
            <div
                class="bg-card border border-border rounded-xl shadow-xl w-full max-w-md p-6 animate-in"
            >
                <h3 class="text-lg font-semibold text-foreground mb-4">
                    Create Zip Archive
                </h3>
                <p class="text-sm text-muted-foreground mb-4">
                    Compressing:
                    <span class="font-medium text-foreground">{{
                        selectedFile?.name
                    }}</span>
                </p>
                <input
                    v-model="zipArchiveName"
                    type="text"
                    placeholder="archive_name"
                    class="w-full bg-muted border border-border rounded-lg px-4 py-3 text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
                    @keyup.enter="compressFile"
                    autofocus
                />
                <p class="text-xs text-muted-foreground mt-2">
                    .zip extension will be added automatically
                </p>
                <div class="flex justify-end gap-3 mt-6">
                    <button
                        @click="showZipModal = false"
                        class="px-4 py-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        @click="compressFile"
                        class="px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
                    >
                        Create Zip
                    </button>
                </div>
            </div>
        </div>

        <!-- Context Menu -->
        <ContextMenu
            :visible="showContextMenu"
            :x="contextMenuX"
            :y="contextMenuY"
            :file="contextMenuFile"
            :has-clipboard="!!clipboard"
            @close="showContextMenu = false"
            @action="handleContextAction"
        />

        <!-- Directory Picker Modal for Move/Copy -->
        <DirectoryPickerModal
            :visible="showMoveModal"
            :action="moveAction"
            :file-count="selectedFiles.length"
            :exclude-paths="selectedFiles.map(f => f.path)"
            @close="showMoveModal = false"
            @select="handleBatchMove"
        />
    </FileManagerLayout>
</template>

<style scoped>
.animate-in {
    animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
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
