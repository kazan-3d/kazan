extern crate bindgen;
extern crate regex;
extern crate xmltree;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
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
        ).clang_arg("-target")
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

struct GeneratedCode(String);

impl GeneratedCode {
    fn new() -> Self {
        GeneratedCode(String::new())
    }
}

impl fmt::Display for GeneratedCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const END_OF_LINE: &'static str = "\n";
        let mut s: &str = &self.0;
        if f.alternate() && s.ends_with(END_OF_LINE) {
            s = &s[..s.len() - END_OF_LINE.len()];
        }
        f.write_str(s)
    }
}

impl io::Write for GeneratedCode {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0 += str::from_utf8(buf).unwrap();
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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
        builder = builder
            .blacklist_type(name)
            .blacklist_type(format!("{}_T", name));
    }
    builder = builder
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
    let mut extensions_enum = GeneratedCode::new();
    let mut extensions_struct = GeneratedCode::new();
    let mut extensions_default_init = GeneratedCode::new();
    let mut extensions_get_dependencies = GeneratedCode::new();
    for extension in parsed_xml.get_child("extensions").unwrap().children.iter() {
        match &**extension.attributes.get("supported").unwrap() {
            "vulkan" => {}
            "disabled" => continue,
            supported => panic!("unknown supported field: {:?}", supported),
        }
        let name = extension.attributes.get("name").unwrap();
        let mut requires = extension
            .attributes
            .get("requires")
            .map(Deref::deref)
            .unwrap_or("")
            .split(',')
            .filter(|v| v != &"")
            .peekable();
        if requires.peek().is_some() {
            writeln!(extensions_get_dependencies, "            {} => {{", name);
            for require in requires {
                writeln!(
                    extensions_get_dependencies,
                    "                retval.{} = true;",
                    require
                );
            }
            writeln!(extensions_get_dependencies, "            }}");
        } else {
            writeln!(extensions_get_dependencies, "            {} => {{}}", name);
        }
        writeln!(extensions_enum, "    {},", name)?;
        writeln!(extensions_struct, "    {}: bool,", name)?;
        writeln!(extensions_default_init, "            {}: false,", name)?;
    }
    let mut code = io::Cursor::new(Vec::new());
    writeln!(
        code,
        r"/* automatically generated code */

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Extension {{
{extensions_enum:#}
}}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Extensions {{
{extensions_struct:#}
}}

impl Default for Extensions {{
    fn default() -> Self {{
        Self {{
{extensions_default_init:#}
        }}
    }}
}}

impl Extension {{
    pub fn get_dependencies(self) -> Extensions {{
        let mut retval = Extensions::default();
        match self {{
{extensions_get_dependencies:#}
        }}
        retval
    }}
}}",
        extensions_enum = extensions_enum,
        extensions_struct = extensions_struct,
        extensions_default_init = extensions_default_init,
        extensions_get_dependencies = extensions_get_dependencies
    )?;
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("vulkan-properties.rs"),
        code.into_inner(),
    )?;
    Ok(())
}
