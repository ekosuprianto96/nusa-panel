import api from './api'
import type { 
  ApiResponse 
} from '@/types'

/**
 * Web Server Service
 * Menangani VirtualHost management dan SSL certificates.
 */
export const webserverService = {
  // Virtual Host Endpoints
  listVHosts: () => 
    api.get<ApiResponse<any[]>>('/web-server'),

  getVHost: (id: string) => 
    api.get<ApiResponse<any>>(`/web-server/${id}`),

  createVHost: (data: { domain_id: string, php_version?: string, ssl_enabled?: boolean }) => 
    api.post<ApiResponse<any>>('/web-server', data),

  updateVHost: (id: string, data: { php_version?: string, force_https?: boolean, custom_config?: string }) => 
    api.put<ApiResponse<any>>(`/web-server/${id}`, data),

  deleteVHost: (id: string) => 
    api.delete<ApiResponse<void>>(`/web-server/${id}`),

  // SSL Endpoints
  requestSsl: (data: { domain_id: string, email?: string }) => 
    api.post<ApiResponse<any>>('/web-server/ssl', data),

  getSslStatus: (vhostId: string) => 
    api.get<ApiResponse<any>>(`/web-server/ssl/${vhostId}`)
}

export default webserverService
