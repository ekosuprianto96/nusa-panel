import api from './api';
import type { ApiResponse, NpmPackage, Pm2Process } from '@/types';

export interface NodejsStatus {
    nvm_installed: boolean;
    current_version: string | null;
    installed_versions: string[];
    lts_versions: string[];
}

export const nodejsService = {
    /**
     * Get Node.js manager status
     */
    getStatus: () => api.get<ApiResponse<NodejsStatus>>('/nodejs'),

    /**
     * Install NVM for user
     */
    installNvm: () => api.post<ApiResponse<string>>('/nodejs/install-nvm'),

    /**
     * Bootstrap NVM + Node.js default
     */
    bootstrap: (version?: string) =>
        api.post<ApiResponse<string>>('/nodejs/bootstrap', { version }),

    /**
     * List available LTS versions
     */
    getAvailableVersions: () => api.get<ApiResponse<string[]>>('/nodejs/available'),

    /**
     * Install specific Node.js version
     */
    installVersion: (version: string) =>
        api.post<ApiResponse<string>>('/nodejs/install', { version }),

    /**
     * Uninstall specific Node.js version
     */
    uninstallVersion: (version: string) =>
        api.post<ApiResponse<string>>('/nodejs/uninstall', { version }),

    /**
     * Set default Node.js version
     */
    setDefault: (version: string) =>
        api.put<ApiResponse<string>>('/nodejs/default', { version }),
    /**
     * Get Environment Variables
     */
    getEnvVars: (path: string) => api.get<ApiResponse<string[]>>(`/nodejs/env?path=${encodeURIComponent(path)}`),

    /**
     * Save Environment Variable
     */
    saveEnvVar: (path: string, key: string, value: string) =>
        api.post<ApiResponse<void>>('/nodejs/env', { path, key, value }),

    /**
     * Delete Environment Variable
     */
    deleteEnvVar: (path: string, key: string) =>
        api.post<ApiResponse<void>>('/nodejs/env/delete', { path, key }),

    /**
     * List PM2 Processes
     */
    listPm2: () => api.get<ApiResponse<Pm2Process[]>>('/nodejs/pm2'),

    /**
     * PM2 Action (start, stop, restart, delete)
     */
    pm2Action: (action: 'start' | 'stop' | 'restart' | 'delete', target: string) =>
        api.post<ApiResponse<void>>('/nodejs/pm2/action', { action, target }),

    /**
     * NPM Operations
     */
    npmList: (path?: string) =>
        api.get<ApiResponse<NpmPackage[]>>('/nodejs/npm', { params: { path } }),

    npmInstall: (package_name?: string, version?: string, dev = false, path?: string) =>
        api.post<ApiResponse<void>>('/nodejs/npm/install', { package: package_name, version, dev, path }),

    npmUninstall: (package_name: string, path?: string) =>
        api.post<ApiResponse<void>>('/nodejs/npm/uninstall', { package: package_name, path }),

    npmInstallAll: (path?: string) =>
        api.post<ApiResponse<void>>('/nodejs/npm/install', { path }),

    npmRun: (script: string, path?: string) =>
        api.post<ApiResponse<void>>('/nodejs/npm/run', { script, path })
};

// Export interfaces from types for convenience if needed, 
// or remove local definition if now importing from @/types
// For now, we removed local Pm2Process definition to use the one from types.

export default nodejsService;
