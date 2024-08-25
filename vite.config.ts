import { createLogger, defineConfig } from "vite";
import { spawn } from "child_process";
import path from "path";

export default defineConfig({
  plugins: [
    {
      name: "wgsl-hmr",
      handleHotUpdate({ file, server }) {
        if (file.endsWith(".wgsl")) {
          server.ws.send({
            type: "custom",
            event: "wgsl-update",
            data: { file: file.replace(process.cwd() + "/assets/", "") },
          });
          return [];
        }
      },
    },
    {
      name: "rust-wasm",
      configureServer(server) {
        server.watcher.add(path.resolve(__dirname, "src/**/*.rs"));
        const logger = createLogger("info", {
          prefix: "[rust-wasm]",
        });
        server.watcher.on("change", (file) => {
          if (file.endsWith(".rs") && file.includes(path.join("src", ""))) {
            logger.info("rust file changed. rebuilding wasm...", {
              timestamp: true,
            });
            spawn("pnpm", ["run", "build:wasm"], { stdio: "inherit" });
          }
        });
      },
    },
  ],
});
