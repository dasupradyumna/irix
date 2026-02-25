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

use core::{convert::TryFrom, ops::Range};
use std::{marker::PhantomData, sync::Arc};

use self::{
    buffer::Buffer,
    layout::{LayoutMarker, RowMajor, canonical_strides_for},
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

        let strides = canonical_strides_for(L::ORDER, &shape).ok_or(TensorError::InvalidShape)?;

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

    /// Returns the shape (extent of each axis) for this tensor.
    pub fn shape(&self) -> &[usize; N] {
        &self.shape
    }

    /// Returns the strides for each axis, expressed in element offsets.
    pub fn strides(&self) -> &[isize; N] {
        &self.strides
    }

    /// Logical buffer offset for the origin index `(0, ..., 0)`.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Checks whether this tensor's strides match the canonical layout definition.
    pub fn is_contiguous(&self) -> bool {
        canonical_strides_for(L::ORDER, &self.shape)
            .map(|canonical| canonical == self.strides)
            .unwrap_or(false)
    }

    /// Immutable access to the element at `idx` if it lies within the tensor bounds.
    pub fn get(&self, idx: [usize; N]) -> Option<&T> {
        let linear = self.linear_index(&idx)?;
        self.buffer.as_slice().get(linear)
    }

    /// Mutable access guarded by explicit aliasing checks; returns `None` if aliased or out of bounds.
    pub fn get_mut(&mut self, idx: [usize; N]) -> Option<&mut T> {
        let linear = self.linear_index(&idx)?;
        if !self.buffer.is_unique() {
            return None;
        }
        let slice = self.buffer.as_mut_slice()?;
        slice.get_mut(linear)
    }

    /// Returns a view defined by per-axis ranges sharing the underlying buffer.
    pub fn slice(&self, ranges: [Range<usize>; N]) -> Result<Self, TensorError> {
        let mut new_shape = self.shape;
        let mut new_offset =
            isize::try_from(self.offset).map_err(|_| TensorError::IndexOutOfBounds)?;

        for (axis, range) in ranges.into_iter().enumerate() {
            if range.start >= range.end {
                return Err(TensorError::InvalidShape);
            }
            let axis_extent = self.shape[axis];
            if range.end > axis_extent {
                return Err(TensorError::IndexOutOfBounds);
            }

            let start_isize =
                isize::try_from(range.start).map_err(|_| TensorError::IndexOutOfBounds)?;
            let delta = self.strides[axis]
                .checked_mul(start_isize)
                .ok_or(TensorError::IndexOutOfBounds)?;
            new_offset = new_offset.checked_add(delta).ok_or(TensorError::IndexOutOfBounds)?;

            new_shape[axis] = range.end - range.start;
        }

        let new_offset = usize::try_from(new_offset).map_err(|_| TensorError::IndexOutOfBounds)?;

        Ok(Self {
            buffer: self.buffer.clone(),
            shape: new_shape,
            strides: self.strides,
            offset: new_offset,
            _layout: PhantomData,
        })
    }
}

fn validate_shape_dims<const N: usize>(shape: &[usize; N]) -> Result<(), TensorError> {
    if shape.contains(&0) {
        return Err(TensorError::InvalidShape);
    }
    Ok(())
}

fn element_count<const N: usize>(shape: &[usize; N]) -> Result<usize, TensorError> {
    shape
        .iter()
        .try_fold(1usize, |acc, &dim| acc.checked_mul(dim).ok_or(TensorError::InvalidShape))
}

impl<const N: usize, T, L> Tensor<N, T, L>
where
    L: LayoutMarker,
{
    fn linear_index(&self, idx: &[usize; N]) -> Option<usize> {
        let mut linear = isize::try_from(self.offset).ok()?;
        for (axis, &coord) in idx.iter().enumerate() {
            if coord >= self.shape[axis] {
                return None;
            }
            let coord_isize = isize::try_from(coord).ok()?;
            let step = self.strides[axis].checked_mul(coord_isize)?;
            linear = linear.checked_add(step)?;
        }

        let linear = usize::try_from(linear).ok()?;
        if linear >= self.buffer.len() {
            return None;
        }
        Some(linear)
    }
}
