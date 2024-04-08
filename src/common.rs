#[derive(PartialEq, Debug)]
pub enum Holder<'a, T> {
    Borrow(&'a T),
    Owned(T),
}

impl <'a, T> AsRef<T> for Holder<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrow(br) => br,
            Self::Owned(own) => own
        }
    }
}

impl <'a,T> From<T> for Holder<'a, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl <'a,T> From<&'a T> for Holder<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrow(value)
    }
}