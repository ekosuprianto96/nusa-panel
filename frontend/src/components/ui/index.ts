/**
 * UI Components - Barrel Export
 * 
 * Single entry point untuk semua UI components
 * 
 * @example
 * import { Button, Input, Card, Badge } from '@/components/ui'
 */

// Button
export { Button, buttonVariants, type ButtonVariants, buttonVariantOptions } from './button'

// Input
export { Input, inputVariants, type InputVariants, inputVariantOptions } from './input'

// Card
export {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
    CardFooter
} from './card'

// Badge
export { Badge, badgeVariants, type BadgeVariants, badgeVariantOptions } from './badge'

// Textarea
export { Textarea } from './textarea'

// Legacy exports - untuk backward compatibility
export { default as BaseModal } from './BaseModal.vue'
export { default as Label } from './Label.vue'
export { default as ThemeToggle } from './ThemeToggle.vue'
export { default as Toast } from './Toast.vue'
export { default as ToastContainer } from './ToastContainer.vue'
