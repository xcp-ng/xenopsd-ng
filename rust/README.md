Basic POC based on [Wendel's rust binding of libxc](https://github.com/Wenzel/xenctrl-sys) and [xenstore](https://github.com/Wenzel/xenstore-sys) to pause and unpause or shutdown a domain.

# Requirements

Install `cargo`, `rustc` & `xen-devel`

## On XCP-ng

To install what's needed, run following commands:
```
yum install llvm-devel clang xen-devel --enablerepo="*"

curl https://sh.rustup.rs -sSf | sh

source $HOME/.cargo/env
```
