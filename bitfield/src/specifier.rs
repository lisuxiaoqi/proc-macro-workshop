pub trait Specifier {
    const BITS: usize;
    type Base;
    type Face;
}

pub trait FromTrans<T> {
    fn fromt(t: T) -> Self;
}

pub trait IntoTrans<T> {
    fn intot(self) -> T;
}

impl<T, U> IntoTrans<T> for U
where
    T: FromTrans<U>,
{
    fn intot(self) -> T {
        T::fromt(self)
    }
}

impl<T> FromTrans<T> for T {
    fn fromt(v: T) -> Self {
        v
    }
}
