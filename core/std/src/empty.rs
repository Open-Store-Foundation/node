
pub trait Empty<T> {
    fn or_empty(self) -> T;
}

impl Empty<String> for Option<String> {
    fn or_empty(self) -> String {
        return self.unwrap_or("".into());
    }
}

impl <T> Empty<Vec<T>> for Option<Vec<T>> {
    fn or_empty(self) -> Vec<T> {
        return self.unwrap_or(vec![])
    }
}
