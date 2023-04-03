use core::{num::NonZeroUsize, ops::ControlFlow};

use crate::{
    try_trait_v2::{FromResidual, Try},
    DoubleEndedLender, Fuse, FusedLender, Lender, Lending,
};

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Chain<A, B> {
    a: Fuse<A>,
    b: Fuse<B>,
}
impl<A, B> Chain<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self { Self { a: Fuse::new(a), b: Fuse::new(b) } }
}
impl<'lend, A, B> Lending<'lend> for Chain<A, B>
where
    A: Lender,
    B: Lender + for<'all> Lending<'all, Lend = <A as Lending<'all>>::Lend>,
{
    type Lend = <A as Lending<'lend>>::Lend;
}
impl<A, B> Lender for Chain<A, B>
where
    A: Lender,
    B: Lender + for<'all> Lending<'all, Lend = <A as Lending<'all>>::Lend>,
{
    fn next(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.a.next().or_else(|| self.b.next()) }
    #[inline]
    fn count(self) -> usize { self.a.count() + self.b.count() }
    fn try_fold<Acc, F, R>(&mut self, mut acc: Acc, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = Acc>,
    {
        acc = match self.a.try_fold(acc, &mut f).branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
        };
        acc = match self.b.try_fold(acc, f).branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
        };
        Try::from_output(acc)
    }
    fn fold<Acc, F>(self, mut acc: Acc, mut f: F) -> Acc
    where
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> Acc,
    {
        acc = self.a.fold(acc, &mut f);
        acc = self.b.fold(acc, f);
        acc
    }
    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        match self.a.advance_by(n) {
            Ok(()) => Ok(()),
            Err(k) => self.b.advance_by(k.get()),
        }
    }
    #[inline]
    fn nth(&mut self, mut n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        n = match self.a.advance_by(n) {
            Ok(()) => match self.a.next() {
                None => 0,
                x => return x,
            },
            Err(k) => k.get(),
        };
        self.b.nth(n)
    }
    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<<Self as Lending<'_>>::Lend>
    where
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        self.a.find(&mut predicate).or_else(|| self.b.find(predicate))
    }
    #[inline]
    fn last<'call>(self) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized + 'call,
    {
        let a_last = self.a.last();
        let b_last = self.b.last();
        b_last.or(a_last)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();
        let lower = a_lower.saturating_add(b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => x.checked_add(y),
            _ => None,
        };
        (lower, upper)
    }
}
impl<A, B> DoubleEndedLender for Chain<A, B>
where
    A: DoubleEndedLender,
    B: DoubleEndedLender + for<'lend> Lending<'lend, Lend = <A as Lending<'lend>>::Lend>,
{
    #[inline]
    fn next_back(&mut self) -> Option<<Self as Lending<'_>>::Lend> { self.b.next_back().or_else(|| self.a.next_back()) }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZeroUsize> {
        match self.b.advance_back_by(n) {
            Ok(()) => Ok(()),
            Err(k) => self.a.advance_back_by(k.get()),
        }
    }
    #[inline]
    fn nth_back(&mut self, mut n: usize) -> Option<<Self as Lending<'_>>::Lend> {
        n = match self.b.advance_back_by(n) {
            Ok(()) => match self.b.next_back() {
                None => 0,
                x => return x,
            },
            Err(k) => k.get(),
        };
        self.a.nth_back(n)
    }

    #[inline]
    fn rfind<'call, P>(&'call mut self, mut predicate: P) -> Option<<Self as Lending<'call>>::Lend>
    where
        Self: Sized,
        P: FnMut(&<Self as Lending<'_>>::Lend) -> bool,
    {
        self.b.rfind(&mut predicate).or_else(|| self.a.rfind(predicate))
    }

    fn try_rfold<Acc, F, R>(&mut self, init: Acc, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> R,
        R: Try<Output = Acc>,
    {
        let mut acc = match self.b.try_rfold(init, &mut f).branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
        };
        acc = match self.a.try_rfold(acc, f).branch() {
            ControlFlow::Continue(x) => x,
            ControlFlow::Break(x) => return FromResidual::from_residual(x),
        };
        Try::from_output(acc)
    }

    fn rfold<Acc, F>(self, init: Acc, mut f: F) -> Acc
    where
        Self: Sized,
        F: FnMut(Acc, <Self as Lending<'_>>::Lend) -> Acc,
    {
        let mut acc = self.b.rfold(init, &mut f);
        acc = self.a.rfold(acc, f);
        acc
    }
}
impl<A, B> FusedLender for Chain<A, B>
where
    A: FusedLender,
    B: FusedLender + for<'lend> Lending<'lend, Lend = <A as Lending<'lend>>::Lend>,
{
}
impl<A: Default, B: Default> Default for Chain<A, B> {
    fn default() -> Self { Chain::new(Default::default(), Default::default()) }
}
