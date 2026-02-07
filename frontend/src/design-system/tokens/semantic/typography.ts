/**
 * Semantic Typography Tokens (Level 1)
 * 
 * Purpose-driven typography assignments.
 * 
 * @layer semantic
 */

import {
    primitiveFontFamily,
    primitiveFontSize,
    primitiveFontWeight,
    primitiveLineHeight,
    primitiveLetterSpacing
} from '../primitives/typography'

// ============================================
// TEXT STYLES
// ============================================

export interface TextStyle {
    fontFamily: string
    fontSize: string
    fontWeight: string
    lineHeight: string
    letterSpacing?: string
}

export const semanticTextStyles: Record<string, TextStyle> = {
    // ----------------------------------------
    // DISPLAY (Hero text)
    // ----------------------------------------
    displayLarge: {
        fontFamily: primitiveFontFamily.display,
        fontSize: primitiveFontSize['5xl'],
        fontWeight: primitiveFontWeight.bold,
        lineHeight: primitiveLineHeight.tight,
        letterSpacing: primitiveLetterSpacing.tight
    },
    displayMedium: {
        fontFamily: primitiveFontFamily.display,
        fontSize: primitiveFontSize['4xl'],
        fontWeight: primitiveFontWeight.bold,
        lineHeight: primitiveLineHeight.tight,
        letterSpacing: primitiveLetterSpacing.tight
    },
    displaySmall: {
        fontFamily: primitiveFontFamily.display,
        fontSize: primitiveFontSize['3xl'],
        fontWeight: primitiveFontWeight.bold,
        lineHeight: primitiveLineHeight.tight
    },

    // ----------------------------------------
    // HEADINGS
    // ----------------------------------------
    headingLarge: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize['2xl'],
        fontWeight: primitiveFontWeight.semibold,
        lineHeight: primitiveLineHeight.snug
    },
    headingMedium: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.xl,
        fontWeight: primitiveFontWeight.semibold,
        lineHeight: primitiveLineHeight.snug
    },
    headingSmall: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.lg,
        fontWeight: primitiveFontWeight.semibold,
        lineHeight: primitiveLineHeight.normal
    },

    // ----------------------------------------
    // BODY TEXT
    // ----------------------------------------
    bodyLarge: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.lg,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    },
    bodyMedium: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.base,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    },
    bodySmall: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.sm,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    },

    // ----------------------------------------
    // UI TEXT
    // ----------------------------------------
    label: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.sm,
        fontWeight: primitiveFontWeight.medium,
        lineHeight: primitiveLineHeight.normal
    },
    caption: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.xs,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.normal
    },
    overline: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.xs,
        fontWeight: primitiveFontWeight.semibold,
        lineHeight: primitiveLineHeight.normal,
        letterSpacing: primitiveLetterSpacing.wider
    },

    // ----------------------------------------
    // INTERACTIVE
    // ----------------------------------------
    button: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.sm,
        fontWeight: primitiveFontWeight.medium,
        lineHeight: primitiveLineHeight.none
    },
    buttonSmall: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.xs,
        fontWeight: primitiveFontWeight.medium,
        lineHeight: primitiveLineHeight.none
    },
    buttonLarge: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.base,
        fontWeight: primitiveFontWeight.medium,
        lineHeight: primitiveLineHeight.none
    },
    link: {
        fontFamily: primitiveFontFamily.sans,
        fontSize: primitiveFontSize.base,
        fontWeight: primitiveFontWeight.medium,
        lineHeight: primitiveLineHeight.normal
    },

    // ----------------------------------------
    // CODE
    // ----------------------------------------
    codeLarge: {
        fontFamily: primitiveFontFamily.mono,
        fontSize: primitiveFontSize.base,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    },
    codeMedium: {
        fontFamily: primitiveFontFamily.mono,
        fontSize: primitiveFontSize.sm,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    },
    codeSmall: {
        fontFamily: primitiveFontFamily.mono,
        fontSize: primitiveFontSize.xs,
        fontWeight: primitiveFontWeight.normal,
        lineHeight: primitiveLineHeight.relaxed
    }
} as const

export type TextStyleKey = keyof typeof semanticTextStyles

/**
 * Get text style
 */
export function getTextStyle(key: TextStyleKey): TextStyle {
    return semanticTextStyles[key] as TextStyle
}

export const semanticTypography = {
    textStyles: semanticTextStyles,
    getTextStyle
} as const

export default semanticTypography
