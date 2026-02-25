//! Layout marker definitions for tensors.

/// Marker for row-major (C-order) tensor layouts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowMajor;

/// Marker for column-major (Fortran-order) tensor layouts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnMajor;

/// Type-level constraint describing valid tensor layout markers.
pub trait LayoutMarker: sealed::Sealed {}

impl LayoutMarker for RowMajor {}
impl LayoutMarker for ColumnMajor {}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::RowMajor {}
    impl Sealed for super::ColumnMajor {}
}
