//! Layout marker definitions for tensors.

use core::convert::TryFrom;

/// Marker for row-major (C-order) tensor layouts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowMajor;

/// Marker for column-major (Fortran-order) tensor layouts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnMajor;

/// Storage ordering used when computing canonical strides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutOrder {
    RowMajor,
    ColumnMajor,
}

/// Type-level constraint describing valid tensor layout markers.
pub trait LayoutMarker: sealed::Sealed {
    /// Compile-time layout identity for the marker.
    const ORDER: LayoutOrder;
}

impl LayoutMarker for RowMajor {
    const ORDER: LayoutOrder = LayoutOrder::RowMajor;
}

impl LayoutMarker for ColumnMajor {
    const ORDER: LayoutOrder = LayoutOrder::ColumnMajor;
}

pub(crate) fn canonical_row_major_strides<const N: usize>(
    shape: &[usize; N],
) -> Option<[isize; N]> {
    let mut strides = [0isize; N];
    let mut stride = 1isize;

    for axis in (0..N).rev() {
        strides[axis] = stride;
        let extent = shape[axis];
        let extent_isize = isize::try_from(extent).ok()?;
        stride = stride.checked_mul(extent_isize)?;
    }

    Some(strides)
}

pub(crate) fn canonical_column_major_strides<const N: usize>(
    shape: &[usize; N],
) -> Option<[isize; N]> {
    let mut strides = [0isize; N];
    let mut stride = 1isize;

    for axis in 0..N {
        strides[axis] = stride;
        let extent = shape[axis];
        let extent_isize = isize::try_from(extent).ok()?;
        stride = stride.checked_mul(extent_isize)?;
    }

    Some(strides)
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::RowMajor {}
    impl Sealed for super::ColumnMajor {}
}
/// Compute canonical strides for the provided layout order.
pub fn canonical_strides_for<const N: usize>(
    order: LayoutOrder,
    shape: &[usize; N],
) -> Option<[isize; N]> {
    match order {
        LayoutOrder::RowMajor => canonical_row_major_strides(shape),
        LayoutOrder::ColumnMajor => canonical_column_major_strides(shape),
    }
}
