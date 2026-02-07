/**
 * Border Radius Tokens
 * 
 * Consistent border radius values untuk UI elements
 */

import { themeConfig } from '../theme.config'

export const radius = themeConfig.radius

/**
 * Semantic radius - penggunaan yang lebih spesifik
 */
export const semanticRadius = {
    // No rounding
    none: radius.none,

    // Subtle rounding
    subtle: radius.sm,

    // Default rounding for most elements
    default: radius.md,

    // Cards, panels
    card: radius.lg,

    // Buttons, badges
    button: radius.md,

    // Input fields
    input: radius.md,

    // Modals, dialogs
    modal: radius.xl,

    // Tooltips, popovers
    popover: radius.lg,

    // Pills, tags
    pill: radius.full,

    // Avatar images
    avatar: radius.full,

    // Large containers
    container: radius['2xl']
} as const

/**
 * Helper untuk mendapatkan CSS radius value
 */
export function getRadius(key: keyof typeof radius): string {
    return radius[key]
}

/**
 * Generate clipping radius (untuk nested elements)
 * Kompensasi untuk border width
 */
export function getInnerRadius(outerRadius: keyof typeof radius, borderWidth: number = 1): string {
    const outerValue = parseFloat(radius[outerRadius]) || 0
    const innerValue = Math.max(0, outerValue - borderWidth / 16) // Convert px to rem approx
    return `${innerValue}rem`
}

export default {
    radius,
    semanticRadius,
    getRadius,
    getInnerRadius
}
