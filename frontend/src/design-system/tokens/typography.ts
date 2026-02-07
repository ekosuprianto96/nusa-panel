/**
 * Typography Tokens
 * 
 * Font families, sizes, weights, dan line heights
 */

import { themeConfig } from '../theme.config'

export const typography = themeConfig.typography

/**
 * Font family utilities
 */
export const fontFamily = {
    sans: typography.fontFamily.sans,
    mono: typography.fontFamily.mono,
    display: typography.fontFamily.display
}

/**
 * Font size scale dengan line height yang sesuai
 */
export const fontSizes = {
    xs: { size: typography.fontSize.xs, lineHeight: '1rem' },      // 12px/16px
    sm: { size: typography.fontSize.sm, lineHeight: '1.25rem' },   // 14px/20px
    base: { size: typography.fontSize.base, lineHeight: '1.5rem' }, // 16px/24px
    lg: { size: typography.fontSize.lg, lineHeight: '1.75rem' },   // 18px/28px
    xl: { size: typography.fontSize.xl, lineHeight: '1.75rem' },   // 20px/28px
    '2xl': { size: typography.fontSize['2xl'], lineHeight: '2rem' }, // 24px/32px
    '3xl': { size: typography.fontSize['3xl'], lineHeight: '2.25rem' }, // 30px/36px
    '4xl': { size: typography.fontSize['4xl'], lineHeight: '2.5rem' }  // 36px/40px
}

/**
 * Font weights
 */
export const fontWeight = typography.fontWeight

/**
 * Line heights
 */
export const lineHeight = typography.lineHeight

/**
 * Letter spacing
 */
export const letterSpacing = typography.letterSpacing

/**
 * Typography presets - kombinasi yang sering digunakan
 */
export const textStyles = {
    // Headings
    h1: {
        fontSize: typography.fontSize['4xl'],
        fontWeight: typography.fontWeight.bold,
        lineHeight: typography.lineHeight.tight,
        letterSpacing: typography.letterSpacing.tight
    },
    h2: {
        fontSize: typography.fontSize['3xl'],
        fontWeight: typography.fontWeight.semibold,
        lineHeight: typography.lineHeight.tight,
        letterSpacing: typography.letterSpacing.tight
    },
    h3: {
        fontSize: typography.fontSize['2xl'],
        fontWeight: typography.fontWeight.semibold,
        lineHeight: typography.lineHeight.snug
    },
    h4: {
        fontSize: typography.fontSize.xl,
        fontWeight: typography.fontWeight.semibold,
        lineHeight: typography.lineHeight.snug
    },
    h5: {
        fontSize: typography.fontSize.lg,
        fontWeight: typography.fontWeight.medium,
        lineHeight: typography.lineHeight.normal
    },
    h6: {
        fontSize: typography.fontSize.base,
        fontWeight: typography.fontWeight.medium,
        lineHeight: typography.lineHeight.normal
    },

    // Body text
    body: {
        fontSize: typography.fontSize.base,
        fontWeight: typography.fontWeight.normal,
        lineHeight: typography.lineHeight.relaxed
    },
    bodySmall: {
        fontSize: typography.fontSize.sm,
        fontWeight: typography.fontWeight.normal,
        lineHeight: typography.lineHeight.relaxed
    },
    bodyLarge: {
        fontSize: typography.fontSize.lg,
        fontWeight: typography.fontWeight.normal,
        lineHeight: typography.lineHeight.relaxed
    },

    // UI text
    label: {
        fontSize: typography.fontSize.sm,
        fontWeight: typography.fontWeight.medium,
        lineHeight: typography.lineHeight.normal
    },
    caption: {
        fontSize: typography.fontSize.xs,
        fontWeight: typography.fontWeight.normal,
        lineHeight: typography.lineHeight.normal
    },
    button: {
        fontSize: typography.fontSize.sm,
        fontWeight: typography.fontWeight.medium,
        lineHeight: typography.lineHeight.none
    },

    // Code
    code: {
        fontFamily: typography.fontFamily.mono,
        fontSize: typography.fontSize.sm,
        fontWeight: typography.fontWeight.normal,
        lineHeight: typography.lineHeight.normal
    }
} as const

export type TextStyleKey = keyof typeof textStyles

export default {
    fontFamily,
    fontSizes,
    fontWeight,
    lineHeight,
    letterSpacing,
    textStyles
}
