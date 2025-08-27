//! Automatically generated!
//! See build.rs


/// Only the debug server build will accept messages marked with codes generated with this function.
pub fn gen_code(api_key: u64, counter: u64) -> u64 {
    api_key.wrapping_add(counter)
}
