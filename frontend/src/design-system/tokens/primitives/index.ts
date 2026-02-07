/**
 * Primitive Tokens - Barrel Export
 * 
 * Level 0 tokens - raw design values
 * These are NEVER used directly in components
 * 
 * @layer primitives
 */

export {
    primitiveColors,
    getPrimitiveColor,
    type ColorScale,
    type ColorShade
} from './colors'

export {
    primitiveSpacing,
    getSpacing,
    type SpacingKey
} from './spacing'

export {
    primitiveTypography,
    primitiveFontFamily,
    primitiveFontSize,
    primitiveFontWeight,
    primitiveLineHeight,
    primitiveLetterSpacing,
    type FontSizeKey,
    type FontWeightKey,
    type LineHeightKey,
    type LetterSpacingKey
} from './typography'

export {
    primitiveShape,
    primitiveRadius,
    primitiveBorderWidth,
    primitiveShadow,
    primitiveShadowDark,
    primitiveOpacity,
    primitiveZIndex,
    type RadiusKey,
    type ShadowKey,
    type ZIndexKey
} from './shape'

export {
    primitiveMotion,
    primitiveDuration,
    primitiveEasing,
    primitiveKeyframes,
    type DurationKey,
    type EasingKey,
    type KeyframeKey
} from './motion'
