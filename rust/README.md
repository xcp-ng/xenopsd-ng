# Xenops in Rust

Basic PoC based on [Wendel's rust binding of libxc](https://github.com/Wenzel/xenctrl-sys) and [xenstore](https://github.com/Wenzel/xenstore-sys) to pause, unpause, shutdown a domain or list them.

## Components

There's 2 ways to use it: the daemon/server way, or via a CLI.

### xenopsd-ng daemon

It's basically having `xenopsd-ng` running and listening on port `3030` for HTTP and JSON-RPC communications. You can then do your request for any host remotely with JSON-RPC calls, eg with `curl`.

See the dedicated README for more details.

### CLI

Use calls directly without having any daemon running.

See the dedicated README for more details.

## Requirements

Install `cargo`, `rustc` & `xen-devel`.

### On XCP-ng

To install what's needed, run following commands:
```
yum install llvm-devel clang gcc xen-devel --enablerepo="*"

curl https://sh.rustup.rs -sSf | sh

source $HOME/.cargo/env
```
