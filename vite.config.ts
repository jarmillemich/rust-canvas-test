import { fileURLToPath, URL } from 'node:url'

import { defineConfig, PluginOption } from 'vite'
import vue from '@vitejs/plugin-vue'
import wasm from 'vite-plugin-wasm'

/**
 * @param newFilename {string}
 * @returns {import('vite').Plugin}
 */
const githubPagesHack = () => {
  return {
    name: 'renameIndex',
    enforce: 'post',
    generateBundle(options, bundle) {
      // Duplicate our index to 404 so we can use history routing
      bundle['404.html'] = {
        ...bundle['index.html'],
        fileName: '404.html',
      }
      
    },
  } as PluginOption
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    wasm(),
    githubPagesHack(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      '@engine': fileURLToPath(new URL('./engine/pkg', import.meta.url)),
    }
  },
  build: {
    target: 'esnext'
  }
})
