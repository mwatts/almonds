// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  components: [
    { path: "~/components" },
  ],
  app: {
    head: {
      meta: [
        {
          name: "viewport",
          content: "width=device-width, initial-scale=1, viewport-fit=cover",
        },
      ],
    },
  },
  css: [
    "highlight.js/styles/atom-one-dark.css",
    "@domternal/theme",
    "@/assets/css/main.css",
  ],
  ssr: false,
  modules: [
    // "@nuxt/a11y",
    "@nuxtjs/apollo",
    "@nuxt/eslint",
    "@nuxt/hints",
    "@nuxt/image",
    "@nuxt/ui",
    "@nuxtjs/device",
    "@nuxtjs/google-fonts",
    "@nuxtjs/i18n",
    "@pinia/nuxt",
    "pinia-plugin-persistedstate/nuxt",
    "@vueuse/nuxt",
  ],

  hooks: {
    "imports:extend": (imports) => {
      for (let i = imports.length - 1; i >= 0; i--) {
        if (imports[i].name === "options") {
          imports.splice(i, 1);
        }
      }
    },
  },

  apollo: {
    clients: {
      default: {
        httpEndpoint:
          process.env.NUXT_PUBLIC_APOLLO_ENDPOINT ||
          "http://localhost:8000/orchard",
      },
    },
  },

  runtimeConfig: {
    public: {
      serverUrl:
        process.env.SERVER_URL ||
        process.env.NUXT_PUBLIC_SERVER_URL ||
        "http://localhost:8000",
    },
  },

  icon: {
    serverBundle: {
      collections: ["heroicons", "lucide", "ri"],
    },
  },

  vite: {
    optimizeDeps: {
      include: [
        "@nuxt/ui > prosemirror-state",
        "@nuxt/ui > prosemirror-transform",
        "@nuxt/ui > prosemirror-model",
        "@nuxt/ui > prosemirror-view",
        "@nuxt/ui > prosemirror-gapcursor",
        "rehackt",
      ],
      // PGlite ships its own wasm + FS assets; pre-bundling it breaks
      // `new PGlite("idb://lunar")` in dev ("Invalid FS bundle size").
      exclude: ["@electric-sql/pglite"],
    },
    worker: {
      format: "es",
    },
  },
  devServer: {
    host: "0.0.0.0",
  },

  colorMode: {
    preference: "system",
    fallback: "light",
    globalName: "__NUXT_COLOR_MODE__",
    componentName: "ColorScheme",
    classPrefix: "",
    classSuffix: "",
    storage: "localStorage",
    storageKey: "nuxt-color-mode",
  },
});
