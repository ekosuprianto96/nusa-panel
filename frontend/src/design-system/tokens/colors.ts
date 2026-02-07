/**
 * Color Tokens
 * 
 * Definisi warna yang di-derive dari theme.config.ts
 * Menyediakan akses programmatic ke color values
 */

import { themeConfig, type SemanticColors, type ComponentColors } from '../theme.config'

export type ColorMode = 'light' | 'dark'

/**
 * Get color value untuk mode tertentu
 */
export function getColorValue(
    colorKey: keyof SemanticColors,
    mode: ColorMode = 'light'
): string {
    const color = themeConfig.colors[colorKey]
    return color[mode]
}

/**
 * Get component color value
 */
export function getComponentColorValue(
    component: keyof ComponentColors,
    property: string,
    mode: ColorMode = 'light'
): string {
    const componentConfig = themeConfig.componentColors[component] as Record<string, { light: string; dark: string }>
    if (componentConfig && componentConfig[property]) {
        return componentConfig[property][mode]
    }
    return ''
}

/**
 * Generate CSS color variable name
 */
export function getColorCSSVar(colorKey: string): string {
    // Convert camelCase to kebab-case
    return `--${colorKey.replace(/([A-Z])/g, '-$1').toLowerCase()}`
}

/**
 * Color palette untuk referensi
 * Ini adalah raw HSL values yang bisa digunakan di mana saja
 */
export const colorPalette = {
    // Blues
    blue: {
        50: '214 100% 97%',
        100: '214 95% 93%',
        200: '213 97% 87%',
        300: '212 96% 78%',
        400: '213 94% 68%',
        500: '217 91% 60%',
        600: '221 83% 53%',
        700: '224 76% 48%',
        800: '226 71% 40%',
        900: '224 64% 33%',
        950: '226 57% 21%'
    },

    // Grays (Slate)
    slate: {
        50: '210 40% 98%',
        100: '210 40% 96%',
        200: '214 32% 91%',
        300: '213 27% 84%',
        400: '215 20% 65%',
        500: '215 16% 47%',
        600: '215 19% 35%',
        700: '217 33% 17%',
        800: '222 47% 11%',
        900: '222 47% 8%',
        950: '222 47% 6%'
    },

    // Reds
    red: {
        50: '0 86% 97%',
        100: '0 93% 94%',
        200: '0 96% 89%',
        300: '0 94% 82%',
        400: '0 91% 71%',
        500: '0 84% 60%',
        600: '0 72% 51%',
        700: '0 74% 42%',
        800: '0 70% 35%',
        900: '0 63% 31%',
        950: '0 75% 15%'
    },

    // Greens
    green: {
        50: '138 76% 97%',
        100: '141 84% 93%',
        200: '141 79% 85%',
        300: '142 77% 73%',
        400: '142 69% 58%',
        500: '142 71% 45%',
        600: '142 76% 36%',
        700: '142 72% 29%',
        800: '143 64% 24%',
        900: '144 61% 20%',
        950: '145 80% 10%'
    },

    // Yellows/Ambers
    amber: {
        50: '48 100% 96%',
        100: '48 96% 89%',
        200: '48 97% 77%',
        300: '46 97% 65%',
        400: '43 96% 56%',
        500: '38 92% 50%',
        600: '32 95% 44%',
        700: '26 90% 37%',
        800: '23 83% 31%',
        900: '22 78% 26%',
        950: '21 92% 14%'
    },

    // Cyans
    cyan: {
        50: '183 100% 96%',
        100: '185 96% 90%',
        200: '186 94% 82%',
        300: '187 92% 69%',
        400: '188 86% 53%',
        500: '189 94% 43%',
        600: '192 91% 36%',
        700: '193 82% 31%',
        800: '194 70% 27%',
        900: '196 64% 24%',
        950: '197 79% 15%'
    }
} as const

export type ColorPalette = typeof colorPalette
export type ColorPaletteKey = keyof ColorPalette
export type ColorShade = keyof ColorPalette['blue']

/**
 * Helper untuk mendapatkan warna dari palette
 */
export function getPaletteColor(color: ColorPaletteKey, shade: ColorShade): string {
    return colorPalette[color][shade]
}

export default {
    getColorValue,
    getComponentColorValue,
    getColorCSSVar,
    colorPalette,
    getPaletteColor
}
