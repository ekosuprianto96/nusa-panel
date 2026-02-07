/**
 * Primitive Typography Tokens (Level 0)
 * 
 * Raw typography values - font families, sizes, weights.
 * 
 * @layer primitives
 */

// ============================================
// FONT FAMILIES
// ============================================

export const primitiveFontFamily = {
    sans: [
        'Inter',
        'ui-sans-serif',
        'system-ui',
        '-apple-system',
        'BlinkMacSystemFont',
        '"Segoe UI"',
        'Roboto',
        '"Helvetica Neue"',
        'Arial',
        '"Noto Sans"',
        'sans-serif'
    ].join(', '),

    mono: [
        'ui-monospace',
        'SFMono-Regular',
        '"SF Mono"',
        'Menlo',
        'Consolas',
        '"Liberation Mono"',
        '"Courier New"',
        'monospace'
    ].join(', '),

    display: [
        'Inter',
        'ui-sans-serif',
        'system-ui'
    ].join(', ')
} as const

// ============================================
// FONT SIZES (Type Scale)
// ============================================

export const primitiveFontSize = {
    xs: '0.75rem',      // 12px
    sm: '0.875rem',     // 14px
    base: '1rem',       // 16px
    lg: '1.125rem',     // 18px
    xl: '1.25rem',      // 20px
    '2xl': '1.5rem',    // 24px
    '3xl': '1.875rem',  // 30px
    '4xl': '2.25rem',   // 36px
    '5xl': '3rem',      // 48px
    '6xl': '3.75rem',   // 60px
    '7xl': '4.5rem',    // 72px
    '8xl': '6rem',      // 96px
    '9xl': '8rem'       // 128px
} as const

// ============================================
// FONT WEIGHTS
// ============================================

export const primitiveFontWeight = {
    thin: '100',
    extralight: '200',
    light: '300',
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
    extrabold: '800',
    black: '900'
} as const

// ============================================
// LINE HEIGHTS
// ============================================

export const primitiveLineHeight = {
    none: '1',
    tight: '1.25',
    snug: '1.375',
    normal: '1.5',
    relaxed: '1.625',
    loose: '2',
    // Specific sizes
    3: '0.75rem',   // 12px
    4: '1rem',      // 16px
    5: '1.25rem',   // 20px
    6: '1.5rem',    // 24px
    7: '1.75rem',   // 28px
    8: '2rem',      // 32px
    9: '2.25rem',   // 36px
    10: '2.5rem'    // 40px
} as const

// ============================================
// LETTER SPACING
// ============================================

export const primitiveLetterSpacing = {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0em',
    wide: '0.025em',
    wider: '0.05em',
    widest: '0.1em'
} as const

// ============================================
// EXPORTS
// ============================================

export const primitiveTypography = {
    fontFamily: primitiveFontFamily,
    fontSize: primitiveFontSize,
    fontWeight: primitiveFontWeight,
    lineHeight: primitiveLineHeight,
    letterSpacing: primitiveLetterSpacing
} as const

export type FontSizeKey = keyof typeof primitiveFontSize
export type FontWeightKey = keyof typeof primitiveFontWeight
export type LineHeightKey = keyof typeof primitiveLineHeight
export type LetterSpacingKey = keyof typeof primitiveLetterSpacing

export default primitiveTypography
