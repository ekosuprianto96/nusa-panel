/**
 * Primitive Motion Tokens (Level 0)
 * 
 * Raw values for transitions and animations.
 * 
 * @layer primitives
 */

// ============================================
// DURATIONS
// ============================================

export const primitiveDuration = {
    0: '0ms',
    75: '75ms',
    100: '100ms',
    150: '150ms',
    200: '200ms',
    300: '300ms',
    500: '500ms',
    700: '700ms',
    1000: '1000ms'
} as const

// ============================================
// TIMING FUNCTIONS (Easing)
// ============================================

export const primitiveEasing = {
    linear: 'linear',
    in: 'cubic-bezier(0.4, 0, 1, 1)',
    out: 'cubic-bezier(0, 0, 0.2, 1)',
    inOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
    spring: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)'
} as const

// ============================================
// KEYFRAMES (for CSS @keyframes)
// ============================================

export const primitiveKeyframes = {
    fadeIn: {
        from: { opacity: '0' },
        to: { opacity: '1' }
    },
    fadeOut: {
        from: { opacity: '1' },
        to: { opacity: '0' }
    },
    slideInUp: {
        from: { opacity: '0', transform: 'translateY(10px)' },
        to: { opacity: '1', transform: 'translateY(0)' }
    },
    slideInDown: {
        from: { opacity: '0', transform: 'translateY(-10px)' },
        to: { opacity: '1', transform: 'translateY(0)' }
    },
    slideInLeft: {
        from: { opacity: '0', transform: 'translateX(-10px)' },
        to: { opacity: '1', transform: 'translateX(0)' }
    },
    slideInRight: {
        from: { opacity: '0', transform: 'translateX(10px)' },
        to: { opacity: '1', transform: 'translateX(0)' }
    },
    scaleIn: {
        from: { opacity: '0', transform: 'scale(0.95)' },
        to: { opacity: '1', transform: 'scale(1)' }
    },
    scaleOut: {
        from: { opacity: '1', transform: 'scale(1)' },
        to: { opacity: '0', transform: 'scale(0.95)' }
    },
    spin: {
        from: { transform: 'rotate(0deg)' },
        to: { transform: 'rotate(360deg)' }
    },
    pulse: {
        '0%, 100%': { opacity: '1' },
        '50%': { opacity: '0.5' }
    },
    bounce: {
        '0%, 100%': { transform: 'translateY(-5%)' },
        '50%': { transform: 'translateY(0)' }
    }
} as const

// ============================================
// EXPORTS
// ============================================

export const primitiveMotion = {
    duration: primitiveDuration,
    easing: primitiveEasing,
    keyframes: primitiveKeyframes
} as const

export type DurationKey = keyof typeof primitiveDuration
export type EasingKey = keyof typeof primitiveEasing
export type KeyframeKey = keyof typeof primitiveKeyframes

export default primitiveMotion
