/**
 * Semantic Motion Tokens (Level 1)
 * 
 * Purpose-driven animation assignments.
 * 
 * @layer semantic
 */

import { primitiveDuration, primitiveEasing } from '../primitives/motion'

// ============================================
// TRANSITION PRESETS
// ============================================

export const semanticTransitions = {
    /** Quick feedback (hover states) */
    fast: `${primitiveDuration[150]} ${primitiveEasing.inOut}`,

    /** Standard transitions */
    normal: `${primitiveDuration[300]} ${primitiveEasing.inOut}`,

    /** Slow, deliberate transitions */
    slow: `${primitiveDuration[500]} ${primitiveEasing.inOut}`,

    /** Instant (no transition) */
    none: 'none',

    // ----------------------------------------
    // SPECIFIC USE CASES
    // ----------------------------------------

    /** Color changes (background, border, text) */
    colors: `color ${primitiveDuration[150]} ${primitiveEasing.inOut}, background-color ${primitiveDuration[150]} ${primitiveEasing.inOut}, border-color ${primitiveDuration[150]} ${primitiveEasing.inOut}`,

    /** Opacity changes */
    opacity: `opacity ${primitiveDuration[200]} ${primitiveEasing.inOut}`,

    /** Transform changes */
    transform: `transform ${primitiveDuration[200]} ${primitiveEasing.inOut}`,

    /** Shadow changes */
    shadow: `box-shadow ${primitiveDuration[200]} ${primitiveEasing.inOut}`,

    /** All properties */
    all: `all ${primitiveDuration[200]} ${primitiveEasing.inOut}`,

    /** Bounce effect */
    bounce: `all ${primitiveDuration[300]} ${primitiveEasing.bounce}`,

    /** Spring effect */
    spring: `all ${primitiveDuration[500]} ${primitiveEasing.spring}`
} as const

// ============================================
// ANIMATION PRESETS
// ============================================

export const semanticAnimations = {
    /** Fade in */
    fadeIn: {
        keyframes: 'fadeIn',
        duration: primitiveDuration[200],
        easing: primitiveEasing.out
    },

    /** Fade out */
    fadeOut: {
        keyframes: 'fadeOut',
        duration: primitiveDuration[150],
        easing: primitiveEasing.in
    },

    /** Slide in from top (dropdowns) */
    slideDown: {
        keyframes: 'slideInUp',
        duration: primitiveDuration[200],
        easing: primitiveEasing.out
    },

    /** Slide in from bottom (modals, toasts) */
    slideUp: {
        keyframes: 'slideInDown',
        duration: primitiveDuration[300],
        easing: primitiveEasing.out
    },

    /** Scale in (popovers, tooltips) */
    scaleIn: {
        keyframes: 'scaleIn',
        duration: primitiveDuration[200],
        easing: primitiveEasing.out
    },

    /** Scale out */
    scaleOut: {
        keyframes: 'scaleOut',
        duration: primitiveDuration[150],
        easing: primitiveEasing.in
    },

    /** Loading spinner */
    spin: {
        keyframes: 'spin',
        duration: primitiveDuration[1000],
        easing: primitiveEasing.linear,
        iteration: 'infinite'
    },

    /** Pulse effect */
    pulse: {
        keyframes: 'pulse',
        duration: primitiveDuration[1000],
        easing: primitiveEasing.inOut,
        iteration: 'infinite'
    }
} as const

// ============================================
// REDUCED MOTION
// ============================================

export const reducedMotionPresets = {
    /** No animation */
    none: 'none',

    /** Minimal fade only */
    fade: `opacity ${primitiveDuration[100]} ${primitiveEasing.linear}`
} as const

// ============================================
// COMBINED EXPORT
// ============================================

export const semanticMotion = {
    transitions: semanticTransitions,
    animations: semanticAnimations,
    reducedMotion: reducedMotionPresets
} as const

export default semanticMotion
