/**
 * Spacing Tokens
 * 
 * Consistent spacing scale untuk margins, paddings, gaps
 */

import { themeConfig } from '../theme.config'

export const spacing = themeConfig.spacing

/**
 * Semantic spacing - nama yang lebih mudah diingat
 */
export const semanticSpacing = {
    // Component internal spacing
    none: spacing[0],
    xxs: spacing[0.5],   // 2px - very tight
    xs: spacing[1],       // 4px - tight
    sm: spacing[2],       // 8px - compact
    md: spacing[4],       // 16px - default
    lg: spacing[6],       // 24px - comfortable
    xl: spacing[8],       // 32px - spacious
    '2xl': spacing[12],   // 48px - very spacious
    '3xl': spacing[16],   // 64px - extra spacious

    // Page-level spacing
    pageX: spacing[4],           // horizontal page padding (mobile)
    pageXMd: spacing[6],         // horizontal page padding (tablet)
    pageXLg: spacing[8],         // horizontal page padding (desktop)
    pageY: spacing[6],           // vertical page spacing

    // Section spacing
    sectionGap: spacing[8],      // gap between sections

    // Card spacing
    cardPadding: spacing[6],     // card internal padding
    cardGap: spacing[4],         // gap between card elements

    // Form spacing
    formGap: spacing[4],         // gap between form fields
    inputPaddingX: spacing[3],   // input horizontal padding
    inputPaddingY: spacing[2],   // input vertical padding

    // Stack/inline spacing
    stackGap: spacing[2],        // gap in vertical stacks
    inlineGap: spacing[2],       // gap in horizontal layouts

    // Icon spacing
    iconGap: spacing[2],         // gap between icon and text
} as const

/**
 * Helper untuk generate CSS spacing value
 */
export function getSpacing(key: keyof typeof spacing): string {
    return spacing[key]
}

/**
 * Helper untuk menghitung spacing berdasarkan multiplier
 * Berguna untuk custom spacing calculations
 */
export function calcSpacing(baseKey: keyof typeof spacing, multiplier: number): string {
    const baseValue = parseFloat(spacing[baseKey])
    const unit = spacing[baseKey].replace(/[\d.]/g, '')
    return `${baseValue * multiplier}${unit}`
}

export default {
    spacing,
    semanticSpacing,
    getSpacing,
    calcSpacing
}
