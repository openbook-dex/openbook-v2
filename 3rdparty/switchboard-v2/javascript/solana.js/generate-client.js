const shell = require('shelljs');
const path = require('path');
const fs = require('fs');
const { execSync } = require('child_process');

const projectRoot = __dirname;
// const shx = path.join(projectRoot, 'node_modules', '.bin', 'shx');

/**
 * Fetch a list of filepaths for a given directory and desired file extension
 * @param [dirPath] Filesystem path to a directory to search.
 * @param [arrayOfFiles] An array of existing file paths for recursive calls
 * @param [extensions] Optional, an array of desired extensions with the leading separator '.'
 * @throws {String}
 * @returns {string[]}
 */
const getAllFiles = (dirPath, arrayOfFiles, extensions) => {
  const files = fs.readdirSync(dirPath, 'utf8');

  arrayOfFiles = arrayOfFiles || [];

  files.forEach(file => {
    if (fs.statSync(dirPath + '/' + file).isDirectory()) {
      arrayOfFiles = getAllFiles(
        dirPath + '/' + file,
        arrayOfFiles,
        extensions
      );
    } else {
      const ext = path.extname(file);
      if (extensions && Array.isArray(extensions) && extensions.includes(ext)) {
        arrayOfFiles.push(path.join(dirPath, '/', file));
      } else {
        arrayOfFiles.push(path.join(dirPath, '/', file));
      }
      // if (!(extensions === undefined) || extensions.includes(ext)) {
      //   arrayOfFiles.push(path.join(dirPath, '/', file));
      // }
    }
  });

  return arrayOfFiles;
};

async function main() {
  shell.cd(projectRoot);

  if (!shell.which('anchor')) {
    shell.echo(
      "Sorry, this script requires 'anchor' to be installed in your $PATH"
    );
    shell.exit(1);
  }

  execSync(
    'anchor idl fetch -o ./src/idl/mainnet.json SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f --provider.cluster mainnet'
  );
  execSync(
    'anchor idl fetch -o ./src/idl/devnet.json 2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG --provider.cluster devnet'
  );

  execSync(
    'rm -rf ./src/generated && npx anchor-client-gen --program-id SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f ../../../switchboard-core/switchboard_v2/target/idl/switchboard_v2.json ./src/generated'
  );
  fs.writeFileSync(
    './src/generated/index.ts',
    [
      "export * from './accounts/index.js';",
      "export * from './errors/index.js';",
      "export * from './instructions/index.js';",
      "export * from './types/index.js';",
    ].join('\n')
  );

  // loop through directory and run regex replaces
  for await (const file of [
    ...getAllFiles('./src/generated/accounts'),
    ...getAllFiles('./src/generated/errors'),
    ...getAllFiles('./src/generated/instructions'),
    // ...getAllFiles('./src/generated/types'),
  ]) {
    if (file.includes('index.ts')) {
      continue;
    }
    const fileString = fs.readFileSync(file, 'utf-8');
    fs.writeFileSync(
      file,
      `import { SwitchboardProgram } from "../../program"\n${fileString}`
    );

    console.log(file);
    // remove PROGRAM_ID import, we will use SwitchboardProgram instead
    execSync(
      `sed -i '' 's/import { PROGRAM_ID } from "..\\/programId"/ /g' ${file}`
    );
    // replace PROGRAM_ID with program.programId
    execSync(`sed -i '' 's/PROGRAM_ID/program.programId/g' ${file}`);
    // replace Connection with SwitchboardProgram
    execSync(
      `sed -i '' 's/c: Connection,/program: SwitchboardProgram,/g' ${file}`
    );
    // replace c.getAccountInfo with the SwitchboardProgram connection
    execSync(
      `sed -i '' 's/c.getAccountInfo/program.connection.getAccountInfo/g' ${file}`
    );
    // replace c.getMultipleAccountsInfo with the SwitchboardProgram connection
    execSync(
      `sed -i '' 's/c.getMultipleAccountsInfo/program.connection.getMultipleAccountsInfo/g' ${file}`
    );

    // add program as first arguement to instructions
    if (file.includes('/instructions/')) {
      execSync(
        `sed -i '' 's/args:/program: SwitchboardProgram, args:/g' ${file}`
      );
    }
  }

  execSync('npx prettier ./src/generated --write');
}

main()
  .then(() => {
    // console.log("Executed successfully");
  })
  .catch(err => {
    console.error(err);
  });
