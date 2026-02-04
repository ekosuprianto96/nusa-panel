import {
    Users, Package, Activity, Server
} from 'lucide-vue-next'

export interface DashboardCard {
    title: string
    icon: any
    color: string
    bgLight: string
    bgDark: string
    link?: string
    description?: string
}

export interface DashboardSection {
    title: string
    icon: any
    cards: DashboardCard[]
}

export const adminFeatures: DashboardSection[] = [
    {
        title: 'Administration',
        icon: Users,
        cards: [
            { title: 'Users', icon: Users, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/users', description: 'Manage accounts' },
            { title: 'Packages', icon: Package, color: 'text-amber-600 dark:text-amber-400', bgLight: 'bg-amber-50', bgDark: 'dark:bg-amber-500/10', link: '/dashboard/packages', description: 'Plan settings' },
        ]
    },
    {
        title: 'Monitoring',
        icon: Activity,
        cards: [
            { title: 'Web Server', icon: Server, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/web-server', description: 'Nginx/Apache' },
        ]
    }
]
