//! # Tensor Module Semantics (Design Freeze — Phase 0)
//!
//! This module freezes the non-negotiable semantics for the upcoming tensor implementation.
//! No code may rely on implicit reinterpretation of these rules, and later phases must treat
//! every invariant here as normative.
//!
//! ## Axis Ordering
//! - Axes are positional only; the integer index of an axis is the entirety of its meaning.
//! - Axis `0` is always the outermost dimension and axis `N-1` the innermost.
//! - Higher layers that require semantic axis labels must encode them separately instead of
//!   overloading tensor axes.
//!
//! ## Layout Semantics
//! - Layout is a type-level marker that defines how multidimensional indices lower into linear
//!   memory.
//! - `RowMajor` (C-order) means the last axis (`N-1`) has stride `1` and strides grow when moving
//!   toward the outer axes.
//! - `ColumnMajor` (Fortran-order) means the first axis (`0`) has stride `1` and strides grow when
//!   moving toward the inner axes.
//! - Layout markers are semantic contracts, not performance hints; no automatic conversion occurs.
//!
//! ## Contiguity Definition
//! - A tensor is contiguous iff its stride array exactly matches the canonical strides implied by
//!   `shape` and the declared layout.
//! - APIs that demand contiguous memory must check this flag explicitly and fail instead of
//!   silently materializing a copy.
//!
//! ## Aliasing Model
//! - Storage is owned by an internal buffer that supports shared ownership (e.g., via `Arc`).
//! - Aliasing between tensors is always explicit: slices and views share the buffer, while distinct
//!   tensors clone or reallocate.
//! - Mutation requires unique buffer ownership; aliased tensors must reject mutable access instead
//!   of falling back to copy-on-write.
//!
//! ## Constructor Safety Rules
//! - Safe constructors validate every semantic invariant: positive dimensions, matching data
//!   length, canonical stride computation, and non-negative offsets.
//! - Unsafe constructors are allowed but must state their full contract, covering layout consistency,
//!   bounds validity, and exclusivity assumptions.
//! - No API—safe or unsafe—may reinterpret layout, axes, or strides behind the caller's back.

// XXX: is this lint too harsh? purely best-practices oriented, worth the ergonomics?
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod layout;

mod buffer;
mod error;

pub use error::TensorError;

use std::{marker::PhantomData, sync::Arc};

use self::{
    buffer::Buffer,
    layout::{
        canonical_column_major_strides, canonical_row_major_strides, LayoutMarker, LayoutOrder,
        RowMajor,
    },
};

/// Multi-dimensional numeric storage with explicit layout semantics.
///
/// # Invariants
/// - `shape` encodes the extent of every axis (no zero dimensions once constructors enforce it).
/// - `offset` is a logical index into the underlying buffer and must be non-negative.
/// - The tuple `(shape, strides, offset)` fully describes how to interpret the shared buffer.
#[derive(Clone, Debug)]
pub struct Tensor<const N: usize, T, L = RowMajor>
where
    L: LayoutMarker,
{
    buffer: Buffer<T>,
    shape: [usize; N],
    strides: [isize; N],
    offset: usize,
    _layout: PhantomData<L>,
}

impl<const N: usize, T, L> Tensor<N, T, L>
where
    L: LayoutMarker,
{
    /// Construct a tensor from owned data, validating all invariants.
    pub fn from_shape_vec(shape: [usize; N], data: Vec<T>) -> Result<Self, TensorError> {
        validate_shape_dims(&shape)?;
        let expected_len = element_count(&shape)?;
        if expected_len != data.len() {
            return Err(TensorError::LengthMismatch {
                expected: expected_len,
                actual: data.len(),
            });
        }

        let strides = canonical_strides::<N, L>(&shape)?;

        Ok(Self {
            buffer: Buffer::from_vec(data),
            shape,
            strides,
            offset: 0,
            _layout: PhantomData,
        })
    }

    /// Construct a tensor without validating invariants.
    ///
    /// # Safety
    /// Caller must guarantee that `shape`, `strides`, and `offset` describe a valid view into
    /// `data` and uphold the declared layout semantics. Violating these requirements results in
    /// undefined behavior across tensor APIs.
    pub unsafe fn new_unchecked(
        data: Arc<Vec<T>>,
        shape: [usize; N],
        strides: [isize; N],
        offset: usize,
    ) -> Self {
        Self {
            buffer: Buffer::from_arc(data),
            shape,
            strides,
            offset,
            _layout: PhantomData,
        }
    }
}

fn validate_shape_dims<const N: usize>(shape: &[usize; N]) -> Result<(), TensorError> {
    if shape.iter().any(|&dim| dim == 0) {
        return Err(TensorError::InvalidShape);
    }
    Ok(())
}

fn element_count<const N: usize>(shape: &[usize; N]) -> Result<usize, TensorError> {
    shape
        .iter()
        .try_fold(1usize, |acc, &dim| acc.checked_mul(dim).ok_or(TensorError::InvalidShape))
}

fn canonical_strides<const N: usize, L>(shape: &[usize; N]) -> Result<[isize; N], TensorError>
where
    L: LayoutMarker,
{
    match L::ORDER {
        LayoutOrder::RowMajor => canonical_row_major_strides(shape),
        LayoutOrder::ColumnMajor => canonical_column_major_strides(shape),
    }
    .ok_or(TensorError::InvalidShape)
}
