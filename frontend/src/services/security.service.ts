import api from './api'
import type { 
  ApiResponse 
} from '@/types'

/**
 * Security Service
 * Menangani IP blocking, SSH keys, monitoring stats, dan access logs.
 */
export const securityService = {
  // IP Blocking (Admin Only)
  listBlockedIps: () => 
    api.get<ApiResponse<any[]>>('/security/ips'),

  blockIp: (data: { ip_address: string, reason?: string }) => 
    api.post<ApiResponse<any>>('/security/ips', data),

  unblockIp: (id: string) => 
    api.delete<ApiResponse<void>>(`/security/ips/${id}`),

  // SSH Key Endpoints
  listSshKeys: () => 
    api.get<ApiResponse<any[]>>('/security/ssh'),

  addSshKey: (data: { label: string, public_key: string }) => 
    api.post<ApiResponse<any>>('/security/ssh', data),

  deleteSshKey: (id: string) => 
    api.delete<ApiResponse<void>>(`/security/ssh/${id}`),

  // Monitoring Endpoints
  getResourceStats: () => 
    api.get<ApiResponse<any>>('/security/stats'),

  getAccessLogs: () => 
    api.get<ApiResponse<any[]>>('/security/logs')
}

export default securityService
