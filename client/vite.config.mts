import { defineConfig } from 'vite';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    server: {
        https: {
            key: readFileSync(resolve(__dirname, 'localhost-key.pem')),
            cert: readFileSync(resolve(__dirname, 'localhost.pem')),
        },
        port: 3000,
    }
});