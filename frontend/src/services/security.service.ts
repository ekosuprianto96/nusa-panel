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

  getBlockedIps: () =>
    api.get<ApiResponse<any[]>>('/security/ips'),

  blockIp: (data: { ip_address: string, reason?: string, type?: string }) =>
    api.post<ApiResponse<any>>('/security/ips', data),

  unblockIp: (id: string) =>
    api.delete<ApiResponse<void>>(`/security/ips/${id}`),

  // SSH Key Endpoints
  listSshKeys: () =>
    api.get<ApiResponse<any[]>>('/security/ssh'),

  getSshKeys: () =>
    api.get<ApiResponse<any[]>>('/security/ssh'),

  addSshKey: (data: { label: string, public_key: string }) =>
    api.post<ApiResponse<any>>('/security/ssh', data),

  deleteSshKey: (id: string) =>
    api.delete<ApiResponse<void>>(`/security/ssh/${id}`),

  // 2FA Endpoints
  get2faStatus: () =>
    api.get<ApiResponse<{ enabled: boolean }>>('/security/2fa/status'),

  setup2fa: () =>
    api.post<ApiResponse<{ qr_code: string, secret: string, backup_codes?: string[] }>>('/security/2fa/setup'),

  verify2fa: (data: { code: string }) =>
    api.post<ApiResponse<void>>('/security/2fa/verify', data),

  disable2fa: () =>
    api.delete<ApiResponse<void>>('/security/2fa'),

  // SSL/TLS Endpoints
  runAutoSsl: () =>
    api.post<ApiResponse<void>>('/security/ssl/auto'),

  renewSsl: (domainId: string) =>
    api.post<ApiResponse<void>>(`/security/ssl/renew/${domainId}`),

  // Monitoring Endpoints
  getResourceStats: () =>
    api.get<ApiResponse<any>>('/security/stats'),

  getAccessLogs: () =>
    api.get<ApiResponse<any[]>>('/security/logs'),

  // ModSecurity Endpoints
  getModSecurityOverview: () =>
    api.get<ApiResponse<any>>('/security/modsecurity/overview'),

  updateModSecuritySettings: (data: { main_engine?: boolean; paranoia_level?: number; anomaly_threshold?: number }) =>
    api.post<ApiResponse<any>>('/security/modsecurity/settings', data),

  updateModSecurityRule: (ruleId: string, data: { enabled: boolean }) =>
    api.post<ApiResponse<any>>(`/security/modsecurity/rules/${ruleId}`, data),

  updateModSecurityDomain: (domainId: string, data: { enabled: boolean }) =>
    api.post<ApiResponse<void>>(`/security/modsecurity/domains/${domainId}`, data),

  // Custom Rules
  listCustomRules: () =>
    api.get<ApiResponse<any[]>>('/security/modsecurity/custom-rules'),

  createCustomRule: (data: { name: string; description?: string; rule_content: string; enabled?: boolean }) =>
    api.post<ApiResponse<any>>('/security/modsecurity/custom-rules', data),

  updateCustomRule: (ruleId: string, data: { name?: string; description?: string; rule_content?: string; enabled?: boolean }) =>
    api.post<ApiResponse<any>>(`/security/modsecurity/custom-rules/${ruleId}`, data),

  deleteCustomRule: (ruleId: string) =>
    api.delete<ApiResponse<void>>(`/security/modsecurity/custom-rules/${ruleId}`),

  // Audit Logs
  getModSecurityAuditLogs: (params?: { domain_id?: string; limit?: number }) =>
    api.get<ApiResponse<any[]>>('/security/modsecurity/audit-logs', { params })
}

export default securityService

