import api from './api'
import type { 
  ApiResponse, 
  PaginatedResponse, 
  DomainResponse, 
  SubdomainResponse, 
  DnsRecordResponse 
} from '@/types'

/**
 * Domain Management Service
 * Menangani domain, subdomain, dan DNS records.
 */
export const domainService = {
  // Domain CRUD
  listDomains: (page: number = 1, perPage: number = 10) => 
    api.get<PaginatedResponse<DomainResponse>>('/domains', { params: { page, per_page: perPage } }),

  getDomain: (id: string) => 
    api.get<ApiResponse<DomainResponse>>(`/domains/${id}`),

  createDomain: (data: { domain_name: string, document_root?: string }) => 
    api.post<ApiResponse<DomainResponse>>('/domains', data),

  updateDomain: (id: string, data: { document_root?: string, is_active?: boolean }) => 
    api.put<ApiResponse<DomainResponse>>(`/domains/${id}`, data),

  deleteDomain: (id: string) => 
    api.delete<ApiResponse<void>>(`/domains/${id}`),

  // Subdomain
  listSubdomains: (domainId: string) => 
    api.get<ApiResponse<SubdomainResponse[]>>(`/domains/${domainId}/subdomains`),

  createSubdomain: (domainId: string, data: { subdomain_name: string, document_root?: string }) => 
    api.post<ApiResponse<SubdomainResponse>>(`/domains/${domainId}/subdomains`, data),

  deleteSubdomain: (domainId: string, subdomainId: string) => 
    api.delete<ApiResponse<void>>(`/domains/${domainId}/subdomains/${subdomainId}`),

  // DNS Records
  listDnsRecords: (domainId: string) => 
    api.get<ApiResponse<DnsRecordResponse[]>>(`/domains/${domainId}/dns`),

  createDnsRecord: (domainId: string, data: { record_type: string, name: string, value: string, ttl?: number, priority?: number }) => 
    api.post<ApiResponse<DnsRecordResponse>>(`/domains/${domainId}/dns`, data),

  updateDnsRecord: (domainId: string, recordId: string, data: { value?: string, ttl?: number, priority?: number }) => 
    api.put<ApiResponse<DnsRecordResponse>>(`/domains/${domainId}/dns/${recordId}`, data),

  deleteDnsRecord: (domainId: string, recordId: string) => 
    api.delete<ApiResponse<void>>(`/domains/${domainId}/dns/${recordId}`),

  // Redirects
  listRedirects: (domainId: string) => 
    api.get<ApiResponse<any[]>>(`/domains/${domainId}/redirects`),

  createRedirect: (domainId: string, data: { source_path: string, destination_url: string, type: '301' | '302' }) => 
    api.post<ApiResponse<any>>(`/domains/${domainId}/redirects`, data),

  deleteRedirect: (domainId: string, redirectId: string) => 
    api.delete<ApiResponse<void>>(`/domains/${domainId}/redirects/${redirectId}`),

  // Domain Aliases (Parked Domains)
  listAliases: (domainId: string) => 
    api.get<ApiResponse<any[]>>(`/domains/${domainId}/aliases`),

  createAlias: (domainId: string, data: { alias_domain: string }) => 
    api.post<ApiResponse<any>>(`/domains/${domainId}/aliases`, data),

  deleteAlias: (domainId: string, aliasId: string) => 
    api.delete<ApiResponse<void>>(`/domains/${domainId}/aliases/${aliasId}`)
}

export default domainService
