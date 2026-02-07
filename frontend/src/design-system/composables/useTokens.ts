/**
 * useTokens Composable
 * 
 * Provides reactive access to the entire token system.
 * Use this for programmatic access to design tokens.
 * 
 * @example
 * const { primitives, semantic, components, getToken } = useTokens()
 * const primaryColor = getToken('intent.primary', 'light')
 */

import { computed, type ComputedRef } from 'vue'

// Primitives
import {
    primitiveColors,
    primitiveSpacing,
    primitiveTypography,
    primitiveShape,
    primitiveMotion
} from '../tokens/primitives'

// Semantic
import {
    semanticColors,
    semanticSpacing,
    semanticTypography,
    semanticMotion,
    type ColorMode
} from '../tokens/semantic'

// Components
import {
    buttonTokens,
    inputTokens,
    cardTokens,
    badgeTokens
} from '../tokens/components'

// ============================================
// TYPES
// ============================================

export interface TokenSystem {
    primitives: {
        colors: typeof primitiveColors
        spacing: typeof primitiveSpacing
        typography: typeof primitiveTypography
        shape: typeof primitiveShape
        motion: typeof primitiveMotion
    }
    semantic: {
        colors: typeof semanticColors
        spacing: typeof semanticSpacing
        typography: typeof semanticTypography
        motion: typeof semanticMotion
    }
    components: {
        button: typeof buttonTokens
        input: typeof inputTokens
        card: typeof cardTokens
        badge: typeof badgeTokens
    }
}

// ============================================
// COMPOSABLE
// ============================================

export function useTokens() {
    // ----------------------------------------
    // TOKEN SYSTEM
    // ----------------------------------------

    const tokens: ComputedRef<TokenSystem> = computed(() => ({
        primitives: {
            colors: primitiveColors,
            spacing: primitiveSpacing,
            typography: primitiveTypography,
            shape: primitiveShape,
            motion: primitiveMotion
        },
        semantic: {
            colors: semanticColors,
            spacing: semanticSpacing,
            typography: semanticTypography,
            motion: semanticMotion
        },
        components: {
            button: buttonTokens,
            input: inputTokens,
            card: cardTokens,
            badge: badgeTokens
        }
    }))

    // ----------------------------------------
    // PRIMITIVE HELPERS
    // ----------------------------------------

    /**
     * Get primitive color value
     */
    function getPrimitiveColor(
        scale: keyof typeof primitiveColors,
        shade?: number
    ): string {
        const color = primitiveColors[scale]
        if (typeof color === 'string') return color

        const shadeKey = (shade || 500) as keyof typeof color
        return color[shadeKey] ?? color[500]
    }

    /**
     * Get primitive spacing value
     */
    function getPrimitiveSpacing(key: keyof typeof primitiveSpacing): string {
        return primitiveSpacing[key]
    }

    // ----------------------------------------
    // SEMANTIC HELPERS
    // ----------------------------------------

    /**
     * Get semantic color for current/specified mode
     */
    function getSemanticColor(
        category: 'background' | 'foreground' | 'intent' | 'interactive',
        name: string,
        mode: ColorMode = 'light'
    ): string {
        const categoryMap = semanticColors[category]
        const color = categoryMap[name]
        return color ? color[mode] : ''
    }

    // ----------------------------------------
    // COMPONENT HELPERS
    // ----------------------------------------

    /**
     * Get button token
     */
    function getButtonToken<K extends keyof typeof buttonTokens>(
        category: K
    ): (typeof buttonTokens)[K] {
        return buttonTokens[category]
    }

    /**
     * Get input token
     */
    function getInputToken<K extends keyof typeof inputTokens>(
        category: K
    ): (typeof inputTokens)[K] {
        return inputTokens[category]
    }

    // ----------------------------------------
    // CSS VARIABLE HELPERS
    // ----------------------------------------

    /**
     * Generate CSS variable reference
     */
    function cssVar(name: string, fallback?: string): string {
        return fallback
            ? `var(--${name}, ${fallback})`
            : `var(--${name})`
    }

    /**
     * Generate HSL color from CSS variable
     */
    function hslVar(name: string): string {
        return `hsl(var(--${name}))`
    }

    // ----------------------------------------
    // RETURN
    // ----------------------------------------

    return {
        // Full token system
        tokens,

        // Direct access
        primitives: tokens.value.primitives,
        semantic: tokens.value.semantic,
        components: tokens.value.components,

        // Helpers
        getPrimitiveColor,
        getPrimitiveSpacing,
        getSemanticColor,
        getButtonToken,
        getInputToken,
        cssVar,
        hslVar
    }
}

export type UseTokensReturn = ReturnType<typeof useTokens>

export default useTokens
