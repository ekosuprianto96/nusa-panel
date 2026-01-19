import path from 'path';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwind from 'tailwindcss';
import autoprefixer from 'autoprefixer';

// https://vitejs.dev/config/
export default defineConfig({
    css: {
        postcss: {
            plugins: [tailwind(), autoprefixer()],
        },
    },
    plugins: [vue()],
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
    // Optimasi build - split Monaco ke chunk terpisah
    build: {
        rollupOptions: {
            output: {
                manualChunks: {
                    // Monaco editor di chunk terpisah
                    'monaco-editor': ['monaco-editor'],
                },
            },
        },
        // Monaco memang besar
        chunkSizeWarningLimit: 2000,
    },
});
