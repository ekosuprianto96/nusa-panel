/**
 * CSS Variables Generator
 * 
 * Utility untuk generate CSS variables dari theme config
 * Bisa digunakan untuk build-time generation atau runtime switching
 */

import {
    themeConfig,
    customThemes,
    type ThemeConfig,
    type SemanticColors,
    type ComponentColors,
    type CustomTheme,
    type ThemeName
} from '../theme.config'

type ColorMode = 'light' | 'dark'

/**
 * Convert camelCase ke kebab-case
 */
function toKebabCase(str: string): string {
    return str.replace(/([A-Z])/g, '-$1').toLowerCase()
}

/**
 * Generate CSS variables untuk semantic colors
 */
function generateColorVariables(colors: SemanticColors, mode: ColorMode): string {
    const lines: string[] = []

    for (const [key, value] of Object.entries(colors)) {
        const varName = toKebabCase(key)
        const colorValue = value[mode]
        lines.push(`  --${varName}: ${colorValue};`)
    }

    return lines.join('\n')
}

/**
 * Generate CSS variables untuk component colors
 */
function generateComponentColorVariables(componentColors: ComponentColors, mode: ColorMode): string {
    const lines: string[] = []

    for (const [component, properties] of Object.entries(componentColors)) {
        for (const [property, value] of Object.entries(properties as Record<string, { light: string; dark: string }>)) {
            const varName = `${component}-${toKebabCase(property)}`
            lines.push(`  --${varName}: ${value[mode]};`)
        }
    }

    return lines.join('\n')
}

/**
 * Generate CSS variables untuk non-color tokens
 */
function generateStaticTokenVariables(config: ThemeConfig): string {
    const lines: string[] = []

    // Radius
    lines.push('\n  /* Border Radius */')
    lines.push(`  --radius: ${config.radius.lg};`)
    lines.push(`  --radius-sm: ${config.radius.sm};`)
    lines.push(`  --radius-md: ${config.radius.md};`)
    lines.push(`  --radius-lg: ${config.radius.lg};`)
    lines.push(`  --radius-xl: ${config.radius.xl};`)
    lines.push(`  --radius-full: ${config.radius.full};`)

    // Typography
    lines.push('\n  /* Typography */')
    lines.push(`  --font-sans: ${config.typography.fontFamily.sans};`)
    lines.push(`  --font-mono: ${config.typography.fontFamily.mono};`)

    // Transitions
    lines.push('\n  /* Transitions */')
    lines.push(`  --transition-fast: ${config.transitions.duration.fast};`)
    lines.push(`  --transition-normal: ${config.transitions.duration.normal};`)
    lines.push(`  --transition-slow: ${config.transitions.duration.slow};`)

    return lines.join('\n')
}

/**
 * Generate complete CSS file dengan light dan dark themes
 */
export function generateThemeCSS(config: ThemeConfig = themeConfig): string {
    const lightColors = generateColorVariables(config.colors, 'light')
    const lightComponentColors = generateComponentColorVariables(config.componentColors, 'light')
    const darkColors = generateColorVariables(config.colors, 'dark')
    const darkComponentColors = generateComponentColorVariables(config.componentColors, 'dark')
    const staticTokens = generateStaticTokenVariables(config)

    return `/**
 * Design System CSS Variables
 * 
 * Auto-generated from theme.config.ts
 * DO NOT EDIT MANUALLY - Edit theme.config.ts instead
 */

:root {
  /* ============================================
     LIGHT THEME
     ============================================ */
  
  /* Semantic Colors */
${lightColors}

  /* Component Colors */
${lightComponentColors}

  /* Static Tokens */
${staticTokens}
}

.dark {
  /* ============================================
     DARK THEME
     ============================================ */
  
  /* Semantic Colors */
${darkColors}

  /* Component Colors */
${darkComponentColors}
}
`
}

/**
 * Generate CSS untuk custom theme
 * Custom theme hanya override warna yang berbeda
 */
export function generateCustomThemeCSS(
    themeName: string,
    customTheme: CustomTheme,
    mode: ColorMode = 'light'
): string {
    const lines: string[] = []

    if (customTheme.colors) {
        for (const [key, value] of Object.entries(customTheme.colors)) {
            if (value) {
                const varName = toKebabCase(key)
                lines.push(`  --${varName}: ${value[mode]};`)
            }
        }
    }

    const selector = mode === 'dark' ? `.theme-${themeName}.dark` : `.theme-${themeName}`

    return `${selector} {
${lines.join('\n')}
}`
}

/**
 * Generate all custom themes CSS
 */
export function generateAllCustomThemesCSS(): string {
    const sections: string[] = []

    for (const [themeName, theme] of Object.entries(customThemes)) {
        sections.push(`/* Theme: ${theme.name} */`)
        sections.push(generateCustomThemeCSS(themeName, theme, 'light'))
        sections.push(generateCustomThemeCSS(themeName, theme, 'dark'))
        sections.push('')
    }

    return sections.join('\n')
}

/**
 * Apply theme at runtime dengan mengubah CSS variables
 */
export function applyThemeVariables(
    themeName: ThemeName,
    mode: ColorMode = 'light',
    element: HTMLElement = document.documentElement
): void {
    const config = themeName === 'light' || themeName === 'dark'
        ? themeConfig
        : customThemes[themeName]
            ? { ...themeConfig, colors: { ...themeConfig.colors, ...customThemes[themeName].colors } }
            : themeConfig

    // Apply semantic colors
    for (const [key, value] of Object.entries(config.colors)) {
        const varName = toKebabCase(key)
        const colorValue = (value as { light: string; dark: string })[mode]
        element.style.setProperty(`--${varName}`, colorValue)
    }

    // Apply component colors
    for (const [component, properties] of Object.entries(config.componentColors)) {
        for (const [property, value] of Object.entries(properties as Record<string, { light: string; dark: string }>)) {
            const varName = `${component}-${toKebabCase(property)}`
            element.style.setProperty(`--${varName}`, value[mode])
        }
    }
}

/**
 * Reset theme ke default
 */
export function resetThemeVariables(element: HTMLElement = document.documentElement): void {
    // Remove all inline styles yang kita set
    const properties = [
        ...Object.keys(themeConfig.colors).map(k => `--${toKebabCase(k)}`),
        ...Object.entries(themeConfig.componentColors).flatMap(([comp, props]) =>
            Object.keys(props as object).map(p => `--${comp}-${toKebabCase(p)}`)
        )
    ]

    for (const prop of properties) {
        element.style.removeProperty(prop)
    }
}

/**
 * Get CSS variable value
 */
export function getCSSVariable(
    varName: string,
    element: HTMLElement = document.documentElement
): string {
    return getComputedStyle(element).getPropertyValue(varName).trim()
}

/**
 * Set CSS variable
 */
export function setCSSVariable(
    varName: string,
    value: string,
    element: HTMLElement = document.documentElement
): void {
    element.style.setProperty(varName, value)
}

export default {
    generateThemeCSS,
    generateCustomThemeCSS,
    generateAllCustomThemesCSS,
    applyThemeVariables,
    resetThemeVariables,
    getCSSVariable,
    setCSSVariable
}
