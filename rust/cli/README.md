CLI for xenops-ng.

# Compile & run the binary

From the `rust` folder:
```
cargo build -p xenops
```

# Usage

The binary is located in: `xenops-ng/rust/target/debug`.
- `xenops-cli {{pause|unpause|shutdown}} <integer>`: pause/unpause/shutdown a domain, the integer arg must be a valid domain id.
- `xenops-cli domain-list`: list all domains ids.
