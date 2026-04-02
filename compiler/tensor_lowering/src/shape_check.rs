//! Tensor shape validation and broadcasting.
//!
//! Implements NumPy-style broadcasting rules:
//!   1. If tensors have different ranks, prepend 1s to the smaller-rank tensor
//!   2. For each dimension, sizes must be equal or one must be 1
//!   3. Output dimension is the maximum of the two

use std::fmt;
use thiserror::Error;

/// A tensor shape: e.g., [batch, height, width, channels]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TensorShape(pub Vec<usize>);

impl TensorShape {
    pub fn new(dims: Vec<usize>) -> Self {
        TensorShape(dims)
    }

    pub fn rank(&self) -> usize {
        self.0.len()
    }

    pub fn is_scalar(&self) -> bool {
        self.0.is_empty()
    }

    pub fn volume(&self) -> usize {
        self.0.iter().product()
    }
}

impl fmt::Display for TensorShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", 
            self.0.iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", "))
    }
}

/// Error in shape operations.
#[derive(Debug, Clone, Error)]
pub enum ShapeError {
    #[error("shape mismatch: {0}")]
    Mismatch(String),

    #[error("broadcast failed: shapes not compatible")]
    BroadcastFailed,

    #[error("dimension error: {0}")]
    InvalidDimension(String),
}

/// Validates and performs shape operations.
pub struct ShapeChecker;

impl ShapeChecker {
    pub fn new() -> Self {
        ShapeChecker
    }

    /// Broadcast two shapes according to NumPy rules.
    /// Returns the broadcasted shape or an error.
    pub fn broadcast(&self, left: &[usize], right: &[usize]) -> Result<Vec<usize>, ShapeError> {
        let mut left_shape = left.to_vec();
        let mut right_shape = right.to_vec();

        // Align ranks by prepending 1s
        while left_shape.len() < right_shape.len() {
            left_shape.insert(0, 1);
        }
        while right_shape.len() < left_shape.len() {
            right_shape.insert(0, 1);
        }

        // Compute broadcast shape
        let mut result = Vec::new();
        for i in 0..left_shape.len() {
            let l = left_shape[i];
            let r = right_shape[i];

            let dim = if l == 1 {
                r
            } else if r == 1 {
                l
            } else if l == r {
                l
            } else {
                return Err(ShapeError::BroadcastFailed);
            };

            result.push(dim);
        }

        Ok(result)
    }

    /// Check if two shapes could be broadcasted together.
    pub fn are_compatible(&self, left: &[usize], right: &[usize]) -> bool {
        self.broadcast(left, right).is_ok()
    }

    /// Verify matrix multiplication dimensions.
    pub fn check_matmul(&self, left: &[usize], right: &[usize]) -> Result<(), ShapeError> {
        if left.len() < 2 || right.len() < 2 {
            return Err(ShapeError::InvalidDimension(
                "matrices must be at least 2D".to_string(),
            ));
        }

        if left[left.len() - 1] != right[right.len() - 2] {
            return Err(ShapeError::Mismatch(format!(
                "inner dimensions do not match: {} vs {}",
                left[left.len() - 1],
                right[right.len() - 2]
            )));
        }

        Ok(())
    }

    /// Verify tensor addition/subtraction.
    pub fn check_elementwise(&self, shapes: &[&[usize]]) -> Result<(), ShapeError> {
        if shapes.is_empty() {
            return Err(ShapeError::InvalidDimension(
                "at least one shape required".to_string(),
            ));
        }

        let mut broadcast_shape = shapes[0].to_vec();
        for shape in &shapes[1..] {
            broadcast_shape = self.broadcast(&broadcast_shape, shape)?;
        }

        Ok(())
    }
}

impl Default for ShapeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_same_shape() {
        let checker = ShapeChecker::new();
        let result = checker.broadcast(&[3, 4], &[3, 4]).expect("should broadcast");
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn broadcast_scalar() {
        let checker = ShapeChecker::new();
        let result = checker.broadcast(&[3, 4], &[1]).expect("should broadcast");
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn broadcast_1d_2d() {
        let checker = ShapeChecker::new();
        let result = checker.broadcast(&[4], &[3, 4]).expect("should broadcast");
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn broadcast_incompatible() {
        let checker = ShapeChecker::new();
        let result = checker.broadcast(&[3, 4], &[5, 6]);
        assert!(result.is_err());
    }

    #[test]
    fn check_matmul_2d() {
        let checker = ShapeChecker::new();
        let result = checker.check_matmul(&[3, 4], &[4, 5]);
        assert!(result.is_ok());
    }

    #[test]
    fn check_matmul_mismatch() {
        let checker = ShapeChecker::new();
        let result = checker.check_matmul(&[3, 4], &[5, 6]);
        assert!(result.is_err());
    }

    #[test]
    fn tensor_shape_volume() {
        let shape = TensorShape::new(vec![2, 3, 4]);
        assert_eq!(shape.volume(), 24);
    }

    #[test]
    fn tensor_shape_rank() {
        let shape = TensorShape::new(vec![2, 3, 4]);
        assert_eq!(shape.rank(), 3);
    }
}

