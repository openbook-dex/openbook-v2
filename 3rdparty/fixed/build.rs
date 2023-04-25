// Copyright © 2018–2021 Trevor Spiteri

// Copying and distribution of this file, with or without
// modification, are permitted in any medium without royalty provided
// the copyright notice and this notice are preserved. This file is
// offered as-is, without any warranty.

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]

use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    io::{Result as IoResult, Write},
    path::{Path, PathBuf},
    process::Command,
};

struct Environment {
    out_dir: PathBuf,
    rustc: OsString,
}

fn main() {
    let env = Environment {
        out_dir: PathBuf::from(cargo_env("OUT_DIR")),
        rustc: cargo_env("RUSTC"),
    };
}

#[derive(PartialEq)]
struct Optional(bool);

impl Environment {
    //  1. If optional feature is availble (both with and without flag), output:
    //         cargo:rustc-cfg=<name>
    //  2. If feature is available with flag (both optional and not), output:
    //         cargo:rustc-cfg=nightly_<name>
    //  3. If non-optional feature is not available, panic.
    fn check_feature(
        &self,
        name: &str,
        optional: Optional,
        contents: &str,
        nightly_features: Option<&str>,
    ) {
        let try_dir = self.out_dir.join(format!("try_{}", name));
        let filename = format!("try_{}.rs", name);
        create_dir_or_panic(&try_dir);
        println!("$ cd {:?}", try_dir);

        #[derive(PartialEq)]
        enum Iteration {
            Stable,
            Unstable,
        }
        let mut found = false;
        for i in &[Iteration::Stable, Iteration::Unstable] {
            let s;
            let file_contents = match *i {
                Iteration::Stable => contents,
                Iteration::Unstable => match nightly_features {
                    Some(features) => {
                        s = format!("#![feature({})]\n{}", features, contents);
                        &s
                    }
                    None => continue,
                },
            };
            create_file_or_panic(&try_dir.join(&filename), file_contents);
            let mut cmd = Command::new(&self.rustc);
            cmd.current_dir(&try_dir)
                .args(&[&filename, "--emit=dep-info,metadata"]);
            println!("$ {:?}", cmd);
            let status = cmd
                .status()
                .unwrap_or_else(|_| panic!("Unable to execute: {:?}", cmd));
            if status.success() {
                if optional.0 {
                    println!("cargo:rustc-cfg={}", name);
                }
                if *i == Iteration::Unstable {
                    println!("cargo:rustc-cfg=nightly_{}", name);
                }
                found = true;
                break;
            }
        }
        remove_dir_or_panic(&try_dir);
        assert!(
            found || optional.0,
            "essential feature not supported by compiler: {}",
            name
        );
    }
}

fn cargo_env(name: &str) -> OsString {
    env::var_os(name)
        .unwrap_or_else(|| panic!("environment variable not found: {}, please use cargo", name))
}

fn remove_dir(dir: &Path) -> IoResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    assert!(dir.is_dir(), "Not a directory: {:?}", dir);
    println!("$ rm -r {:?}", dir);
    fs::remove_dir_all(dir)
}

fn remove_dir_or_panic(dir: &Path) {
    remove_dir(dir).unwrap_or_else(|_| panic!("Unable to remove directory: {:?}", dir));
}

fn create_dir(dir: &Path) -> IoResult<()> {
    println!("$ mkdir -p {:?}", dir);
    fs::create_dir_all(dir)
}

fn create_dir_or_panic(dir: &Path) {
    create_dir(dir).unwrap_or_else(|_| panic!("Unable to create directory: {:?}", dir));
}

fn create_file_or_panic(filename: &Path, contents: &str) {
    println!("$ printf '%s' {:?}... > {:?}", &contents[0..20], filename);
    let mut file =
        File::create(filename).unwrap_or_else(|_| panic!("Unable to create file: {:?}", filename));
    file.write_all(contents.as_bytes())
        .unwrap_or_else(|_| panic!("Unable to write to file: {:?}", filename));
}
