# sel4-config

[![Crates.io](https://img.shields.io/crates/v/sel4-config.svg?style=flat-square)](https://crates.io/crates/sel4-config)

[Documentation](https://doc.robigalia.org/sel4-config)

Parses an seL4 .config file and generates cargo build script output to stdout
and generated rust code at ```env!("OUT_DIR")/sel4_config.rs```.

For y/n seL4 config entries, sets a rust feature flag for configs with a
value of y.

For seL4 config entries with values, detects whether they are a string or
integer, and writes a corresponding pub const in mod sel4_config in the
generated file.

Additionally, derives feature flags depending on the value of some seL4
config entries. This allows conditional compilation for these secenarios:

    CONFIG_MULTI_CPU feature set if CONFIG_MAX_NUM_NODES != 1

# How to use

Add a build dependency and build script to your Cargo.toml file:

```
[package]
build = "build.rs"

[build-dependencies]
sel4-config = { version = "0.0.1", path = "../sel4-config" }

```

Create a build script that calls sel4_config::process_sel4_config():

```
extern crate sel4_config;

use sel4_config::*;

fn main() {
    process_sel4_config();
}
```

Import the generated sel4_config module in your lib.rs or main.rs:

```
include!(concat!(env!("OUT_DIR"), "/sel4_config.rs"));
```

You can now access seL4 configuration flags using features:

```
#[cfg(feature = "CONFIG_ENABLE_BENCHMARKS")]
fn do_benchmarking() {
}
```

You can also access seL4 configuration values using module sel4_config:

```
println!("We support {} CPUs", sel4_config::CONFIG_MAX_NUM_NODES);
```

# Status

Functional, still needs polish and testing.
