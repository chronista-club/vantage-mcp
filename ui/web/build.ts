import { $ } from "bun";

// Create dist directory
await $`mkdir -p dist`;

// Copy files
await $`cp src/index.html dist/`;
await $`cp src/styles.css dist/`;
await $`cp -r vendor/* dist/`;  // Copy all vendor files including Tabler

// Build TypeScript
await Bun.build({
  entrypoints: ["./src/index.ts"],
  outdir: "./dist",
  minify: true,
  target: "browser",
  format: "iife",
  naming: "[name].js", // Output as index.js
});

console.log("✓ Build completed");