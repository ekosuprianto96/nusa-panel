/**
 * 🎨 Theme Configuration - Single Source of Truth
 * 
 * Semua design tokens didefinisikan di sini.
 * Mengubah nilai di sini akan otomatis update semua components.
 * 
 * Format warna menggunakan HSL untuk flexibility:
 * - Mudah untuk membuat variants (lighter/darker)
 * - Mendukung opacity dengan hsl(var(--color) / 0.5)
 */

export type ThemeName = 'light' | 'dark' | string

// ===========================================
// COLOR TOKENS
// ===========================================

export interface ColorToken {
    light: string  // HSL values tanpa hsl() wrapper, contoh: "217 91% 60%"
    dark: string
}

export interface SemanticColors {
    background: ColorToken
    foreground: ColorToken
    card: ColorToken
    cardForeground: ColorToken
    popover: ColorToken
    popoverForeground: ColorToken
    primary: ColorToken
    primaryForeground: ColorToken
    secondary: ColorToken
    secondaryForeground: ColorToken
    muted: ColorToken
    mutedForeground: ColorToken
    accent: ColorToken
    accentForeground: ColorToken
    destructive: ColorToken
    destructiveForeground: ColorToken
    success: ColorToken
    successForeground: ColorToken
    warning: ColorToken
    warningForeground: ColorToken
    info: ColorToken
    infoForeground: ColorToken
    border: ColorToken
    input: ColorToken
    ring: ColorToken
}

export interface ComponentColors {
    sidebar: {
        background: ColorToken
        border: ColorToken
        icon: ColorToken
        iconActive: ColorToken
        activeBackground: ColorToken
    }
    table: {
        headerBackground: ColorToken
        rowHover: ColorToken
        rowSelected: ColorToken
        border: ColorToken
    }
    panel: {
        background: ColorToken
        border: ColorToken
    }
}

// ===========================================
// TYPOGRAPHY TOKENS
// ===========================================

export interface TypographyTokens {
    fontFamily: {
        sans: string
        mono: string
        display: string
    }
    fontSize: {
        xs: string
        sm: string
        base: string
        lg: string
        xl: string
        '2xl': string
        '3xl': string
        '4xl': string
    }
    fontWeight: {
        normal: string
        medium: string
        semibold: string
        bold: string
    }
    lineHeight: {
        none: string
        tight: string
        snug: string
        normal: string
        relaxed: string
        loose: string
    }
    letterSpacing: {
        tighter: string
        tight: string
        normal: string
        wide: string
        wider: string
    }
}

// ===========================================
// SPACING TOKENS
// ===========================================

export interface SpacingTokens {
    0: string
    0.5: string
    1: string
    1.5: string
    2: string
    2.5: string
    3: string
    3.5: string
    4: string
    5: string
    6: string
    7: string
    8: string
    9: string
    10: string
    11: string
    12: string
    14: string
    16: string
    20: string
    24: string
    28: string
    32: string
    36: string
    40: string
    44: string
    48: string
    52: string
    56: string
    60: string
    64: string
    72: string
    80: string
    96: string
}

// ===========================================
// RADIUS TOKENS
// ===========================================

export interface RadiusTokens {
    none: string
    sm: string
    DEFAULT: string
    md: string
    lg: string
    xl: string
    '2xl': string
    '3xl': string
    full: string
}

// ===========================================
// SHADOW TOKENS
// ===========================================

export interface ShadowTokens {
    none: string
    sm: string
    DEFAULT: string
    md: string
    lg: string
    xl: string
    '2xl': string
    inner: string
}

// ===========================================
// TRANSITION TOKENS
// ===========================================

export interface TransitionTokens {
    duration: {
        fastest: string
        fast: string
        normal: string
        slow: string
        slowest: string
    }
    timing: {
        linear: string
        in: string
        out: string
        inOut: string
        bounce: string
    }
}

// ===========================================
// COMPLETE THEME CONFIG TYPE
// ===========================================

export interface ThemeConfig {
    colors: SemanticColors
    componentColors: ComponentColors
    typography: TypographyTokens
    spacing: SpacingTokens
    radius: RadiusTokens
    shadows: ShadowTokens
    transitions: TransitionTokens
}

// ===========================================
// DEFAULT THEME CONFIGURATION
// ===========================================

export const themeConfig: ThemeConfig = {
    // =========================================
    // SEMANTIC COLORS
    // =========================================
    colors: {
        // Core backgrounds
        background: {
            light: '210 20% 98%',
            dark: '222 47% 6%'
        },
        foreground: {
            light: '222 47% 11%',
            dark: '210 40% 98%'
        },

        // Card surfaces
        card: {
            light: '0 0% 100%',
            dark: '222 47% 8%'
        },
        cardForeground: {
            light: '222 47% 11%',
            dark: '210 40% 98%'
        },

        // Popover/Dropdown surfaces
        popover: {
            light: '0 0% 100%',
            dark: '222 47% 8%'
        },
        popoverForeground: {
            light: '222 47% 11%',
            dark: '210 40% 98%'
        },

        // Primary - Main brand color (Blue)
        primary: {
            light: '217 91% 60%',
            dark: '217 91% 60%'
        },
        primaryForeground: {
            light: '210 40% 98%',
            dark: '222 47% 11%'
        },

        // Secondary - Subtle backgrounds
        secondary: {
            light: '210 40% 96%',
            dark: '217 33% 15%'
        },
        secondaryForeground: {
            light: '222 47% 11%',
            dark: '210 40% 98%'
        },

        // Muted - Even more subtle
        muted: {
            light: '210 40% 96%',
            dark: '217 33% 15%'
        },
        mutedForeground: {
            light: '215 16% 47%',
            dark: '215 20% 65%'
        },

        // Accent - Hover states, highlights
        accent: {
            light: '210 40% 96%',
            dark: '217 33% 15%'
        },
        accentForeground: {
            light: '222 47% 11%',
            dark: '210 40% 98%'
        },

        // Destructive - Errors, delete actions
        destructive: {
            light: '0 84% 60%',
            dark: '0 63% 31%'
        },
        destructiveForeground: {
            light: '0 0% 98%',
            dark: '210 40% 98%'
        },

        // Success - Positive actions, confirmations
        success: {
            light: '142 71% 45%',
            dark: '142 69% 40%'
        },
        successForeground: {
            light: '0 0% 100%',
            dark: '0 0% 100%'
        },

        // Warning - Caution, pending states
        warning: {
            light: '38 92% 50%',
            dark: '38 92% 45%'
        },
        warningForeground: {
            light: '0 0% 0%',
            dark: '0 0% 0%'
        },

        // Info - Informational messages
        info: {
            light: '199 89% 48%',
            dark: '199 89% 48%'
        },
        infoForeground: {
            light: '0 0% 100%',
            dark: '0 0% 100%'
        },

        // Borders & inputs
        border: {
            light: '214 32% 91%',
            dark: '217 33% 17%'
        },
        input: {
            light: '214 32% 91%',
            dark: '217 33% 17%'
        },
        ring: {
            light: '217 91% 60%',
            dark: '224 76% 48%'
        }
    },

    // =========================================
    // COMPONENT-SPECIFIC COLORS
    // =========================================
    componentColors: {
        sidebar: {
            background: {
                light: '0 0% 100%',
                dark: '222 47% 8%'
            },
            border: {
                light: '214 32% 91%',
                dark: '217 33% 17%'
            },
            icon: {
                light: '215 16% 47%',
                dark: '215 20% 55%'
            },
            iconActive: {
                light: '217 91% 60%',
                dark: '217 91% 60%'
            },
            activeBackground: {
                light: '217 91% 97%',
                dark: '217 33% 15%'
            }
        },
        table: {
            headerBackground: {
                light: '0 0% 100%',
                dark: '222 47% 8%'
            },
            rowHover: {
                light: '217 91% 97%',
                dark: '217 33% 13%'
            },
            rowSelected: {
                light: '217 91% 95%',
                dark: '217 40% 18%'
            },
            border: {
                light: '214 32% 95%',
                dark: '217 33% 14%'
            }
        },
        panel: {
            background: {
                light: '0 0% 100%',
                dark: '222 47% 8%'
            },
            border: {
                light: '214 32% 91%',
                dark: '217 33% 17%'
            }
        }
    },

    // =========================================
    // TYPOGRAPHY
    // =========================================
    typography: {
        fontFamily: {
            sans: 'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
            mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace',
            display: 'Inter, ui-sans-serif, system-ui'
        },
        fontSize: {
            xs: '0.75rem',      // 12px
            sm: '0.875rem',     // 14px
            base: '1rem',       // 16px
            lg: '1.125rem',     // 18px
            xl: '1.25rem',      // 20px
            '2xl': '1.5rem',    // 24px
            '3xl': '1.875rem',  // 30px
            '4xl': '2.25rem'    // 36px
        },
        fontWeight: {
            normal: '400',
            medium: '500',
            semibold: '600',
            bold: '700'
        },
        lineHeight: {
            none: '1',
            tight: '1.25',
            snug: '1.375',
            normal: '1.5',
            relaxed: '1.625',
            loose: '2'
        },
        letterSpacing: {
            tighter: '-0.05em',
            tight: '-0.025em',
            normal: '0em',
            wide: '0.025em',
            wider: '0.05em'
        }
    },

    // =========================================
    // SPACING (Tailwind-compatible scale)
    // =========================================
    spacing: {
        0: '0px',
        0.5: '0.125rem',   // 2px
        1: '0.25rem',      // 4px
        1.5: '0.375rem',   // 6px
        2: '0.5rem',       // 8px
        2.5: '0.625rem',   // 10px
        3: '0.75rem',      // 12px
        3.5: '0.875rem',   // 14px
        4: '1rem',         // 16px
        5: '1.25rem',      // 20px
        6: '1.5rem',       // 24px
        7: '1.75rem',      // 28px
        8: '2rem',         // 32px
        9: '2.25rem',      // 36px
        10: '2.5rem',      // 40px
        11: '2.75rem',     // 44px
        12: '3rem',        // 48px
        14: '3.5rem',      // 56px
        16: '4rem',        // 64px
        20: '5rem',        // 80px
        24: '6rem',        // 96px
        28: '7rem',        // 112px
        32: '8rem',        // 128px
        36: '9rem',        // 144px
        40: '10rem',       // 160px
        44: '11rem',       // 176px
        48: '12rem',       // 192px
        52: '13rem',       // 208px
        56: '14rem',       // 224px
        60: '15rem',       // 240px
        64: '16rem',       // 256px
        72: '18rem',       // 288px
        80: '20rem',       // 320px
        96: '24rem'        // 384px
    },

    // =========================================
    // BORDER RADIUS
    // =========================================
    radius: {
        none: '0px',
        sm: '0.125rem',    // 2px
        DEFAULT: '0.25rem', // 4px
        md: '0.375rem',    // 6px
        lg: '0.5rem',      // 8px
        xl: '0.75rem',     // 12px
        '2xl': '1rem',     // 16px
        '3xl': '1.5rem',   // 24px
        full: '9999px'
    },

    // =========================================
    // BOX SHADOWS
    // =========================================
    shadows: {
        none: 'none',
        sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
        DEFAULT: '0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)',
        md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
        lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
        xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
        '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
        inner: 'inset 0 2px 4px 0 rgb(0 0 0 / 0.05)'
    },

    // =========================================
    // TRANSITIONS
    // =========================================
    transitions: {
        duration: {
            fastest: '75ms',
            fast: '150ms',
            normal: '300ms',
            slow: '500ms',
            slowest: '700ms'
        },
        timing: {
            linear: 'linear',
            in: 'cubic-bezier(0.4, 0, 1, 1)',
            out: 'cubic-bezier(0, 0, 0.2, 1)',
            inOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
            bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)'
        }
    }
}

// ===========================================
// CUSTOM THEME REGISTRY
// ===========================================

export interface CustomTheme {
    name: string
    colors: Partial<SemanticColors>
    componentColors?: Partial<ComponentColors>
}

/**
 * Registry untuk custom themes
 * Anda bisa menambahkan custom theme di sini
 */
export const customThemes: Record<string, CustomTheme> = {
    // Contoh custom theme
    ocean: {
        name: 'Ocean',
        colors: {
            primary: {
                light: '199 89% 48%',
                dark: '199 89% 48%'
            },
            accent: {
                light: '180 100% 25%',
                dark: '180 100% 30%'
            }
        }
    },
    sunset: {
        name: 'Sunset',
        colors: {
            primary: {
                light: '25 95% 53%',
                dark: '25 95% 53%'
            },
            accent: {
                light: '330 80% 55%',
                dark: '330 80% 55%'
            }
        }
    }
}

export default themeConfig
