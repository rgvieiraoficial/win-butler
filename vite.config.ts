import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri espera porta fixa e não deve limpar a tela do processo.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  // Evita que o Vite ofusque o binário nativo em builds de produção do Tauri.
  build: {
    target: "es2021",
    sourcemap: false,
  },
});
