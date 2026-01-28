import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import paperclip from '@paperclip-lang/vite-plugin';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    paperclip({
      typescript: true,
      includeStyles: true,
    }),
    react(),
  ],
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx', '.pc'],
  },
});
