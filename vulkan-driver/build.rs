extern crate bindgen;
extern crate regex;
extern crate xmltree;
use std::env;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str;
use xmltree::Element;

const VULKAN_HEADERS_INCLUDE_PATH: &'static str = "../external/Vulkan-Headers/include";

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
        )
        .clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .clang_arg(format!("-I{}", VULKAN_HEADERS_INCLUDE_PATH))
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
    let vulkan_wrapper_header_path = "vulkan-wrapper.h";
    let vulkan_vk_xml_path = "../external/Vulkan-Headers/registry/vk.xml";
    println!("cargo:rerun-if-changed={}", vulkan_wrapper_header_path);
    println!("cargo:rerun-if-changed={}", VULKAN_HEADERS_INCLUDE_PATH);
    println!("cargo:rerun-if-changed={}", vulkan_vk_xml_path);
    let parsed_xml = Element::parse(fs::File::open(&PathBuf::from(vulkan_vk_xml_path))?)
        .map_err(|v| io::Error::new(io::ErrorKind::Other, format!("{}", v)))?;
    let types = parsed_xml.get_child("types").unwrap();
    let header_version: u32 = types
        .children
        .iter()
        .filter_map(|v| {
            if v.get_child("name")
                .and_then(|v| v.text.as_ref().map(Deref::deref))
                == Some("VK_HEADER_VERSION")
            {
                Some(v.text.as_ref().unwrap())
            } else {
                None
            }
        })
        .next()
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let vulkan_calling_convention = detect_vulkan_calling_convention()?;
    let match_calling_convention_regex = regex::Regex::new(r#"extern "([^"]+)""#).unwrap();
    let mut builder = bindgen::builder()
        .header(vulkan_wrapper_header_path)
        .clang_arg("-target")
        .clang_arg(env::var("TARGET").unwrap())
        .clang_arg(format!("-I{}", VULKAN_HEADERS_INCLUDE_PATH))
        .prepend_enum_name(false)
        .layout_tests(false)
        .whitelist_var("VK_.*")
        .whitelist_var("ICD_LOADER_MAGIC");
    for t in types
        .children
        .iter()
        .filter(|v| v.attributes.get("category").map(|v| &**v) == Some("handle"))
    {
        let name = if let Some(name) = t.attributes.get("name") {
            name
        } else {
            t.get_child("name").unwrap().text.as_ref().unwrap()
        };
        if name.ends_with("NVX") {
            continue;
        }
        builder = builder
            .blacklist_type(format!("^{}$", name))
            .blacklist_type(format!("^{}_T$", name));
    }
    builder = builder
        .whitelist_type("PFN_.*")
        .whitelist_type("^Vk.*")
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
        })
        .into_owned();
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("vulkan-types.rs"),
        code,
    )?;
    let driver_name_prefix = if cfg!(unix) {
        "lib"
    } else if cfg!(target_os = "windows") {
        ""
    } else {
        unimplemented!()
    };
    let driver_name_suffix = if cfg!(any(target_os = "linux", target_os = "android")) {
        ".so"
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        ".dylib"
    } else if cfg!(target_os = "windows") {
        ".dll"
    } else {
        unimplemented!()
    };
    let driver_name = format!("{}kazan_driver{}", driver_name_prefix, driver_name_suffix);
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("kazan_driver.json"),
        format!(
            r#"{{
    "file_format_version": "1.0.0",
    "ICD": {{
        "library_path": "{}",
        "api_version": "1.1.{}"
    }}
}}"#,
            PathBuf::from(env::var("OUT_DIR").unwrap())
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .unwrap_or_else(|| Path::new(""))
                .join(driver_name)
                .to_str()
                .unwrap(),
            header_version
        ),
    )?;
    Ok(())
}
