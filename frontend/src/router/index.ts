import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            redirect: '/dashboard',
        },
        {
            path: '/auth/login',
            name: 'Login',
            meta: { guest: true },
            component: () => import('@/pages/auth/LoginPage.vue'),
        },
        {
            path: '/dashboard',
            name: 'Dashboard',
            meta: { requiresAuth: true },
            component: () => import('@/pages/dashboard/DashboardPage.vue'),
        },
        {
            path: '/file-manager',
            redirect: '/dashboard/files',
        },
        {
            path: '/dashboard/files',
            name: 'FileManager',
            meta: {
                requiresAuth: true,
                title: 'File Manager',
                description: 'Upload or manage files',
                icon: '📄',
                color: 'blue',
                link: '/dashboard/files',
            },
            component: () => import('@/pages/files/FileManagerPage.vue'),
        },
        {
            path: '/dashboard/domains',
            name: 'Domains',
            meta: {
                requiresAuth: true,
                title: 'Domains',
                description: 'Manage your domains',
                icon: '🌐',
                color: 'blue',
                link: '/dashboard/domains',
            },
            component: () => import('@/pages/domains/DomainsPage.vue'),
        },
        {
            path: '/dashboard/databases',
            name: 'Databases',
            meta: {
                requiresAuth: true,
                title: 'Databases',
                description: 'Manage MySQL databases',
                icon: '💾',
                color: 'blue',
                link: '/dashboard/databases',
            },
            component: () => import('@/pages/databases/DatabasesPage.vue'),
        },
        {
            path: '/dashboard/emails',
            name: 'Email',
            meta: {
                requiresAuth: true,
                title: 'Email',
                description: 'Manage email accounts',
                icon: '✉️',
                color: 'blue',
                link: '/dashboard/emails',
            },
            component: () => import('@/pages/email/EmailPage.vue'),
        },
        {
            path: '/dashboard/security',
            name: 'Security',
            meta: {
                requiresAuth: true,
                title: 'Security',
                description: 'Manage firewall and SSH',
                icon: '🛡️',
                color: 'blue',
                link: '/dashboard/security',
            },
            component: () => import('@/pages/security/SecurityPage.vue'),
        },
        {
            path: '/dashboard/system',
            name: 'System',
            meta: {
                requiresAuth: true,
                title: 'System',
                description: 'System tools and logs',
                icon: '⚙️',
                color: 'blue',
                link: '/dashboard/system',
            },
            component: () => import('@/pages/system/SystemPage.vue'),
        },
        {
            path: '/dashboard/redis',
            name: 'Redis',
            meta: {
                requiresAuth: true,
                title: 'Redis Manager',
                description: 'Manage Redis instances',
                icon: '⚡',
                color: 'red',
                link: '/dashboard/redis',
            },
            component: () => import('@/pages/redis/RedisPage.vue'),
        },
        {
            path: '/dashboard/web-server',
            name: 'WebServer',
            meta: {
                requiresAuth: true,
                title: 'Web Server',
                description: 'Nginx/Apache Manager',
                icon: '🌐',
                color: 'green',
                link: '/dashboard/web-server',
            },
            component: () => import('@/pages/web_server/WebServerPage.vue'),
        },
        {
            path: '/dashboard/apps',
            name: 'AppInstaller',
            meta: {
                requiresAuth: true,
                title: 'App Installer',
                description: 'Softaculous-like installer',
                icon: '📱',
                color: 'purple',
                link: '/dashboard/apps',
            },
            component: () => import('@/pages/apps/AppInstallerPage.vue'),
        },
    ],
});

router.beforeEach(async (to, _from, next) => {
    // const isAuthenticated = true
    const isAuthenticated = !!localStorage.getItem('access_token');

    if (to.meta.requiresAuth && !isAuthenticated) {
        next('/auth/login');
    } else if (to.meta.guest && isAuthenticated) {
        next('/dashboard');
    } else {
        next();
    }
});

export default router;
