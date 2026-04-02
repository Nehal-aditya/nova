//! Tensor Type Operations
//!
//! Implements type-level rules for tensor operations:
//!   - Matrix multiply (@): shape compatibility checking
//!   - Einsum: rank inference
//!   - Element-wise ops: broadcast rules
//!   - Autodiff: gradient tensor shape inference

use crate::types::{NovaType, TensorShape};
use crate::unify::UnifyError;

/// Result of a tensor operation — the inferred output type.
#[derive(Debug, Clone)]
pub struct TensorOpResult {
    pub output_type: NovaType,
}

/// Check and infer the result type of matrix multiplication: A @ B
///
/// Rules:
///   Tensor[T, (m, k)] @ Tensor[T, (k, n)] -> Tensor[T, (m, n)]
///   Tensor[T, rank=2] @ Tensor[T, rank=2] -> Tensor[T, rank=2]
///   Element types must unify (same type or both Float/UnitFloat of same dim)
pub fn infer_matmul(
    lhs: &NovaType,
    rhs: &NovaType,
    line: u32,
    col: u32,
) -> Result<TensorOpResult, UnifyError> {
    let (l_elem, l_shape) = expect_tensor(lhs, line, col)?;
    let (r_elem, r_shape) = expect_tensor(rhs, line, col)?;

    // Element types must be compatible
    check_elem_compatible(l_elem, r_elem, line, col)?;

    // Shape inference
    let out_shape = infer_matmul_shape(l_shape, r_shape, line, col)?;

    Ok(TensorOpResult {
        output_type: NovaType::Tensor {
            elem: Box::new(l_elem.clone()),
            shape: out_shape,
        },
    })
}

fn infer_matmul_shape(
    lhs: &TensorShape,
    rhs: &TensorShape,
    line: u32,
    col: u32,
) -> Result<TensorShape, UnifyError> {
    use TensorShape::*;
    match (lhs, rhs) {
        // Both shapes fully known: check inner dims match
        (Known(a), Known(b)) => {
            if a.len() < 2 || b.len() < 2 {
                return Err(UnifyError::at(
                    "matmul requires at least rank-2 tensors", line, col,
                ));
            }
            let a_inner = a[a.len() - 1];
            let b_outer = b[b.len() - 2];
            if a_inner != b_outer {
                return Err(UnifyError::at(
                    format!(
                        "matmul shape mismatch: inner dimensions {} vs {} do not match\n  \
                         left:  {:?}\n  right: {:?}\n  \
                         hint:  A @ B requires A.cols == B.rows",
                        a_inner, b_outer, a, b
                    ),
                    line, col,
                ));
            }
            // Result shape: a[0..n-1] + b[last]
            let mut out = a[..a.len()-1].to_vec();
            out.push(b[b.len() - 1]);
            Ok(Known(out))
        }
        // Rank-2 @ Rank-2 → Rank-2
        (Rank(2), Rank(2)) => Ok(Rank(2)),
        // Any rank-2-compatible pair
        (Rank(r1), Rank(r2)) if r1 == r2 && *r1 >= 2 => Ok(Rank(*r1)),
        // Unknown shapes: result is also unknown rank 2
        (Unknown, _) | (_, Unknown) => Ok(Rank(2)),
        _ => Err(UnifyError::at(
            format!("matmul incompatible shapes: {} @ {}", lhs, rhs), line, col,
        )),
    }
}

/// Infer element-wise binary op (add, sub, mul, div) result type.
/// Both tensors must have the same shape (or broadcast-compatible).
pub fn infer_elementwise(
    lhs: &NovaType,
    rhs: &NovaType,
    line: u32,
    col: u32,
) -> Result<TensorOpResult, UnifyError> {
    let (l_elem, l_shape) = expect_tensor(lhs, line, col)?;
    let (r_elem, r_shape) = expect_tensor(rhs, line, col)?;
    check_elem_compatible(l_elem, r_elem, line, col)?;
    let out_shape = broadcast_shapes(l_shape, r_shape, line, col)?;
    Ok(TensorOpResult {
        output_type: NovaType::Tensor {
            elem: Box::new(l_elem.clone()),
            shape: out_shape,
        },
    })
}

/// Infer gradient tensor shape: same shape as the original tensor.
/// `autodiff(loss)` where loss: Tensor[Float, shape] → grad: Tensor[Float, shape]
pub fn infer_gradient_shape(forward_type: &NovaType) -> NovaType {
    match forward_type {
        NovaType::Tensor { elem, shape } => NovaType::Tensor {
            elem: Box::new(strip_unit(*elem.clone())),
            shape: shape.clone(),
        },
        NovaType::UnitFloat { .. } | NovaType::Float => NovaType::Float,
        other => other.clone(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn expect_tensor(
    ty: &NovaType,
    line: u32,
    col: u32,
) -> Result<(&NovaType, &TensorShape), UnifyError> {
    match ty {
        NovaType::Tensor { elem, shape } => Ok((elem.as_ref(), shape)),
        _ => Err(UnifyError::at(
            format!("expected Tensor type, found {}\n  hint: @ operator requires both operands to be Tensor types", ty),
            line, col,
        )),
    }
}

fn check_elem_compatible(
    lhs: &NovaType,
    rhs: &NovaType,
    line: u32,
    col: u32,
) -> Result<(), UnifyError> {
    use NovaType::*;
    match (lhs, rhs) {
        (Float, Float) => Ok(()),
        (UnitFloat { dim: d1, .. }, UnitFloat { dim: d2, .. }) if d1 == d2 => Ok(()),
        (Float, UnitFloat { .. }) | (UnitFloat { .. }, Float) => Ok(()),
        (Int, Int) => Ok(()),
        _ if lhs == rhs => Ok(()),
        _ => Err(UnifyError::at(
            format!("tensor element type mismatch: {} vs {}", lhs, rhs),
            line, col,
        )),
    }
}

fn broadcast_shapes(
    lhs: &TensorShape,
    rhs: &TensorShape,
    line: u32,
    col: u32,
) -> Result<TensorShape, UnifyError> {
    use TensorShape::*;
    match (lhs, rhs) {
        (Known(a), Known(b)) if a == b => Ok(Known(a.clone())),
        (Rank(a), Rank(b)) if a == b   => Ok(Rank(*a)),
        (Unknown, s) | (s, Unknown)    => Ok(s.clone()),
        _ => Err(UnifyError::at(
            format!("tensor shape mismatch for element-wise op: {} vs {}", lhs, rhs),
            line, col,
        )),
    }
}

/// Strip unit annotation from a Float type (for gradient tensors).
fn strip_unit(ty: NovaType) -> NovaType {
    match ty {
        NovaType::UnitFloat { .. } => NovaType::Float,
        other => other,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NovaType, TensorShape};
    use nova_unit_resolver::dimension::Dim;

    fn float_tensor(shape: TensorShape) -> NovaType {
        NovaType::Tensor { elem: Box::new(NovaType::Float), shape }
    }

    #[test]
    fn matmul_known_shapes_ok() {
        let a = float_tensor(TensorShape::Known(vec![4, 8]));
        let b = float_tensor(TensorShape::Known(vec![8, 16]));
        let r = infer_matmul(&a, &b, 0, 0).unwrap();
        assert_eq!(r.output_type, float_tensor(TensorShape::Known(vec![4, 16])));
    }

    #[test]
    fn matmul_inner_dim_mismatch_fails() {
        let a = float_tensor(TensorShape::Known(vec![4, 8]));
        let b = float_tensor(TensorShape::Known(vec![9, 16])); // 8 != 9
        let err = infer_matmul(&a, &b, 5, 3).unwrap_err();
        assert!(err.message.contains("shape mismatch"));
        assert!(err.message.contains("hint"));
        assert_eq!(err.line, 5);
    }

    #[test]
    fn matmul_rank2_symbolic_ok() {
        let a = float_tensor(TensorShape::Rank(2));
        let b = float_tensor(TensorShape::Rank(2));
        let r = infer_matmul(&a, &b, 0, 0).unwrap();
        match r.output_type {
            NovaType::Tensor { shape: TensorShape::Rank(2), .. } => {}
            other => panic!("expected Rank(2) tensor, got {:?}", other),
        }
    }

    #[test]
    fn matmul_non_tensor_fails() {
        let a = NovaType::Float;
        let b = float_tensor(TensorShape::Rank(2));
        assert!(infer_matmul(&a, &b, 0, 0).is_err());
    }

    #[test]
    fn elementwise_same_shape_ok() {
        let a = float_tensor(TensorShape::Known(vec![1024]));
        let b = float_tensor(TensorShape::Known(vec![1024]));
        assert!(infer_elementwise(&a, &b, 0, 0).is_ok());
    }

    #[test]
    fn elementwise_shape_mismatch_fails() {
        let a = float_tensor(TensorShape::Known(vec![512]));
        let b = float_tensor(TensorShape::Known(vec![1024]));
        assert!(infer_elementwise(&a, &b, 0, 0).is_err());
    }

    #[test]
    fn gradient_shape_preserved() {
        let fwd = float_tensor(TensorShape::Known(vec![512, 1024]));
        let grad = infer_gradient_shape(&fwd);
        assert_eq!(grad, float_tensor(TensorShape::Known(vec![512, 1024])));
    }

    #[test]
    fn gradient_strips_unit() {
        let fwd = NovaType::Tensor {
            elem: Box::new(NovaType::UnitFloat { dim: Dim::ENERGY, unit_str: "eV".into() }),
            shape: TensorShape::Rank(1),
        };
        let grad = infer_gradient_shape(&fwd);
        match grad {
            NovaType::Tensor { elem, .. } => assert_eq!(*elem, NovaType::Float),
            _ => panic!("expected tensor"),
        }
    }
}
