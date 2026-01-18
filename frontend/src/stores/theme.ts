import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

/**
 * Theme types yang didukung
 */
export type Theme = 'light' | 'dark'

/**
 * Theme Store
 * Mengelola state tema aplikasi (light/dark mode)
 */
export const useThemeStore = defineStore('theme', () => {
  /**
   * Mendeteksi preferensi sistem untuk dark mode
   */
  const getSystemPreference = (): Theme => {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return 'light'
  }

  /**
   * Mendapatkan tema dari localStorage atau system preference
   */
  const getInitialTheme = (): Theme => {
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem('theme') as Theme | null
      if (stored === 'light' || stored === 'dark') {
        return stored
      }
    }
    return getSystemPreference()
  }

  const theme = ref<Theme>(getInitialTheme())

  /**
   * Toggle tema antara light dan dark
   */
  const toggleTheme = (): void => {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
  }

  /**
   * Set tema secara spesifik
   * @param newTheme - Tema yang akan diset
   */
  const setTheme = (newTheme: Theme): void => {
    theme.value = newTheme
  }

  /**
   * Cek apakah tema saat ini adalah dark
   */
  const isDark = (): boolean => theme.value === 'dark'

  /**
   * Watch perubahan tema dan apply ke document
   */
  watch(
    theme,
    (newTheme) => {
      if (typeof document !== 'undefined') {
        const root = document.documentElement
        if (newTheme === 'dark') {
          root.classList.add('dark')
        } else {
          root.classList.remove('dark')
        }
        localStorage.setItem('theme', newTheme)
      }
    },
    { immediate: true }
  )

  return {
    theme,
    toggleTheme,
    setTheme,
    isDark
  }
})
