import { defineStore } from 'pinia'
import api from '@/services/api'
import type { LoginRequest, TokenPair, ApiResponse, UserResponse } from '@/types'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as UserResponse | null,
    isAuthenticated: false,
    authChecked: false,
  }),

  getters: {
    isLoggedIn: (state) => state.isAuthenticated,
  },

  actions: {
    async login(credentials: LoginRequest) {
      try {
        await api.post<ApiResponse<TokenPair>>('/auth/login', credentials)
        this.isAuthenticated = true

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
        this.isAuthenticated = true
        this.authChecked = true
      } catch (error) {
        this.clearLocalAuth()
      }
    },

    async checkSession() {
      if (this.authChecked) return
      try {
        await this.fetchMe()
      } catch {
        this.authChecked = true
      }
    },

    clearLocalAuth() {
      this.user = null
      this.isAuthenticated = false
      this.authChecked = true
    },

    async logout() {
      try {
        await api.post<ApiResponse<void>>('/auth/logout')
      } finally {
        this.clearLocalAuth()
      }
    }
  }
})
