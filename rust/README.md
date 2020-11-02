Basic POC based on [Wendel's rust binding of libxc](https://github.com/Wenzel/xenctrl-sys) and [xenstore](https://github.com/Wenzel/xenstore-sys) to pause and unpause or shutdown a domain.

# Requirements

Install `cargo` & `rustc`

# Usage

In this folder run `cargo run {pause|unpause|shutdown} <integer>` where integer is a valid domain id.