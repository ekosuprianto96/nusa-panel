/**
 * Design System - Main Public API
 * 
 * Single entry point untuk import semua design system utilities
 * 
 * @example
 * import { cn, useTheme, themeConfig } from '@/design-system'
 * import { Button, Input, Card } from '@/design-system/components'
 */

// Theme configuration
export {
    themeConfig,
    customThemes,
    type ThemeConfig,
    type ThemeName,
    type SemanticColors,
    type ComponentColors,
    type ColorToken,
    type TypographyTokens,
    type SpacingTokens,
    type RadiusTokens,
    type ShadowTokens,
    type TransitionTokens,
    type CustomTheme
} from './theme.config'

// Tokens
export * from './tokens'

// Composables
export {
    useTheme,
    useThemeSingleton,
    useDesignTokens,
    designTokens,
    type UseThemeReturn,
    type ColorMode,
    type UseDesignTokensReturn
} from './composables'

// Utilities
export { cn, createClassResolver, conditionalClass, mapClasses } from './utils/cn'
export {
    generateThemeCSS,
    generateCustomThemeCSS,
    generateAllCustomThemesCSS,
    applyThemeVariables,
    resetThemeVariables,
    getCSSVariable,
    setCSSVariable
} from './utils/generateCSS'
