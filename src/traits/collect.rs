use crate::{Lend, Lender};
/// # Example
/// ```
/// # use lender::prelude::*;
/// struct MyStruct;
/// impl<L: IntoLender> FromLender<L> for MyStruct
/// where
///     L::Lender: for<'all> Lending<'all, Lend = &'all mut [u32]>,
/// {
///     fn from_lender(lender: L) -> Self {
///         lender.into_lender().for_each(|lend| drop(lend));
///         Self
///     }
/// }
/// ```
pub trait FromLender<L: IntoLender>: Sized {
    fn from_lender(lender: L) -> Self;
}

/// Documentation is incomplete. Refer to [`core::iter::IntoIterator`] for more information
pub trait IntoLender {
    type Lender: Lender;
    fn into_lender(self) -> <Self as IntoLender>::Lender;
}
impl<L: Lender> IntoLender for L {
    type Lender = L;
    #[inline]
    fn into_lender(self) -> L {
        self
    }
}

/// Documentation is incomplete. Refer to [`core::iter::Extend`] for more information
pub trait ExtendLender<L: IntoLender> {
    fn extend_lender(&mut self, lender: L);
    /// Extends a collection with exactly one element.
    fn extend_lender_one(&mut self, item: Lend<'_, L::Lender>);
    /// Reserves capacity in a collection for the given number of additional elements.
    ///
    /// The default implementation does nothing.
    fn extend_lender_reserve(&mut self, additional: usize) {
        let _ = additional;
    }
}


/// A trait for an object which can create a leder from a reference to self.
pub trait IterLender {
    type Lender<'lender>: Lender
        where Self: 'lender;
    fn iter_lender(&self) -> Self::Lender<'_>;
}

/// A trait for an object which can create a leder from a mutable reference to self.
pub trait IterMutLender {
    type Lender<'lender>: Lender
        where Self: 'lender;
    fn iter_mut_lender(&mut self) -> Self::Lender<'_>;
}

impl<'iter, 'lend, T> crate::Lending<'lend> for core::slice::Iter<'iter, T> {
    type Lend = &'lend T;
}

impl<'iter, T> crate::Lender for core::slice::Iter<'iter, T> {
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        <Self as Iterator>::next(self)
    }
}

impl<'iter, 'lend, T> crate::Lending<'lend> for core::slice::IterMut<'iter, T> {
    type Lend = &'lend mut T;
}

impl<'iter, T> crate::Lender for core::slice::IterMut<'iter, T> {
    #[inline(always)]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        <Self as Iterator>::next(self)
    }
}

impl<'slice, T> IterLender for &'slice [T] {
    type Lender<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn iter_lender(&self) -> Self::Lender<'_> {
        self.iter()
    }
}

impl<'slice, T> IterLender for &'slice mut [T] {
    type Lender<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn iter_lender(&self) -> Self::Lender<'_> {
        self.iter()
    }
}

impl<'slice, T> IterMutLender for &'slice mut [T] {
    type Lender<'a> = core::slice::IterMut<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn iter_mut_lender(&mut self) -> Self::Lender<'_> {
        self.iter_mut()
    }
}

impl<T> IterLender for alloc::vec::Vec<T> {
    type Lender<'a> = core::slice::Iter<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn iter_lender(&self) -> Self::Lender<'_> {
        self.iter()
    }
}

impl<T> IterMutLender for alloc::vec::Vec<T> {
    type Lender<'a> = core::slice::IterMut<'a, T>
    where
        Self: 'a;

    #[inline(always)]
    fn iter_mut_lender(&mut self) -> Self::Lender<'_> {
        self.iter_mut()
    }
}