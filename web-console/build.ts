import { $ } from "bun";

// Create dist directory
await $`mkdir -p dist`;

// Copy files
await $`cp src/index.html dist/`;
await $`cp vendor/alpine.min.js dist/`;
await $`cp vendor/alpine-persist.min.js dist/`;

// Build TypeScript
await Bun.build({
  entrypoints: ["./src/app.ts"],
  outdir: "./dist",
  minify: true,
  target: "browser",
  format: "iife",
});

console.log("✓ Build completed");