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

use std::marker::PhantomData;

use self::{
    buffer::Buffer,
    layout::{LayoutMarker, RowMajor},
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
