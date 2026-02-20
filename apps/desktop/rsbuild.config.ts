import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
    plugins: [pluginReact()],
    source: {
        entry: {
            index: "./src/main.tsx",
        },
    },
    server: {
        port: 1420,
        host: host || "127.0.0.1",
    },
    output: {
        distPath: {
            root: "dist",
        },
    },
    tools: {
        rspack: {
            watchOptions: {
                ignored: ["**/src-tauri/**"],
            },
        },
    },
});
