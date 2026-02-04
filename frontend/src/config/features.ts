import {
    Folder, Database, Globe, Mail, Shield,
    FileText, Image as ImageIcon, Lock,
    Save, Cloud, ChevronRight, Settings2,
    Activity, MailPlus, Zap, ShieldCheck,
    ShieldAlert, HardDrive, Plus,
    Terminal, Server
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

export const features: DashboardSection[] = [
    {
        title: 'Files',
        icon: Folder,
        cards: [
            { title: 'File Manager', icon: FileText, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/files', description: 'Explore files' },
            { title: 'Images', icon: ImageIcon, color: 'text-orange-600 dark:text-orange-400', bgLight: 'bg-orange-50', bgDark: 'dark:bg-orange-500/10', link: '/dashboard/files', description: 'Optimize assets' },
            { title: 'Directory Privacy', icon: Lock, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/files', description: 'Secure folders' },
            { title: 'Disk Usage', icon: HardDrive, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/system', description: 'View storage' },
            { title: 'FTP Accounts', icon: Cloud, color: 'text-purple-600 dark:text-purple-400', bgLight: 'bg-purple-50', bgDark: 'dark:bg-purple-500/10', link: '/dashboard/files', description: 'FTP management' },
            { title: 'Backup Wizard', icon: Save, color: 'text-slate-600 dark:text-slate-400', bgLight: 'bg-slate-50', bgDark: 'dark:bg-slate-500/10', link: '/dashboard/system', description: 'Save your data' },
        ]
    },
    {
        title: 'Databases',
        icon: Database,
        cards: [
            { title: 'phpMyAdmin', icon: Settings2, color: 'text-orange-600 dark:text-orange-400', bgLight: 'bg-orange-50', bgDark: 'dark:bg-orange-500/10', description: 'Database UI' },
            { title: 'MySQL Databases', icon: Database, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/databases', description: 'Manage DBs' },
            { title: 'Redis Manager', icon: Zap, color: 'text-red-600 dark:text-red-400', bgLight: 'bg-red-50', bgDark: 'dark:bg-red-500/10', link: '/dashboard/redis', description: 'In-memory cache' },
        ]
    },
    {
        title: 'Domains',
        icon: Globe,
        cards: [
            { title: 'Domains', icon: Globe, color: 'text-blue-600 dark:text-blue-400', bgLight: 'bg-blue-50', bgDark: 'dark:bg-blue-500/10', link: '/dashboard/domains', description: 'Main domains' },
            { title: 'Addon Domains', icon: Plus, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/domains', description: 'Extra domains' },
            { title: 'Subdomains', icon: Globe, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/domains', description: 'Sub-level webs' },
            { title: 'Redirects', icon: ChevronRight, color: 'text-slate-600 dark:text-slate-400', bgLight: 'bg-slate-50', bgDark: 'dark:bg-slate-500/10', link: '/dashboard/domains', description: 'URL forwarding' },
        ]
    },
    {
        title: 'Email',
        icon: Mail,
        cards: [
            { title: 'Email Accounts', icon: MailPlus, color: 'text-pink-600 dark:text-pink-400', bgLight: 'bg-pink-50', bgDark: 'dark:bg-pink-500/10', link: '/dashboard/emails', description: 'Professional mail' },
            { title: 'Forwarders', icon: ChevronRight, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/emails', description: 'Mail routing' },
            { title: 'Autoresponders', icon: Zap, color: 'text-amber-600 dark:text-amber-400', bgLight: 'bg-amber-50', bgDark: 'dark:bg-amber-500/10', link: '/dashboard/emails', description: 'Auto reply' },
        ]
    },
    {
        title: 'Security',
        icon: Shield,
        cards: [
            { title: 'SSH Access', icon: Terminal, color: 'text-slate-600 dark:text-slate-400', bgLight: 'bg-slate-50', bgDark: 'dark:bg-slate-500/10', link: '/dashboard/security', description: 'Terminal keys' },
            { title: 'IP Blocker', icon: ShieldAlert, color: 'text-red-600 dark:text-red-400', bgLight: 'bg-red-50', bgDark: 'dark:bg-red-500/10', link: '/dashboard/security', description: 'Deny access' },
            { title: 'SSL/TLS', icon: ShieldCheck, color: 'text-emerald-600 dark:text-emerald-400', bgLight: 'bg-emerald-50', bgDark: 'dark:bg-emerald-500/10', link: '/dashboard/security', description: 'Security certs' },
        ]
    },
    {
        title: 'Software',
        icon: Server,
        cards: [
            { title: 'App Installer', icon: Cloud, color: 'text-purple-600 dark:text-purple-400', bgLight: 'bg-purple-50', bgDark: 'dark:bg-purple-500/10', link: '/dashboard/apps', description: 'One-click install' },
            { title: 'Web Server', icon: Activity, color: 'text-green-600 dark:text-green-400', bgLight: 'bg-green-50', bgDark: 'dark:bg-green-500/10', link: '/dashboard/web-server', description: 'Nginx & PHP' },
            { title: 'PHP Selector', icon: Settings2, color: 'text-indigo-600 dark:text-indigo-400', bgLight: 'bg-indigo-50', bgDark: 'dark:bg-indigo-500/10', link: '/dashboard/system', description: 'Change version' },
        ]
    }
]
