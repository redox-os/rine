# rine

Rine is not an emulator. It is a system call translator for Redox binaries
that runs on Linux. But hey, if it makes you feel better:
"(R)edox (i)n a(n) (e)mulator" is fine too.

## Trying it out

First, build the example that uses Redox system calls:
```
cargo build --example simple
```

Next, run the example with `rine`:
```
cargo run target/debug/examples/simple
```
