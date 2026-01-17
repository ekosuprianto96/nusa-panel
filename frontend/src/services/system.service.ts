import api from './api'
import type { 
  ApiResponse, 
  CronJob, 
  SystemBackup, 
  ServiceStatus 
} from '@/types'

/**
 * System Tools Service
 * Menangani cron jobs, backups, dan status service system.
 */
export const systemService = {
  // Cron Job Endpoints
  listCronJobs: () => 
    api.get<ApiResponse<CronJob[]>>('/system/cron'),

  createCronJob: (data: { schedule: string, command: string, description?: string, email_notification?: string }) => 
    api.post<ApiResponse<CronJob>>('/system/cron', data),

  updateCronJob: (id: string, data: { schedule?: string, command?: string, description?: string, is_active?: boolean, email_notification?: string }) => 
    api.put<ApiResponse<CronJob>>(`/system/cron/${id}`, data),

  deleteCronJob: (id: string) => 
    api.delete<ApiResponse<void>>(`/system/cron/${id}`),

  // Backup Endpoints
  listBackups: () => 
    api.get<ApiResponse<SystemBackup[]>>('/system/backups'),

  createBackup: (data: { backup_type: 'full' | 'database' | 'homedir', description?: string }) => 
    api.post<ApiResponse<SystemBackup>>('/system/backups', data),

  deleteBackup: (id: string) => 
    api.delete<ApiResponse<void>>(`/system/backups/${id}`),

  // Service Status (Admin Only)
  getServicesStatus: () => 
    api.get<ApiResponse<ServiceStatus[]>>('/system/services'),

  // Control Service (Admin Only)
  controlService: (serviceName: string, action: 'start' | 'stop' | 'restart') =>
    api.post<ApiResponse<ServiceStatus>>(`/system/services/${serviceName}`, { action }),

  // PHP Version Management (Admin Only)
  getPhpVersions: () =>
    api.get<ApiResponse<string[]>>('/system/php/versions'),

  getCurrentPhpVersion: () =>
    api.get<ApiResponse<string>>('/system/php/current'),

  changePhpVersion: (version: string) =>
    api.post<ApiResponse<string>>('/system/php/change', { version }),

  // Error Logs (Admin Only)
  getErrorLogs: (lines = 50) =>
    api.get<ApiResponse<string[]>>(`/system/logs?lines=${lines}`),

  clearErrorLogs: () =>
    api.post<ApiResponse<void>>('/system/logs/clear', {})
}

export default systemService
