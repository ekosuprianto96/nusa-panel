# 📌 NusaPanel Feature Summary

Berikut adalah rangkuman fitur yang sudah ada di dalam **NusaPanel** berdasarkan analisis kode **Frontend & Backend**.

---

## ✅ 1. Core Features (Sudah Terimplementasi)

---

### 🖥️ Dashboard (`/dashboard`)

- Landing page utama setelah login.
- Menampilkan ringkasan resource atau status server.
- Catatan: Perlu verifikasi detail konten widget.

---

### 📂 File Manager (`/dashboard/files`)

**Backend:** `FileService`
Mendukung operasi file standar dalam sandbox user (`/home/user_xxxx`).

**Fitur:**

- List files & directories.
- Create file / folder.
- Read & write content (edit file).
- Delete, rename, move, copy.
- Basic permission check (sandbox & path traversal prevention).

**Limitasi Saat Ini:**

- Belum ada implementasi eksplisit:
    - Compress / Extract (ZIP).
    - Change Permissions (CHMOD).

- Walaupun struct sudah tersedia.

---

### 🌐 Domains Management (`/dashboard/domains`)

**Backend:** `DomainService`

**Fitur:**

- CRUD domain (Create, Read, Update, Delete).
- Unlimited subdomains.
- DNS Records (A, CNAME, MX, TXT, dll).
- Document root otomatis (`public_html`).
- Auto DNS:
    - Generate default A & MX records saat domain dibuat.

---

### 💾 Database Management (`/dashboard/databases`)

**Backend:** `DatabaseService`

**Fitur:**

- Isolated databases:
    - Prefix: `userid_dbname`.

- Database users per database.
- Privileges management (ALL / partial).
- Integrasi login phpMyAdmin.

---

### 📧 Email (`/dashboard/emails`)

**Backend:** `email_service.rs`

**Fitur:**

- Create email account.
- Delete email account.
- Update password email.

---

### 🛡️ Security (`/dashboard/security`)

**Backend:** `SecurityService`

**Fitur:**

- Manajemen firewall (UFW / iptables).
- SSH access control (potensial).

---

### ⚙️ System Tools (`/dashboard/system`)

**Backend:** `SystemService`

**Fitur Lengkap:**

- Cron Jobs:
    - Preset: Hourly, Daily, dll.

- Backups:
    - Full backup.
    - DB only.
    - Files only.

- PHP Version Selector:
    - Global / per-user (8.2, 8.3, dll).

- Error Logs Viewer:
    - PHP / Nginx.

- Services Manager:
    - Start / Stop / Restart:
        - Nginx
        - MySQL
        - PHP-FPM

---

### ⚡ Redis Manager (`/dashboard/redis`)

**Backend:** `RedisService`

**Fitur:**

- Manajemen Redis instance.
- Isolasi user atau shared instance management.

---

### 🌐 Web Server (`/dashboard/web-server`)

**Backend:** `WebServerService`

**Fitur:**

- Viewer & editor konfigurasi web server.
- Vhost config Nginx / Apache.

---

### 📱 App Installer (`/dashboard/apps`)

**Backend:** `AppInstallerService`

**Fitur:**

- Softaculous-like installer.
- WordPress:
    - Download WP-CLI.
    - Create database.
    - Generate `wp-config.php`.

- Laravel:
    - `composer create-project`.
    - `artisan key:generate`.
    - Migration.

---

---

## 🚀 2. Peluang Pengembangan Selanjutnya (Next Steps)

Berikut ide pengembangan agar **NusaPanel setara cPanel / CyberPanel**.

---

## 🔥 High Priority (Crusial for Hosting)

---

### 🔐 SSL / TLS Certificates (Let's Encrypt)

**Kondisi:**

- Ada field `ssl_enabled` di tabel domain.
- Belum ada automasi Certbot yang lengkap.

**Fitur:**

- Auto issue SSL.
- Auto renew.
- Upload custom certificate.

---

### 📁 FTP Account Management

**Kondisi:**

- Ada `ftp_service.rs`.
- Belum ada menu di sidebar frontend.

**Fitur:**

- Create FTP account tambahan.
- Directory restriction per user.

---

### 🧩 Advanced File Manager

**Kondisi:**

- Baru CRUD basic.

**Fitur Tambahan:**

- Zip / Unzip.
- Code editor:
    - Monaco / Ace Editor.

- Permissions UI:
    - CHMOD (755, 644, dll).

---

---

## 📈 Medium Priority (Value Add)

---

### 📊 Resource Monitoring (Real-time)

**Fitur:**

- CPU usage.
- RAM usage.
- Disk.
- Bandwidth per user / domain.

**Tech:**

- Netdata integration.
- Custom metrics collector.

---

### 🧑‍💻 Node.js / Python / Go Manager

**Kondisi:**

- Sekarang fokus PHP & Laravel.

**Fitur:**

- PM2 manager.
- Process lifecycle control.
- Multi runtime manager.

---

### ☁️ Cloud Backup (S3 / GDrive)

**Kondisi:**

- Backup masih lokal.

**Fitur:**

- Upload otomatis ke:
    - AWS S3.
    - S3-compatible storage.
    - Google Drive.

---

---

## 🛠️ System Admin Features

---

### 💻 Web Terminal

**Fitur:**

- Terminal berbasis web (xterm.js).
- Command:
    - `git pull`
    - `composer install`
    - `npm install`
    - dll langsung dari browser.

---

### 🔥 Firewall Manager (Advanced)

**Fitur:**

- IP whitelist / blacklist.
- Port management.
- Rule editor UI.

---

---

## ✅ Penutup

Dokumentasi ini bisa dijadikan:

- README project.
- Roadmap development.
- Proposal sistem panel hosting.
