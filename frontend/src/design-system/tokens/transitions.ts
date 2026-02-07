/**
 * Transition Tokens
 * 
 * Animation durations dan easing functions
 */

import { themeConfig } from '../theme.config'

export const transitions = themeConfig.transitions
export const duration = transitions.duration
export const timing = transitions.timing

/**
 * Pre-built transition presets
 * Format: property duration timing-function
 */
export const transitionPresets = {
    // All properties
    all: `all ${duration.normal} ${timing.inOut}`,
    allFast: `all ${duration.fast} ${timing.inOut}`,

    // Colors only
    colors: `color ${duration.normal} ${timing.inOut}, background-color ${duration.normal} ${timing.inOut}, border-color ${duration.normal} ${timing.inOut}`,
    colorsFast: `color ${duration.fast} ${timing.inOut}, background-color ${duration.fast} ${timing.inOut}, border-color ${duration.fast} ${timing.inOut}`,

    // Opacity/visibility
    opacity: `opacity ${duration.normal} ${timing.inOut}`,
    opacityFast: `opacity ${duration.fast} ${timing.inOut}`,

    // Transforms
    transform: `transform ${duration.normal} ${timing.inOut}`,
    transformFast: `transform ${duration.fast} ${timing.inOut}`,
    transformBounce: `transform ${duration.normal} ${timing.bounce}`,

    // Size changes
    size: `width ${duration.normal} ${timing.inOut}, height ${duration.normal} ${timing.inOut}`,

    // Shadow
    shadow: `box-shadow ${duration.normal} ${timing.inOut}`,
    shadowFast: `box-shadow ${duration.fast} ${timing.inOut}`,

    // Common UI patterns
    button: `background-color ${duration.fast} ${timing.inOut}, border-color ${duration.fast} ${timing.inOut}, box-shadow ${duration.fast} ${timing.inOut}, transform ${duration.fast} ${timing.inOut}`,
    input: `border-color ${duration.fast} ${timing.inOut}, box-shadow ${duration.fast} ${timing.inOut}`,
    link: `color ${duration.fast} ${timing.inOut}, text-decoration-color ${duration.fast} ${timing.inOut}`,
    card: `transform ${duration.normal} ${timing.inOut}, box-shadow ${duration.normal} ${timing.inOut}`,

    // Modal/overlay
    modal: `opacity ${duration.normal} ${timing.out}, transform ${duration.normal} ${timing.out}`,
    overlay: `opacity ${duration.normal} ${timing.inOut}`,

    // Accordion/collapse
    collapse: `grid-template-rows ${duration.normal} ${timing.inOut}`,

    // None
    none: 'none'
} as const

/**
 * Animation keyframe presets (CSS animation names)
 */
export const animations = {
    // Fade animations
    fadeIn: 'fadeIn',
    fadeOut: 'fadeOut',

    // Slide animations
    slideInFromTop: 'slideInFromTop',
    slideInFromBottom: 'slideInFromBottom',
    slideInFromLeft: 'slideInFromLeft',
    slideInFromRight: 'slideInFromRight',

    // Scale animations
    scaleIn: 'scaleIn',
    scaleOut: 'scaleOut',

    // Bounce
    bounce: 'bounce',

    // Pulse
    pulse: 'pulse',

    // Spin
    spin: 'spin',

    // Ping (attention)
    ping: 'ping'
} as const

/**
 * Keyframes CSS - yang perlu ditambahkan ke CSS
 */
export const keyframesCSS = `
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fadeOut {
  from { opacity: 1; }
  to { opacity: 0; }
}

@keyframes slideInFromTop {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes slideInFromBottom {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes slideInFromLeft {
  from { opacity: 0; transform: translateX(-10px); }
  to { opacity: 1; transform: translateX(0); }
}

@keyframes slideInFromRight {
  from { opacity: 0; transform: translateX(10px); }
  to { opacity: 1; transform: translateX(0); }
}

@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes scaleOut {
  from { opacity: 1; transform: scale(1); }
  to { opacity: 0; transform: scale(0.95); }
}

@keyframes bounce {
  0%, 100% { transform: translateY(-5%); }
  50% { transform: translateY(0); }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes ping {
  75%, 100% { transform: scale(2); opacity: 0; }
}
`

/**
 * Helper untuk membuat transition string
 */
export function createTransition(
    property: string | string[],
    durationKey: keyof typeof duration = 'normal',
    timingKey: keyof typeof timing = 'inOut'
): string {
    const props = Array.isArray(property) ? property : [property]
    return props
        .map(p => `${p} ${duration[durationKey]} ${timing[timingKey]}`)
        .join(', ')
}

/**
 * Reduced motion helper
 * Mengembalikan value yang aman untuk users yang prefer reduced motion
 */
export function getReducedMotionTransition(normalValue: string): {
    normal: string
    reduced: string
} {
    return {
        normal: normalValue,
        reduced: 'none'
    }
}

export default {
    duration,
    timing,
    transitionPresets,
    animations,
    keyframesCSS,
    createTransition,
    getReducedMotionTransition
}
