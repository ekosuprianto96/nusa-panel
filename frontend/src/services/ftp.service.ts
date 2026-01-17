import api from './api'
import type { 
  ApiResponse 
} from '@/types'

/**
 * FTP Management Service
 * Menangani akun FTP dan informasi koneksi server.
 */
export const ftpService = {
  listAccounts: () => 
    api.get<ApiResponse<any[]>>('/ftp'),

  getAccount: (id: string) => 
    api.get<ApiResponse<any>>(`/ftp/${id}`),

  createAccount: (data: { username: string, password: string, home_directory?: string, quota_mb?: number }) => 
    api.post<ApiResponse<any>>('/ftp', data),

  updateAccount: (id: string, data: { password?: string, home_directory?: string, is_active?: boolean, quota_mb?: number }) => 
    api.put<ApiResponse<any>>(`/ftp/${id}`, data),

  changePassword: (id: string, data: { new_password: string }) => 
    api.put<ApiResponse<void>>(`/ftp/${id}/password`, data),

  toggleStatus: (id: string) => 
    api.put<ApiResponse<any>>(`/ftp/${id}/toggle`),

  deleteAccount: (id: string) => 
    api.delete<ApiResponse<void>>(`/ftp/${id}`),

  getServerInfo: () => 
    api.get<ApiResponse<any>>('/ftp/server-info')
}

export default ftpService
