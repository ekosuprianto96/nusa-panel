import api from './api';
import type { ApiResponse } from '@/types';

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
};

export default nodejsService;
