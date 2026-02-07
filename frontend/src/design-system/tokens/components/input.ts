/**
 * Input Component Tokens (Level 2)
 * 
 * Component-specific token definitions for Input/Textarea.
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
// INPUT COLORS BY STATE
// ============================================

export interface InputColorTokens {
    bg: SemanticColorValue
    border: SemanticColorValue
    borderFocus: SemanticColorValue
    placeholder: SemanticColorValue
    text: SemanticColorValue
    ring?: SemanticColorValue
}

export const inputColors = {
    default: {
        bg: { light: primitiveColors.white, dark: primitiveColors.slate[900] },
        border: { light: primitiveColors.slate[300], dark: primitiveColors.slate[600] },
        borderFocus: { light: primitiveColors.blue[500], dark: primitiveColors.blue[400] },
        placeholder: { light: primitiveColors.slate[500], dark: primitiveColors.slate[400] },
        text: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
        ring: { light: primitiveColors.blue[500], dark: primitiveColors.blue[400] }
    },

    error: {
        bg: { light: primitiveColors.white, dark: primitiveColors.slate[900] },
        border: { light: primitiveColors.red[500], dark: primitiveColors.red[900] },
        borderFocus: { light: primitiveColors.red[500], dark: primitiveColors.red[900] },
        placeholder: { light: primitiveColors.slate[500], dark: primitiveColors.slate[400] },
        text: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
        ring: { light: primitiveColors.red[500], dark: primitiveColors.red[900] }
    },

    success: {
        bg: { light: primitiveColors.white, dark: primitiveColors.slate[900] },
        border: { light: primitiveColors.green[500], dark: primitiveColors.green[600] },
        borderFocus: { light: primitiveColors.green[500], dark: primitiveColors.green[600] },
        placeholder: { light: primitiveColors.slate[500], dark: primitiveColors.slate[400] },
        text: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
        ring: { light: primitiveColors.green[500], dark: primitiveColors.green[600] }
    }
} as const

// ============================================
// INPUT SIZE TOKENS
// ============================================

export interface InputSizeTokens {
    height: string
    paddingX: string
    paddingY: string
    fontSize: string
}

export const inputSizes = {
    sm: {
        height: '2rem',        // 32px
        paddingX: semanticElementSpacing.inputPaddingX,
        paddingY: '0.25rem',
        fontSize: '0.75rem'    // 12px
    },

    default: {
        height: '2.5rem',      // 40px
        paddingX: semanticElementSpacing.inputPaddingX,
        paddingY: semanticElementSpacing.inputPaddingY,
        fontSize: '0.875rem'   // 14px
    },

    lg: {
        height: '2.75rem',     // 44px
        paddingX: '1rem',
        paddingY: '0.625rem',
        fontSize: '1rem'       // 16px
    }
} as const

// ============================================
// INPUT SHAPE TOKENS
// ============================================

export const inputShape = {
    borderRadius: primitiveRadius.md,
    borderWidth: '1px',

    /** Focus ring */
    focusRingWidth: '2px',
    focusRingOffset: '2px'
} as const

// ============================================
// INPUT MOTION TOKENS
// ============================================

export const inputMotion = {
    transition: semanticTransitions.colors,

    /** Disabled state */
    disabledOpacity: '0.5'
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const inputTokens = {
    colors: inputColors,
    sizes: inputSizes,
    shape: inputShape,
    motion: inputMotion
} as const

export type InputVariant = keyof typeof inputColors
export type InputSize = keyof typeof inputSizes

export default inputTokens
