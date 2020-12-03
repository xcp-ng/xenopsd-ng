// See: https://rust-lang.github.io/rust-bindgen/tutorial-3.html
use std::{env, path::PathBuf};

const WRAPPER: &str = "lib/wrapper.h";

fn main () {
  println!("cargo:rerun-if-changed={}", WRAPPER);
  println!("cargo:rustc-link-lib={}={}", "dylib", "xenctrl");
  println!("cargo:rustc-link-lib={}={}", "dylib", "xenstore");

  let bindings = bindgen::Builder::default()
    .header(WRAPPER)
    .default_enum_style(bindgen::EnumVariation::Rust {
      non_exhaustive: false,
    })
    .layout_tests(false)
    .rustfmt_bindings(true)
    .derive_default(true)
    .generate()
    .expect("Unable to generate bindings.");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("Failed to get OUT_DIR env var."));
    bindings
      .write_to_file(out_path.join("bindings.rs"))
      .expect("Couldn't write bindings!");
}
