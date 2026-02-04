<?php
/**
 * phpMyAdmin Configuration untuk Nusa Panel SSO
 * 
 * File ini harus di-include atau replace config.inc.php phpMyAdmin.
 * Lokasi default phpMyAdmin: /usr/share/phpmyadmin/
 * 
 * @author Nusa Panel Team
 */

// Use Server 1 directly
$server = 1;

// Authentication type: signon untuk SSO
$cfg['Servers'][$server]['auth_type'] = 'signon';

// Session name yang digunakan oleh signon.php
$cfg['Servers'][$server]['SignonSession'] = 'NusaPanelSignonSession';

// URL untuk redirect jika session expired (Auth Required)
$cfg['Servers'][$server]['SignonURL'] = getenv('PANEL_URL') ?: '/panel/databases';

// Path ke script signon
// WARNING: Defining SignonScript causes "Cannot use object as array" crash in PMA 5.2
// Since we use SignonURL for redirection, we don't strictly need to include the script here 
// as long as the external flow works.
// $cfg['Servers'][$server]['SignonScript'] = '/usr/share/phpmyadmin/signon.php';

// URL untuk logout - kembali ke panel
$cfg['Servers'][$server]['LogoutURL'] = getenv('PANEL_URL') ?: '/panel/databases';

// MySQL Server configuration
$cfg['Servers'][$server]['host'] = getenv('MYSQL_HOST') ?: 'localhost';
$cfg['Servers'][$server]['port'] = (int)(getenv('MYSQL_PORT') ?: 3306);
$cfg['Servers'][$server]['socket'] = getenv('MYSQL_SOCKET') ?: '';
$cfg['Servers'][$server]['compress'] = false;
$cfg['Servers'][$server]['AllowNoPassword'] = false;

// Connection type (socket lebih cepat untuk localhost)
if (!empty($cfg['Servers'][$server]['socket'])) {
    $cfg['Servers'][$server]['connect_type'] = 'socket';
} else {
    $cfg['Servers'][$server]['connect_type'] = 'tcp';
}

/**
 * Session settings
 */
$cfg['LoginCookieValidity'] = 86400;
$cfg['LoginCookieStore'] = 0;
$cfg['LoginCookieDeleteAll'] = true;

/**
 * Security settings
 */
$cfg['blowfish_secret'] = getenv('PMA_BLOWFISH_SECRET') 
    ?: 'change-this-to-your-secret-32chars';

// Directories
$cfg['TempDir'] = '/tmp/phpmyadmin';
$cfg['UploadDir'] = '';
$cfg['SaveDir'] = '';

/**
 * UI Settings
 */
$cfg['ThemeDefault'] = 'pmahomme';
$cfg['ShowDatabasesNavigationAsTree'] = true;
$cfg['MaxRows'] = 50;
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
