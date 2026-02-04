import api from './api'
import type {
    ApiResponse,
    Package,
    CreatePackageRequest,
    UpdatePackageRequest
} from '@/types'

/**
 * Package Management Service
 * Menangani operasi CRUD untuk hosting packages (Admin only).
 */
export const packageService = {
    /**
     * List semua packages
     * @returns Daftar packages
     */
    listPackages: () =>
        api.get<ApiResponse<Package[]>>('/packages'),

    /**
     * Get package by ID
     * @param id Package ID
     */
    getPackage: (id: string) =>
        api.get<ApiResponse<Package>>(`/packages/${id}`),

    /**
     * Get default package
     */
    getDefaultPackage: () =>
        api.get<ApiResponse<Package>>('/packages/default'),

    /**
     * Create new package (Admin only)
     * @param data Package data
     */
    createPackage: (data: CreatePackageRequest) =>
        api.post<ApiResponse<Package>>('/packages', data),

    /**
     * Update package (Admin only)
     * @param id Package ID
     * @param data Package data
     */
    updatePackage: (id: string, data: UpdatePackageRequest) =>
        api.put<ApiResponse<Package>>(`/packages/${id}`, data),

    /**
     * Delete package (Admin only)
     * @param id Package ID
     */
    deletePackage: (id: string) =>
        api.delete<ApiResponse<void>>(`/packages/${id}`)
}

export default packageService
