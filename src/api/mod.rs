mod load_events;
pub use load_events::*;
mod error;
pub use error::*;

fn gen_code(api_key: u64, counter: u64) -> u64 {
    let mut key = (api_key + 143 * counter) as u128;
    for _ in 0..11 {
        key = key * key + 453;
        if key <= 0xffff_ffff {
            key += 0x4242424242424242424242424242;
        }
        key &= 0x0000_0000_ffff_ffff_ffff_ffff_0000_0000;
        key >>= 32;
    }
    key as u64
}

fn save_counter(counter: u64) {
    let window = web_sys::window().unwrap();
    let local_storage = window.local_storage().unwrap().unwrap();
    local_storage.set("counter", &counter.to_string()).unwrap();
}
