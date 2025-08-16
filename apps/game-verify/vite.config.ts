import { defineConfig, UserConfig } from 'vite'
import { fileURLToPath, URL } from 'url';
import environment from 'vite-plugin-environment';
import react from '@vitejs/plugin-react'
import injectHTML from 'vite-plugin-html-inject';

// https://vite.dev/config/
export default defineConfig(({ mode }: UserConfig) => {
  const config: UserConfig = {
    envDir: '../../',
    build: {
      emptyOutDir: true,
    },
    optimizeDeps: {
      esbuildOptions: {
        define: {
          global: "globalThis",
        },
      },
    },
    server: {
      host: "0.0.0.0",
      // 兼容 Windows WSL2
      watch: {
        usePolling: true 
      },
      proxy: {
        "/api": {
          target: ['dev', 'test', 'prod'].includes(mode || '') ? "https://ic0.app" : "http://127.0.0.1:8080",
          changeOrigin: true,
        },
      },
    },
    esbuild: {
      drop: mode === 'prod' ? ['console', 'debugger'] : []
    },
    plugins: [
      react(),
      injectHTML(),
      environment("all", { prefix: "CANISTER_" }),
      environment("all", { prefix: "DFX_" }),
    ],
    resolve: {
      alias: [
        {
          find: "@/",
          replacement: fileURLToPath(
            new URL("./src/", import.meta.url)
          ),
        },
        {
          find: "declarations",
          replacement: fileURLToPath(
            new URL("../declarations", import.meta.url)
          ),
        },
      ],
    }
  };
  return config;
});
