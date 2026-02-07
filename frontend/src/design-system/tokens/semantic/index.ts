/**
 * Semantic Tokens - Barrel Export
 * 
 * Level 1 tokens - purpose-driven values
 * These map primitives to meanings
 * 
 * @layer semantic
 */

export {
    semanticColors,
    semanticBackground,
    semanticForeground,
    semanticIntent,
    semanticInteractive,
    getSemanticColor,
    type ColorMode,
    type SemanticColorValue
} from './colors'

export {
    semanticSpacing,
    semanticLayoutSpacing,
    semanticComponentSpacing,
    semanticElementSpacing
} from './spacing'

export {
    semanticTypography,
    semanticTextStyles,
    getTextStyle,
    type TextStyle,
    type TextStyleKey
} from './typography'

export {
    semanticMotion,
    semanticTransitions,
    semanticAnimations,
    reducedMotionPresets
} from './motion'
