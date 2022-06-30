# dump-memory

Small memory dumper in Rust.

## TL; DR. How to use ?

```bash
cargo build --release
${CARGO_TARGET_DIR:-target}/release/dump-memory $PID
${CARGO_TARGET_DIR:-target}/release/dump-memory $PID /some/path/to/output_dir
```

If `$PID` is not specified, it defaults to:
```bash
$(basename $(perl -pe 's/\0.*$//' /proc/$PID/cmdline))-$PID
```

## Note on "cross" building
If you want to use it on an old Linux, you might encounter a error like
`GLIBC 2.XX not found`. Then just build with musl

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target=x86_64-unknown-linux-musl
${CARGO_TARGET_DIR:-target}/x86_64-unknown-linux-musl/release/dump-memory $PID
```
