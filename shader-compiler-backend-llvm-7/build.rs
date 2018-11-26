// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

// Partially based on llvm-sys; llvm-sys's license is reproduced below:

// Copyright (c) 2015 Peter Marheine
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate bindgen;
extern crate cc;
extern crate cmake;
extern crate fs2;
extern crate reqwest;
extern crate ring;
extern crate tar;
extern crate which;
extern crate xz2;
use fs2::FileExt;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const LLVM_7_SOURCE_TAR_XZ_URL: &'static str =
    "http://releases.llvm.org/7.0.0/llvm-7.0.0.src.tar.xz";

const LLVM_7_SOURCE_TAR_XZ_SHA256_HASH: &'static [u8; 32] = &[
    0x8b, 0xc1, 0xf8, 0x44, 0xe6, 0xcb, 0xde, 0x1b, 0x65, 0x2c, 0x19, 0xc1, 0xed, 0xeb, 0xc1, 0x86,
    0x44, 0x56, 0xfd, 0x9c, 0x78, 0xb8, 0xc1, 0xbe, 0xa0, 0x38, 0xe5, 0x1b, 0x36, 0x3f, 0xe2, 0x22,
];

const LLVM_7_SOURCE_DIR_SUFFIX: &'static str = "llvm-7.0.0.src";

fn verify_sha256(mut f: fs::File, file_path: &Path) -> fs::File {
    f.seek(io::SeekFrom::Start(0)).unwrap();
    let mut context = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buffer = [0; 1 << 12]; // 4KiB
    loop {
        let count = f.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    let hash = context.finish();
    if hash.as_ref() != LLVM_7_SOURCE_TAR_XZ_SHA256_HASH {
        panic!(
            "file is corrupted: SHA256 doesn't match; try deleting {} and rerunning cargo",
            file_path.display(),
        );
    }
    f.seek(io::SeekFrom::Start(0)).unwrap();
    f
}

fn download_llvm_7(out_dir: &Path) -> io::Result<fs::File> {
    let filename = LLVM_7_SOURCE_TAR_XZ_URL.rsplit('/').next().unwrap();
    let file_path = out_dir.join(filename);
    match fs::File::open(&file_path) {
        Ok(file) => return Ok(verify_sha256(file, &file_path)),
        Err(ref error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    };
    let response = reqwest::get(LLVM_7_SOURCE_TAR_XZ_URL)
        .map_err(|v| io::Error::new(io::ErrorKind::Other, v))?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)?;
    { response }
        .copy_to(&mut { file })
        .map_err(|v| io::Error::new(io::ErrorKind::Other, v))?;
    Ok(verify_sha256(fs::File::open(&file_path)?, &file_path))
}

fn extract_tar_xz<R: Read, T: AsRef<Path>>(r: R, target_path: T) -> io::Result<()> {
    tar::Archive::new(xz2::read::XzDecoder::new(r)).unpack(target_path)
}

fn download_and_extract_llvm_7_if_needed(llvm_dir: &Path) -> io::Result<PathBuf> {
    let source_dir = llvm_dir.join(LLVM_7_SOURCE_DIR_SUFFIX);
    match fs::File::open(source_dir.join("CMakeLists.txt")) {
        Ok(_) => return Ok(source_dir),
        Err(ref error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    extract_tar_xz(download_llvm_7(llvm_dir)?, llvm_dir)?;
    Ok(source_dir)
}

fn make_config(llvm_dir: &Path) -> cmake::Config {
    let mut retval = cmake::Config::new(llvm_dir.join(LLVM_7_SOURCE_DIR_SUFFIX));
    let found_ccache = match which::which("ccache") {
        Err(ref error) if error.kind() == which::ErrorKind::CannotFindBinaryPath => false,
        result => {
            result.unwrap();
            true
        }
    };
    retval
        .generator("Ninja")
        .define("LLVM_TARGETS_TO_BUILD", "host")
        .define("LLVM_CCACHE_BUILD", if found_ccache { "ON" } else { "OFF" })
        .define("LLVM_APPEND_VC_REV", "OFF") // stop llvm needing relink after git commit
        .define(
            "LLVM_TARGET_ARCH",
            env::var("TARGET").unwrap().split("-").next().unwrap(),
        )
        .out_dir(llvm_dir)
        .profile("Debug")
        .always_configure(false);
    retval
}

fn llvm_config<A: IntoIterator<Item = S>, S: AsRef<OsStr>>(
    llvm_config_path: &Path,
    args: A,
) -> String {
    String::from_utf8(
        Command::new(llvm_config_path)
            .arg("--link-static")
            .args(args)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

fn get_libs<A: IntoIterator<Item = S>, S: AsRef<OsStr>>(
    llvm_config_path: &Path,
    args: A,
) -> Vec<String> {
    llvm_config(llvm_config_path, args)
        .split_whitespace()
        .chain(llvm_config(llvm_config_path, Some("--system-libs")).split_whitespace())
        .filter_map(|flag| {
            if flag == "" {
                None
            } else if cfg!(target_env = "msvc") {
                // Same as --libnames, foo.lib
                assert!(flag.ends_with(".lib"));
                Some(&flag[..flag.len() - 4])
            } else {
                // Linker flags style, -lfoo
                assert!(flag.starts_with("-l"));
                Some(&flag[2..])
            }
        })
        .map(Into::into)
        .collect()
}

struct LockedFile(fs::File);

impl Drop for LockedFile {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

impl LockedFile {
    fn new<T: AsRef<Path>>(file_path: T) -> io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;
        file.lock_exclusive()?;
        Ok(LockedFile(file))
    }
}

fn build_llvm() -> PathBuf {
    assert_eq!(
        env::var_os("TARGET"),
        env::var_os("HOST"),
        "cross-compilation is not supported"
    );
    let llvm_dir = env::current_dir()
        .unwrap()
        .join("..")
        .join("external")
        .join("llvm-7")
        .join(env::var_os("TARGET").unwrap());
    fs::create_dir_all(&llvm_dir).unwrap();
    let _locked_file = LockedFile::new(llvm_dir.join(".build-lock")).unwrap();
    download_and_extract_llvm_7_if_needed(&llvm_dir).unwrap();
    make_config(&llvm_dir).build_target("install").build();
    #[cfg(windows)]
    let llvm_config_path = llvm_dir.join("bin").join("llvm-config.exe");
    #[cfg(not(windows))]
    let llvm_config_path = llvm_dir.join("bin").join("llvm-config");
    llvm_config_path
}

fn main() {
    let out_dir = Path::new(&env::var_os("OUT_DIR").unwrap()).to_path_buf();
    let llvm_config_path = build_llvm();
    println!(
        "cargo:rustc-link-search=native={}",
        llvm_config(&llvm_config_path, Some("--libdir"))
    );
    let llvm_libs = get_libs(
        &llvm_config_path,
        &["--libs", "orcjit", "native", "analysis"],
    );
    let header = r#"
#include "llvm-c/Core.h"
#include "llvm-c/OrcBindings.h"
#include "llvm-c/Target.h"
#include "llvm-c/Analysis.h"
#include <stdbool.h>

#ifdef __cplusplus
extern "C"
{
#endif

void LLVM_InitializeNativeTarget(void);
void LLVM_InitializeNativeAsmParser(void);
void LLVM_InitializeNativeAsmPrinter(void);
void LLVM_InitializeNativeDisassembler(void);

#ifdef __cplusplus
}
#endif
"#;
    let header_path = out_dir.join("llvm_bindings.h");
    fs::write(&header_path, header).unwrap();
    let llvm_bindings_source = format!("#include {:?}\n", header_path)
        + r#"
void LLVM_InitializeNativeTarget(void)
{
    LLVM_NATIVE_TARGETINFO();
    LLVM_NATIVE_TARGET();
    LLVM_NATIVE_TARGETMC();
}

void LLVM_InitializeNativeAsmParser(void)
{
    LLVM_NATIVE_ASMPARSER();
}

void LLVM_InitializeNativeAsmPrinter(void)
{
    LLVM_NATIVE_ASMPRINTER();
}

void LLVM_InitializeNativeDisassembler(void)
{
    LLVM_NATIVE_DISASSEMBLER();
}
"#;
    let llvm_bindings_path = out_dir.join("llvm_bindings.c");
    fs::write(&llvm_bindings_path, llvm_bindings_source).unwrap();
    let include_dir: String = llvm_config(&llvm_config_path, Some("--includedir"))
        .trim_right()
        .into();
    let builder = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .clang_arg("-I")
        .clang_arg(&include_dir as &str)
        .rustfmt_bindings(true)
        .whitelist_type("LLVM.*")
        .whitelist_function("LLVM.*")
        .whitelist_var("LLVM.*")
        .blacklist_type("^__.*")
        .prepend_enum_name(false)
        .constified_enum("LLVM.*");
    builder
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("llvm_c.rs"))
        .unwrap();
    let build_llvm_bindings = || {
        let mut retval = cc::Build::new();
        retval
            .cpp(true)
            .file(&llvm_bindings_path)
            .include(&include_dir);
        retval
    };
    build_llvm_bindings()
        .cpp_link_stdlib(None)
        .compile("llvm_bindings");
    for lib in llvm_libs {
        println!("cargo:rustc-link-lib={}", lib);
    }
    // build twice to get the c++ standard library linked after LLVM with llvm_bindings before LLVM
    build_llvm_bindings().compile("llvm_bindings");
}
