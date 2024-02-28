import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import ElementPlus from "unplugin-element-plus/vite";
import { cdn } from "vite-plugin-cdn2";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    ElementPlus({}),
    cdn({
      modules: [
        {
          name: "vue",
          relativeModule: "./dist/vue.global.prod.js",
        },
        "vue-demi",
        {
          name: "vue-router",
          relativeModule: "./dist/vue-router.global.prod.js",
        },
        { name: "pinia", relativeModule: "./dist/pinia.iife.prod.js" },
        { name: "vue-i18n", relativeModule: "./dist/vue-i18n.global.prod.js" },
        {
          name: "element-plus",
          aliases: ["es", "lib"],
          relativeModule: "./dist/index.full.min.js",
        },
        { name: "naive-ui", relativeModule: "./dist/index.prod.js" },
        "axios",
        "chart.js",
        "lodash-es",
      ],
    }),
  ],
  server: {
    proxy: {
      // with options
      "/api": {
        target: "https://127.0.0.1:3344",
        changeOrigin: true,
        secure: false,
      },
      "/socket": {
        target: "wss://127.0.0.1:3344",
        changeOrigin: true,
        secure: false,
        ws: true,
      },
    },
  },
});
