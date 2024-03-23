import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasmPack from 'vite-plugin-wasm-pack';

// https://vitejs.dev/config/
export default defineConfig({
  base: '',
  plugins: [react(), wasmPack('./rs-pixel-maps')],
})
