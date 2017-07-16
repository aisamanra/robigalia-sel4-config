/* Copyright (c) 2017 The Robigalia Project Developers
 * Licensed under the Apache License, Version 2.0
 * <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT
 * license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
 * at your option. All files in the project carrying such
 * notice may not be copied, modified, or distributed except
 * according to those terms.
 */

use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, Write, BufWriter};

/// Parse seL4 config file ../sel4/.config and turn yes/no configs into rust
/// features and configs with values into a pub const in mod sel4_config
/// in a generated .rs file
///
/// To use the generated sel4_config module:
/// include!(concat!(env!("OUT_DIR"), "/sel4_config.rs"));
pub fn process_sel4_config() {
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=SEL4_CONFIG_PATH");

    let out_dir = env::var("OUT_DIR").expect("expected OUT_DIR environment variable");

    // open file to write rustified config values to
    let out_file = File::create(&*format!("{}/sel4_config.rs", out_dir)).unwrap();
    let mut f = BufWriter::new(out_file);

    // start rust module
    writeln!(f, "#[allow(dead_code)]").unwrap();
    writeln!(f, "mod sel4_config {{").unwrap();

    // read seL4 config from ../sel4/.config
    let config = env::var("SEL4_CONFIG_PATH")
        .map(Cow::from)
        .unwrap_or(Cow::from("../sel4/.config"));
    println!("cargo:rerun-if-changed={}", config);

    let file = BufReader::new(File::open(&*config)
        .expect("Could not open ../sel4/.config: Please clone and configure\
                 https://gitlab.com/robigalia/sel4"));

    for line in file.lines().filter_map(|result| result.ok()) {
        let line = line.trim();

        // skip comments and empty lines
        if line.starts_with("#") || !line.contains("=") {
            continue;
        }

        // parse line into key and value
        let mid = line.find("=").expect("expected token '='");
        let key = &line[..mid];
        let value = &line[mid + 1..];

        // validate key
        assert!(key.starts_with("CONFIG_") && key.chars().all(|x| x.is_uppercase() ||
                                                                  x.is_digit(10) ||
                                                                  x == '_'),
                "invalid key: '{}'", key);

        // if value is y or n, set up a feature for key, if y
        // otherwise, set up key with value
        match value {
            "Y" | "y" => set_boolean(key, true),
            "N" | "n" => set_boolean(key, false),
            _ => set_num_or_string(key, value, &mut f),
        }

        // Turn some keyed values into feature flags
        match key {
            "CONFIG_MAX_NUM_NODES" =>
                set_boolean("CONFIGX_ENABLE_SMP_SUPPORT", value != "1"),
            _ => (),
        }
    }

    // close rust module
    writeln!(f, "}}").unwrap();
}

fn set_boolean(key: &str, cond: bool) {
    if cond {
        println!("cargo:rustc-cfg=feature=\"{}\"", key);
    }
}

fn set_num_or_string(key: &str, value: &str, f: &mut BufWriter<File>) {
    match value.parse::<u32>() {
        Ok(_) => writeln!(f, "    pub const {}: usize = {};", key, value).unwrap(),
        _ if value.starts_with('"') && value.ends_with('"') =>
            writeln!(f, "    pub const {}: &'static str = {};", key, value).unwrap(),
        _ => panic!("unknown type: key='{}'; value='{}'", key, value),
    }
}
