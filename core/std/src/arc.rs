
#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! arc {
        ($value:expr) => {
            std::sync::Arc::new($value)
        };
    }
}
