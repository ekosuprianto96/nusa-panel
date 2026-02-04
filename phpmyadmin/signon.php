<?php
/**
 * Nusa Panel phpMyAdmin Signon Script
 * 
 * Script ini menerima token dari panel dan mengambil
 * kredensial dari Rust API untuk auto-login.
 * 
 * @author Nusa Panel Team
 * @version 1.0.0
 */

declare(strict_types=1);

// Configuration
$config = [
    // Rust API endpoint untuk validasi token
    'api_url' => getenv('NUSA_PANEL_API_URL') ?: 'http://localhost:8095',
    
    // Internal key untuk autentikasi ke API
    'internal_key' => getenv('PMA_INTERNAL_KEY') ?: 'change-me-in-production',
    
    // Session name untuk phpMyAdmin
    'session_name' => 'NusaPanelSignonSession',
    
    // Redirect URL jika gagal
    'error_redirect' => getenv('PANEL_URL') ?: '/panel/databases',
    
    // Timeout untuk request ke API (detik)
    'api_timeout' => 5,
];

/**
 * Log error ke file
 * 
 * @param string $message Pesan error
 */
function logError(string $message): void {
    $logFile = getenv('PMA_SIGNON_LOG') ?: '/var/log/phpmyadmin/signon.log';
    $timestamp = date('Y-m-d H:i:s');
    @file_put_contents($logFile, "[$timestamp] $message\n", FILE_APPEND);
}

/**
 * Redirect dengan error message
 * 
 * @param string $errorCode Kode error untuk query string
 */
function redirectWithError(string $errorCode): void {
    global $config;
    
    session_destroy();
    header('Location: ' . $config['error_redirect'] . '?error=' . urlencode($errorCode));
    exit;
}

/**
 * Validasi token dengan Rust API
 * 
 * @param string $token Token dari query string
 * @return array|null Credentials jika valid, null jika tidak
 */
function validateToken(string $token): ?array {
    global $config;
    
    $url = rtrim($config['api_url'], '/') . '/api/phpmyadmin/validate';
    
    $ch = curl_init($url);
    $postData = json_encode(['token' => $token]);
    
    curl_setopt_array($ch, [
        CURLOPT_POST => true,
        CURLOPT_POSTFIELDS => $postData,
        CURLOPT_HTTPHEADER => [
            'Content-Type: application/json',
            'X-Internal-Key: ' . $config['internal_key'],
        ],
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_TIMEOUT => $config['api_timeout'],
        CURLOPT_CONNECTTIMEOUT => 3,
        // Untuk localhost, skip SSL verification
        CURLOPT_SSL_VERIFYPEER => !str_contains($config['api_url'], 'localhost'),
    ]);
    
    $response = curl_exec($ch);
    $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);
    $error = curl_error($ch);
    curl_close($ch);
    
    if ($error) {
        logError("cURL error: $error");
        return null;
    }
    
    if ($httpCode !== 200) {
        logError("API returned HTTP $httpCode: $response");
        return null;
    }
    
    $data = json_decode($response, true);
    
    if (!$data || !isset($data['user']) || !isset($data['password'])) {
        logError('Invalid API response: ' . $response);
        return null;
    }
    
    return $data;
}

// ==========================================
// MAIN EXECUTION
// ==========================================

// Start named session for phpMyAdmin
session_name($config['session_name']);
session_start();

// Get token from URL
$token = $_GET['token'] ?? null;

if (empty($token)) {
    logError('No token provided');
    redirectWithError('no_token');
}

// Sanitize token (hanya izinkan UUID format)
if (!preg_match('/^[a-f0-9\-]{36}$/i', $token)) {
    logError('Invalid token format: ' . substr($token, 0, 50));
    redirectWithError('invalid_token_format');
}

// Validate token dengan API
$credentials = validateToken($token);

if ($credentials === null) {
    redirectWithError('invalid_token');
}

// Set session variables for phpMyAdmin
$_SESSION['PMA_single_signon_user'] = $credentials['user'];
$_SESSION['PMA_single_signon_password'] = $credentials['password'];
$_SESSION['PMA_single_signon_host'] = $credentials['host'] ?? 'localhost';
$_SESSION['PMA_single_signon_port'] = (int)($credentials['port'] ?? 3306);

// Set database jika ada
if (!empty($credentials['database'])) {
    $_SESSION['PMA_single_signon_database'] = $credentials['database'];
}

// Log successful signon (tanpa password)
logError('Successful signon for user: ' . $credentials['user']);

// Redirect to phpMyAdmin
header('Location: index.php');
exit;
