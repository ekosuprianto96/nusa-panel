/**
 * Shadow Tokens
 * 
 * Box shadow definitions untuk depth dan elevation
 */

import { themeConfig } from '../theme.config'

export const shadows = themeConfig.shadows

/**
 * Elevation levels - semantic shadows
 */
export const elevation = {
    // Flat - no shadow
    level0: shadows.none,

    // Subtle lift - untuk cards pada hover
    level1: shadows.sm,

    // Default elevation - untuk cards
    level2: shadows.DEFAULT,

    // Medium elevation - untuk dropdowns
    level3: shadows.md,

    // High elevation - untuk dialogs, modals
    level4: shadows.lg,

    // Highest elevation - untuk important overlays
    level5: shadows.xl
} as const

/**
 * Semantic shadows untuk use cases spesifik
 */
export const semanticShadows = {
    // Cards & panels
    card: shadows.DEFAULT,
    cardHover: shadows.md,

    // Buttons
    button: shadows.sm,
    buttonHover: shadows.DEFAULT,

    // Dropdowns & menus
    dropdown: shadows.lg,
    popover: shadows.lg,

    // Modals & dialogs
    modal: shadows.xl,
    dialog: shadows.xl,

    // Toast notifications
    toast: shadows.lg,

    // Focused input
    inputFocus: '0 0 0 2px hsl(var(--ring) / 0.3)',

    // Inner shadows
    inner: shadows.inner,
    inset: shadows.inner
} as const

/**
 * Dark mode shadow adjustments
 * Shadows are typically lighter/more subtle in dark mode
 */
export const darkShadows = {
    sm: '0 1px 2px 0 rgb(0 0 0 / 0.2)',
    DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.3), 0 1px 2px -1px rgb(0 0 0 / 0.3)',
    md: '0 4px 6px -1px rgb(0 0 0 / 0.3), 0 2px 4px -2px rgb(0 0 0 / 0.3)',
    lg: '0 10px 15px -3px rgb(0 0 0 / 0.4), 0 4px 6px -4px rgb(0 0 0 / 0.3)',
    xl: '0 20px 25px -5px rgb(0 0 0 / 0.4), 0 8px 10px -6px rgb(0 0 0 / 0.3)',
    '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.5)'
} as const

/**
 * Helper untuk mendapatkan shadow value
 */
export function getShadow(key: keyof typeof shadows): string {
    return shadows[key]
}

/**
 * Get elevation shadow
 */
export function getElevation(level: keyof typeof elevation): string {
    return elevation[level]
}

export default {
    shadows,
    elevation,
    semanticShadows,
    darkShadows,
    getShadow,
    getElevation
}
