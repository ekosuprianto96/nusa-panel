import api from './api';
import type { ApiResponse, ManagedDatabase, DatabaseUser } from '@/types';

/**
 * Database Management Service
 * Menangani managed MySQL databases dan users.
 */
export const databaseService = {
    // Database CRUD
    listDatabases: () => api.get<ApiResponse<ManagedDatabase[]>>('/databases'),

    getDatabase: (id: string) =>
        api.get<ApiResponse<ManagedDatabase>>(`/databases/${id}`),

    createDatabase: (data: {
        name: string;
        description?: string;
        charset?: string;
        collation?: string;
    }) => api.post<ApiResponse<ManagedDatabase>>('/databases', data),

    updateDatabase: (id: string, data: { description?: string }) =>
        api.put<ApiResponse<ManagedDatabase>>(`/databases/${id}`, data),

    deleteDatabase: (id: string) =>
        api.delete<ApiResponse<void>>(`/databases/${id}`),

    // Database Users
    listDatabaseUsers: () =>
        api.get<ApiResponse<DatabaseUser[]>>('/databases/users'),

    getDatabaseUser: (id: string) =>
        api.get<ApiResponse<DatabaseUser>>(`/databases/users/${id}`),

    createDatabaseUser: (data: {
        username: string;
        password: string;
        database_id: string;
        host?: string;
        privileges?: string;
    }) => api.post<ApiResponse<DatabaseUser>>('/databases/users', data),

    updateDatabaseUser: (
        id: string,
        data: {
            password?: string;
            host?: string;
            privileges?: string;
            is_active?: boolean;
            database_id?: string;
        },
    ) => api.put<ApiResponse<DatabaseUser>>(`/databases/users/${id}`, data),

    deleteDatabaseUser: (id: string) =>
        api.delete<ApiResponse<void>>(`/databases/users/${id}`),

    // phpMyAdmin SSO
    /**
     * Generate signon token untuk SSO ke phpMyAdmin
     * @param dbUserId ID database user yang akan di-SSO
     * @returns URL untuk redirect ke phpMyAdmin dengan token
     */
    generateSignonToken: (dbUserId: string) =>
        api.post<ApiResponse<{ signon_url: string; expires_in: number }>>(
            `/phpmyadmin/signon/${dbUserId}`,
        ),
};

export default databaseService;
