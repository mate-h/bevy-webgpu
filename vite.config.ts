import { createLogger, defineConfig, type Plugin, type Logger } from "vite";
import { spawn } from "child_process";
import path from "path";
import fs from "fs";

function buildWasm(logger: Logger): Promise<void> {
  logger.info("building wasm...", { timestamp: true });
  return new Promise((resolve, reject) => {
    const child = spawn("pnpm", ["run", "build:wasm"], {
      stdio: "inherit",
      shell: process.platform === "win32",
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        logger.info("wasm build complete", { timestamp: true });
        resolve();
      } else {
        reject(new Error(`wasm build failed with exit code ${code}`));
      }
    });
  });
}

function rustWasmPlugin(): Plugin {
  const logger = createLogger("info", {
    prefix: "[rust-wasm]",
  });
  let building: Promise<void> | null = null;

  const rebuild = () => {
    if (building) return building;
    building = buildWasm(logger).finally(() => {
      building = null;
    });
    return building;
  };

  return {
    name: "rust-wasm",
    async configureServer(server) {
      server.watcher.add(path.resolve(__dirname, "src/**/*.rs"));
      await rebuild();
      server.watcher.on("change", (file) => {
        if (file.endsWith(".rs") && file.includes(path.join("src", ""))) {
          logger.info("rust file changed. rebuilding wasm...", {
            timestamp: true,
          });
          rebuild().catch((err) => {
            logger.error(String(err), { timestamp: true });
          });
        }
      });
    },
  };
}

export default defineConfig({
  plugins: [
    {
      name: "copy-assets",
      generateBundle() {
        const files = fs.readdirSync("assets", { recursive: true });
        for (const file of files) {
          const p = "assets/" + file;
          if (fs.statSync(p).isDirectory()) continue;
          this.emitFile({
            type: "asset",
            fileName: p,
            source: fs.readFileSync(p),
          });
        }
      },
    },
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
    rustWasmPlugin(),
  ],
});
