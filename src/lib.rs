//! Algorithms and utilities to work with bifiltered graphs. In particular,
//! algorithms to remove edges from a bifiltered graph while maintaining the topological
//! properties of its clique complex, see [crate::removal].

#![warn(clippy::shadow_unrelated)]
#![warn(clippy::needless_pass_by_value)]
#![allow(clippy::needless_range_loop)]

use num::{Bounded, Zero};
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

pub mod edges;

pub mod distance_matrix;
pub mod mpfree;
pub mod points;
pub mod removal;

mod chain_complex;
mod filtration;
mod io_utils;
mod simplicial_complex;

/// A generic value, like usize or i32, that we can use as grades in a bifiltered graph.
pub trait Value:
    Zero
    + Ord
    + Bounded
    + Copy
    + Clone
    + Hash
    + std::fmt::Debug
    + std::fmt::Display
    + std::marker::Send
    + std::marker::Sync
{
}

impl<T> Value for T where
    T: Zero
        + Ord
        + Bounded
        + Copy
        + Clone
        + Hash
        + std::fmt::Debug
        + std::fmt::Display
        + std::marker::Send
        + std::marker::Sync
{
}

/// The grade in which a simplex enters a filtration.
pub trait CriticalGrade:
    Clone + PartialOrd + Ord + std::fmt::Debug + std::marker::Sync + std::marker::Send
{
    fn min_value() -> Self;
    fn max_value() -> Self;
    fn zero() -> Self;

    #[must_use]
    fn join(&self, other: &Self) -> Self;

    fn lte(&self, other: &Self) -> bool;
    fn gte(&self, other: &Self) -> bool;

    /// Returns true if self is incomparable to other, meaning that neither self <= other and neither
    /// self >= other.
    fn is_incomparable_to(&self, other: &Self) -> bool {
        !self.lte(other) && !self.gte(other)
    }

    fn parameters() -> usize;
}

/// A 1-critical grade. The default order is lexicographic.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OneCriticalGrade<VF, const N: usize>(pub [VF; N]);

impl<VF: Value, const N: usize> OneCriticalGrade<VF, N> {
    pub fn iter(&self) -> Iter<'_, VF> {
        self.0.iter()
    }
}

impl<VF: Value, const N: usize> OneCriticalGrade<VF, N> {
    pub fn cmp_colexicographically(&self, b: &Self) -> Ordering {
        for i in (0..N).rev() {
            match self.0[i].cmp(&b.0[i]) {
                Ordering::Less => {
                    return Ordering::Less;
                }
                Ordering::Equal => {
                    continue;
                }
                Ordering::Greater => {
                    return Ordering::Greater;
                }
            }
        }
        Ordering::Equal
    }
}

impl<VF: Value, const N: usize> CriticalGrade for OneCriticalGrade<VF, N> {
    fn min_value() -> OneCriticalGrade<VF, N> {
        OneCriticalGrade([VF::min_value(); N])
    }

    fn max_value() -> OneCriticalGrade<VF, N> {
        OneCriticalGrade([VF::max_value(); N])
    }

    fn zero() -> OneCriticalGrade<VF, N> {
        OneCriticalGrade([VF::zero(); N])
    }

    fn join(&self, other: &Self) -> Self {
        let mut join = *self;
        for n in 0..N {
            join[n] = std::cmp::max(join[n], other[n]);
        }
        join
    }

    /// Returns true if, for all i in 0..(N-1), the i'th value of self is less than or equal to
    /// the i'th value of the other.
    fn lte(&self, other: &Self) -> bool {
        for n in 0..N {
            if self[n] > other[n] {
                return false;
            }
        }
        true
    }

    fn gte(&self, other: &Self) -> bool {
        for n in 0..N {
            if self[n] < other[n] {
                return false;
            }
        }
        true
    }

    fn parameters() -> usize {
        N
    }
}

impl<VF: Value, const N: usize> Index<usize> for OneCriticalGrade<VF, N> {
    type Output = VF;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<VF: Value, const N: usize> IndexMut<usize> for OneCriticalGrade<VF, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<VF: Value, const N: usize> From<[VF; N]> for OneCriticalGrade<VF, N> {
    fn from(grade: [VF; N]) -> Self {
        Self(grade)
    }
}

impl<VF: Value> From<VF> for OneCriticalGrade<VF, 1> {
    fn from(grade: VF) -> Self {
        Self([grade])
    }
}

impl<VF: Value, const N: usize> std::fmt::Display for OneCriticalGrade<VF, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            write!(f, "{}", self.0[i])?;
            if i != N - 1 {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}
