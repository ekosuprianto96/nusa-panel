import api from './api';
import type {
    ApiResponse,
    FileInfo,
    FileListResponse,
    FileContentResponse,
} from '@/types';

/**
 * File Management Service
 * Menangani operasi file system seperti list, read, write, rename, dll.
 */
export const fileService = {
    /**
     * Mendapatkan daftar file dalam direktori
     * @param path Path relatif dari user home
     * @param showHidden Tampilkan file tersembunyi
     */
    listFiles: (path: string = '', showHidden: boolean = false) =>
        api.get<ApiResponse<FileListResponse>>('/files', {
            params: { path, show_hidden: showHidden },
        }),

    /**
     * Mendapatkan konten file teks
     * @param path Path file lengkap
     */
    getFileContent: (path: string) =>
        api.get<ApiResponse<FileContentResponse>>('/files/content', {
            params: { path },
        }),

    /**
     * Membuat file atau direktori baru
     * @param data Info file/direktori yang akan dibuat
     */
    create: (data: {
        path: string;
        name: string;
        file_type: 'file' | 'directory';
        content?: string;
        is_directory?: boolean;
    }) => api.post<ApiResponse<FileInfo>>('/files/create', data),

    /**
     * Menulis/menyimpan konten ke file
     * @param data Data konten dan path file
     */
    writeFile: (data: {
        path: string;
        content: string | undefined;
        encoding?: string;
        create_if_not_exists?: boolean;
    }) => api.put<ApiResponse<FileInfo>>('/files/content', data),

    /**
     * Me-rename file atau direktori
     * @param data Path asal dan nama baru
     */
    rename: (data: { path: string; new_name: string }) =>
        api.put<ApiResponse<FileInfo>>('/files/rename', data),

    /**
     * Menyalin file atau direktori
     * @param data Path sumber dan tujuan
     */
    copy: (data: {
        source: string;
        destination: string;
        overwrite?: boolean;
    }) => api.post<ApiResponse<FileInfo>>('/files/copy', data),

    /**
     * Memindahkan file atau direktori
     * @param data Path sumber dan tujuan
     */
    move: (data: {
        source: string;
        destination: string;
        overwrite?: boolean;
    }) => api.post<ApiResponse<FileInfo>>('/files/move', data),

    /**
     * Menghapus file atau direktori
     * @param data Path dan opsi rekursif
     */
    delete: (data: { path: string; recursive?: boolean }) =>
        api.post<ApiResponse<void>>('/files/delete', data),

    /**
     * Compress files/directories ke zip archive
     * @param data Files yang akan di-compress dan nama archive
     */
    compress: (data: {
        paths: string[];
        archive_name: string;
        format?: string;
    }) => api.post<ApiResponse<FileInfo>>('/files/compress', data),

    /**
     * Extract zip archive
     * @param data Path archive dan destination
     */
    extract: (data: {
        archive_path: string;
        destination: string;
        overwrite?: boolean;
    }) => api.post<ApiResponse<FileInfo>>('/files/extract', data),

    /**
     * Mencari file berdasarkan nama
     * @param query Query pencarian
     * @param path Optional path untuk memulai pencarian
     * @param limit Maximum hasil (default 50)
     */
    search: (query: string, path?: string, limit: number = 50) =>
        api.get<ApiResponse<FileInfo[]>>('/files/search', {
            params: { query, path, limit },
        }),

    /**
     * Upload file dengan progress tracking
     * @param file File yang akan diupload
     * @param targetPath Path tujuan
     * @param onProgress Callback untuk progress update (0-100)
     * @returns Promise dengan FileInfo
     */
    uploadWithProgress: (
        file: File | null,
        targetPath: string,
        onProgress: (percent: number) => void,
    ): Promise<ApiResponse<FileInfo>> => {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();

            reader.onload = async (e) => {
                const result = e.target?.result as string;
                const content = result.split(',')[1]; // Extract base64

                const xhr = new XMLHttpRequest();

                xhr.upload.addEventListener('progress', (event) => {
                    if (event.lengthComputable) {
                        const percent = Math.round(
                            (event.loaded / event.total) * 100,
                        );
                        onProgress(percent);
                    }
                });

                xhr.addEventListener('load', () => {
                    if (xhr.status >= 200 && xhr.status < 300) {
                        try {
                            const response = JSON.parse(xhr.responseText);
                            resolve(response);
                        } catch {
                            reject(new Error('Invalid response'));
                        }
                    } else {
                        reject(new Error(`Upload failed: ${xhr.status}`));
                    }
                });

                xhr.addEventListener('error', () => {
                    reject(new Error('Upload failed'));
                });

                xhr.open('PUT', '/api/files/content');
                xhr.setRequestHeader('Content-Type', 'application/json');

                const token = localStorage.getItem('access_token');
                if (token) {
                    xhr.setRequestHeader('Authorization', `Bearer ${token}`);
                }

                xhr.send(
                    JSON.stringify({
                        path: targetPath
                            ? `${targetPath}/${file?.name}`
                            : file?.name,
                        content: content,
                        encoding: 'base64',
                        create_if_not_exists: true,
                    }),
                );
            };

            reader.onerror = () => {
                reject(new Error('Failed to read file'));
            };

            if (file) {
                reader.readAsDataURL(file);
            }
        });
    },
};

export default fileService;
