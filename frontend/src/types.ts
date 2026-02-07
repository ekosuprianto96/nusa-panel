export interface ApiResponse<T> {
    status: string;
    message: string;
    data: T;
}

export interface PaginatedResponse<T> {
    status: string;
    message: string;
    data: T[];
    pagination: {
        has_next: boolean;
        has_prev: boolean;
        total: number;
        total_pages: number;
        page: number;
        per_page: number;
    };
}

export interface TokenPair {
    access_token: string;
    refresh_token: string;
}

export interface UserResourceUsage {
    disk_used_mb: number;
    disk_limit_mb: number;
    bandwidth_used_mb: number;
    bandwidth_limit_mb: number;
    domains_count: number;
    domains_limit: number;
    databases_count: number;
    databases_limit: number;
    email_accounts_count: number;
    email_accounts_limit: number;
}

export interface UserResponse {
    id: string;
    username: string;
    email: string;
    first_name: string | null;
    last_name: string | null;
    full_name: string;
    role: string;
    status: string;
    package_id: string | null;
    created_by: string | null;
    company: string | null;
    phone: string | null;
    address: string | null;
    created_at: string;
    last_login_at: string | null;
    usage?: UserResourceUsage;
}

export interface AdminUserStats {
    total_users: number;
    active_users: number;
    blocked_users: number;
    new_signups_7d: number;
}

export interface LoginRequest {
    username_or_email: string;
    password: string;
    two_fa_code?: string;
}

export interface CreateUserRequest {
    username: string;
    email: string;
    password: string;
    first_name?: string;
    last_name?: string;
}

export interface CreateUserByAdminRequest {
    username: string;
    email: string;
    password: string;
    first_name?: string;
    last_name?: string;
    role?: 'user' | 'reseller';
    package_id?: string;
    company?: string;
    phone?: string;
    address?: string;
    status?: 'active' | 'inactive' | 'pending';
}

export interface UpdateUserRequest {
    email?: string;
    first_name?: string;
    last_name?: string;
    company?: string;
    phone?: string;
    address?: string;
}

export interface AssignPackageRequest {
    package_id: string;
}

// Package Types
export interface Package {
    id: string;
    name: string;
    description: string | null;
    disk_quota_mb: number;
    bandwidth_mb: number;
    max_domains: number;
    max_subdomains: number;
    max_databases: number;
    max_email_accounts: number;
    max_ftp_accounts: number;
    max_cron_jobs: number;
    price_monthly: number;
    price_yearly: number;
    is_active: boolean;
    is_default: boolean;
    sort_order: number;
    created_at: string;
    updated_at: string;
    users_count?: number;
}

export interface CreatePackageRequest {
    name: string;
    description?: string;
    disk_quota_mb?: number;
    bandwidth_mb?: number;
    max_domains?: number;
    max_subdomains?: number;
    max_databases?: number;
    max_email_accounts?: number;
    max_ftp_accounts?: number;
    max_cron_jobs?: number;
    price_monthly?: number;
    price_yearly?: number;
    is_active?: boolean;
    is_default?: boolean;
    sort_order?: number;
}

export interface UpdatePackageRequest {
    name?: string;
    description?: string;
    disk_quota_mb?: number;
    bandwidth_mb?: number;
    max_domains?: number;
    max_subdomains?: number;
    max_databases?: number;
    max_email_accounts?: number;
    max_ftp_accounts?: number;
    max_cron_jobs?: number;
    price_monthly?: number;
    price_yearly?: number;
    is_active?: boolean;
    is_default?: boolean;
    sort_order?: number;
}

// File Interface
export interface FileInfo {
    name: string;
    path: string;
    file_type: 'file' | 'directory' | 'symlink';
    size: number;
    permissions: string;
    permissions_octal: string;
    owner: string;
    group: string;
    modified_at: string;
    accessed_at?: string;
    created_at?: string;
    extension?: string;
    mime_type?: string;
    is_hidden: boolean;
}

export interface FileListResponse {
    current_path: string;
    parent_path: string | null;
    items: FileInfo[];
    total_items: number;
    total_size: number;
    files: FileInfo[];
}

export interface FileContentResponse {
    path: string;
    name: string;
    content: string;
    encoding: string;
    size: number;
    mime_type?: string;
}

// System Interface
export interface CronJob {
    id: string;
    user_id: string;
    schedule: string;
    command: string;
    description?: string;
    is_active: boolean;
    email_notification?: string;
    created_at: string;
    updated_at: string;
}

export interface SystemBackup {
    id: string;
    user_id: string;
    filename: string;
    size_bytes: number;
    backup_type: string;
    status: string;
    created_at: string;
    completed_at?: string;
}

export interface ServiceStatus {
    name: string;
    status: string;
    uptime: string;
    memory_usage: string;
}

// Domain Interface
export interface Domain {
    id: string;
    user_id: string;
    domain_name: string;
    document_root: string;
    is_active: boolean;
    ssl_enabled: boolean;
    ssl_status?: string;
    ssl_provider?: string | null;
    ssl_expiry_at?: string | null;
    modsecurity_enabled?: boolean;
    created_at: string;
    updated_at: string;
}

export interface DomainResponse extends Domain {
    subdomains_count: number;
    dns_records_count: number;
}

export interface DdnsHostResponse {
    id: string;
    domain_id: string;
    hostname: string;
    description?: string | null;
    last_ip?: string | null;
    last_updated_at?: string | null;
    created_at: string;
    updated_at: string;
}

export interface DdnsHostCreateResponse {
    id: string;
    domain_id: string;
    hostname: string;
    description?: string | null;
    api_key: string;
    created_at: string;
}

export interface Subdomain {
    id: string;
    domain_id: string;
    subdomain_name: string;
    full_name: string;
    document_root: string;
    is_active: boolean;
    created_at: string;
}

export interface DnsRecord {
    id: string;
    domain_id: string;
    record_type: string;
    name: string;
    value: string;
    ttl: number;
    priority?: number;
    created_at: string;
    updated_at: string;
}

// System Tools
export interface ResourceUsage {
    cpu: number;
    memory: number;
    disk: number;
    processes: ProcessInfo[];
}

export interface ProcessInfo {
    pid: number;
    name: number;
    cpu: number;
    memory: number;
}

export interface NpmPackage {
    name: string;
    version: string;
    type: string;
    hasUpdate?: boolean; // Frontend helper
    latest?: string; // Frontend helper
}

export interface Pm2Process {
    pid: number;
    name: string;
    pm_id: number;
    status: string; // online, stopped, errored
    memory: number;
    cpu: number;
    uptime: number;
    restarts: number;
}

// Database Interface
export interface ManagedDatabase {
    id: string;
    user_id: string;
    db_name: string;
    description?: string;
    size_bytes: number;
    size_mb: number;
    charset: string;
    collation: string;
    created_at: string;
    updated_at: string;
    users_count: number;
}

export interface DatabaseUser {
    id: string;
    user_id: string;
    database_id: string;
    db_name: string;
    db_username: string;
    host: string;
    privileges: string;
    is_active: boolean;
    created_at: string;
    updated_at: string;
    phpmyadmin_info: {
        url: string;
        username: string;
        database: string;
        mysql_host: string;
    };
}

// Domain Types
export interface DomainResponse {
    id: string;
    user_id: string;
    domain_name: string;
    document_root: string;
    is_active: boolean;
    ssl_enabled: boolean;
    ssl_status?: string;
    ssl_provider?: string | null;
    ssl_expiry_at?: string | null;
    modsecurity_enabled?: boolean;
    created_at: string;
    updated_at: string;
}

export interface SubdomainResponse {
    id: string;
    domain_id: string;
    subdomain_name: string;
    full_domain: string;
    document_root: string;
    is_active: boolean;
    created_at: string;
}

export interface DnsRecordResponse {
    id: string;
    domain_id: string;
    record_type: string;
    name: string;
    value: string;
    ttl: number;
    priority: number | null;
    created_at: string;
    updated_at: string;
}
