import { defineConfig } from 'vite';
export default defineConfig({
  root: 'desktop',
  server: { host:'127.0.0.1', port:1420, strictPort:true, fs:{ allow:['..'] } },
  clearScreen:false,
  build:{outDir:'../dist',emptyOutDir:true,target:'es2022'}
});
