import api from './api'
import type { 
  ApiResponse 
} from '@/types'

/**
 * Redis Management Service
 * Menangani aktivasi dan monitoring instance Redis user.
 */
export const redisService = {
  /**
   * Mendapatkan status instance Redis user
   */
  getStatus: () => 
    api.get<ApiResponse<any>>('/redis'),

  /**
   * Mengaktifkan instance Redis
   * @param data Opsi limit memori
   */
  enableRedis: (data: { max_memory?: string }) => 
    api.post<ApiResponse<any>>('/redis', data),

  /**
   * Menonaktifkan instance Redis
   */
  disableRedis: () => 
    api.delete<ApiResponse<any>>('/redis')
}

export default redisService
