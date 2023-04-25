#!/usr/bin/env node
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable import/no-extraneous-dependencies */

// bundle d.ts declarations
const { build, ts, tsconfig, dirname, glob, log } = require("estrella");

// Automatically exclude all node_modules from the bundled version
const { nodeExternalsPlugin } = require("esbuild-node-externals");

build({
  entryPoints: ["./src/main.ts"],
  // outdir: "lib",
  outfile: "dist/main.js",
  bundle: true,
  minify: false,
  platform: "node",
  target: "node16",
  tslint: true,
  sourcemap: "inline",
  plugins: [nodeExternalsPlugin()],
  onEnd(config) {
    // Generate type declaration files
    const dtsFilesOutdir = dirname(config.outfile);
    generateTypeDefs(tsconfig(config), config.entry, dtsFilesOutdir);
  },
});

function generateTypeDefs(tscConfig, entryfiles, outdir) {
  const filenames = [
    ...new Set(
      // eslint-disable-next-line unicorn/prefer-spread
      (Array.isArray(entryfiles) ? entryfiles : [entryfiles]).concat(
        tscConfig.include || []
      )
    ),
  ].filter(Boolean);
  // log.info("Generating type declaration files for", filenames.join(", "));
  const compilerOptions = {
    ...tscConfig.compilerOptions,
    moduleResolution: undefined,
    declaration: true,
    outDir: outdir,
  };
  const program = ts.ts.createProgram(filenames, compilerOptions);
  const targetSourceFile = undefined;
  const writeFile = undefined;
  const cancellationToken = undefined;
  const emitOnlyDtsFiles = true;
  program.emit(
    targetSourceFile,
    writeFile,
    cancellationToken,
    emitOnlyDtsFiles
  );
  // log.info("Wrote", glob(outdir + "/*.d.ts").join(", "));
}
