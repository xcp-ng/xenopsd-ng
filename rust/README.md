Basic POC based on [Wendel's rust binding of libxc](https://github.com/Wenzel/xenctrl-sys) to pause and unpause a domain.

# Requirements

Install `cargo` & `rustc`

# Usage

In this folder run `cargo run {pause|unpause} <integer>` where integer is a valid domid.