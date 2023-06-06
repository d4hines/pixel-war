import { defineConfig, mergeConfig } from "vite";
import path from "path";

// https://vitejs.dev/config/
export default ({ command }) => {
  const isBuild = command === "build";
  return defineConfig({
    define: {
      global: 'globalThis'
    },
    build: {
     rollupOptions: {
        input: {
          index: './index.html',
          desktop: './desktop.html'
        }
      },
      target: "esnext",
      commonjsOptions: {
        transformMixedEsModules: true
      }
    },
    server: {
      port: 4000
    },
    resolve: {
      alias: {
        // dedupe @airgap/beacon-sdk
        // I almost have no idea why it needs `cjs` on dev and `esm` on build, but this is how it works 🤷‍♂️
        "@airgap/beacon-sdk": path.resolve(
          path.resolve(),
          `./node_modules/@airgap/beacon-sdk/dist/${
            isBuild ? "esm" : "cjs"
          }/index.js`
        ),
        // polyfills
        "readable-stream": "vite-compatible-readable-stream",
        stream: "vite-compatible-readable-stream",
      }
    }
  });
};