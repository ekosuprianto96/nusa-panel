/**
 * Class Name Utilities
 * 
 * Utility functions untuk menggabungkan class names dengan smart merging
 * Menggunakan clsx untuk conditional classes dan tailwind-merge untuk conflict resolution
 */

import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * Combine class names with intelligent Tailwind class merging
 * 
 * @example
 * cn('px-4 py-2', 'px-6') // returns 'py-2 px-6'
 * cn('bg-red-500', condition && 'bg-blue-500') // conditional classes
 * cn({ 'text-red-500': hasError, 'text-green-500': isSuccess })
 */
export function cn(...inputs: ClassValue[]): string {
    return twMerge(clsx(inputs))
}

/**
 * Create a variant class resolver
 * Helper untuk membuat class resolver dengan base classes
 * 
 * @example
 * const buttonClasses = createClassResolver('inline-flex items-center')
 * buttonClasses('px-4 py-2', isPrimary && 'bg-blue-500')
 */
export function createClassResolver(...baseClasses: ClassValue[]) {
    return (...inputs: ClassValue[]): string => {
        return cn(...baseClasses, ...inputs)
    }
}

/**
 * Conditional class helper
 * Shorthand untuk conditional class application
 * 
 * @example
 * conditionalClass(isActive, 'bg-blue-500', 'bg-gray-200')
 */
export function conditionalClass(
    condition: boolean,
    trueClass: string,
    falseClass: string = ''
): string {
    return condition ? trueClass : falseClass
}

/**
 * Map object of conditions to classes
 * 
 * @example
 * mapClasses({
 *   'text-red-500': hasError,
 *   'text-green-500': isSuccess,
 *   'opacity-50': isDisabled
 * })
 */
export function mapClasses(classMap: Record<string, boolean>): string {
    return Object.entries(classMap)
        .filter(([, condition]) => condition)
        .map(([className]) => className)
        .join(' ')
}

export default cn
