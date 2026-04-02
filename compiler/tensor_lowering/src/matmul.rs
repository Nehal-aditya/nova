//! Matrix multiplication optimization and lowering.
//!
//! Chooses appropriate implementation strategy based on shape:
//!   - Small dense matrices → direct LLVM code
//!   - Batched operations → vectorized routines
//!   - Large matrices → delegate to BLAS library stub

use std::fmt;

/// Strategy for implementing matrix multiplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatmulStrategy {
    /// Direct inline LLVM IR for small matrices
    DirectLLVM,
    /// Use vector instructions for batched operations
    Vectorized,
    /// Delegate to BLAS library (not available at compile time)
    BLASDelegate,
}

impl fmt::Display for MatmulStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatmulStrategy::DirectLLVM => write!(f, "direct_llvm"),
            MatmulStrategy::Vectorized => write!(f, "vectorized"),
            MatmulStrategy::BLASDelegate => write!(f, "blas_delegate"),
        }
    }
}

/// Optimizes matrix multiplication operations.
pub struct MatmulOptimizer;

impl MatmulOptimizer {
    pub fn new() -> Self {
        MatmulOptimizer
    }

    /// Choose an implementation strategy based on matrix shapes.
    pub fn choose_strategy(&self, left: &[usize], right: &[usize]) -> MatmulStrategy {
        let left_cols = left[left.len() - 1];
        let left_rows = left[left.len() - 2];
        let right_cols = right[right.len() - 1];

        // For small 2D matrices, use direct LLVM
        if left.len() == 2 && right.len() == 2 {
            if left_rows <= 1024 && left_cols <= 1024 && right_cols <= 1024 {
                return MatmulStrategy::DirectLLVM;
            }
        }

        // For batched operations with small inner dimensions
        if left.len() > 2 || right.len() > 2 {
            if left_cols <= 512 && right_cols <= 512 {
                return MatmulStrategy::Vectorized;
            }
        }

        // Large matrices: delegate to BLAS
        MatmulStrategy::BLASDelegate
    }

    /// Estimate the computational cost of a matmul (operations count).
    pub fn estimate_cost(&self, left: &[usize], right: &[usize]) -> usize {
        let m = left[left.len() - 2]; // output rows
        let n = right[right.len() - 1]; // output cols
        let k = left[left.len() - 1]; // inner dimension

        // Number of flops: m * n * k multiplications + accumulations
        let mut flops = m * n * k * 2;

        // Account for batch dimensions
        for &d in &left[..left.len() - 2] {
            flops *= d;
        }

        flops
    }

    /// Recommend whether to transpose operands for cache efficiency.
    pub fn should_transpose_left(&self, left: &[usize]) -> bool {
        if left.len() != 2 {
            return false;
        }
        // Transpose if more columns than rows (row-major layout preferred)
        left[1] > left[0]
    }
}

impl Default for MatmulOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_strategy_small_2d() {
        let optimizer = MatmulOptimizer::new();
        let strategy = optimizer.choose_strategy(&[10, 20], &[20, 30]);
        assert_eq!(strategy, MatmulStrategy::DirectLLVM);
    }

    #[test]
    fn choose_strategy_batched() {
        let optimizer = MatmulOptimizer::new();
        let strategy = optimizer.choose_strategy(&[5, 100, 200], &[200, 300]);
        assert_eq!(strategy, MatmulStrategy::Vectorized);
    }

    #[test]
    fn choose_strategy_large() {
        let optimizer = MatmulOptimizer::new();
        let strategy = optimizer.choose_strategy(&[2000, 3000], &[3000, 4000]);
        assert_eq!(strategy, MatmulStrategy::BLASDelegate);
    }

    #[test]
    fn estimate_cost_2d() {
        let optimizer = MatmulOptimizer::new();
        let cost = optimizer.estimate_cost(&[10, 20], &[20, 30]);
        // 10 * 30 * 20 * 2 = 12000
        assert_eq!(cost, 12000);
    }

    #[test]
    fn estimate_cost_batched() {
        let optimizer = MatmulOptimizer::new();
        let cost = optimizer.estimate_cost(&[5, 10, 20], &[20, 30]);
        // (5 * 10 * 30 * 20 * 2) = 60000
        assert_eq!(cost, 60000);
    }

    #[test]
    fn transpose_recommendation_wide() {
        let optimizer = MatmulOptimizer::new();
        assert!(optimizer.should_transpose_left(&[10, 100]));
    }

    #[test]
    fn transpose_recommendation_tall() {
        let optimizer = MatmulOptimizer::new();
        assert!(!optimizer.should_transpose_left(&[100, 10]));
    }
}

