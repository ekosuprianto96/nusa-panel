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
};

export default fileService;
