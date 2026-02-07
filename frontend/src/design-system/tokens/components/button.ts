/**
 * Button Component Tokens (Level 2)
 * 
 * Component-specific token definitions for Button.
 * These are what the Button component actually consumes.
 * 
 * @layer components
 */

import { semanticElementSpacing } from '../semantic/spacing'
import { semanticTransitions } from '../semantic/motion'
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
// BUTTON COLORS BY VARIANT
// ============================================

export interface ButtonColorTokens {
    bg: SemanticColorValue
    bgHover: SemanticColorValue
    bgActive?: SemanticColorValue
    fg: SemanticColorValue
    border?: SemanticColorValue
}

export const buttonColors = {
    default: {
        bg: { light: primitiveColors.blue[500], dark: primitiveColors.blue[500] },
        bgHover: { light: primitiveColors.blue[600], dark: primitiveColors.blue[400] },
        bgActive: { light: primitiveColors.blue[700], dark: primitiveColors.blue[300] },
        fg: { light: primitiveColors.white, dark: primitiveColors.slate[900] }
    },

    destructive: {
        bg: { light: primitiveColors.red[500], dark: primitiveColors.red[900] },
        bgHover: { light: primitiveColors.red[600], dark: primitiveColors.red[800] },
        fg: { light: primitiveColors.white, dark: primitiveColors.red[50] }
    },

    success: {
        bg: { light: primitiveColors.green[500], dark: primitiveColors.green[600] },
        bgHover: { light: primitiveColors.green[600], dark: primitiveColors.green[500] },
        fg: { light: primitiveColors.white, dark: primitiveColors.white }
    },

    warning: {
        bg: { light: primitiveColors.amber[500], dark: primitiveColors.amber[500] },
        bgHover: { light: primitiveColors.amber[600], dark: primitiveColors.amber[400] },
        fg: { light: primitiveColors.black, dark: primitiveColors.black }
    },

    outline: {
        bg: { light: 'transparent', dark: 'transparent' },
        bgHover: { light: primitiveColors.slate[100], dark: primitiveColors.slate[800] },
        fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
        border: { light: primitiveColors.slate[200], dark: primitiveColors.slate[700] }
    },

    secondary: {
        bg: { light: primitiveColors.slate[100], dark: primitiveColors.slate[800] },
        bgHover: { light: primitiveColors.slate[200], dark: primitiveColors.slate[700] },
        fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] }
    },

    ghost: {
        bg: { light: 'transparent', dark: 'transparent' },
        bgHover: { light: primitiveColors.slate[100], dark: primitiveColors.slate[800] },
        fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] }
    },

    link: {
        bg: { light: 'transparent', dark: 'transparent' },
        bgHover: { light: 'transparent', dark: 'transparent' },
        fg: { light: primitiveColors.blue[500], dark: primitiveColors.blue[500] }
    }
} as const

// ============================================
// BUTTON SIZE TOKENS
// ============================================

export interface ButtonSizeTokens {
    height: string
    paddingX: string
    paddingY: string
    fontSize: string
    iconSize: string
    iconGap: string
}

export const buttonSizes = {
    sm: {
        height: '2rem',        // 32px
        paddingX: semanticElementSpacing.buttonPadding.sm.x,
        paddingY: semanticElementSpacing.buttonPadding.sm.y,
        fontSize: '0.75rem',   // 12px
        iconSize: '0.875rem',  // 14px
        iconGap: '0.375rem'    // 6px
    },

    default: {
        height: '2.5rem',      // 40px
        paddingX: semanticElementSpacing.buttonPadding.md.x,
        paddingY: semanticElementSpacing.buttonPadding.md.y,
        fontSize: '0.875rem',  // 14px
        iconSize: '1rem',      // 16px
        iconGap: '0.5rem'      // 8px
    },

    lg: {
        height: '2.75rem',     // 44px
        paddingX: semanticElementSpacing.buttonPadding.lg.x,
        paddingY: semanticElementSpacing.buttonPadding.lg.y,
        fontSize: '1rem',      // 16px
        iconSize: '1.125rem',  // 18px
        iconGap: '0.5rem'      // 8px
    },

    xl: {
        height: '3rem',        // 48px
        paddingX: semanticElementSpacing.buttonPadding.xl.x,
        paddingY: semanticElementSpacing.buttonPadding.xl.y,
        fontSize: '1rem',      // 16px
        iconSize: '1.25rem',   // 20px
        iconGap: '0.625rem'    // 10px
    },

    icon: {
        height: '2.5rem',      // 40px
        paddingX: '0',
        paddingY: '0',
        fontSize: '1rem',
        iconSize: '1rem',
        iconGap: '0'
    }
} as const

// ============================================
// BUTTON SHAPE TOKENS
// ============================================

export const buttonShape = {
    borderRadius: primitiveRadius.md,
    borderWidth: '1px',

    /** Focus ring */
    focusRingWidth: '2px',
    focusRingOffset: '2px'
} as const

// ============================================
// BUTTON MOTION TOKENS
// ============================================

export const buttonMotion = {
    transition: semanticTransitions.colors,

    /** Disabled state */
    disabledOpacity: '0.5',

    /** Loading spinner speed */
    spinnerDuration: '600ms'
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const buttonTokens = {
    colors: buttonColors,
    sizes: buttonSizes,
    shape: buttonShape,
    motion: buttonMotion
} as const

export type ButtonVariant = keyof typeof buttonColors
export type ButtonSize = keyof typeof buttonSizes

export default buttonTokens
