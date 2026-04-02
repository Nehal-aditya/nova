//! NOVA Tensor Lowering Phase
//!
//! Phase 4 of the NOVA compiler pipeline.
//!
//! ## What this crate does
//!
//! Lowers high-level tensor operations to primitives suitable for LLVM IR emission:
//!   - Shape inference and validation
//!   - Dimension checking for operations (matmul, add, etc.)
//!   - Broadcasting rule application
//!   - Optimization patterns for common cases
//!   - Memory layout decisions

pub mod shape_check;
pub mod matmul;

pub use shape_check::{ShapeChecker, ShapeError, TensorShape};
pub use matmul::{MatmulOptimizer, MatmulStrategy};

use thiserror::Error;

/// Comprehensive error type for tensor lowering.
#[derive(Debug, Clone, Error)]
pub enum LoweringError {
    #[error("shape error: {0}")]
    ShapeError(String),

    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("broadcasting failed: shapes {left} and {right} incompatible")]
    BroadcastingFailed { left: String, right: String },

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("{message}")]
    Other { message: String },
}

/// Main tensor lowering orchestrator.
pub struct TensorLowerer {
    shape_checker: ShapeChecker,
    matmul_optimizer: MatmulOptimizer,
}

impl TensorLowerer {
    pub fn new() -> Self {
        TensorLowerer {
            shape_checker: ShapeChecker::new(),
            matmul_optimizer: MatmulOptimizer::new(),
        }
    }

    /// Check tensor dimensions for an operation.
    pub fn check_dimensions(
        &self,
        left: &[usize],
        right: &[usize],
    ) -> Result<Vec<usize>, LoweringError> {
        self.shape_checker
            .broadcast(left, right)
            .map_err(|e| LoweringError::ShapeError(e.to_string()))
    }

    /// Infer output shape for matmul.
    pub fn infer_matmul_shape(
        &self,
        left: &[usize],
        right: &[usize],
    ) -> Result<Vec<usize>, LoweringError> {
        if left.len() < 2 || right.len() < 2 {
            return Err(LoweringError::DimensionMismatch {
                expected: 2,
                actual: if left.len() < 2 { left.len() } else { right.len() },
            });
        }

        if left[left.len() - 1] != right[right.len() - 2] {
            return Err(LoweringError::BroadcastingFailed {
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            });
        }

        let mut result = Vec::new();
        // Batch dimensions
        for &d in &left[..left.len() - 2] {
            result.push(d);
        }
        // Output rows
        result.push(left[left.len() - 2]);
        // Output cols
        result.push(right[right.len() - 1]);
        Ok(result)
    }

    /// Optimize matmul operation.
    pub fn optimize_matmul(
        &self,
        left_shape: &[usize],
        right_shape: &[usize],
    ) -> MatmulStrategy {
        self.matmul_optimizer.choose_strategy(left_shape, right_shape)
    }

    /// Validate that shapes are compatible for element-wise operations.
    pub fn validate_elementwise(
        &self,
        shapes: &[Vec<usize>],
    ) -> Result<Vec<usize>, LoweringError> {
        if shapes.is_empty() {
            return Err(LoweringError::Other {
                message: "need at least one shape for element-wise operation".to_string(),
            });
        }

        let mut result = shapes[0].clone();
        for shape in &shapes[1..] {
            result = self
                .shape_checker
                .broadcast(&result, shape)
                .map_err(|e| LoweringError::ShapeError(e.to_string()))?;
        }
        Ok(result)
    }
}

impl Default for TensorLowerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowerer_creation() {
        let lowerer = TensorLowerer::new();
        let _ = lowerer;
    }

    #[test]
    fn infer_matmul_2d() {
        let lowerer = TensorLowerer::new();
        let left = vec![3, 4];
        let right = vec![4, 5];
        let result = lowerer.infer_matmul_shape(&left, &right).expect("should infer");
        assert_eq!(result, vec![3, 5]);
    }

    #[test]
    fn infer_matmul_batched() {
        let lowerer = TensorLowerer::new();
        let left = vec![2, 3, 4];
        let right = vec![4, 5];
        let result = lowerer.infer_matmul_shape(&left, &right).expect("should infer");
        assert_eq!(result, vec![2, 3, 5]);
    }

    #[test]
    fn matmul_shape_mismatch_error() {
        let lowerer = TensorLowerer::new();
        let left = vec![3, 4];
        let right = vec![5, 6]; // inner dimensions don't match
        let result = lowerer.infer_matmul_shape(&left, &right);
        assert!(result.is_err());
    }

    #[test]
    fn elementwise_same_shape() {
        let lowerer = TensorLowerer::new();
        let shapes = vec![vec![3, 4], vec![3, 4]];
        let result = lowerer.validate_elementwise(&shapes).expect("should validate");
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn elementwise_broadcast_scalar() {
        let lowerer = TensorLowerer::new();
        let shapes = vec![vec![3, 4], vec![1]];
        let result = lowerer.validate_elementwise(&shapes).expect("should validate");
        assert_eq!(result, vec![3, 4]);
    }

    #[test]
    fn check_dimensions() {
        let lowerer = TensorLowerer::new();
        let left = vec![3, 4];
        let right = vec![3, 4];
        let result = lowerer
            .check_dimensions(&left, &right)
            .expect("should check");
        assert_eq!(result, vec![3, 4]);
    }
}

