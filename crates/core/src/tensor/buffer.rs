//! Internal shared buffer abstraction.

use std::sync::Arc;

/// Shared owner of tensor element storage.
///
/// The buffer intentionally exposes only crate-private APIs so that tensor semantics stay
/// centralized in this module. All aliasing and uniqueness checks flow through this type.
#[derive(Clone, Debug)]
pub(crate) struct Buffer<T> {
    data: Arc<Vec<T>>,
}

impl<T> Buffer<T> {
    /// Wrap a freshly allocated vector inside the shared buffer.
    pub(crate) fn from_vec(data: Vec<T>) -> Self {
        Self { data: Arc::new(data) }
    }

    /// Wrap an existing shared allocation.
    pub(crate) fn from_arc(data: Arc<Vec<T>>) -> Self {
        Self { data }
    }

    /// Current logical length of the underlying storage.
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if there are no other owners of the underlying allocation.
    pub(crate) fn is_unique(&self) -> bool {
        Arc::strong_count(&self.data) == 1
    }

    /// Immutable slice view for read-only tensor operations.
    pub(crate) fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    /// Mutable slice view, available only when the buffer is uniquely owned.
    pub(crate) fn as_mut_slice(&mut self) -> Option<&mut [T]> {
        Arc::get_mut(&mut self.data).map(Vec::as_mut_slice)
    }
}
