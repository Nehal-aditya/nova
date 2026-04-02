pub mod graph;
pub mod gradient;

pub use graph::{ComputationGraph, NodeId, OpType, Node};
pub use gradient::{GradientAccumulator, DifferentiationEngine};

use std::collections::HashMap;

/// Main interface for automatic differentiation.
pub struct AutoDiffEngine {
    graph: ComputationGraph,
    values: HashMap<NodeId, f64>,
    gradients: GradientAccumulator,
}

impl AutoDiffEngine {
    pub fn new() -> Self {
        AutoDiffEngine {
            graph: ComputationGraph::new(),
            values: HashMap::new(),
            gradients: GradientAccumulator::new(),
        }
    }

    /// Get mutable reference to the computation graph.
    pub fn graph_mut(&mut self) -> &mut ComputationGraph {
        &mut self.graph
    }

    /// Get immutable reference to the computation graph.
    pub fn graph(&self) -> &ComputationGraph {
        &self.graph
    }

    /// Store a forward-pass value for a node.
    pub fn set_value(&mut self, node_id: NodeId, value: f64) {
        self.values.insert(node_id, value);
    }

    /// Get a forward-pass value.
    pub fn get_value(&self, node_id: NodeId) -> Option<f64> {
        self.values.get(&node_id).copied()
    }

    /// Perform backward pass from output node.
    pub fn backward(&mut self, output_node: NodeId, d_output: f64) -> Result<(), String> {
        // Ensure topological order is computed
        self.graph.compute_reverse_order(output_node)?;

        // Run differentiation engine
        let result_gradients = DifferentiationEngine::backward(&self.graph, output_node, d_output, &self.values)?;

        // Store gradients
        for (node_id, grad) in result_gradients {
            self.gradients.add_gradient(node_id, grad);
        }

        Ok(())
    }

    /// Get gradient for a node.
    pub fn get_gradient(&self, node_id: NodeId) -> f64 {
        self.gradients.get_gradient(node_id)
    }

    /// Get all gradients.
    pub fn get_all_gradients(&self) -> &HashMap<NodeId, f64> {
        self.gradients.get_all()
    }

    /// Clear all gradients.
    pub fn clear_gradients(&mut self) {
        self.gradients.clear();
    }

    /// Reset the entire engine.
    pub fn reset(&mut self) {
        self.graph.clear();
        self.values.clear();
        self.gradients.clear();
    }
}

impl Default for AutoDiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_creation() {
        let engine = AutoDiffEngine::new();
        assert_eq!(engine.graph().node_count(), 0);
    }

    #[test]
    fn create_and_store_value() {
        let mut engine = AutoDiffEngine::new();
        let node = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        engine.set_value(node, 3.14);
        assert!((engine.get_value(node).unwrap() - 3.14).abs() < 1e-6);
    }

    #[test]
    fn graph_operations() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let z = engine.graph_mut().create_node(OpType::Add, 1, 1);
        
        engine.graph_mut().add_input(z, x).unwrap();
        engine.graph_mut().add_input(z, y).unwrap();
        
        assert_eq!(engine.graph().node_count(), 3);
    }

    #[test]
    fn backward_simple() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let z = engine.graph_mut().create_node(OpType::Add, 1, 1);
        
        engine.graph_mut().add_input(z, x).unwrap();
        engine.graph_mut().add_input(z, y).unwrap();
        
        engine.set_value(x, 2.0);
        engine.set_value(y, 3.0);
        engine.set_value(z, 5.0);
        
        engine.backward(z, 1.0).expect("backward");
        
        assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
        assert!((engine.get_gradient(y) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn backward_with_multiplication() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let z = engine.graph_mut().create_node(OpType::Mul, 1, 1);
        
        engine.graph_mut().add_input(z, x).unwrap();
        engine.graph_mut().add_input(z, y).unwrap();
        
        engine.set_value(x, 2.0);
        engine.set_value(y, 3.0);
        engine.set_value(z, 6.0);
        
        engine.backward(z, 1.0).expect("backward");
        
        // ∂(x*y)/∂x = y = 3.0
        // ∂(x*y)/∂y = x = 2.0
        assert!((engine.get_gradient(x) - 3.0).abs() < 1e-6);
        assert!((engine.get_gradient(y) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn reset_engine() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        engine.set_value(x, 5.0);
        
        engine.reset();
        
        assert_eq!(engine.graph().node_count(), 0);
        assert_eq!(engine.get_value(x), None);
    }

    #[test]
    fn clear_gradients() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let x2 = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::Add, 1, 1);
        
        engine.graph_mut().add_input(y, x).unwrap();
        engine.graph_mut().add_input(y, x2).unwrap();
        
        engine.set_value(x, 2.0);
        engine.set_value(x2, 3.0);
        engine.set_value(y, 5.0);
        
        engine.backward(y, 1.0).expect("backward");
        assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
        
        engine.clear_gradients();
        assert_eq!(engine.get_gradient(x), 0.0);
    }

    #[test]
    fn backward_with_transcendental() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::Sin, 1, 1);
        
        engine.graph_mut().add_input(y, x).unwrap();
        
        engine.set_value(x, 0.0);
        engine.set_value(y, 0.0);
        
        engine.backward(y, 1.0).expect("backward");
        
        // ∂(sin(x))/∂x at x=0 = cos(0) = 1.0
        assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn backward_with_relu() {
        let mut engine = AutoDiffEngine::new();
        let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
        let y = engine.graph_mut().create_node(OpType::ReLU, 1, 1);
        
        engine.graph_mut().add_input(y, x).unwrap();
        
        // Test positive case
        engine.set_value(x, 3.0);
        engine.set_value(y, 3.0);
        
        engine.backward(y, 1.0).expect("backward");
        assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
        
        // Test negative case
        engine.clear_gradients();
        engine.set_value(x, -2.0);
        engine.set_value(y, 0.0);
        
        engine.backward(y, 1.0).expect("backward");
        assert_eq!(engine.get_gradient(x), 0.0);
    }
}
