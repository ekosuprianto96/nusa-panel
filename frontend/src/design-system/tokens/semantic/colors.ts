/**
 * Semantic Color Tokens (Level 1)
 * 
 * Purpose-driven color assignments.
 * Maps primitive colors to semantic meanings.
 * 
 * @layer semantic
 */

import { primitiveColors } from '../primitives/colors'

// ============================================
// TYPES
// ============================================

export type ColorMode = 'light' | 'dark'

export interface SemanticColorValue {
    light: string
    dark: string
}

// ============================================
// BACKGROUND & FOREGROUND
// ============================================

export const semanticBackground: Record<string, SemanticColorValue> = {
    /** Page background */
    default: {
        light: '210 20% 98%',  // Very light gray
        dark: primitiveColors.slate[950]
    },

    /** Alternative background for sections */
    subtle: {
        light: primitiveColors.slate[100],
        dark: primitiveColors.slate[900]
    },

    /** Card/elevated surface background */
    surface: {
        light: primitiveColors.white,
        dark: primitiveColors.slate[900]
    },

    /** Elevated surface (modals, popovers) */
    elevated: {
        light: primitiveColors.white,
        dark: primitiveColors.slate[800]
    },

    /** Overlay/backdrop */
    overlay: {
        light: primitiveColors.black,
        dark: primitiveColors.black
    }
}

export const semanticForeground: Record<string, SemanticColorValue> = {
    /** Primary text color */
    default: {
        light: primitiveColors.slate[900],
        dark: primitiveColors.slate[50]
    },

    /** Secondary/muted text */
    muted: {
        light: primitiveColors.slate[500],
        dark: primitiveColors.slate[400]
    },

    /** Subtle/disabled text */
    subtle: {
        light: primitiveColors.slate[400],
        dark: primitiveColors.slate[500]
    },

    /** Inverted text (on dark backgrounds) */
    inverted: {
        light: primitiveColors.slate[50],
        dark: primitiveColors.slate[900]
    }
}

// ============================================
// INTENT COLORS
// ============================================

export const semanticIntent: Record<string, SemanticColorValue> = {
    // Primary - main brand/action color
    primary: {
        light: primitiveColors.blue[500],
        dark: primitiveColors.blue[500]
    },
    primaryHover: {
        light: primitiveColors.blue[600],
        dark: primitiveColors.blue[400]
    },
    primaryActive: {
        light: primitiveColors.blue[700],
        dark: primitiveColors.blue[300]
    },
    primaryForeground: {
        light: primitiveColors.white,
        dark: primitiveColors.slate[900]
    },

    // Secondary - alternative actions
    secondary: {
        light: primitiveColors.slate[100],
        dark: primitiveColors.slate[800]
    },
    secondaryHover: {
        light: primitiveColors.slate[200],
        dark: primitiveColors.slate[700]
    },
    secondaryForeground: {
        light: primitiveColors.slate[900],
        dark: primitiveColors.slate[50]
    },

    // Destructive - dangerous actions
    destructive: {
        light: primitiveColors.red[500],
        dark: primitiveColors.red[900]
    },
    destructiveHover: {
        light: primitiveColors.red[600],
        dark: primitiveColors.red[800]
    },
    destructiveForeground: {
        light: primitiveColors.white,
        dark: primitiveColors.red[50]
    },

    // Success - positive outcomes
    success: {
        light: primitiveColors.green[500],
        dark: primitiveColors.green[600]
    },
    successHover: {
        light: primitiveColors.green[600],
        dark: primitiveColors.green[500]
    },
    successForeground: {
        light: primitiveColors.white,
        dark: primitiveColors.white
    },

    // Warning - caution states
    warning: {
        light: primitiveColors.amber[500],
        dark: primitiveColors.amber[500]
    },
    warningHover: {
        light: primitiveColors.amber[600],
        dark: primitiveColors.amber[400]
    },
    warningForeground: {
        light: primitiveColors.black,
        dark: primitiveColors.black
    },

    // Info - informational states
    info: {
        light: primitiveColors.cyan[500],
        dark: primitiveColors.cyan[500]
    },
    infoHover: {
        light: primitiveColors.cyan[600],
        dark: primitiveColors.cyan[400]
    },
    infoForeground: {
        light: primitiveColors.white,
        dark: primitiveColors.white
    }
}

// ============================================
// INTERACTIVE COLORS
// ============================================

export const semanticInteractive: Record<string, SemanticColorValue> = {
    /** Border color */
    border: {
        light: primitiveColors.slate[200],
        dark: primitiveColors.slate[700]
    },

    /** Input border */
    borderInput: {
        light: primitiveColors.slate[300],
        dark: primitiveColors.slate[600]
    },

    /** Focus ring */
    ring: {
        light: primitiveColors.blue[500],
        dark: primitiveColors.blue[400]
    },

    /** Accent/highlight */
    accent: {
        light: primitiveColors.slate[100],
        dark: primitiveColors.slate[800]
    },
    accentForeground: {
        light: primitiveColors.slate[900],
        dark: primitiveColors.slate[50]
    },

    /** Muted backgrounds */
    muted: {
        light: primitiveColors.slate[100],
        dark: primitiveColors.slate[800]
    },
    mutedForeground: {
        light: primitiveColors.slate[500],
        dark: primitiveColors.slate[400]
    }
}

// ============================================
// HELPER FUNCTIONS
// ============================================

/**
 * Get semantic color value for a mode
 */
export function getSemanticColor(
    category: 'background' | 'foreground' | 'intent' | 'interactive',
    name: string,
    mode: ColorMode = 'light'
): string {
    const map = {
        background: semanticBackground,
        foreground: semanticForeground,
        intent: semanticIntent,
        interactive: semanticInteractive
    }

    const color = map[category][name]
    return color ? color[mode] : ''
}

// ============================================
// COMBINED EXPORT
// ============================================

export const semanticColors = {
    background: semanticBackground,
    foreground: semanticForeground,
    intent: semanticIntent,
    interactive: semanticInteractive
} as const

export default semanticColors
