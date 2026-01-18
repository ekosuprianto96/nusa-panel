import { defineStore } from 'pinia'
import api from '@/services/api'
import type { LoginRequest, TokenPair, ApiResponse, UserResponse } from '@/types'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as UserResponse | null,
    accessToken: localStorage.getItem('access_token'),
    refreshToken: localStorage.getItem('refresh_token'),
  }),
  
  getters: {
    isAuthenticated: (state) => !!state.accessToken,
  },
  
  actions: {
    async login(credentials: LoginRequest) {
      try {
        const response = await api.post<ApiResponse<TokenPair>>('/auth/login', credentials)
        const { access_token, refresh_token } = response.data.data
        
        this.accessToken = access_token
        this.refreshToken = refresh_token
        
        localStorage.setItem('access_token', access_token)
        localStorage.setItem('refresh_token', refresh_token)
        
        await this.fetchMe()
        return true
      } catch (error) {
        console.error('Login failed:', error)
        throw error
      }
    },
    
    async fetchMe() {
      try {
        const response = await api.get<ApiResponse<UserResponse>>('/auth/me')
        this.user = response.data.data
      } catch (error) {
        this.logout()
      }
    },
    
    logout() {
      this.user = null
      this.accessToken = null
      this.refreshToken = null
      localStorage.removeItem('access_token')
      localStorage.removeItem('refresh_token')
    }
  }
})
