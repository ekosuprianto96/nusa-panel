<?php
/**
 * phpMyAdmin Configuration untuk Nusa Panel SSO
 * 
 * File ini harus di-include atau replace config.inc.php phpMyAdmin.
 * Lokasi default phpMyAdmin: /usr/share/phpmyadmin/
 * 
 * @author Nusa Panel Team
 */

declare(strict_types=1);

/**
 * Server configuration
 */
$i = 0;
$i++;

// Authentication type: signon untuk SSO
$cfg['Servers'][$i]['auth_type'] = 'signon';

// Session name yang digunakan oleh signon.php
$cfg['Servers'][$i]['SignonSession'] = 'NusaPanelSignonSession';

// URL untuk redirect jika session expired
$cfg['Servers'][$i]['SignonURL'] = getenv('PANEL_URL') ?: '/panel/databases';

// Path ke script signon
$cfg['Servers'][$i]['SignonScript'] = __DIR__ . '/signon.php';

// URL untuk logout - kembali ke panel
$cfg['Servers'][$i]['LogoutURL'] = getenv('PANEL_URL') ?: '/panel/databases';

// MySQL Server configuration
$cfg['Servers'][$i]['host'] = getenv('MYSQL_HOST') ?: 'localhost';
$cfg['Servers'][$i]['port'] = (int)(getenv('MYSQL_PORT') ?: 3306);
$cfg['Servers'][$i]['socket'] = getenv('MYSQL_SOCKET') ?: '';
$cfg['Servers'][$i]['compress'] = false;
$cfg['Servers'][$i]['AllowNoPassword'] = false;

// Connection type (socket lebih cepat untuk localhost)
if (!empty($cfg['Servers'][$i]['socket'])) {
    $cfg['Servers'][$i]['connect_type'] = 'socket';
} else {
    $cfg['Servers'][$i]['connect_type'] = 'tcp';
}

/**
 * Session settings
 */
// Cookie validity in seconds (24 hours)
$cfg['LoginCookieValidity'] = 86400;

// Remember login dalam browser session
$cfg['LoginCookieStore'] = 0;

// Logout dari semua server sekaligus
$cfg['LoginCookieDeleteAll'] = true;

/**
 * Security settings
 */
// Blowfish secret untuk cookie encryption
$cfg['blowfish_secret'] = getenv('PMA_BLOWFISH_SECRET') 
    ?: 'change-this-to-your-secret-32chars';

// Directories
$cfg['TempDir'] = '/tmp/phpmyadmin';
$cfg['UploadDir'] = '';
$cfg['SaveDir'] = '';

/**
 * UI Settings
 */
// Theme
$cfg['ThemeDefault'] = 'pmahomme';

// Show database list on navigation panel
$cfg['ShowDatabasesNavigationAsTree'] = true;

// Maximum rows to display
$cfg['MaxRows'] = 50;

// Default language
$cfg['DefaultLang'] = 'en';

/**
 * Export/Import settings
 */
$cfg['Export']['method'] = 'quick';
$cfg['Import']['charset'] = 'utf-8';

/**
 * Disable version check (untuk keamanan)
 */
$cfg['VersionCheck'] = false;

/**
 * Additional security
 */
// Aktifkan untuk production
// $cfg['CaptchaLoginPublicKey'] = '';
// $cfg['CaptchaLoginPrivateKey'] = '';
