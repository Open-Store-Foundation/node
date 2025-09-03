use std::env;

pub fn is_debug() -> bool {
    if env::var_os("IS_DEBUG").is_some() {
        return true;
    }

    if cfg!(debug_assertions) {
        return true;
    } else {
        return false;
    }
}
