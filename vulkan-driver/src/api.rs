#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#[cfg(unix)]
use xcb::ffi::{xcb_connection_t, xcb_visualid_t, xcb_window_t};
include!(concat!(env!("OUT_DIR"), "/vulkan-types.rs"));
