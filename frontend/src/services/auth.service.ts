import api from './api'
import type { 
  ApiResponse, 
  TokenPair, 
  LoginRequest, 
  CreateUserRequest, 
  UserResponse 
} from '@/types'

/**
 * Authentication Service
 * Menangani registrasi, login, logout, dan manajemen token.
 */
export const authService = {
  /**
   * Register user baru
   * @param data Data registrasi user
   */
  register: (data: CreateUserRequest) => 
    api.post<ApiResponse<UserResponse>>('/auth/register', data),

  /**
   * Login user
   * @param data Kredensial login
   */
  login: (data: LoginRequest) => 
    api.post<ApiResponse<TokenPair>>('/auth/login', data),

  /**
   * Refresh access token menggunakan refresh token
   * @param refreshToken Refresh token yang valid
   */
  refresh: (refreshToken: string) => 
    api.post<ApiResponse<TokenPair>>('/auth/refresh', { refresh_token: refreshToken }),

  /**
   * Mendapatkan informasi user yang sedang login
   */
  getCurrentUser: () => 
    api.get<ApiResponse<UserResponse>>('/auth/me'),

  /**
   * Mengubah password user
   * @param data Data password lama dan baru
   */
  changePassword: (data: any) => 
    api.post<ApiResponse<void>>('/auth/change-password', data),

  /**
   * Logout user dari sistem
   */
  logout: () => 
    api.post<ApiResponse<void>>('/auth/logout')
}

export default authService
