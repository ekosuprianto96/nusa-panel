/**
 * useTheme Composable
 * 
 * Enhanced theme management dengan support untuk:
 * - Light/Dark mode toggle
 * - Custom themes
 * - System preference detection
 * - Persistent storage
 * - Runtime CSS variable switching
 */

import { ref, computed, watch, onMounted, type Ref, type ComputedRef } from 'vue'
import { themeConfig, customThemes, type ThemeName } from '../theme.config'
import { applyThemeVariables, resetThemeVariables } from '../utils/generateCSS'

// ============================================
// TYPES
// ============================================

export type ColorMode = 'light' | 'dark'

export interface ThemeState {
    colorMode: ColorMode
    themeName: ThemeName
}

export interface UseThemeReturn {
    // State
    colorMode: Ref<ColorMode>
    themeName: Ref<ThemeName>
    isDark: ComputedRef<boolean>
    isLight: ComputedRef<boolean>

    // Actions
    setColorMode: (mode: ColorMode) => void
    toggleColorMode: () => void
    setTheme: (name: ThemeName) => void
    setSystemPreference: () => void

    // Utilities
    availableThemes: ComputedRef<string[]>
    currentThemeConfig: ComputedRef<typeof themeConfig>
}

// ============================================
// CONSTANTS
// ============================================

const STORAGE_KEY_MODE = 'theme-mode'
const STORAGE_KEY_THEME = 'theme-name'

// ============================================
// COMPOSABLE
// ============================================

/**
 * useTheme composable untuk theme management
 * 
 * @example
 * const { colorMode, isDark, toggleColorMode, setTheme } = useTheme()
 * 
 * // Toggle dark mode
 * toggleColorMode()
 * 
 * // Set custom theme
 * setTheme('ocean')
 */
export function useTheme(): UseThemeReturn {
    // ========================================
    // STATE
    // ========================================

    const colorMode = ref<ColorMode>(getInitialColorMode())
    const themeName = ref<ThemeName>(getInitialTheme())

    // ========================================
    // COMPUTED
    // ========================================

    const isDark = computed(() => colorMode.value === 'dark')
    const isLight = computed(() => colorMode.value === 'light')

    const availableThemes = computed(() => [
        'default',
        ...Object.keys(customThemes)
    ])

    const currentThemeConfig = computed(() => {
        if (themeName.value === 'default' || themeName.value === 'light' || themeName.value === 'dark') {
            return themeConfig
        }

        const custom = customThemes[themeName.value]
        if (custom) {
            return {
                ...themeConfig,
                colors: {
                    ...themeConfig.colors,
                    ...custom.colors
                }
            }
        }

        return themeConfig
    })

    // ========================================
    // ACTIONS
    // ========================================

    /**
     * Set color mode (light/dark)
     */
    function setColorMode(mode: ColorMode): void {
        colorMode.value = mode
    }

    /**
     * Toggle between light and dark modes
     */
    function toggleColorMode(): void {
        colorMode.value = colorMode.value === 'light' ? 'dark' : 'light'
    }

    /**
     * Set theme by name
     */
    function setTheme(name: ThemeName): void {
        themeName.value = name
    }

    /**
     * Use system color scheme preference
     */
    function setSystemPreference(): void {
        colorMode.value = getSystemPreference()
    }

    // ========================================
    // WATCHERS
    // ========================================

    // Watch color mode changes
    watch(colorMode, (newMode) => {
        applyColorMode(newMode)
        persistColorMode(newMode)
    }, { immediate: true })

    // Watch theme changes
    watch(themeName, (newTheme) => {
        applyTheme(newTheme, colorMode.value)
        persistTheme(newTheme)
    })

    // Combined watcher untuk apply theme + mode
    watch([colorMode, themeName], ([mode, theme]) => {
        applyTheme(theme, mode)
    })

    // ========================================
    // LIFECYCLE
    // ========================================

    onMounted(() => {
        // Listen untuk system preference changes
        if (typeof window !== 'undefined') {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

            mediaQuery.addEventListener('change', (e) => {
                // Only auto-update if user hasn't set a preference
                const stored = localStorage.getItem(STORAGE_KEY_MODE)
                if (!stored) {
                    colorMode.value = e.matches ? 'dark' : 'light'
                }
            })
        }
    })

    // ========================================
    // RETURN
    // ========================================

    return {
        // State
        colorMode,
        themeName,
        isDark,
        isLight,

        // Actions
        setColorMode,
        toggleColorMode,
        setTheme,
        setSystemPreference,

        // Utilities
        availableThemes,
        currentThemeConfig
    }
}

// ============================================
// HELPER FUNCTIONS
// ============================================

/**
 * Detect system color scheme preference
 */
function getSystemPreference(): ColorMode {
    if (typeof window !== 'undefined' && window.matchMedia) {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return 'light'
}

/**
 * Get initial color mode from storage or system
 */
function getInitialColorMode(): ColorMode {
    if (typeof localStorage !== 'undefined') {
        const stored = localStorage.getItem(STORAGE_KEY_MODE)
        if (stored === 'light' || stored === 'dark') {
            return stored
        }
    }
    return getSystemPreference()
}

/**
 * Get initial theme from storage
 */
function getInitialTheme(): ThemeName {
    if (typeof localStorage !== 'undefined') {
        const stored = localStorage.getItem(STORAGE_KEY_THEME)
        if (stored) {
            return stored
        }
    }
    return 'default'
}

/**
 * Apply color mode to document
 */
function applyColorMode(mode: ColorMode): void {
    if (typeof document !== 'undefined') {
        const root = document.documentElement
        if (mode === 'dark') {
            root.classList.add('dark')
        } else {
            root.classList.remove('dark')
        }
    }
}

/**
 * Apply theme with CSS variables
 */
function applyTheme(theme: ThemeName, mode: ColorMode): void {
    if (typeof document === 'undefined') return

    const root = document.documentElement

    // Remove previous theme classes
    const themeClasses = Array.from(root.classList).filter(c => c.startsWith('theme-'))
    themeClasses.forEach(c => root.classList.remove(c))

    if (theme !== 'default' && theme !== 'light' && theme !== 'dark') {
        // Add custom theme class
        root.classList.add(`theme-${theme}`)

        // Apply custom theme CSS variables
        if (customThemes[theme]) {
            applyThemeVariables(theme, mode, root)
        }
    } else {
        // Reset to default
        resetThemeVariables(root)
    }
}

/**
 * Persist color mode to storage
 */
function persistColorMode(mode: ColorMode): void {
    if (typeof localStorage !== 'undefined') {
        localStorage.setItem(STORAGE_KEY_MODE, mode)
    }
}

/**
 * Persist theme to storage
 */
function persistTheme(theme: ThemeName): void {
    if (typeof localStorage !== 'undefined') {
        localStorage.setItem(STORAGE_KEY_THEME, theme)
    }
}

// ============================================
// SINGLETON PATTERN (Optional)
// ============================================

let themeInstance: UseThemeReturn | null = null

/**
 * Get singleton theme instance
 * Berguna untuk memastikan state konsisten di seluruh app
 */
export function useThemeSingleton(): UseThemeReturn {
    if (!themeInstance) {
        themeInstance = useTheme()
    }
    return themeInstance
}

export default useTheme
