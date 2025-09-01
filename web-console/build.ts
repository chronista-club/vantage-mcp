import { $ } from "bun";

// Create dist directory
await $`mkdir -p dist`;

// Copy files
await $`cp src/index.html dist/`;
await $`cp vendor/alpine.min.js dist/`;

// Build JS
await Bun.build({
  entrypoints: ["./src/app.js"],
  outdir: "./dist",
  minify: true,
});

console.log("✓ Build completed");