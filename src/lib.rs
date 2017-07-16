/* Copyright (c) 2017 The Robigalia Project Developers
 * Licensed under the Apache License, Version 2.0
 * <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT
 * license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
 * at your option. All files in the project carrying such
 * notice may not be copied, modified, or distributed except
 * according to those terms.
 */

use std::fs::File;
use std::io::{BufReader, BufRead, Write, BufWriter};
use std::env;

/// Parse seL4 config file ../sel4/.config and turn yes/no configs into rust
/// features and configs with values into a pub const in mod sel4_config
/// in a generated .rs file 
///
/// To use the generated sel4_config module:
/// include!(concat!(env!("OUT_DIR"), "/sel4_config.rs"));
pub fn process_sel4_config() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // open file to write rustified config values to
    let out_file = File::create(&*format!("{}/sel4_config.rs", out_dir)).unwrap();
    let mut f = BufWriter::new(out_file);

    // start rust module
    let _ = writeln!(f, "#[allow(dead_code)]");
    let _ = writeln!(f, "mod sel4_config {{");

    // read seL4 config from ../sel4/.config
    let file = BufReader::new(File::open("../sel4/.config")
        .expect("Could not open ../sel4/.config: \
                Please clone and configure https://gitlab.com/robigalia/sel4"));

    for line in file.lines().filter_map(|result| result.ok()) {
        let line = line.trim();

        // skip comments and empty lines
        if line.starts_with("#") || !line.contains("=") {
            continue;
        }

        // parse line into name and value
        let parsed: Vec<&str> = line.split("=").collect();
        assert_eq!(parsed.len(), 2);
        let name = parsed[0];
        let value = parsed[1];

        // if value is y or n, set up a feature for name, if y
        // otherwise, set up name with value
        match value {
            "Y" | "y" => set_feature_flag(name, &mut f),
            "N" | "n" => (),
            _ => set_named_value(name, value, &mut f),
        }

        // Turn some named values into feature flags
        match name {
            "CONFIG_MAX_NUM_NODES" => if value != "1" {
                set_feature_flag("CONFIG_MULTI_CPU", &mut f);
            },
            _ => (),
        }
    }

    // close rust module
    let _ = writeln!(f, "}}");
}

fn set_feature_flag(name: &str, _: &mut BufWriter<File>) {
    println!("cargo:rustc-cfg=feature=\"{}\"", name);
}

fn set_named_value(name: &str, value: &str, f: &mut BufWriter<File>) {
    let _ = match value.parse::<u32>() {
        Ok(_) => writeln!(f, "    pub const {}: usize = {};", name, value),
        _ => writeln!(f, "    pub const {}: &'static str = {};", name, value),
    };
}
