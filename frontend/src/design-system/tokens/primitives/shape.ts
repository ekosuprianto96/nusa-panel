/**
 * Primitive Shape Tokens (Level 0)
 * 
 * Raw values for radius, borders, shadows.
 * 
 * @layer primitives
 */

// ============================================
// BORDER RADIUS
// ============================================

export const primitiveRadius = {
    none: '0px',
    sm: '0.125rem',    // 2px
    DEFAULT: '0.25rem', // 4px
    md: '0.375rem',    // 6px
    lg: '0.5rem',      // 8px
    xl: '0.75rem',     // 12px
    '2xl': '1rem',     // 16px
    '3xl': '1.5rem',   // 24px
    full: '9999px'
} as const

// ============================================
// BORDER WIDTH
// ============================================

export const primitiveBorderWidth = {
    0: '0px',
    DEFAULT: '1px',
    2: '2px',
    4: '4px',
    8: '8px'
} as const

// ============================================
// BOX SHADOWS
// ============================================

export const primitiveShadow = {
    none: 'none',
    sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
    DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
    md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
    lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
    xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
    '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
    inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)'
} as const

// Dark mode shadows (more pronounced)
export const primitiveShadowDark = {
    none: 'none',
    sm: '0 1px 2px 0 rgb(0 0 0 / 0.2)',
    DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.3), 0 1px 2px -1px rgb(0 0 0 / 0.3)',
    md: '0 4px 6px -1px rgb(0 0 0 / 0.3), 0 2px 4px -2px rgb(0 0 0 / 0.3)',
    lg: '0 10px 15px -3px rgb(0 0 0 / 0.4), 0 4px 6px -4px rgb(0 0 0 / 0.3)',
    xl: '0 20px 25px -5px rgb(0 0 0 / 0.4), 0 8px 10px -6px rgb(0 0 0 / 0.3)',
    '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.5)',
    inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.2)'
} as const

// ============================================
// OPACITY
// ============================================

export const primitiveOpacity = {
    0: '0',
    5: '0.05',
    10: '0.1',
    20: '0.2',
    25: '0.25',
    30: '0.3',
    40: '0.4',
    50: '0.5',
    60: '0.6',
    70: '0.7',
    75: '0.75',
    80: '0.8',
    90: '0.9',
    95: '0.95',
    100: '1'
} as const

// ============================================
// Z-INDEX
// ============================================

export const primitiveZIndex = {
    auto: 'auto',
    0: '0',
    10: '10',
    20: '20',
    30: '30',
    40: '40',
    50: '50',
    // Named layers
    dropdown: '1000',
    sticky: '1020',
    fixed: '1030',
    modalBackdrop: '1040',
    modal: '1050',
    popover: '1060',
    tooltip: '1070',
    toast: '1080'
} as const

// ============================================
// EXPORTS
// ============================================

export const primitiveShape = {
    radius: primitiveRadius,
    borderWidth: primitiveBorderWidth,
    shadow: primitiveShadow,
    shadowDark: primitiveShadowDark,
    opacity: primitiveOpacity,
    zIndex: primitiveZIndex
} as const

export type RadiusKey = keyof typeof primitiveRadius
export type ShadowKey = keyof typeof primitiveShadow
export type ZIndexKey = keyof typeof primitiveZIndex

export default primitiveShape
