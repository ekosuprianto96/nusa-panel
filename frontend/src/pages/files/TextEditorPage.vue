<script setup lang="ts">
/**
 * TextEditorPage - Halaman Text Editor dengan Monaco Editor (Optimized)
 * 
 * Features:
 * - Lazy-loaded Monaco 
 * - Limited languages (via Vite plugin)
 * - Runtime optimizations
 * - Ctrl+S to save
 */
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick, shallowRef } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import FileManagerLayout from '@/layouts/FileManagerLayout.vue'
import { fileService } from '@/services/file.service'
import { 
  Save, ArrowLeft, FileCode, AlertCircle, Loader2, Sun, Moon
} from 'lucide-vue-next'

// Type import untuk Monaco
import type * as Monaco from 'monaco-editor'

const route = useRoute()
const router = useRouter()

/** File path dari query */
const filePath = computed(() => (route.query.path as string) || '')
const fileName = computed(() => filePath.value.split('/').pop() || 'Untitled')

/** Editor state - gunakan shallowRef untuk performance */
const editorContainer = ref<HTMLElement | null>(null)
const editor = shallowRef<Monaco.editor.IStandaloneCodeEditor | null>(null)
let monaco: typeof Monaco | null = null

/** File content */
const content = ref('')
const originalContent = ref('')
const isLoading = ref(true)
const isSaving = ref(false)
const error = ref('')
const saveSuccess = ref(false)
const editorReady = ref(false)

/** Theme */
const isDark = ref(document.documentElement.classList.contains('dark'))

/** Check if file has unsaved changes */
const hasChanges = computed(() => content.value !== originalContent.value)

/** Detect language from file extension - untuk file hosting panel */
const detectLanguage = (filename: string): string => {
  const ext = filename.split('.').pop()?.toLowerCase()
  const name = filename.toLowerCase()
  
  // Special files tanpa extension
  if (name === '.htaccess' || name === 'htaccess') return 'apache'
  if (name === '.env' || name.startsWith('.env.')) return 'ini'
  if (name === 'dockerfile') return 'dockerfile'
  if (name === 'makefile') return 'makefile'
  if (name === '.gitignore' || name === '.dockerignore') return 'ignore'
  if (name === 'nginx.conf' || name.includes('nginx')) return 'nginx'
  if (name === 'apache.conf' || name === 'httpd.conf') return 'apache'
  if (name === 'wp-config.php') return 'php'
  
  const languageMap: Record<string, string> = {
    // Web Languages
    'js': 'javascript',
    'mjs': 'javascript',
    'cjs': 'javascript',
    'jsx': 'javascript', 
    'ts': 'typescript',
    'tsx': 'typescript',
    'vue': 'html',
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'scss',
    'less': 'less',
    
    // Data Formats
    'json': 'json',
    'xml': 'xml',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'ini',
    
    // Server-side (common di hosting)
    'php': 'php',
    'phtml': 'php',
    'php3': 'php',
    'php4': 'php',
    'php5': 'php',
    'php7': 'php',
    'phps': 'php',
    'py': 'python',
    'rb': 'ruby',
    'pl': 'perl',
    
    // Config files (hosting)
    'conf': 'ini',
    'ini': 'ini',
    'cfg': 'ini',
    'env': 'ini',
    'htaccess': 'apache',
    'htpasswd': 'plaintext',
    
    // Database
    'sql': 'sql',
    'mysql': 'sql',
    
    // Shell/Scripts
    'sh': 'shell',
    'bash': 'shell',
    'zsh': 'shell',
    'fish': 'shell',
    'csh': 'shell',
    
    // Documentation
    'md': 'markdown',
    'markdown': 'markdown',
    'txt': 'plaintext',
    'log': 'plaintext',
    
    // Cron
    'cron': 'shell',
    'crontab': 'shell',
  }
  return languageMap[ext || ''] || 'plaintext'
}

/** Load file content */
const loadFile = async (): Promise<void> => {
  if (!filePath.value) {
    error.value = 'No file path specified'
    isLoading.value = false
    return
  }

  try {
    isLoading.value = true
    error.value = ''
    const response = await fileService.getFileContent(filePath.value)
    content.value = response.data.data.content
    originalContent.value = response.data.data.content
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal memuat file'
  } finally {
    isLoading.value = false
  }
}

/** Save file */
const saveFile = async (): Promise<void> => {
  if (!filePath.value || !hasChanges.value) return

  try {
    isSaving.value = true
    error.value = ''
    await fileService.writeFile({
      path: filePath.value,
      content: content.value
    })
    originalContent.value = content.value
    saveSuccess.value = true
    setTimeout(() => saveSuccess.value = false, 2000)
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Gagal menyimpan file'
  } finally {
    isSaving.value = false
  }
}

/** Go back to file manager */
const goBack = (): void => {
  if (hasChanges.value) {
    if (!confirm('Anda memiliki perubahan yang belum disimpan. Yakin ingin keluar?')) {
      return
    }
  }
  const parentPath = filePath.value.split('/').slice(0, -1).join('/')
  router.push({ 
    path: '/dashboard/files', 
    query: parentPath ? { path: parentPath } : undefined 
  })
}

/** Toggle theme */
const toggleTheme = (): void => {
  isDark.value = !isDark.value
  if (monaco) {
    monaco.editor.setTheme(isDark.value ? 'vs-dark' : 'vs')
  }
}

/** Lazy load dan initialize Monaco Editor */
const initMonacoEditor = async (): Promise<void> => {
  if (!editorContainer.value) return
  
  try {
    // Lazy import Monaco - workers di-handle oleh vite-plugin-monaco-editor
    monaco = await import('monaco-editor')

    const language = detectLanguage(fileName.value)

    // Create editor dengan error markers enabled
    editor.value = monaco.editor.create(editorContainer.value, {
      value: content.value || '',
      language: language,
      theme: isDark.value ? 'vs-dark' : 'vs',
      // Performance optimizations
      minimap: { enabled: false },
      folding: true,
      renderLineHighlight: 'line',
      // Basic settings
      lineNumbers: 'on',
      fontSize: 14,
      fontFamily: "Consolas, 'Courier New', monospace",
      wordWrap: 'on',
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      // Error markers settings
      glyphMargin: true,
      scrollbar: {
        vertical: 'auto',
        horizontal: 'auto',
      },
    })

    // Update content on change
    editor.value.onDidChangeModelContent(() => {
      content.value = editor.value?.getValue() || ''
    })

    // Ctrl+S to save
    editor.value.addAction({
      id: 'save-file',
      label: 'Save File',
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
      run: () => saveFile()
    })

    editorReady.value = true
  } catch (err) {
    console.error('Failed to initialize Monaco:', err)
    error.value = 'Gagal memuat editor'
  }
}

onMounted(async () => {
  // Load file content first
  await loadFile()
  
  // Wait for DOM then init editor
  await nextTick()
  await initMonacoEditor()
})

onBeforeUnmount(() => {
  editor.value?.dispose()
})

watch(isDark, (newVal) => {
  if (monaco) {
    monaco.editor.setTheme(newVal ? 'vs-dark' : 'vs')
  }
})
</script>

<template>
  <FileManagerLayout>
    <div class="flex flex-col h-full bg-background">
      <!-- Header -->
      <header class="h-14 border-b border-border bg-card px-4 flex items-center justify-between flex-shrink-0">
        <div class="flex items-center gap-3">
          <button 
            @click="goBack"
            class="p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
          >
            <ArrowLeft class="w-5 h-5" />
          </button>
          <FileCode class="w-5 h-5 text-primary" />
          <div class="flex items-center gap-2">
            <span class="font-semibold text-foreground">{{ fileName }}</span>
            <span v-if="hasChanges" class="text-xs text-orange-500">●</span>
          </div>
        </div>
        
        <div class="flex items-center gap-2">
          <!-- Theme toggle -->
          <button
            @click="toggleTheme"
            class="p-2 rounded-lg hover:bg-muted text-muted-foreground transition-colors"
            :title="isDark ? 'Light Mode' : 'Dark Mode'"
          >
            <component :is="isDark ? Sun : Moon" class="w-4 h-4" />
          </button>

          <!-- Save button -->
          <button
            @click="saveFile"
            :disabled="!hasChanges || isSaving"
            :class="[
              'px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all',
              hasChanges 
                ? 'bg-primary text-primary-foreground hover:bg-primary/90' 
                : 'bg-muted text-muted-foreground cursor-not-allowed'
            ]"
          >
            <Loader2 v-if="isSaving" class="w-4 h-4 animate-spin" />
            <Save v-else class="w-4 h-4" />
            {{ isSaving ? 'Saving...' : 'Save' }}
          </button>
        </div>
      </header>

      <!-- Save success indicator -->
      <div 
        v-if="saveSuccess"
        class="absolute top-16 right-4 bg-green-500 text-white px-4 py-2 rounded-lg shadow-lg text-sm font-medium z-50"
      >
        ✓ Saved!
      </div>

      <!-- Error message -->
      <div 
        v-if="error" 
        class="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg flex items-center gap-2 text-destructive text-sm"
      >
        <AlertCircle class="w-4 h-4 flex-shrink-0" />
        {{ error }}
      </div>

      <!-- Loading state -->
      <div v-if="isLoading && !editorReady" class="flex-1 flex items-center justify-center">
        <Loader2 class="w-8 h-8 text-primary animate-spin" />
      </div>

      <!-- Editor Container -->
      <div 
        ref="editorContainer" 
        class="editor-container flex-1"
        :class="{ 'hidden': isLoading && !editorReady }"
      />
    </div>
  </FileManagerLayout>
</template>

<style>
/* Monaco Editor container */
.editor-container {
  min-height: 300px;
  width: 100%;
}
</style>
