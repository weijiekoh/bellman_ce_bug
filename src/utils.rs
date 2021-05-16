pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

use num_bigint::BigUint;
use num_traits::Num;
use std::fmt::Display;

pub fn repr_to_big<T: Display>(r: T) -> String {
    BigUint::from_str_radix(&format!("{}", r)[2..], 16).unwrap().to_str_radix(10)
}
