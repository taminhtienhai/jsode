use std::hash::Hasher;

// https://github.com/rust-lang/rust/issues/104061
// `std::hash::DefaultHasher::new` is not yet stable as a const fn
// pub static HASHER: std::hash::DefaultHasher = std::hash::DefaultHasher::new();

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

pub fn hash_str(input: &str) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(input.as_bytes());
    hasher.finish()
}