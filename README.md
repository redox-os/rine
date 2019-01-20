# rine

Rine is not an emulator. It is a system call translator for Redox binaries
that runs on Linux. But hey, if it makes you feel better:
"(R)edox (i)n a(n) (e)mulator" is fine too.

## Trying it out

First, set up a Redox environment by running this inside of a
[redox](https://gitlab.redox-os.org/redox-os/redox) repository:
```
make env
```

Next, build the example that uses Redox system calls:
```
xargo build --target x86_64-unknown-redox --example simple
```

Finally, run the example with `rine`:
```
cargo run target/x86_64-unknown-redox/debug/examples/simple
```

The `example.sh` script will do all of this for you.
