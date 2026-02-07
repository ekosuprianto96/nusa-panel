/**
 * Card Component Tokens (Level 2)
 * 
 * Component-specific token definitions for Card.
 * 
 * @layer components
 */

import { semanticComponentSpacing } from '../semantic/spacing'
import { semanticTransitions } from '../semantic/motion'
import { primitiveRadius, primitiveShadow } from '../primitives/shape'
import { primitiveColors } from '../primitives/colors'

// ============================================
// CARD COLOR TOKENS
// ============================================

export const cardColors = {
    bg: { light: primitiveColors.white, dark: primitiveColors.slate[900] },
    fg: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },
    border: { light: primitiveColors.slate[200], dark: primitiveColors.slate[700] },

    /** Title color */
    title: { light: primitiveColors.slate[900], dark: primitiveColors.slate[50] },

    /** Description/muted text */
    description: { light: primitiveColors.slate[500], dark: primitiveColors.slate[400] }
} as const

// ============================================
// CARD SPACING TOKENS
// ============================================

export const cardSpacing = {
    /** Card internal padding */
    padding: semanticComponentSpacing.cardPadding,

    /** Header padding */
    headerPadding: {
        top: semanticComponentSpacing.cardPadding,
        horizontal: semanticComponentSpacing.cardPadding,
        bottom: '0'
    },

    /** Content padding */
    contentPadding: {
        vertical: '0',
        horizontal: semanticComponentSpacing.cardPadding
    },

    /** Footer padding */
    footerPadding: {
        top: '0',
        horizontal: semanticComponentSpacing.cardPadding,
        bottom: semanticComponentSpacing.cardPadding
    },

    /** Gap between header elements */
    headerGap: '0.375rem',

    /** Gap between sections */
    sectionGap: semanticComponentSpacing.cardGap
} as const

// ============================================
// CARD SHAPE TOKENS
// ============================================

export const cardShape = {
    borderRadius: primitiveRadius.lg,
    borderWidth: '1px',

    /** Default shadow */
    shadow: primitiveShadow.sm,

    /** Hover shadow (for hoverable cards) */
    shadowHover: primitiveShadow.md
} as const

// ============================================
// CARD MOTION TOKENS
// ============================================

export const cardMotion = {
    transition: semanticTransitions.shadow
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const cardTokens = {
    colors: cardColors,
    spacing: cardSpacing,
    shape: cardShape,
    motion: cardMotion
} as const

export default cardTokens
