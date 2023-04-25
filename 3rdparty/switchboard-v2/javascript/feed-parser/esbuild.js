#!/usr/bin/env node
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable import/no-extraneous-dependencies */

// bundle d.ts declarations
const { build } = require("estrella");

// Automatically exclude all node_modules from the bundled version
const { nodeExternalsPlugin } = require("esbuild-node-externals");

build({
  entryPoints: ["./src/main.ts"],
  outfile: "dist/index.js",
  bundle: true,
  minify: true,
  platform: "node",
  target: "node16",
  tslint: true,
  sourcemap: "inline",
  plugins: [nodeExternalsPlugin()],
});
