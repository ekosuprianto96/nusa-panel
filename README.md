# 🚀 NusaPanel

**NusaPanel** adalah panel kontrol hosting web modern yang dibangun dengan teknologi terkini. Panel ini dirancang sebagai alternatif ringan dan cepat untuk cPanel/CyberPanel, dengan fokus pada performa tinggi dan keamanan.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Vue](https://img.shields.io/badge/vue-3.5-green.svg)

---

## 📋 Daftar Isi

- [Tech Stack](#-tech-stack)
- [Kelebihan](#-kelebihan)
- [Fitur Utama](#-fitur-utama)
- [Persyaratan Sistem](#-persyaratan-sistem)
- [Instalasi](#-instalasi)
- [Konfigurasi](#-konfigurasi)
- [Menjalankan Aplikasi](#-menjalankan-aplikasi)
- [Struktur Proyek](#-struktur-proyek)
- [Lisensi](#-lisensi)

---

## 🛠 Tech Stack

### Backend
| Teknologi | Versi | Deskripsi |
|-----------|-------|-----------|
| **Rust** | 1.70+ | Bahasa pemrograman sistem yang aman dan cepat |
| **Rocket** | 0.5 | Web framework async untuk Rust dengan fitur JSON & secrets |
| **SQLx** | 0.7 | Database toolkit async untuk MySQL dengan compile-time verification |
| **Tokio** | 1.0 | Async runtime untuk concurrent operations |
| **Argon2** | 0.5 | Library hashing password yang aman |
| **JWT** | 9.0 | JSON Web Token untuk autentikasi |

### Frontend
| Teknologi | Versi | Deskripsi |
|-----------|-------|-----------|
| **Vue.js** | 3.5 | Framework JavaScript progresif untuk UI |
| **TypeScript** | 5.9 | JavaScript dengan static typing |
| **Vite** | 7.2 | Build tool next-generation yang super cepat |
| **TailwindCSS** | 3.4 | Utility-first CSS framework |
| **Pinia** | 3.0 | State management untuk Vue |
| **Monaco Editor** | 0.55 | Code editor powerful (sama seperti VS Code) |
| **Radix Vue** | 1.9 | Komponen UI headless yang accessible |

### Database & Services
| Teknologi | Deskripsi |
|-----------|-----------|
| **MySQL** | Database utama untuk menyimpan data panel |
| **Nginx** | Web server untuk serving aplikasi |
| **PHP-FPM** | PHP FastCGI Process Manager |
| **Redis** | In-memory cache dan session storage |

---

## ✨ Kelebihan

### 🚀 Performa Tinggi
- **Backend Rust** - Kecepatan mendekati C/C++ dengan memory safety
- **Zero-cost abstractions** - Tidak ada overhead runtime
- **Async I/O** - Menangani ribuan koneksi concurrent dengan efisien

### 🔒 Keamanan Terjamin
- **Memory Safety** - Rust mencegah buffer overflow dan memory leaks
- **Argon2 Hashing** - Password hashing dengan standar industri terkini
- **JWT Authentication** - Token-based auth yang stateless dan aman
- **Sandbox Isolation** - Setiap user terisolasi di direktori home masing-masing
- **Path Traversal Prevention** - Mencegah akses file di luar sandbox

### 💡 Ringan & Efisien
- **Low Memory Footprint** - Penggunaan RAM minimal dibanding panel PHP
- **Fast Compilation** - Build time yang optimal dengan binary tunggal
- **No Runtime Dependency** - Tidak memerlukan interpreter seperti PHP/Python

### 🎨 UI Modern
- **Vue 3 Composition API** - Kode frontend yang bersih dan maintainable
- **Monaco Editor** - Editor kode built-in sekelas VS Code
- **Responsive Design** - Tampilan optimal di desktop dan mobile
- **Dark/Light Theme** - Dukungan tema gelap dan terang

### 📦 Fitur Lengkap
- **Multi-Domain Support** - Kelola unlimited domain dan subdomain
- **Database Management** - Buat dan kelola database MySQL dengan mudah
- **Email Management** - Buat akun email untuk domain anda
- **App Installer** - Install WordPress, Laravel dengan satu klik
- **File Manager** - Kelola file langsung dari browser

---

## 🎯 Fitur Utama

| Fitur | Deskripsi |
|-------|-----------|
| 📂 **File Manager** | Upload, edit, delete, copy, move file dengan UI intuitif |
| 🌐 **Domain Manager** | Kelola domain, subdomain, dan DNS records |
| 💾 **Database** | Buat database MySQL dengan user isolation |
| 📧 **Email** | Buat dan kelola akun email |
| ⚙️ **System Tools** | Cron jobs, backup, PHP version selector, error logs |
| 🛡️ **Security** | Firewall management dan SSH access control |
| 📱 **App Installer** | Install WordPress, Laravel otomatis |
| ⚡ **Redis Manager** | Kelola Redis instance |
| 🌐 **Web Server** | Konfigurasi Nginx/Apache vhost |

---

## 📌 Persyaratan Sistem

### Minimum
- **OS**: Ubuntu 20.04 LTS / Debian 11+
- **RAM**: 1 GB
- **CPU**: 1 Core
- **Disk**: 20 GB

### Recommended
- **OS**: Ubuntu 22.04 LTS / Ubuntu 24.04 LTS
- **RAM**: 2 GB+
- **CPU**: 2+ Core
- **Disk**: 40 GB+ SSD

### Software Requirements
- Rust 1.70+
- Node.js 18+
- MySQL 8.0+
- Nginx 1.18+
- PHP 8.1+ (dengan PHP-FPM)

---

## 📥 Instalasi

### 1. Clone Repository

```bash
git clone https://github.com/your-username/nusa-panel-rust.git
cd nusa-panel-rust
```

### 2. Install Dependencies

#### Backend (Rust)
```bash
# Install Rust jika belum ada
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build backend
cd backend
cargo build --release
```

#### Frontend (Vue)
```bash
cd frontend
npm install
npm run build
```

### 3. Setup Database

```bash
# Login ke MySQL
mysql -u root -p

# Buat database
CREATE DATABASE nusa_panel;
CREATE USER 'nusapanel'@'localhost' IDENTIFIED BY 'your_password';
GRANT ALL PRIVILEGES ON nusa_panel.* TO 'nusapanel'@'localhost';
FLUSH PRIVILEGES;
```

### 4. Konfigurasi Environment

```bash
# Backend
cd backend
cp .env.example .env
nano .env
```

Isi file `.env`:
```env
DATABASE_URL=mysql://nusapanel:your_password@localhost/nusa_panel
JWT_SECRET=your_super_secret_jwt_key_here
ROCKET_PORT=8000
ROCKET_ADDRESS=127.0.0.1
```

---

## ⚙️ Konfigurasi

### Backend Configuration

Edit file `.env` di folder `backend/`:

| Variable | Deskripsi | Default |
|----------|-----------|---------|
| `DATABASE_URL` | URL koneksi MySQL | - |
| `JWT_SECRET` | Secret key untuk JWT | - |
| `ROCKET_PORT` | Port API | 8000 |
| `ROCKET_ADDRESS` | Bind address | 127.0.0.1 |

### Frontend Configuration

Edit file `.env` di folder `frontend/`:

| Variable | Deskripsi | Default |
|----------|-----------|---------|
| `VITE_API_URL` | URL Backend API | http://localhost:8000 |

---

## ▶️ Menjalankan Aplikasi

### Development Mode

#### Backend
```bash
cd backend
cargo run
```

#### Frontend
```bash
cd frontend
npm run dev
```

Akses aplikasi di: `http://localhost:5173`

### Production Mode

#### Backend
```bash
cd backend
cargo build --release
./target/release/nusa-panel
```

#### Frontend
```bash
cd frontend
npm run build
# Deploy folder dist/ ke web server
```

### Menggunakan Systemd (Production)

Buat file service untuk backend:

```bash
sudo nano /etc/systemd/system/nusa-panel.service
```

```ini
[Unit]
Description=NusaPanel Backend API
After=network.target mysql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/nusa-panel/backend
ExecStart=/opt/nusa-panel/backend/target/release/nusa-panel
Restart=always
RestartSec=5
Environment=ROCKET_ENV=production

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable nusa-panel
sudo systemctl start nusa-panel
```

---

## 📁 Struktur Proyek

```
nusa-panel-rust/
├── backend/                 # Rust Backend API
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── routes/         # API endpoints
│   │   ├── services/       # Business logic
│   │   ├── models/         # Data models
│   │   └── middleware/     # Auth & middleware
│   ├── Cargo.toml          # Rust dependencies
│   └── .env                # Environment config
│
├── frontend/               # Vue.js Frontend
│   ├── src/
│   │   ├── pages/          # Page components
│   │   ├── components/     # Reusable components
│   │   ├── stores/         # Pinia stores
│   │   ├── services/       # API services
│   │   └── layouts/        # Layout components
│   ├── package.json        # NPM dependencies
│   └── vite.config.ts      # Vite configuration
│
└── README.md               # Dokumentasi ini
```

---

## 📄 Lisensi

Proyek ini dilisensikan di bawah [MIT License](LICENSE).

---

## 🤝 Kontribusi

Kontribusi sangat diterima! Silakan buat issue atau pull request.

1. Fork repository
2. Buat branch fitur (`git checkout -b feature/AmazingFeature`)
3. Commit perubahan (`git commit -m 'Add some AmazingFeature'`)
4. Push ke branch (`git push origin feature/AmazingFeature`)
5. Buat Pull Request

---

## 📞 Kontak

Jika ada pertanyaan atau saran, silakan buat issue di repository ini.

---

<p align="center">
  Made with ❤️ by NusaPanel Team
</p>
