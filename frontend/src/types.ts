export interface ApiResponse<T> {
  status: string
  message: string
  data: T
}

export interface PaginatedResponse<T> {
  status: string
  message: string
  data: {
    items: T[]
    total_items: number
    page: number
    per_page: number
    total_pages: number
  }
}

export interface TokenPair {
  access_token: string
  refresh_token: string
}

export interface UserResourceUsage {
  disk_used_mb: number
  disk_limit_mb: number
  bandwidth_used_mb: number
  bandwidth_limit_mb: number
  domains_count: number
  domains_limit: number
  databases_count: number
  databases_limit: number
  email_accounts_count: number
  email_accounts_limit: number
}

export interface UserResponse {
  id: string
  username: string
  email: string
  first_name: string | null
  last_name: string | null
  full_name: string
  role: string
  status: string
  created_at: string
  last_login_at: string | null
  usage?: UserResourceUsage
}

export interface LoginRequest {
  username_or_email: string
  password: string
}

export interface CreateUserRequest {
  username: string
  email: string
  password: string
  first_name?: string
  last_name?: string
}

export interface UpdateUserRequest {
  email?: string
  first_name?: string
  last_name?: string
}

// File Interface
export interface FileInfo {
  name: string
  path: string
  file_type: 'file' | 'directory' | 'symlink'
  size: number
  permissions: string
  permissions_octal: string
  owner: string
  group: string
  modified_at: string
  accessed_at?: string
  created_at?: string
  extension?: string
  mime_type?: string
  is_hidden: boolean
}

export interface FileListResponse {
  current_path: string
  parent_path: string | null
  items: FileInfo[]
  total_items: number
  total_size: number
}

export interface FileContentResponse {
  path: string
  name: string
  content: string
  encoding: string
  size: number
  mime_type?: string
}

// System Interface
export interface CronJob {
  id: string
  user_id: string
  schedule: string
  command: string
  description?: string
  is_active: boolean
  email_notification?: string
  created_at: string
  updated_at: string
}

export interface SystemBackup {
  id: string
  user_id: string
  filename: string
  size_bytes: number
  backup_type: string
  status: string
  created_at: string
  completed_at?: string
}

export interface ServiceStatus {
  name: string
  status: string
  uptime: string
  memory_usage: string
}

// Domain Interface
export interface Domain {
  id: string
  user_id: string
  domain_name: string
  document_root: string
  is_active: boolean
  ssl_enabled: boolean
  created_at: string
  updated_at: string
}

export interface DomainResponse extends Domain {
  subdomains_count: number
  dns_records_count: number
}

export interface Subdomain {
  id: string
  domain_id: string
  subdomain_name: string
  full_name: string
  document_root: string
  is_active: boolean
  created_at: string
}

export interface DnsRecord {
  id: string
  domain_id: string
  record_type: string
  name: string
  value: string
  ttl: number
  priority?: number
  created_at: string
  updated_at: string
}

// Database Interface
export interface ManagedDatabase {
  id: string
  user_id: string
  db_name: string
  description?: string
  size_bytes: number
  size_mb: number
  charset: string
  collation: string
  created_at: string
  updated_at: string
  users_count: number
}

export interface DatabaseUser {
  id: string
  user_id: string
  database_id: string
  db_name: string
  db_username: string
  host: string
  privileges: string
  is_active: boolean
  created_at: string
  updated_at: string
  phpmyadmin_info: {
    url: string
    username: string
    database: string
    mysql_host: string
  }
}

// Domain Types
export interface DomainResponse {
  id: string
  user_id: string
  domain_name: string
  document_root: string
  is_active: boolean
  ssl_enabled: boolean
  created_at: string
  updated_at: string
}

export interface SubdomainResponse {
  id: string
  domain_id: string
  subdomain_name: string
  full_domain: string
  document_root: string
  is_active: boolean
  created_at: string
}

export interface DnsRecordResponse {
  id: string
  domain_id: string
  record_type: string
  name: string
  value: string
  ttl: number
  priority: number | null
  created_at: string
  updated_at: string
}
