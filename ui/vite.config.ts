import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { viteSingleFile } from 'vite-plugin-singlefile'
import { resolve } from 'path'

export default defineConfig(() => {
  const isWebBuild = process.env.VITE_RUNTIME === 'web'

  return {
    plugins: [
      vue(),
      tailwindcss(),
      ...(!isWebBuild
        ? [
            viteSingleFile({
              useRecommendedBuildConfig: true,
              removeViteModuleLoader: true,
              deleteInlinedFiles: true,
            }),
          ]
        : []),
    ],
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    define: {
      'import.meta.env.VITE_RUNTIME': JSON.stringify(process.env.VITE_RUNTIME || 'juce'),
    },
    build: {
      target: 'esnext',
      minify: 'terser' as const,
      outDir: isWebBuild ? 'dist-web' : 'dist',
    },
    base: isWebBuild ? '/' : './',
  }
})
