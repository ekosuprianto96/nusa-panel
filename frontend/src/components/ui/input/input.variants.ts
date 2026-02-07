/**
 * Input Variants - CVA Configuration
 */

import { cva, type VariantProps } from 'class-variance-authority'

export const inputVariants = cva(
    // Base styles
    [
        'flex w-full rounded-md border bg-background px-3 py-2',
        'text-base ring-offset-background',
        'file:border-0 file:bg-transparent file:text-sm file:font-medium',
        'placeholder:text-muted-foreground',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'md:text-sm'
    ],
    {
        variants: {
            /**
             * Visual variant
             */
            variant: {
                default: 'border-input',
                error: 'border-destructive focus-visible:ring-destructive',
                success: 'border-success focus-visible:ring-success'
            },
            /**
             * Size variant
             */
            size: {
                default: 'h-10',
                sm: 'h-9 text-xs',
                lg: 'h-11 text-base'
            }
        },
        defaultVariants: {
            variant: 'default',
            size: 'default'
        }
    }
)

export type InputVariants = VariantProps<typeof inputVariants>

export const inputVariantOptions = {
    variant: ['default', 'error', 'success'] as const,
    size: ['default', 'sm', 'lg'] as const
}

export default inputVariants
