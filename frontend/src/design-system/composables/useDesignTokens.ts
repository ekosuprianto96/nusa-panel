/**
 * useDesignTokens Composable
 * 
 * Provides reactive access to design tokens dalam Vue components
 * Useful untuk dynamic styling dan computed styles
 */

import { computed, type ComputedRef } from 'vue'
import { themeConfig, type ThemeConfig } from '../theme.config'
import * as tokens from '../tokens'
import { useTheme } from './useTheme'
import { getCSSVariable } from '../utils/generateCSS'

// ============================================
// TYPES
// ============================================

export interface UseDesignTokensReturn {
    // Direct token access
    tokens: typeof tokens
    config: ThemeConfig

    // Color helpers
    getColor: (colorKey: keyof typeof themeConfig.colors) => ComputedRef<string>
    getHslColor: (colorKey: keyof typeof themeConfig.colors) => ComputedRef<string>

    // CSS Variable helpers
    getCSSVar: (varName: string) => string
    cssVar: (varName: string) => string

    // Typography
    getFontSize: (size: keyof typeof themeConfig.typography.fontSize) => string
    getFontFamily: (family: keyof typeof themeConfig.typography.fontFamily) => string

    // Spacing
    getSpacing: (key: keyof typeof themeConfig.spacing) => string

    // Radius
    getRadius: (key: keyof typeof themeConfig.radius) => string

    // Shadows
    getShadow: (key: keyof typeof themeConfig.shadows) => string

    // Transitions
    getTransition: (preset: keyof typeof tokens.semanticTransitions) => string
}

// ============================================
// COMPOSABLE
// ============================================

/**
 * useDesignTokens composable
 * 
 * @example
 * const { getColor, getSpacing, cssVar } = useDesignTokens()
 * 
 * // Get color as computed ref (reactive to theme changes)
 * const primaryColor = getColor('primary')
 * 
 * // Get CSS variable value
 * const bgColor = cssVar('--background')
 * 
 * // Get spacing value
 * const padding = getSpacing(4) // '1rem'
 */
export function useDesignTokens(): UseDesignTokensReturn {
    const { colorMode } = useTheme()

    // ========================================
    // COLOR HELPERS
    // ========================================

    /**
     * Get color value as computed ref
     * Reactive terhadap perubahan color mode
     */
    function getColor(colorKey: keyof typeof themeConfig.colors): ComputedRef<string> {
        return computed(() => {
            const color = themeConfig.colors[colorKey]
            return color[colorMode.value]
        })
    }

    /**
     * Get color as HSL string
     */
    function getHslColor(colorKey: keyof typeof themeConfig.colors): ComputedRef<string> {
        return computed(() => {
            const color = themeConfig.colors[colorKey]
            return `hsl(${color[colorMode.value]})`
        })
    }

    // ========================================
    // CSS VARIABLE HELPERS
    // ========================================

    /**
     * Get CSS variable value from DOM
     */
    function getCSSVar(varName: string): string {
        if (typeof document === 'undefined') return ''
        return getCSSVariable(varName)
    }

    /**
     * Shorthand untuk membuat CSS var reference
     */
    function cssVar(varName: string): string {
        // Ensure var name starts with --
        const name = varName.startsWith('--') ? varName : `--${varName}`
        return `var(${name})`
    }

    // ========================================
    // TOKEN ACCESSORS
    // ========================================

    function getFontSize(size: keyof typeof themeConfig.typography.fontSize): string {
        return themeConfig.typography.fontSize[size]
    }

    function getFontFamily(family: keyof typeof themeConfig.typography.fontFamily): string {
        return themeConfig.typography.fontFamily[family]
    }

    function getSpacing(key: keyof typeof themeConfig.spacing): string {
        return themeConfig.spacing[key]
    }

    function getRadius(key: keyof typeof themeConfig.radius): string {
        return themeConfig.radius[key]
    }

    function getShadow(key: keyof typeof themeConfig.shadows): string {
        return themeConfig.shadows[key]
    }

    function getTransition(preset: keyof typeof tokens.semanticTransitions): string {
        return tokens.semanticTransitions[preset]
    }

    // ========================================
    // RETURN
    // ========================================

    return {
        // Direct access
        tokens,
        config: themeConfig,

        // Color helpers
        getColor,
        getHslColor,

        // CSS Variable helpers
        getCSSVar,
        cssVar,

        // Token accessors
        getFontSize,
        getFontFamily,
        getSpacing,
        getRadius,
        getShadow,
        getTransition
    }
}

// ============================================
// STANDALONE TOKEN ACCESS
// ============================================

/**
 * Direct token access tanpa composable
 * Untuk digunakan diluar setup() atau di file non-component
 */
export const designTokens = {
    colors: tokens.primitiveColors,
    typography: tokens.semanticTextStyles,
    spacing: tokens.primitiveSpacing,
    radius: tokens.primitiveRadius,
    shadows: tokens.primitiveShadow,
    transitions: tokens.semanticTransitions
}

export default useDesignTokens
