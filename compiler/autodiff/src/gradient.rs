//! Automatic differentiation via reverse-mode (backpropagation).
//!
//! Gradient computation rules for all operation types. The gradient of an operation
//! is computed according to the chain rule:
//!
//!   ∂L/∂x = (∂L/∂y) * (∂y/∂x)
//!
//! where y = f(x).

use crate::graph::{ComputationGraph, NodeId, OpType};
use std::collections::HashMap;

/// Stores accumulated gradients for each node.
#[derive(Debug, Clone)]
pub struct GradientAccumulator {
    gradients: HashMap<NodeId, f64>,
}

impl GradientAccumulator {
    pub fn new() -> Self {
        GradientAccumulator {
            gradients: HashMap::new(),
        }
    }

    /// Add a gradient to the accumulator.
    pub fn add_gradient(&mut self, node_id: NodeId, grad: f64) {
        *self.gradients.entry(node_id).or_insert(0.0) += grad;
    }

    /// Get the gradient for a node.
    pub fn get_gradient(&self, node_id: NodeId) -> f64 {
        self.gradients.get(&node_id).copied().unwrap_or(0.0)
    }

    /// Clear all gradients.
    pub fn clear(&mut self) {
        self.gradients.clear();
    }

    /// Get all accumulated gradients.
    pub fn get_all(&self) -> &HashMap<NodeId, f64> {
        &self.gradients
    }

    /// Get mutable gradients map.
    pub fn get_all_mut(&mut self) -> &mut HashMap<NodeId, f64> {
        &mut self.gradients
    }
}

impl Default for GradientAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Backward pass context during automatic differentiation.
pub struct BackwardContext<'a> {
    graph: &'a ComputationGraph,
    accumulator: &'a mut GradientAccumulator,
    values: &'a HashMap<NodeId, f64>,  // Forward pass values
}

impl<'a> BackwardContext<'a> {
    pub fn new(
        graph: &'a ComputationGraph,
        accumulator: &'a mut GradientAccumulator,
        values: &'a HashMap<NodeId, f64>,
    ) -> Self {
        BackwardContext {
            graph,
            accumulator,
            values,
        }
    }

    /// Get the forward pass value of a node.
    pub fn get_value(&self, node_id: NodeId) -> f64 {
        *self.values.get(&node_id).unwrap_or(&0.0)
    }

    /// Accumulate gradient to an input node.
    pub fn add_gradient(&mut self, node_id: NodeId, d_node: f64) {
        self.accumulator.add_gradient(node_id, d_node);
    }
}

/// Computes gradients using reverse-mode automatic differentiation.
pub struct DifferentiationEngine;

impl DifferentiationEngine {
    /// Perform a complete backward pass.
    pub fn backward(
        graph: &ComputationGraph,
        output_node: NodeId,
        d_output: f64,
        values: &HashMap<NodeId, f64>,
    ) -> Result<HashMap<NodeId, f64>, String> {
        let mut accumulator = GradientAccumulator::new();

        // Initialize gradient of output node
        accumulator.add_gradient(output_node, d_output);

        // Process nodes in reverse topological order
        let reverse_order = graph.get_reverse_order();
        for &node_id in reverse_order {
            let d_node = accumulator.get_gradient(node_id);
            if d_node.abs() < 1e-10 {
                continue; // Skip near-zero gradients
            }

            let node = match graph.get_node(node_id) {
                Some(n) => n,
                None => return Err(format!("Node {} not found in graph", node_id)),
            };

            let mut ctx = BackwardContext::new(graph, &mut accumulator, values);

            match node.op_type {
                OpType::Parameter | OpType::Constant => {
                    // Leaf nodes don't propagate further (already accumulated)
                }
                OpType::Add => Self::backward_add(&mut ctx, node_id, node, d_node)?,
                OpType::Sub => Self::backward_sub(&mut ctx, node_id, node, d_node)?,
                OpType::Mul => Self::backward_mul(&mut ctx, node_id, node, d_node)?,
                OpType::Div => Self::backward_div(&mut ctx, node_id, node, d_node)?,
                OpType::Neg => Self::backward_neg(&mut ctx, node_id, node, d_node)?,
                OpType::Pow => Self::backward_pow(&mut ctx, node_id, node, d_node)?,
                OpType::Sin => Self::backward_sin(&mut ctx, node_id, node, d_node)?,
                OpType::Cos => Self::backward_cos(&mut ctx, node_id, node, d_node)?,
                OpType::Tan => Self::backward_tan(&mut ctx, node_id, node, d_node)?,
                OpType::Exp => Self::backward_exp(&mut ctx, node_id, node, d_node)?,
                OpType::Log => Self::backward_log(&mut ctx, node_id, node, d_node)?,
                OpType::Sqrt => Self::backward_sqrt(&mut ctx, node_id, node, d_node)?,
                OpType::ReLU => Self::backward_relu(&mut ctx, node_id, node, d_node)?,
                OpType::Sigmoid => Self::backward_sigmoid(&mut ctx, node_id, node, d_node)?,
                OpType::Tanh => Self::backward_tanh(&mut ctx, node_id, node, d_node)?,
                _ => return Err(format!("Gradient not implemented for {:?}", node.op_type)),
            }
        }

        Ok(accumulator.gradients)
    }

    // Gradient rules for each operation
    // ∂(a + b)/∂a = 1, ∂(a + b)/∂b = 1
    fn backward_add(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.len() < 2 {
            return Err("Add requires 2 inputs".to_string());
        }
        ctx.add_gradient(node.inputs[0], d_node);
        ctx.add_gradient(node.inputs[1], d_node);
        Ok(())
    }

    // ∂(a - b)/∂a = 1, ∂(a - b)/∂b = -1
    fn backward_sub(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.len() < 2 {
            return Err("Sub requires 2 inputs".to_string());
        }
        ctx.add_gradient(node.inputs[0], d_node);
        ctx.add_gradient(node.inputs[1], -d_node);
        Ok(())
    }

    // ∂(a * b)/∂a = b, ∂(a * b)/∂b = a
    fn backward_mul(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.len() < 2 {
            return Err("Mul requires 2 inputs".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let b = ctx.get_value(node.inputs[1]);
        ctx.add_gradient(node.inputs[0], b * d_node);
        ctx.add_gradient(node.inputs[1], a * d_node);
        Ok(())
    }

    // ∂(a / b)/∂a = 1/b, ∂(a / b)/∂b = -a/b²
    fn backward_div(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.len() < 2 {
            return Err("Div requires 2 inputs".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let b = ctx.get_value(node.inputs[1]);
        ctx.add_gradient(node.inputs[0], d_node / b);
        ctx.add_gradient(node.inputs[1], -a * d_node / (b * b));
        Ok(())
    }

    // ∂(-a)/∂a = -1
    fn backward_neg(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Neg requires 1 input".to_string());
        }
        ctx.add_gradient(node.inputs[0], -d_node);
        Ok(())
    }

    // ∂(a^b)/∂a = b * a^(b-1), ∂(a^b)/∂b = a^b * ln(a)
    fn backward_pow(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.len() < 2 {
            return Err("Pow requires 2 inputs".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let b = ctx.get_value(node.inputs[1]);
        ctx.add_gradient(node.inputs[0], b * a.powf(b - 1.0) * d_node);
        ctx.add_gradient(node.inputs[1], a.powf(b) * a.ln() * d_node);
        Ok(())
    }

    // ∂(sin(a))/∂a = cos(a)
    fn backward_sin(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Sin requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        ctx.add_gradient(node.inputs[0], a.cos() * d_node);
        Ok(())
    }

    // ∂(cos(a))/∂a = -sin(a)
    fn backward_cos(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Cos requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        ctx.add_gradient(node.inputs[0], -a.sin() * d_node);
        Ok(())
    }

    // ∂(tan(a))/∂a = sec²(a) = 1/cos²(a)
    fn backward_tan(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Tan requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let cos_a = a.cos();
        ctx.add_gradient(node.inputs[0], d_node / (cos_a * cos_a));
        Ok(())
    }

    // ∂(exp(a))/∂a = exp(a)
    fn backward_exp(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Exp requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        ctx.add_gradient(node.inputs[0], a.exp() * d_node);
        Ok(())
    }

    // ∂(ln(a))/∂a = 1/a
    fn backward_log(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Log requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        ctx.add_gradient(node.inputs[0], d_node / a);
        Ok(())
    }

    // ∂(√a)/∂a = 1/(2√a)
    fn backward_sqrt(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Sqrt requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        ctx.add_gradient(node.inputs[0], d_node / (2.0 * a.sqrt()));
        Ok(())
    }

    // ∂(ReLU(a))/∂a = 1 if a > 0 else 0
    fn backward_relu(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("ReLU requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let grad = if a > 0.0 { d_node } else { 0.0 };
        ctx.add_gradient(node.inputs[0], grad);
        Ok(())
    }

    // ∂(sigmoid(a))/∂a = sigmoid(a) * (1 - sigmoid(a))
    fn backward_sigmoid(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Sigmoid requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let sigmoid = 1.0 / (1.0 + (-a).exp());
        ctx.add_gradient(node.inputs[0], sigmoid * (1.0 - sigmoid) * d_node);
        Ok(())
    }

    // ∂(tanh(a))/∂a = 1 - tanh²(a)
    fn backward_tanh(
        ctx: &mut BackwardContext,
        _node_id: NodeId,
        node: &crate::graph::Node,
        d_node: f64,
    ) -> Result<(), String> {
        if node.inputs.is_empty() {
            return Err("Tanh requires 1 input".to_string());
        }
        let a = ctx.get_value(node.inputs[0]);
        let tanh_a = a.tanh();
        ctx.add_gradient(node.inputs[0], (1.0 - tanh_a * tanh_a) * d_node);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ComputationGraph;

    fn setup_simple_graph() -> (ComputationGraph, NodeId, NodeId, NodeId) {
        let mut graph = ComputationGraph::new();
        let x = graph.create_node(OpType::Parameter, 1, 1);
        let y = graph.create_node(OpType::Parameter, 1, 1);
        let z = graph.create_node(OpType::Add, 1, 1);
        graph.add_input(z, x).unwrap();
        graph.add_input(z, y).unwrap();
        let _ = graph.compute_reverse_order(z);
        (graph, x, y, z)
    }

    #[test]
    fn gradient_accumulator_basic() {
        let mut acc = GradientAccumulator::new();
        let node = NodeId::new(0);
        acc.add_gradient(node, 1.5);
        acc.add_gradient(node, 0.5);
        assert_eq!(acc.get_gradient(node), 2.0);
    }

    #[test]
    fn gradient_accumulator_multiple_nodes() {
        let mut acc = GradientAccumulator::new();
        let n1 = NodeId::new(0);
        let n2 = NodeId::new(1);
        acc.add_gradient(n1, 1.0);
        acc.add_gradient(n2, 2.0);
        acc.add_gradient(n1, -0.5);
        assert_eq!(acc.get_gradient(n1), 0.5);
        assert_eq!(acc.get_gradient(n2), 2.0);
    }

    #[test]
    fn backward_add_gradient() {
        let (graph, x, y, z) = setup_simple_graph();
        let mut values = HashMap::new();
        values.insert(x, 2.0);
        values.insert(y, 3.0);
        values.insert(z, 5.0);

        let gradients = DifferentiationEngine::backward(&graph, z, 1.0, &values).unwrap();
        assert!((gradients.get(&x).copied().unwrap_or(0.0) - 1.0).abs() < 1e-6);
        assert!((gradients.get(&y).copied().unwrap_or(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn backward_mul_gradient() {
        let mut graph = ComputationGraph::new();
        let x = graph.create_node(OpType::Parameter, 1, 1);
        let y = graph.create_node(OpType::Parameter, 1, 1);
        let z = graph.create_node(OpType::Mul, 1, 1);
        graph.add_input(z, x).unwrap();
        graph.add_input(z, y).unwrap();
        let _ = graph.compute_reverse_order(z);

        let mut values = HashMap::new();
        values.insert(x, 2.0);
        values.insert(y, 3.0);
        values.insert(z, 6.0);

        let gradients = DifferentiationEngine::backward(&graph, z, 1.0, &values).unwrap();
        assert!((gradients.get(&x).copied().unwrap_or(0.0) - 3.0).abs() < 1e-6);
        assert!((gradients.get(&y).copied().unwrap_or(0.0) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn backward_sin_gradient() {
        let mut graph = ComputationGraph::new();
        let x = graph.create_node(OpType::Parameter, 1, 1);
        let y = graph.create_node(OpType::Sin, 1, 1);
        graph.add_input(y, x).unwrap();
        let _ = graph.compute_reverse_order(y);

        let mut values = HashMap::new();
        values.insert(x, 0.0); // sin(0) = 0, cos(0) = 1
        values.insert(y, 0.0);

        let gradients = DifferentiationEngine::backward(&graph, y, 1.0, &values).unwrap();
        assert!((gradients.get(&x).copied().unwrap_or(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn backward_relu_gradient() {
        let mut graph = ComputationGraph::new();
        let x = graph.create_node(OpType::Parameter, 1, 1);
        let y = graph.create_node(OpType::ReLU, 1, 1);
        graph.add_input(y, x).unwrap();
        let _ = graph.compute_reverse_order(y);

        let mut values = HashMap::new();
        values.insert(x, 2.0);
        values.insert(y, 2.0);

        let gradients = DifferentiationEngine::backward(&graph, y, 1.0, &values).unwrap();
        assert!((gradients.get(&x).copied().unwrap_or(0.0) - 1.0).abs() < 1e-6);
    }
}

