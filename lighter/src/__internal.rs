use core::convert::Infallible;
pub use lighter_derive::lighter;

#[repr(transparent)]
pub struct Wrap<T>(pub Option<T>);

impl<T> Wrap<T> {
    #[inline(always)]
    fn inner(&mut self) -> T {
        // SAFETY: we never instantiate a Wrap(None),
        // and the user never should either; this is hidden.
        // self.inner() should only be called *once*.
        unsafe { self.0.take().unwrap_unchecked() }
    }
}

// Wrap(T).bytes() always calls the "best" implementation to convert to Iterator<Item = u8>
// https://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
pub trait MatchIterator<E> {
    type Iter: Iterator<Item = Result<u8, E>>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<T: IntoIterator<Item = u8>> MatchIterator<Infallible> for Wrap<T> {
    type Iter = core::iter::Map<T::IntoIter, fn(u8) -> Result<u8, Infallible>>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().into_iter().map(Result::Ok)
    }
}

impl<E, T: IntoIterator<Item = Result<u8, E>>> MatchIterator<E> for &mut Wrap<T> {
    type Iter = T::IntoIter;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().into_iter()
    }
}

pub trait MatchRefIterator<E> {
    type Iter: Iterator<Item = Result<u8, E>>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<'a, T: IntoIterator<Item = &'a u8>> MatchRefIterator<Infallible> for Wrap<T> {
    type Iter = core::iter::Map<core::iter::Copied<T::IntoIter>, fn(u8) -> Result<u8, Infallible>>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().into_iter().copied().map(Result::Ok)
    }
}

impl<'a, E, T: IntoIterator<Item = Result<&'a u8, E>>> MatchRefIterator<E> for &mut Wrap<T> {
    type Iter = core::iter::Map<T::IntoIter, fn(Result<&'a u8, E>) -> Result<u8, E>>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().into_iter().map(Result::<&u8, E>::copied)
    }
}

pub trait MatchStr<'a> {
    type Iter: Iterator<Item = Result<u8, Infallible>>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<'a> MatchStr<'a> for Wrap<&'a str> {
    type Iter = core::iter::Map<core::str::Bytes<'a>, fn(u8) -> Result<u8, Infallible>>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().bytes().map(Result::Ok)
    }
}

#[cfg(feature = "std")]
impl<'a> MatchStr<'a> for Wrap<String> {
    type Iter = core::iter::Map<std::vec::IntoIter<u8>, fn(u8) -> Result<u8, Infallible>>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        self.inner().into_bytes().into_iter().map(Result::Ok)
    }
}

// Automatically unwrap Result<T, Infallible>, but not any other Result<T, E>
pub trait MaybeUnwrap {
    type Unwrapped;
    fn maybe_unwrap(&mut self) -> Self::Unwrapped;
}

impl<T, E> MaybeUnwrap for Wrap<Result<T, E>> {
    type Unwrapped = Result<T, E>;

    #[inline(always)]
    fn maybe_unwrap(&mut self) -> Self::Unwrapped {
        self.inner()
    }
}

impl<T> MaybeUnwrap for &mut Wrap<Result<T, Infallible>> {
    type Unwrapped = T;

    #[inline(always)]
    fn maybe_unwrap(&mut self) -> Self::Unwrapped {
        self.inner().unwrap()
    }
}
