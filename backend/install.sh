#!/bin/bash
# NusaPanel Automated Installer Script
# Tested on Ubuntu 20.04/22.04 LTS

set -e

# ==========================================
# CONSTANTS & CONFIG
# ==========================================
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

APP_DIR="/opt/nusa-panel"
REPO_URL="https://github.com/yourusername/nusa-panel-rust.git" # Replace with actual repo
DB_NAME="nusa_panel"
DB_USER="nusa_admin"
SERVICE_USER="nusa-panel"

# ==========================================
# HELPER FUNCTIONS
# ==========================================
info() {
    echo -e "${GREEN}[INFO] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}"
    exit 1
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        error "Please run strictly as root (sudo bash install.sh)"
    fi
}

check_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        if [[ "$ID" != "ubuntu" && "$ID" != "debian" ]]; then
            error "This script currently supports Ubuntu and Debian only."
        fi
    else
        error "Unsupported OS."
    fi
}

prompt_input() {
    read -p "$1: " val
    echo $val
}

# ==========================================
# MAIN INSTALLATION STEPS
# ==========================================

# 1. System Prep
setup_system() {
    info "Step 1/7: Updating System & Installing Core Dependencies..."
    apt-get update
    apt-get install -y software-properties-common curl git unzip build-essential pkg-config libssl-dev \
        acl ufw tar htop
    
    # Install Node.js & NPM (for Frontend)
    if ! command -v node &> /dev/null; then
        info "Installing Node.js 20..."
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        apt-get install -y nodejs
    fi
}

# 2. Add Repositories & Install Stacks
install_stack() {
    info "Step 2/7: Installing LEMP Stack & Redis..."
    
    # PHP Repository
    add-apt-repository -y ppa:ondrej/php
    apt-get update

    # Nginx, MariaDB, Redis, Certbot
    apt-get install -y nginx mariadb-server redis-server certbot python3-certbot-nginx

    # Enable Services
    systemctl enable --now nginx
    systemctl enable --now mariadb
    systemctl enable --now redis-server
}

# 3. Install PHP Versions
install_php() {
    info "Step 3/7: Installing Multiple PHP Versions..."
    
    # Versions to install
    VERSIONS=("7.4" "8.0" "8.1" "8.2" "8.3")
    
    for ver in "${VERSIONS[@]}"; do
        info "Installing PHP $ver..."
        apt-get install -y php$ver php$ver-fpm php$ver-cli php$ver-common \
            php$ver-mysql php$ver-zip php$ver-gd php$ver-mbstring \
            php$ver-curl php$ver-xml php$ver-bcmath php$ver-soap php$ver-intl \
            php$ver-readline php$ver-pcov php$ver-msgpack php$ver-igbinary \
            php$ver-ldap php$ver-redis
            
        systemctl enable --now php$ver-fpm
    done
    
    # Set default PHP CLI to latest
    update-alternatives --set php /usr/bin/php8.3
}

# 4. Install Tools (Composer, WP-CLI)
install_tools() {
    info "Step 4/7: Installing Helper Tools (Composer, WP-CLI)..."
    
    # Composer
    if [ ! -f /usr/local/bin/composer ]; then
        curl -sS https://getcomposer.org/installer | php
        mv composer.phar /usr/local/bin/composer
    else
        composer self-update
    fi

    # WP-CLI
    if [ ! -f /usr/local/bin/wp ]; then
        curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar
        chmod +x wp-cli.phar
        mv wp-cli.phar /usr/local/bin/wp
    else
        wp cli update --allow-root --yes
    fi
}

# 5. Build Application
build_app() {
    info "Step 5/7: Building NusaPanel form Source..."
    
    # Create System User for Service
    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd -r -s /bin/false -d $APP_DIR $SERVICE_USER
    fi
    
    mkdir -p $APP_DIR
    chown $SERVICE_USER:$SERVICE_USER $APP_DIR
    
    # Install Rust
    if ! command -v cargo &> /dev/null; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi

    # Clone/Copy Source (Logic: If .git exists locally assume dev, else clone)
    # Since this script is likely curl'd, we should ask for Repo or assume current context
    # For MVP: We assume the user Curls this script, then we clone into APP_DIR
    
    else
        info "Downloading/Cloning Repository..."
        # Default to github default branch
        git clone $REPO_URL $APP_DIR
    fi

    # 5.1 Enable Real System Integration (Important for Linux VM)
    info "Step 5.1: Switching to Real System Services..."
    cd $APP_DIR/src/routes
    
    # Replace Service imports with Real implementations
    # Targets: UserService, SystemService, WebServerService, SecurityService, RedisService
    # Pattern: use crate::services::Service; -> use crate::services::ServiceReal as Service;
    
    # 1. UserService
    sed -i 's/use crate::services::UserService;/use crate::services::UserServiceReal as UserService;/g' *.rs
    
    # 2. SystemService
    sed -i 's/use crate::services::SystemService;/use crate::services::SystemServiceReal as SystemService;/g' *.rs
    
    # 3. WebServerService
    sed -i 's/use crate::services::WebServerService;/use crate::services::WebServerServiceReal as WebServerService;/g' *.rs
    
    # 4. SecurityService
    sed -i 's/use crate::services::SecurityService;/use crate::services::SecurityServiceReal as SecurityService;/g' *.rs
    
    # 5. RedisService
    sed -i 's/use crate::services::RedisService;/use crate::services::RedisServiceReal as RedisService;/g' *.rs

    info "Real System Services enabled."

    # Attempt Build if Cargo.toml exists
    if [ -f "$APP_DIR/Cargo.toml" ]; then
        cd $APP_DIR
        info "Building Release Binary (Backend)..."
        # Ensure build tools are available
        source $HOME/.cargo/env
        cargo build --release
    else
        info "Skipping backend build (Source code not found)."
    fi

    # 5.2 Build Frontend
    if [ -d "$APP_DIR/frontend" ]; then
        info "Building Frontend Application..."
        cd "$APP_DIR/frontend"
        npm install
        npm run build
        info "Frontend Built successfully."
    else
        info "Skipping frontend build (Directory not found)."
    fi
}

# 5.3 Setup phpMyAdmin
setup_phpmyadmin() {
    info "Step 5.3: Installing & Configuring phpMyAdmin..."
    
    # Pre-configure debconf to avoid prompts
    # We choose NOT to use dbconfig-common to auto-create db, as we manage it manually or it's just a GUI
    # Or strictly use defaults.
    echo "phpmyadmin phpmyadmin/dbconfig-install boolean false" | debconf-set-selections
    echo "phpmyadmin phpmyadmin/reconfigure-webserver multiselect" | debconf-set-selections
    
    export DEBIAN_FRONTEND=noninteractive
    apt-get install -y phpmyadmin
    
    # 1. Copy Config
    if [ -f "$APP_DIR/phpmyadmin/config.inc.php" ]; then
        info "Applying custom config.inc.php..."
        cp "$APP_DIR/phpmyadmin/config.inc.php" /etc/phpmyadmin/config.inc.php
        chown root:www-data /etc/phpmyadmin/config.inc.php
        chmod 640 /etc/phpmyadmin/config.inc.php
    else
        info "Warning: Custom config.inc.php not found in $APP_DIR/phpmyadmin/"
    fi

    # 2. Copy Signon Script
    if [ -f "$APP_DIR/phpmyadmin/signon.php" ]; then
        info "Deploying signon.php..."
        cp "$APP_DIR/phpmyadmin/signon.php" /usr/share/phpmyadmin/signon.php
        chown www-data:www-data /usr/share/phpmyadmin/signon.php
        chmod 644 /usr/share/phpmyadmin/signon.php
        
        # Inject Secrets (PMA_INTERNAL_KEY)
        # Use simple sed, assuming variable is set from setup_database
        if [ ! -z "$PMA_INTERNAL_KEY" ]; then
            info "Injecting PMA_INTERNAL_KEY into signon.php..."
            sed -i "s/change-me-in-production/$PMA_INTERNAL_KEY/g" /usr/share/phpmyadmin/signon.php
        fi
    else
        info "Warning: signon.php not found in $APP_DIR/phpmyadmin/"
    fi
    
    # 3. Inject Blowfish Secret into Config
    info "Generating Blowfish Secret..."
    BLOWFISH_SECRET=$(openssl rand -base64 32 | tr -d '/+' | cut -c1-32) # alphanumeric only for safety
    sed -i "s/change-this-to-your-secret-32chars/$BLOWFISH_SECRET/g" /etc/phpmyadmin/config.inc.php
    
    # 4. Ensure permissions for signon log
    mkdir -p /var/log/phpmyadmin
    chown www-data:www-data /var/log/phpmyadmin
}

# 6. Database Setup
setup_database() {
    info "Step 6/7: Configuring Database..."
    
    # Generate Password
    DB_PASS=$(openssl rand -base64 12)
    
    info "Creating Database '$DB_NAME' with User '$DB_USER'..."
    mysql -e "CREATE DATABASE IF NOT EXISTS $DB_NAME;"
    mysql -e "CREATE USER IF NOT EXISTS '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASS';"
    mysql -e "GRANT ALL PRIVILEGES ON $DB_NAME.* TO '$DB_USER'@'localhost';"
    mysql -e "FLUSH PRIVILEGES;"
    
    # Write .env
    # Generate PMA Key
    PMA_INTERNAL_KEY=$(openssl rand -hex 32)
    
    cat > "$APP_DIR/.env" <<EOF
DATABASE_URL=mysql://$DB_USER:$DB_PASS@localhost/$DB_NAME
JWT_SECRET=$(openssl rand -hex 32)
RUST_LOG=info
APP_ENV=production
APP_PORT=8000
PMA_INTERNAL_KEY=$PMA_INTERNAL_KEY
EOF

    chown $SERVICE_USER:$SERVICE_USER "$APP_DIR/.env"
    chmod 600 "$APP_DIR/.env"
    
    info "Database Configured. Check $APP_DIR/.env"
}

# 7. Security & Service
setup_security_service() {
    info "Step 7/7: Finalizing Security & Services..."
    
    # Sudoers for Service User
    # Allow nusa-panel service user to run specific commands as root without password
    echo "$SERVICE_USER OFF=(ALL) NOPASSWD: /usr/bin/systemctl, /usr/sbin/useradd, /usr/sbin/usermod, /usr/bin/passwd, /usr/bin/chown, /usr/bin/chmod, /usr/bin/openssl, /usr/bin/cp, /usr/bin/rm, /usr/bin/mkdir, /usr/bin/cat, /usr/bin/apt-get, /usr/bin/nginx, /usr/bin/certbot" > /etc/sudoers.d/nusa-panel
    chmod 0440 /etc/sudoers.d/nusa-panel

    # Setup Systemd
    cat > /etc/systemd/system/nusa-panel.service <<EOF
[Unit]
Description=NusaPanel Hosting Control Panel
After=network.target mysql.service redis-server.service

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/target/release/nusa-panel
Restart=always
EnvironmentFile=$APP_DIR/.env

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable nusa-panel
    systemctl start nusa-panel
    
    # Setup Nginx Proxy
    info "Configuring Nginx as Reverse Proxy..."
    cat > /etc/nginx/sites-available/nusa-panel <<EOF
server {
    listen 80;
    server_name _; # Change to domain if needed

    root $APP_DIR/frontend/dist;
    index index.html;

    # Backend API
    location /api/ {
        proxy_pass http://localhost:8000;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    # phpMyAdmin
    location ^~ /phpmyadmin/ {
        alias /usr/share/phpmyadmin/;
        index index.php;
        location ~ \.php$ {
            include fastcgi_params;
            fastcgi_pass unix:/run/php/php8.3-fpm.sock;
            fastcgi_param SCRIPT_FILENAME \$request_filename;
        }
    }

    # Frontend SPA
    location / {
        try_files \$uri \$uri/ /index.html;
    }
}
EOF
    ln -sf /etc/nginx/sites-available/nusa-panel /etc/nginx/sites-enabled/default
    systemctl restart nginx

    # Config UFW
    ufw allow 80/tcp
    ufw allow 443/tcp
    # ufw enable # Careful enforcing this automatically
}

# ==========================================
# EXECUTION
# ==========================================
check_root
check_os

echo "=========================================="
echo "   NusaPanel Automated Installer v1.0"
echo "=========================================="

setup_system
install_stack
install_php
install_tools
setup_database
build_app
setup_phpmyadmin
setup_security_service

info "Installation Completed Successfully!"
info "1. Upload your source code to $APP_DIR (if git clone was skipped)"
info "2. Run 'cd $APP_DIR && cargo build --release'"
info "3. Run 'systemctl start nusa-panel'"
info "Panel will be available at http://$(curl -s ifconfig.me):8000"
