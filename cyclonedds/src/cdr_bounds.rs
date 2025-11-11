//! Traits and types for describing the serialized CDR size bounds of keys.
//!
//! DDS requires a maximum serialized size for keyed topic types when
//! computing keyhashes. Implement [`CdrBounds`] on your key type to provide
//! this information.
//!
//! # Examples
//!
//! ## Implementing bounds for bounded data
//!
//! ```
//! use cyclonedds::cdr_bounds::{CdrBounds, CdrSize};
//!
//! struct BoundedData {
//!     w: f64,
//!     x: i32,
//!     y: u16,
//!     z: bool,
//! }
//!
//! impl CdrBounds for BoundedData {
//!     fn max_serialized_cdr_size() -> CdrSize {
//!         // This corresponds to (f64 max serialized CDR size padded to the i32 alignment
//!         //                        + i32 max serialized CDR size padded to the u16 alignment
//!         //                          + u16 max serialized CDR size padded to the bool alignment
//!         //                            + bool max serialized CDR size)
//!         //                              + bool::max_serialized_cdr_size())
//!         //                        all padded to the alignment of f64
//!         <(f64, i32, u16, bool)>::max_serialized_cdr_size()
//!     }
//!
//!     fn alignment() -> usize {
//!         // This corresponds to the max alignment of the four fields (i.e. f64::alignment()).
//!         <(f64, i32, u16, bool)>::alignment()
//!     }
//! }
//!
//! assert_eq!(BoundedData::max_serialized_cdr_size(), CdrSize::Bounded(16));
//! assert_eq!(BoundedData::alignment(), 8);
//! ```
//!
//! ## Implementing bounds for unbounded data
//!
//! ```
//! use cyclonedds::cdr_bounds::{CdrBounds, CdrSize};
//!
//! struct UnboundedData {
//!     data: Vec<u8>,
//! }
//!
//! impl CdrBounds for UnboundedData {
//!     fn max_serialized_cdr_size() -> CdrSize {
//!         Vec::<u8>::max_serialized_cdr_size()
//!     }
//!
//!     fn alignment() -> usize {
//!         // This corresponds to the max alignment of the length and element type.
//!         // u32::alignment().max(u8::alignment())
//!         Vec::<u8>::alignment()
//!     }
//! }
//!
//! assert_eq!(UnboundedData::max_serialized_cdr_size(), CdrSize::Unbounded);
//! assert_eq!(UnboundedData::alignment(), 4);
//! ```

/// The maximum serialized CDR size of a type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CdrSize {
    /// The type has a known maximum serialized size in bytes.
    Bounded(usize),
    /// The type has no fixed upper bound on its serialized size.
    Unbounded,
}

pub(crate) trait Padding {
    /// Returns `self` rounded up to the nearest multiple of `align`.
    fn with_padding(self, align: usize) -> Self;
}

impl Padding for usize {
    #[inline]
    fn with_padding(self, align: usize) -> usize {
        self + (align - (self % align)) % align
    }
}

impl Padding for CdrSize {
    #[inline]
    fn with_padding(self, align: usize) -> Self {
        match self {
            CdrSize::Bounded(size) => CdrSize::Bounded(size.with_padding(align)),
            CdrSize::Unbounded => CdrSize::Unbounded,
        }
    }
}

macro_rules! impl_cdr_size_op {
    ($trait:ident, $method:ident, $op:tt) => {
        // CdrSize {op} CdrSize
        impl $trait<CdrSize> for CdrSize {
            type Output = Self;

            fn $method(self, rhs: CdrSize) -> Self::Output {
                match (self, rhs) {
                    (CdrSize::Bounded(lhs), CdrSize::Bounded(rhs)) => CdrSize::Bounded(lhs $op rhs),
                    _ => CdrSize::Unbounded,
                }
            }
        }

        // CdrSize {op} usize
        impl $trait<usize> for CdrSize {
            type Output = Self;

            fn $method(self, rhs: usize) -> Self::Output {
                match self {
                    CdrSize::Bounded(lhs) => CdrSize::Bounded(lhs $op rhs),
                    _ => CdrSize::Unbounded,
                }
            }
        }

        // usize {op} CdrSize
        impl $trait<CdrSize> for usize {
            type Output = CdrSize;

            fn $method(self, rhs: CdrSize) -> Self::Output {
                match rhs {
                    CdrSize::Bounded(r) => CdrSize::Bounded(self $op r),
                    _ => CdrSize::Unbounded,
                }
            }
        }
    };
}

use std::ops::{Add, Div, Mul, Sub};
impl_cdr_size_op!(Add, add, +);
impl_cdr_size_op!(Sub, sub, -);
impl_cdr_size_op!(Mul, mul, *);
impl_cdr_size_op!(Div, div, /);

macro_rules! impl_cdr_size_op_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        // CdrSize {op}= CdrSize
        impl $trait<CdrSize> for CdrSize {
            fn $method(&mut self, rhs: CdrSize) {
                *self = *self $op rhs;
            }
        }

        // CdrSize {op}= usize
        impl $trait<usize> for CdrSize {
            fn $method(&mut self, rhs: usize) {
                *self = *self $op rhs;
            }
        }
    }
}

use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};
impl_cdr_size_op_assign!(AddAssign, add_assign, +);
impl_cdr_size_op_assign!(DivAssign, div_assign, /);
impl_cdr_size_op_assign!(MulAssign, mul_assign, *);
impl_cdr_size_op_assign!(SubAssign, sub_assign, -);

/// Describes the CDR serialization bounds of a topic key.
///
/// This information is used when computing keyhashes. If the maximum serialized
/// size of a topic type is 16 bytes or fewer the keyhash is based on the
/// big-endian serialized form of the key. Otherwise, it is based on the md5
/// hash of the data. If the key type contains unbounded fields (e.g. `Vec`,
/// `String`), return [`CdrSize::Unbounded`].
pub trait CdrBounds {
    /// Returns the maximum serialized CDR size of this type.
    fn max_serialized_cdr_size() -> CdrSize;
    /// Returns the required CDR alignment of this type in bytes.
    fn alignment() -> usize;
}

macro_rules! impl_cdr_bounds_by_size {
    ($t:ty, $size:expr) => {
        impl CdrBounds for $t {
            fn max_serialized_cdr_size() -> CdrSize {
                CdrSize::Bounded($size)
            }

            fn alignment() -> usize {
                $size
            }
        }
    };
}

impl_cdr_bounds_by_size!((), 0);
impl_cdr_bounds_by_size!(char, 1);
impl_cdr_bounds_by_size!(bool, 1);
impl_cdr_bounds_by_size!(u8, 1);
impl_cdr_bounds_by_size!(u16, 2);
impl_cdr_bounds_by_size!(u32, 4);
impl_cdr_bounds_by_size!(u64, 8);
impl_cdr_bounds_by_size!(i8, 1);
impl_cdr_bounds_by_size!(i16, 2);
impl_cdr_bounds_by_size!(i32, 4);
impl_cdr_bounds_by_size!(i64, 8);
impl_cdr_bounds_by_size!(f32, 4);
impl_cdr_bounds_by_size!(f64, 8);

impl<T, const N: usize> CdrBounds for [T; N]
where
    T: CdrBounds,
{
    fn max_serialized_cdr_size() -> CdrSize {
        T::max_serialized_cdr_size().with_padding(T::alignment()) * N
    }

    fn alignment() -> usize {
        T::alignment()
    }
}

macro_rules! impl_cdr_bounds_tuple {
    ($head:ident $(, $tail:ident)* $(,)?) => {
        impl<$head: CdrBounds $(, $tail: CdrBounds)*> CdrBounds for ($head, $($tail, )*) {
            fn max_serialized_cdr_size() -> CdrSize {
                ($head::max_serialized_cdr_size() $(.with_padding($tail::alignment()) + $tail::max_serialized_cdr_size())*).with_padding($head::alignment())
            }

            fn alignment() -> usize {
                let alignment = $head::alignment();
                $(let alignment = alignment.max($tail::alignment());)*
                    alignment
            }
        }

        impl_cdr_bounds_tuple!($($tail),*);
    };

    () => {};
}

impl_cdr_bounds_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, }

impl<T> CdrBounds for Vec<T>
where
    T: CdrBounds,
{
    fn max_serialized_cdr_size() -> CdrSize {
        CdrSize::Unbounded
    }

    fn alignment() -> usize {
        // max between alignment of length and type.
        u32::alignment().max(T::alignment())
    }
}

impl CdrBounds for String {
    fn max_serialized_cdr_size() -> CdrSize {
        CdrSize::Unbounded
    }

    fn alignment() -> usize {
        // length field as alignment
        u32::alignment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdr_size_bounds_comparisons() {
        let min_bounded = CdrSize::Bounded(usize::MIN);
        let max_bounded = CdrSize::Bounded(usize::MAX);
        let unbounded = CdrSize::Unbounded;

        assert!(min_bounded < max_bounded);
        assert!(min_bounded < unbounded);

        assert!(max_bounded > min_bounded);
        assert!(max_bounded < unbounded);

        assert!(unbounded > min_bounded);
        assert!(unbounded > max_bounded);
    }

    #[test]
    fn test_cdr_size_bounds_arithmetic() {
        let lhs = 10;
        let rhs = 5;

        assert_eq!(
            CdrSize::Bounded(lhs + rhs),
            CdrSize::Bounded(lhs) + CdrSize::Bounded(rhs)
        );
        assert_eq!(
            CdrSize::Unbounded,
            CdrSize::Bounded(lhs) + CdrSize::Unbounded
        );
        assert_eq!(CdrSize::Bounded(lhs + rhs), CdrSize::Bounded(lhs) + rhs);
        assert_eq!(CdrSize::Bounded(lhs + rhs), lhs + CdrSize::Bounded(rhs));
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded + rhs);
        assert_eq!(CdrSize::Unbounded, lhs + CdrSize::Unbounded);

        assert_eq!(
            CdrSize::Bounded(lhs - rhs),
            CdrSize::Bounded(lhs) - CdrSize::Bounded(rhs)
        );
        assert_eq!(
            CdrSize::Unbounded,
            CdrSize::Bounded(lhs) - CdrSize::Unbounded
        );
        assert_eq!(CdrSize::Bounded(lhs - rhs), CdrSize::Bounded(lhs) - rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded - rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded - rhs);
        assert_eq!(CdrSize::Unbounded, lhs - CdrSize::Unbounded);

        assert_eq!(
            CdrSize::Bounded(lhs * rhs),
            CdrSize::Bounded(lhs) * CdrSize::Bounded(rhs)
        );
        assert_eq!(
            CdrSize::Unbounded,
            CdrSize::Bounded(lhs) * CdrSize::Unbounded
        );
        assert_eq!(CdrSize::Bounded(lhs * rhs), CdrSize::Bounded(lhs) * rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded * rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded * rhs);
        assert_eq!(CdrSize::Unbounded, lhs * CdrSize::Unbounded);

        assert_eq!(
            CdrSize::Bounded(lhs / rhs),
            CdrSize::Bounded(lhs) / CdrSize::Bounded(rhs)
        );
        assert_eq!(
            CdrSize::Unbounded,
            CdrSize::Bounded(lhs) / CdrSize::Unbounded
        );
        assert_eq!(CdrSize::Bounded(lhs / rhs), CdrSize::Bounded(lhs) / rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded / rhs);
        assert_eq!(CdrSize::Unbounded, CdrSize::Unbounded / rhs);
        assert_eq!(CdrSize::Unbounded, lhs / CdrSize::Unbounded);

        let mut size = CdrSize::Bounded(10);

        size += CdrSize::Bounded(1);
        assert_eq!(size, CdrSize::Bounded(11));
        size += 1;
        assert_eq!(size, CdrSize::Bounded(12));

        size -= CdrSize::Bounded(1);
        assert_eq!(size, CdrSize::Bounded(11));
        size -= 1;
        assert_eq!(size, CdrSize::Bounded(10));

        size *= CdrSize::Bounded(10);
        assert_eq!(size, CdrSize::Bounded(100));
        size *= 10;
        assert_eq!(size, CdrSize::Bounded(1000));

        size /= CdrSize::Bounded(10);
        assert_eq!(size, CdrSize::Bounded(100));
        size /= 10;
        assert_eq!(size, CdrSize::Bounded(10));

        size = CdrSize::Bounded(1);
        size += CdrSize::Unbounded;
        assert_eq!(size, CdrSize::Unbounded);

        size = CdrSize::Bounded(1);
        size -= CdrSize::Unbounded;
        assert_eq!(size, CdrSize::Unbounded);

        size = CdrSize::Bounded(1);
        size *= CdrSize::Unbounded;
        assert_eq!(size, CdrSize::Unbounded);

        size = CdrSize::Bounded(1);
        size /= CdrSize::Unbounded;
        assert_eq!(size, CdrSize::Unbounded);
    }

    #[test]
    fn test_cdr_size_bounds_various_types() {
        assert_eq!(CdrSize::Unbounded, <Vec<i32>>::max_serialized_cdr_size());
        assert_eq!(4, <Vec<i32>>::alignment());

        assert_eq!(CdrSize::Unbounded, <Vec<i64>>::max_serialized_cdr_size());
        assert_eq!(8, <Vec<i64>>::alignment());

        assert_eq!(CdrSize::Unbounded, <String>::max_serialized_cdr_size());
        assert_eq!(4, <String>::alignment());

        assert_eq!(CdrSize::Bounded(4), <i32>::max_serialized_cdr_size());
        assert_eq!(4, <i32>::alignment());

        assert_eq!(
            CdrSize::Bounded(16),
            <(i32, i64)>::max_serialized_cdr_size()
        );
        assert_eq!(8, <(i32, i64)>::alignment());

        assert_eq!(
            CdrSize::Bounded(16 * 10),
            <[(i32, i64); 10]>::max_serialized_cdr_size()
        );
        assert_eq!(8, <[(i32, i64); 10]>::alignment());

        assert_eq!(
            CdrSize::Unbounded,
            <(Vec<i32>, i64)>::max_serialized_cdr_size()
        );
        assert_eq!(8, <(Vec<i32>, i64)>::alignment());

        assert_eq!(
            CdrSize::Bounded(20),
            <(u32, u32, u32, u32, u8)>::max_serialized_cdr_size()
        );
    }
}
