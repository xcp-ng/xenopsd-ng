#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

mod bindings {
  include!(concat!(env!("OUT_DIR"), "/wrapper/bindings.rs"));
}

pub use bindings::*;
