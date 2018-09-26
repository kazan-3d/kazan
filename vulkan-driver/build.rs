extern crate bindgen;
extern crate regex;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn detect_vulkan_calling_convention() -> io::Result<String> {
    let code = bindgen::builder()
        .header_contents(
            "vulkan_detect.h",
            r#"#include <vulkan/vk_platform.h>
#ifdef __cplusplus
extern "C" {
#endif

VKAPI_ATTR void VKAPI_CALL detect_fn();

#ifdef __cplusplus
}
#endif
"#,
        ).clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .clang_arg("-I../external/Vulkan-Headers/include")
        .whitelist_function("detect_fn")
        .generate()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "generate() failed"))?
        .to_string();
    if let Some(captures) = regex::Regex::new(r#"extern "([^"]+)""#)
        .unwrap()
        .captures(&code)
    {
        Ok(captures[1].to_owned())
    } else {
        eprintln!("code:\n{}", code);
        Err(io::Error::new(io::ErrorKind::Other, "regex not found"))
    }
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=vulkan-wrapper.h");
    println!("cargo:rerun-if-changed=../external/Vulkan-Headers/include");
    let vulkan_calling_convention = detect_vulkan_calling_convention()?;
    let match_calling_convention_regex = regex::Regex::new(r#"extern "([^"]+)""#).unwrap();
    let builder = bindgen::builder()
        .header("vulkan-wrapper.h")
        .clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .clang_arg("-I../external/Vulkan-Headers/include")
        .prepend_enum_name(false)
        .layout_tests(false)
        .whitelist_var("VK_.*")
        .whitelist_var("ICD_LOADER_MAGIC")
        .whitelist_type("Vk.*")
        .whitelist_type("PFN_.*")
        .blacklist_type("^xcb_.*")
        .derive_debug(false)
        .ignore_functions()
        .constified_enum(".*");
    let mut code = builder
        .generate()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "generate() failed"))?
        .to_string();
    code = match_calling_convention_regex
        .replace_all(&code, |captures: &regex::Captures| {
            if captures[1] == vulkan_calling_convention {
                r#"extern "system""#
            } else {
                let _ = fs::write(
                    PathBuf::from(env::var("OUT_DIR").unwrap()).join("vulkan-types.rs"),
                    &code,
                );
                eprintln!("vulkan_calling_convention: {:?}", vulkan_calling_convention);
                panic!("unhandled non-vulkan calling convention");
            }
        }).into_owned();
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("vulkan-types.rs"),
        code,
    )?;
    Ok(())
}
