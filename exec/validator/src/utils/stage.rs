
pub enum Stage<T> {
    Retry(Option<T>),
    Checking(Option<T>),
    Value(T)
}

impl <T> Stage<T> {
    pub fn retry(value: T)-> Stage<T> {
        return Self::Retry(Some(value));
    }

    pub fn retry_none() -> Stage<T> {
        return Self::Retry(None);
    }

    pub fn check(value: T)-> Stage<T> {
        return Self::Checking(Some(value));
    }

    pub fn check_none() -> Stage<T> {
        return Self::Checking(None);
    }
}
