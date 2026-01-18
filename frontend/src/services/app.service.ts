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
    api.post<ApiResponse<any>>('/apps/install', data)
}

export default appService
