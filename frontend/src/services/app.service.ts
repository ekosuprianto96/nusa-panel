import api from './api'
import type {
  ApiResponse
} from '@/types'

/**
 * App Installer Service
 * Menangani instalasi aplikasi (WordPress, Laravel, dll).
 */
export const appService = {
  /**
   * Menginstal aplikasi ke domain tertentu
   * @param data Payload instalasi aplikasi
   */
  installApp: (data: {
    domain_id: string,
    app_type: 'wordpress' | 'laravel',
    site_title: string,
    admin_username: string,
    admin_password: string,
    admin_email: string
  }) =>
    api.post<ApiResponse<any>>('/apps/install', data),

  /**
   * Mendapatkan daftar aplikasi yang sudah terinstal
   */
  listInstallations: () =>
    api.get<ApiResponse<any>>('/apps/installations'),

  /**
   * Admin/Reseller: list installations untuk user tertentu
   * @param userId User ID
   */
  listInstallationsByUser: (userId: string) =>
    api.get<ApiResponse<any>>(`/apps/admin/user/${userId}/installations`)
}

export default appService
