import api from './api'
import type { 
  ApiResponse 
} from '@/types'

/**
 * Email Management Service
 * Menangani akun email, forwarder, dan autoresponder.
 */
export const emailService = {
  // Email Accounts
  listAccounts: () => 
    api.get<ApiResponse<any[]>>('/emails'),

  getAccount: (id: string) => 
    api.get<ApiResponse<any>>(`/emails/${id}`),

  createAccount: (data: { domain_id: string, username: string, password: string, quota_mb?: number }) => 
    api.post<ApiResponse<any>>('/emails', data),

  updateAccount: (id: string, data: { password?: string, quota_mb?: number, is_active?: boolean }) => 
    api.put<ApiResponse<any>>(`/emails/${id}`, data),

  deleteAccount: (id: string) => 
    api.delete<ApiResponse<void>>(`/emails/${id}`),

  // Forwarders
  listForwarders: () => 
    api.get<ApiResponse<any[]>>('/emails/forwarders'),

  createForwarder: (data: { domain_id: string, source_username: string, destination_email: string }) => 
    api.post<ApiResponse<any>>('/emails/forwarders', data),

  deleteForwarder: (id: string) => 
    api.delete<ApiResponse<void>>(`/emails/forwarders/${id}`),

  // Autoresponders
  listAutoresponders: () => 
    api.get<ApiResponse<any[]>>('/emails/autoresponders'),

  createAutoresponder: (data: { email_account_id: string, subject: string, body: string, start_date?: string, end_date?: string }) => 
    api.post<ApiResponse<any>>('/emails/autoresponders', data),

  updateAutoresponder: (id: string, data: { subject?: string, body?: string, is_active?: boolean }) => 
    api.put<ApiResponse<any>>(`/emails/autoresponders/${id}`, data),

  deleteAutoresponder: (id: string) => 
    api.delete<ApiResponse<void>>(`/emails/autoresponders/${id}`)
}

export default emailService
