// See: https://rust-lang.github.io/rust-bindgen/tutorial-3.html
use std::{env, path::PathBuf};
use std::fs;
use std::io::Write;
use std::process::Command;

// Because bindgen cannot create const rust variables correctly using
// HVM_SAVE_LENGTH/HVM_SAVE_CODE macros, we must use this binary to
// generate HVM_SAVE_LENGTH_<X> HVM_SAVE_CODE_<X> defines instead.
// If in the future bindgen supports macros and constant expressions correctly,
// we could get rid of this binary.
const GEN_HVM_SAVE_VARIABLES_BIN: &str = "gen-hvm-save-variables";

const WRAPPER_DIR: &str = "wrapper";
const WRAPPER_C_HEADER: &str = "wrapper.h";

// ================================================================================================

fn compile_c_gen (c_gen: &str, out_dir: &std::path::PathBuf) -> String {
  let c_gen_path = out_dir.join(c_gen).to_str().expect("Failed to get gen path.").to_string();
  let status = Command::new("gcc")
    .args(&[
      WRAPPER_DIR.to_string() + "/" + c_gen + ".c",
      "-o".to_string(),
      c_gen_path.clone()
    ])
    .status()
    .expect("Unable to execute gcc.");

  if !status.success() {
    eprintln!("Failed to compile `{}`.", c_gen);
    std::process::exit(1);
  }

  c_gen_path
}

fn apply_c_gen (c_gen_path: &str, bindings_file: &mut fs::File) {
  let output = Command::new(c_gen_path)
    .output()
    .expect("Failed to execute gen binary.");

  if !output.status.success() {
    eprintln!("Failed to execute correctly `{}`.", c_gen_path);
    std::process::exit(1);
  }

  bindings_file.write_all(&output.stdout).expect("Failed to write in bindings file.");
}

fn main () {
  let wrapper_file = WRAPPER_DIR.to_string() + "/" + WRAPPER_C_HEADER;

  println!("cargo:rerun-if-changed={}", &wrapper_file);
  println!("cargo:rerun-if-changed={}/{}.c", "wrapper", GEN_HVM_SAVE_VARIABLES_BIN);
  println!("cargo:rustc-link-lib={}={}", "dylib", "xenctrl");
  println!("cargo:rustc-link-lib={}={}", "dylib", "xenforeignmemory");
  println!("cargo:rustc-link-lib={}={}", "dylib", "xenstore");

  let bindings = bindgen::Builder::default()
    .header(&wrapper_file)
    .default_enum_style(bindgen::EnumVariation::Rust {
      non_exhaustive: false,
    })
    .layout_tests(false)
    .rustfmt_bindings(true)
    .derive_default(true)
    .generate()
    .expect("Unable to generate bindings.");

  // Write the bindings to the $OUT_DIR/<WRAPPER_DIR>/bindings.rs file.
  let out_dir = PathBuf::from(
    env::var("OUT_DIR").expect("Failed to get OUT_DIR env var.")
  ).join(WRAPPER_DIR);
  fs::create_dir_all(&out_dir).expect("Failed to create wrapper output path.");

  let bindings_path = out_dir.join("bindings.rs");
  bindings
    .write_to_file(bindings_path.clone())
    .expect("Couldn't write bindings!");

  let mut bindings_file = fs::OpenOptions::new()
    .append(true)
    .open(bindings_path)
    .expect("Failed to open bindings file.");

  apply_c_gen(&compile_c_gen(GEN_HVM_SAVE_VARIABLES_BIN, &out_dir), &mut bindings_file);
}
