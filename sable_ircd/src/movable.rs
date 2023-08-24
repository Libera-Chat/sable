/// A utility type to allow a struct implementing Drop to contain a field which
/// will, in normal use, always contain a value, but which can be moved out by
/// a method that consumes the type.
///
/// Implements [`Deref`](std::ops::Deref) for transparent access to the underlying
/// value; this operation will panic if the `Movable` has been emptied.
pub enum Movable<T> {
    Empty,
    Full(T),
}

impl<T> Movable<T> {
    pub fn new(obj: T) -> Self {
        Self::Full(obj)
    }

    pub fn take(&mut self) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::Full(_) => match std::mem::replace(self, Self::Empty) {
                Self::Empty => None,
                Self::Full(value) => Some(value),
            },
        }
    }

    pub fn unwrap(&mut self) -> T {
        self.take().unwrap()
    }
}

impl<T> std::ops::Deref for Movable<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Self::Empty => panic!("Attempted to deref an empty Movable"),
            Self::Full(value) => value,
        }
    }
}

impl<T> std::ops::DerefMut for Movable<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self {
            Self::Empty => panic!("Attempted to deref an empty Movable"),
            Self::Full(value) => value,
        }
    }
}
