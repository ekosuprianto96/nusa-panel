import api from './api'
import type { 
  ApiResponse, 
  PaginatedResponse, 
  UserResponse, 
  UpdateUserRequest,
  UserResourceUsage
} from '@/types'

/**
 * User Management Service
 * Menangani operasi CRUD user dan resource usage (khusus Admin).
 */
export const userService = {
  /**
   * List semua user (Admin Only)
   * @param page Halaman yang diminta
   * @param perPage Jumlah item per halaman
   */
  listUsers: (page: number = 1, perPage: number = 10) => 
    api.get<PaginatedResponse<UserResponse>>('/users', { params: { page, per_page: perPage } }),

  /**
   * Mendapatkan detail user berdasarkan ID
   * @param id User ID
   */
  getUser: (id: string) => 
    api.get<ApiResponse<UserResponse>>(`/users/${id}`),

  /**
   * Update data user
   * @param id User ID
   * @param data Data yang akan diupdate
   */
  updateUser: (id: string, data: UpdateUserRequest) => 
    api.put<ApiResponse<UserResponse>>(`/users/${id}`, data),

  /**
   * Menghapus user (Admin Only)
   * @param id User ID
   */
  deleteUser: (id: string) => 
    api.delete<ApiResponse<void>>(`/users/${id}`),

  /**
   * Update status user (Admin Only)
   * @param id User ID
   * @param status Status baru (active|inactive|blocked)
   */
  updateStatus: (id: string, status: string) => 
    api.put<ApiResponse<UserResponse>>(`/users/${id}/status`, { status }),

  /**
   * Update role user (Admin Only)
   * @param id User ID
   * @param role Role baru (admin|reseller|user)
   */
  updateRole: (id: string, role: string) => 
    api.put<ApiResponse<UserResponse>>(`/users/${id}/role`, { role }),

  /**
   * Mendapatkan statistik penggunaan resource user
   * @param id User ID
   */
  getUsage: (id: string) => 
    api.get<ApiResponse<UserResourceUsage>>(`/users/${id}/usage`)
}

export default userService
