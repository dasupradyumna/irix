//! Tensor-specific error definitions.

use core::fmt;

/// Errors originating from tensor construction or runtime checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TensorError {
    /// One or more dimensions were zero or overflowed representable ranges.
    InvalidShape,
    /// Provided element buffer length did not match the product of the shape.
    LengthMismatch { expected: usize, actual: usize },
    /// Operation requires contiguous storage but tensor is viewed with non-canonical strides.
    NonContiguousRequired,
    /// Mutation attempted on an aliased tensor without exclusive buffer ownership.
    AliasedMutationForbidden,
    /// Index or slice bounds exceeded tensor shape.
    IndexOutOfBounds,
}

impl fmt::Display for TensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TensorError::InvalidShape => f.write_str("invalid tensor shape"),
            TensorError::LengthMismatch { expected, actual } => {
                write!(f, "data length mismatch: expected {expected}, got {actual}")
            },
            TensorError::NonContiguousRequired => {
                f.write_str("operation requires contiguous tensor")
            },
            TensorError::AliasedMutationForbidden => {
                f.write_str("mutation forbidden because tensor storage is aliased")
            },
            TensorError::IndexOutOfBounds => f.write_str("tensor index out of bounds"),
        }
    }
}

impl std::error::Error for TensorError {}

/// Convenient result type for tensor operations.
pub type TensorResult<T> = Result<T, TensorError>;
