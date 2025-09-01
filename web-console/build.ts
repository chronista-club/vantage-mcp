import { $ } from "bun";

// Clean and create dist
await $`rm -rf dist && mkdir -p dist/vendor`;

// Copy files
await $`cp src/index.html dist/`;
await $`cp ../static/vendor/libs/alpine.min.js dist/`;
await $`cp ../static/vendor/css/tabler.min.css dist/vendor/`;
await $`cp ../static/vendor/js/tabler.min.js dist/vendor/`;
await $`cp ../static/vendor/libs/tabler-icons.min.css dist/vendor/`;

// Build JS
await Bun.build({
  entrypoints: ["./src/app.js"],
  outdir: "./dist",
  minify: true,
});

console.log("✓ Build completed");