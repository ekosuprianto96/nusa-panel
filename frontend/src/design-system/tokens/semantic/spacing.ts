/**
 * Semantic Spacing Tokens (Level 1)
 * 
 * Purpose-driven spacing assignments.
 * 
 * @layer semantic
 */

import { primitiveSpacing } from '../primitives/spacing'

// ============================================
// LAYOUT SPACING
// ============================================

export const semanticLayoutSpacing = {
    /** Page horizontal padding - responsive */
    pageInline: {
        sm: primitiveSpacing[4],   // 16px on mobile
        md: primitiveSpacing[6],   // 24px on tablet
        lg: primitiveSpacing[8]    // 32px on desktop
    },

    /** Page vertical padding */
    pageBlock: primitiveSpacing[6],

    /** Section gap (between major sections) */
    sectionGap: primitiveSpacing[12],

    /** Content gap (between content blocks) */
    contentGap: primitiveSpacing[8],

    /** Container max width */
    containerMax: '1400px'
} as const

// ============================================
// COMPONENT SPACING
// ============================================

export const semanticComponentSpacing = {
    /** Card internal padding */
    cardPadding: primitiveSpacing[6],

    /** Card header/content/footer gap */
    cardGap: primitiveSpacing[4],

    /** Modal padding */
    modalPadding: primitiveSpacing[6],

    /** Form field gap */
    formGap: primitiveSpacing[4],

    /** Form group gap (related fields) */
    formGroupGap: primitiveSpacing[6],

    /** Button group gap */
    buttonGroupGap: primitiveSpacing[2],

    /** Stack gap (generic vertical) */
    stackGap: {
        xs: primitiveSpacing[1],
        sm: primitiveSpacing[2],
        md: primitiveSpacing[4],
        lg: primitiveSpacing[6],
        xl: primitiveSpacing[8]
    },

    /** Inline gap (generic horizontal) */
    inlineGap: {
        xs: primitiveSpacing[1],
        sm: primitiveSpacing[2],
        md: primitiveSpacing[3],
        lg: primitiveSpacing[4],
        xl: primitiveSpacing[6]
    }
} as const

// ============================================
// ELEMENT SPACING
// ============================================

export const semanticElementSpacing = {
    /** Icon spacing from text */
    iconGap: primitiveSpacing[2],

    /** Badge/tag internal padding */
    badgePaddingX: primitiveSpacing[2.5],
    badgePaddingY: primitiveSpacing[0.5],

    /** Input internal padding */
    inputPaddingX: primitiveSpacing[3],
    inputPaddingY: primitiveSpacing[2],

    /** Button internal padding by size */
    buttonPadding: {
        sm: { x: primitiveSpacing[3], y: primitiveSpacing[1.5] },
        md: { x: primitiveSpacing[4], y: primitiveSpacing[2] },
        lg: { x: primitiveSpacing[6], y: primitiveSpacing[2.5] },
        xl: { x: primitiveSpacing[8], y: primitiveSpacing[3] }
    },

    /** Tooltip padding */
    tooltipPadding: {
        x: primitiveSpacing[3],
        y: primitiveSpacing[1.5]
    }
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const semanticSpacing = {
    layout: semanticLayoutSpacing,
    component: semanticComponentSpacing,
    element: semanticElementSpacing
} as const

export default semanticSpacing
