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
            path: '/admin',
            name: 'AdminDashboard',
            meta: { requiresAuth: true, requiresAdmin: true },
            component: () => import('@/pages/admin/AdminDashboardPage.vue'),
        },
        // User Management (Admin/Reseller only)
        {
            path: '/dashboard/users',
            name: 'Users',
            meta: {
                requiresAuth: true,
                requiresAdmin: true,
                title: 'User Management',
                description: 'Manage users and access',
                icon: '👥',
                color: 'purple',
            },
            component: () => import('@/pages/users/UsersPage.vue'),
        },
        {
            path: '/dashboard/users/:id',
            name: 'UserDetail',
            meta: {
                requiresAuth: true,
                requiresAdmin: true,
                title: 'User Detail',
                description: 'User account overview',
                icon: '👤',
                color: 'purple',
            },
            component: () => import('@/pages/users/UserDetailPage.vue'),
        },
        // Package Management (Admin only)
        {
            path: '/dashboard/packages',
            name: 'Packages',
            meta: {
                requiresAuth: true,
                requiresAdmin: true,
                title: 'Package Management',
                description: 'Hosting packages configuration',
                icon: '📦',
                color: 'orange',
            },
            component: () => import('@/pages/packages/PackagesPage.vue'),
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
            path: '/dashboard/files/upload',
            name: 'FileUpload',
            meta: {
                requiresAuth: true,
                title: 'Upload Files',
            },
            component: () => import('@/pages/files/UploadPage.vue'),
        },
        {
            path: '/dashboard/files/editor',
            name: 'TextEditor',
            meta: {
                requiresAuth: true,
                title: 'Text Editor',
            },
            component: () => import('@/pages/files/TextEditorPage.vue'),
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
            path: '/dashboard/security/ssl-tls',
            name: 'SslTls',
            meta: {
                requiresAuth: true,
                title: 'SSL/TLS Status',
            },
            component: () => import('@/pages/security/SslTlsPage.vue'),
        },
        {
            path: '/dashboard/security/ip-blocker',
            name: 'IpBlocker',
            meta: {
                requiresAuth: true,
                title: 'IP Blocker',
            },
            component: () => import('@/pages/security/IpBlockerPage.vue'),
        },
        {
            path: '/dashboard/security/ssh-access',
            name: 'SshAccess',
            meta: {
                requiresAuth: true,
                title: 'SSH Access',
            },
            component: () => import('@/pages/security/SshAccessPage.vue'),
        },
        {
            path: '/dashboard/security/two-factor-auth',
            name: 'TwoFactorAuth',
            meta: {
                requiresAuth: true,
                title: '2FA Setup',
            },
            component: () => import('@/pages/security/TwoFactorAuthPage.vue'),
        },
        {
            path: '/dashboard/security/mod-security',
            name: 'ModSecurity',
            meta: {
                requiresAuth: true,
                title: 'ModSecurity',
            },
            component: () => import('@/pages/security/ModSecurityPage.vue'),
        },
        {
            path: '/dashboard/system',
            name: 'System',
            meta: {
                requiresAuth: true,
                requiresAdmin: false,
                title: 'System',
                description: 'System tools and logs',
                icon: '⚙️',
                color: 'blue',
                link: '/dashboard/system',
            },
            component: () => import('@/pages/system/SystemPage.vue'),
        },
        {
            path: '/dashboard/system/cron-jobs',
            name: 'CronJobs',
            meta: { requiresAuth: true, title: 'Cron Jobs' },
            component: () => import('@/pages/system/CronJobsPage.vue'),
        },
        {
            path: '/dashboard/system/php-manager',
            name: 'PhpManager',
            meta: { requiresAuth: true, title: 'PHP Manager' },
            component: () => import('@/pages/system/PhpManagerPage.vue'),
        },
        {
            path: '/dashboard/system/process-manager',
            name: 'ProcessManager',
            meta: { requiresAuth: true, title: 'Process Manager' },
            component: () => import('@/pages/system/ProcessManagerPage.vue'),
        },
        {
            path: '/dashboard/system/dns-tracker',
            name: 'DnsTracker',
            meta: { requiresAuth: true, title: 'DNS Tracker' },
            component: () => import('@/pages/system/DnsTrackerPage.vue'),
        },
        {
            path: '/dashboard/system/npm-manager',
            name: 'NpmManager',
            meta: { requiresAuth: true, title: 'NPM Manager' },
            component: () => import('@/pages/system/NpmManagerPage.vue'),
        },
        {
            path: '/dashboard/system/error-logs',
            name: 'ErrorLogs',
            meta: { requiresAuth: true, title: 'Error Logs' },
            component: () => import('@/pages/system/ErrorLogsPage.vue'),
        },
        {
            path: '/dashboard/system/resource-usage',
            name: 'ResourceUsage',
            meta: { requiresAuth: true, title: 'Resource Usage' },
            component: () => import('@/pages/system/ResourceUsagePage.vue'),
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
                requiresAdmin: true,
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
    } else if (to.meta.requiresAdmin && isAuthenticated) {
        // Check for admin/reseller role
        const { useAuthStore } = await import('@/stores/auth');
        const authStore = useAuthStore();

        // Ensure user data is loaded
        if (!authStore.user && authStore.accessToken) {
            await authStore.fetchMe();
        }

        const userRole = authStore.user?.role || 'user';
        const allowedRoles = ['admin', 'reseller'];

        if (!allowedRoles.includes(userRole)) {
            // Redirect unauthorized users to dashboard
            next('/dashboard');
        } else {
            next();
        }
    } else {
        if (isAuthenticated) {
            const { useAuthStore } = await import('@/stores/auth');
            const authStore = useAuthStore();

            if (!authStore.user && authStore.accessToken) {
                await authStore.fetchMe();
            }

            const userRole = authStore.user?.role || 'user';
            const isAdmin = ['admin', 'reseller'].includes(userRole);

            if (to.path === '/dashboard' && isAdmin) {
                next('/admin');
                return;
            }
            if (to.path === '/admin' && !isAdmin) {
                next('/dashboard');
                return;
            }
        }
        next();
    }
});

export default router;
