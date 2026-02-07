/**
 * Button Variants - CVA Configuration
 * 
 * Menggunakan class-variance-authority untuk type-safe variant system
 * Dipisahkan dari component untuk reusability dan testing
 */

import { cva, type VariantProps } from 'class-variance-authority'

/**
 * Button variants configuration
 * 
 * @example
 * import { buttonVariants } from './button.variants'
 * 
 * // Get classes for specific variant
 * const classes = buttonVariants({ variant: 'destructive', size: 'lg' })
 */
export const buttonVariants = cva(
    // Base styles
    [
        'inline-flex items-center justify-center gap-2',
        'whitespace-nowrap rounded-md text-sm font-medium',
        'ring-offset-background transition-colors',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        'disabled:pointer-events-none disabled:opacity-50',
        '[&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0'
    ],
    {
        variants: {
            /**
             * Visual variant
             */
            variant: {
                default: 'bg-primary text-primary-foreground hover:bg-primary/90',
                destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
                success: 'bg-success text-success-foreground hover:bg-success/90',
                warning: 'bg-warning text-warning-foreground hover:bg-warning/90',
                outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
                secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
                ghost: 'hover:bg-accent hover:text-accent-foreground',
                link: 'text-primary underline-offset-4 hover:underline'
            },
            /**
             * Size variant
             */
            size: {
                default: 'h-10 px-4 py-2',
                sm: 'h-9 rounded-md px-3 text-xs',
                lg: 'h-11 rounded-md px-8',
                xl: 'h-12 rounded-lg px-10 text-base',
                icon: 'h-10 w-10'
            }
        },
        defaultVariants: {
            variant: 'default',
            size: 'default'
        }
    }
)

/**
 * Button variant props type
 */
export type ButtonVariants = VariantProps<typeof buttonVariants>

/**
 * Available variant options for documentation/autocomplete
 */
export const buttonVariantOptions = {
    variant: ['default', 'destructive', 'success', 'warning', 'outline', 'secondary', 'ghost', 'link'] as const,
    size: ['default', 'sm', 'lg', 'xl', 'icon'] as const
}

export default buttonVariants
