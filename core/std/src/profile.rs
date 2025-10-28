use std::env;

pub fn is_debug() -> bool {
    if let Some(value) = env::var_os("IS_DEBUG") {
        return value == "true";
    }

    if cfg!(debug_assertions) {
        return true;
    } else {
        return false;
    }
}
