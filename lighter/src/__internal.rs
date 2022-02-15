pub use lighter_derive::lighter;

pub struct Wrap<T>(pub Option<T>);

pub trait MatchIterator {
    type Iter: Iterator<Item = u8>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<T: IntoIterator<Item = u8>> MatchIterator for Wrap<T> {
    type Iter = T::IntoIter;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        // SAFETY: we never instantiate a Wrap(None),
        // and the user never should either; this is hidden
        unsafe { self.0.take().unwrap_unchecked() }.into_iter()
    }
}

pub trait MatchRefIterator {
    type Iter: Iterator<Item = u8>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<'a, T: IntoIterator<Item = &'a u8>> MatchRefIterator for Wrap<T> {
    type Iter = core::iter::Copied<T::IntoIter>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        unsafe { self.0.take().unwrap_unchecked() }
            .into_iter()
            .copied()
    }
}

pub trait MatchStr<'a> {
    type Iter: Iterator<Item = u8>;
    fn bytes(&mut self) -> Self::Iter;
}

impl<'a> MatchStr<'a> for Wrap<&'a str> {
    type Iter = core::str::Bytes<'a>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        unsafe { self.0.as_ref().unwrap_unchecked() }.bytes()
    }
}

#[cfg(feature = "std")]
impl<'a> MatchStr<'a> for Wrap<String> {
    type Iter = std::vec::IntoIter<u8>;
    #[inline]
    fn bytes(&mut self) -> Self::Iter {
        unsafe { self.0.take().unwrap_unchecked() }
            .into_bytes()
            .into_iter()
    }
}
