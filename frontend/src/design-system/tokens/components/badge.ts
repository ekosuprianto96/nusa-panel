/**
 * Badge Component Tokens (Level 2)
 * 
 * Component-specific token definitions for Badge.
 * 
 * @layer components
 */

import { semanticElementSpacing } from '../semantic/spacing'
import { primitiveRadius } from '../primitives/shape'
import { primitiveColors } from '../primitives/colors'

// ============================================
// TYPES
// ============================================

export interface SemanticColorValue {
    light: string
    dark: string
}

// ============================================
// BADGE COLOR TOKENS
// ============================================

export interface BadgeColorTokens {
    bg: SemanticColorValue
    fg: SemanticColorValue
    border?: SemanticColorValue
}

export const badgeColors = {
    default: {
        bg: { light: primitiveColors.blue[500], dark: primitiveColors.blue[500] },
        fg: { light: primitiveColors.white, dark: primitiveColors.slate[900] }
    },

    secondary: {
        bg: { light: primitiveColors.slate[100], dark: primitiveColors.slate[800] },
        fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] }
    },

    destructive: {
        bg: { light: primitiveColors.red[500], dark: primitiveColors.red[900] },
        fg: { light: primitiveColors.white, dark: primitiveColors.red[50] }
    },

    success: {
        bg: { light: primitiveColors.green[500], dark: primitiveColors.green[600] },
        fg: { light: primitiveColors.white, dark: primitiveColors.white }
    },

    warning: {
        bg: { light: primitiveColors.amber[500], dark: primitiveColors.amber[500] },
        fg: { light: primitiveColors.black, dark: primitiveColors.black }
    },

    info: {
        bg: { light: primitiveColors.cyan[500], dark: primitiveColors.cyan[500] },
        fg: { light: primitiveColors.white, dark: primitiveColors.white }
    },

    outline: {
        bg: { light: 'transparent', dark: 'transparent' },
        fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
        border: { light: primitiveColors.slate[200], dark: primitiveColors.slate[700] }
    }
} as const

// ============================================
// BADGE SIZE TOKENS
// ============================================

export interface BadgeSizeTokens {
    paddingX: string
    paddingY: string
    fontSize: string
    lineHeight: string
}

export const badgeSizes = {
    sm: {
        paddingX: '0.5rem',
        paddingY: '0.0625rem',  // 1px
        fontSize: '0.625rem',   // 10px
        lineHeight: '1'
    },

    default: {
        paddingX: semanticElementSpacing.badgePaddingX,
        paddingY: semanticElementSpacing.badgePaddingY,
        fontSize: '0.75rem',    // 12px
        lineHeight: '1'
    },

    lg: {
        paddingX: '0.75rem',
        paddingY: '0.25rem',
        fontSize: '0.875rem',   // 14px
        lineHeight: '1'
    }
} as const

// ============================================
// BADGE SHAPE TOKENS
// ============================================

export const badgeShape = {
    borderRadius: primitiveRadius.full,
    borderWidth: '1px'
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const badgeTokens = {
    colors: badgeColors,
    sizes: badgeSizes,
    shape: badgeShape
} as const

export type BadgeVariant = keyof typeof badgeColors
export type BadgeSize = keyof typeof badgeSizes

export default badgeTokens
