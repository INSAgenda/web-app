use std::{fs::*, io::{Read, Write}};

static DEBUG_BUILD_ALGO: &str = r#"
/// Only the debug server build will accept messages marked with codes generated with this function.
pub fn gen_code(api_key: u64, counter: u64) -> u64 {
    api_key + counter
}
"#;

fn read_production_build_algo() -> Option<String> {
    let mut file = match File::open("src/api/gen_code_prod.rs") {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read src/api/gen_code_prod.rs");

    Some(content)
}

fn main() {
    println!("cargo:rerun-if-changed=src/api/gen_code_prod.rs");

    let algo = read_production_build_algo().unwrap_or_else(|| DEBUG_BUILD_ALGO.to_string());

    let mut output_file = File::create("src/api/gen_code.rs").expect("Failed to open src/api/gen_code.rs");
    output_file.write_all(b"//! Automatically generated!\n//! See build.rs\n\n").expect("Failed to write to src/api/gen_code.rs");
    output_file.write_all(algo.as_bytes()).expect("Failed to write to src/api/gen_code.rs");
}
