//! Computation graph for automatic differentiation.
//!
//! A computation graph is a DAG (directed acyclic graph) where:
//!   - Nodes represent operations (add, mul, sin, etc.)
//!   - Edges represent data flow (tensor through operations)
//!   - Each node has an associated type and shape
//!
//! The graph is built during forward pass and then traversed in topological order
//! during the backward pass to compute gradients.

use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a node in the computation graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        NodeId(id)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// Types of operations in the computation graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpType {
    // Literals and constants
    Parameter,
    Constant,
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Pow,
    
    // Transcendental
    Sin,
    Cos,
    Tan,
    Exp,
    Log,
    Sqrt,
    
    // Linear algebra
    Matmul,
    Transpose,
    Reshape,
    
    // Neural network
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
    
    // Loss
    MSE,
    CrossEntropy,
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Parameter => write!(f, "param"),
            OpType::Constant => write!(f, "const"),
            OpType::Add => write!(f, "add"),
            OpType::Sub => write!(f, "sub"),
            OpType::Mul => write!(f, "mul"),
            OpType::Div => write!(f, "div"),
            OpType::Neg => write!(f, "neg"),
            OpType::Pow => write!(f, "pow"),
            OpType::Sin => write!(f, "sin"),
            OpType::Cos => write!(f, "cos"),
            OpType::Tan => write!(f, "tan"),
            OpType::Exp => write!(f, "exp"),
            OpType::Log => write!(f, "log"),
            OpType::Sqrt => write!(f, "sqrt"),
            OpType::Matmul => write!(f, "matmul"),
            OpType::Transpose => write!(f, "transpose"),
            OpType::Reshape => write!(f, "reshape"),
            OpType::ReLU => write!(f, "relu"),
            OpType::Sigmoid => write!(f, "sigmoid"),
            OpType::Tanh => write!(f, "tanh"),
            OpType::Softmax => write!(f, "softmax"),
            OpType::MSE => write!(f, "mse"),
            OpType::CrossEntropy => write!(f, "crossentropy"),
        }
    }
}

/// A node in the computation graph.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub op_type: OpType,
    pub inputs: Vec<NodeId>,
    pub output_shape: String,  // e.g., "(batch, features)"
    pub output_type: String,   // e.g., "Float[eV]"
    pub line: u32,
    pub col: u32,
}

impl Node {
    pub fn new(id: NodeId, op_type: OpType, line: u32, col: u32) -> Self {
        Node {
            id,
            op_type,
            inputs: Vec::new(),
            output_shape: String::new(),
            output_type: String::new(),
            line,
            col,
        }
    }

    pub fn with_inputs(mut self, inputs: Vec<NodeId>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_output(mut self, shape: impl Into<String>, ty: impl Into<String>) -> Self {
        self.output_shape = shape.into();
        self.output_type = ty.into();
        self
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.op_type, OpType::Parameter | OpType::Constant)
    }
}

/// Represents the computational flow of a tensor through the computation graph.
#[derive(Debug, Clone)]
pub struct Gradient {
    pub node_id: NodeId,
    pub value: f64,  // Current accumulated gradient
    pub shape: String,
}

/// The computation graph for reverse-mode AD.
#[derive(Debug)]
pub struct ComputationGraph {
    next_id: u32,
    nodes: HashMap<NodeId, Node>,
    reverse_order: Vec<NodeId>,  // Nodes in reverse topological order
}

impl ComputationGraph {
    pub fn new() -> Self {
        ComputationGraph {
            next_id: 0,
            nodes: HashMap::new(),
            reverse_order: Vec::new(),
        }
    }

    /// Create a new node in the graph and return its ID.
    pub fn create_node(&mut self, op_type: OpType, line: u32, col: u32) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        let node = Node::new(id, op_type, line, col);
        self.nodes.insert(id, node);
        id
    }

    /// Add an input to a node.
    pub fn add_input(&mut self, node_id: NodeId, input_id: NodeId) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.inputs.push(input_id);
            Ok(())
        } else {
            Err(format!("Node {} not found", node_id))
        }
    }

    /// Set the output type and shape for a node.
    pub fn set_output(&mut self, node_id: NodeId, shape: impl Into<String>, ty: impl Into<String>) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.output_shape = shape.into();
            node.output_type = ty.into();
            Ok(())
        } else {
            Err(format!("Node {} not found", node_id))
        }
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Compute topological order (for backward pass).
    pub fn compute_reverse_order(&mut self, output_node: NodeId) -> Result<(), String> {
        self.reverse_order.clear();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        let mut order = Vec::new();

        self.topological_sort_dfs(output_node, &mut visited, &mut visiting, &mut order)?;
        order.reverse();
        self.reverse_order = order;
        Ok(())
    }

    fn topological_sort_dfs(
        &self,
        node_id: NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        visiting: &mut std::collections::HashSet<NodeId>,
        order: &mut Vec<NodeId>,
    ) -> Result<(), String> {
        if visited.contains(&node_id) {
            return Ok(());
        }
        if visiting.contains(&node_id) {
            return Err("Cycle detected in computation graph".to_string());
        }

        visiting.insert(node_id);

        if let Some(node) = self.get_node(node_id) {
            for &input_id in &node.inputs {
                self.topological_sort_dfs(input_id, visited, visiting, order)?;
            }
        }

        visiting.remove(&node_id);
        visited.insert(node_id);
        order.push(node_id);
        Ok(())
    }

    /// Get nodes in reverse topological order.
    pub fn get_reverse_order(&self) -> &[NodeId] {
        &self.reverse_order
    }

    /// Get all nodes.
    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a node exists.
    pub fn has_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Clear the graph.
    pub fn clear(&mut self) {
        self.next_id = 0;
        self.nodes.clear();
        self.reverse_order.clear();
    }
}

impl Default for ComputationGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_node() {
        let mut graph = ComputationGraph::new();
        let id = graph.create_node(OpType::Parameter, 1, 1);
        assert_eq!(id, NodeId::new(0));
        assert!(graph.has_node(id));
    }

    #[test]
    fn create_multiple_nodes() {
        let mut graph = ComputationGraph::new();
        let n1 = graph.create_node(OpType::Parameter, 1, 1);
        let n2 = graph.create_node(OpType::Constant, 2, 1);
        let n3 = graph.create_node(OpType::Add, 3, 1);
        assert_eq!(graph.node_count(), 3);
        assert_ne!(n1, n2);
        assert_ne!(n2, n3);
    }

    #[test]
    fn add_inputs() {
        let mut graph = ComputationGraph::new();
        let n1 = graph.create_node(OpType::Parameter, 1, 1);
        let n2 = graph.create_node(OpType::Parameter, 1, 1);
        let n3 = graph.create_node(OpType::Add, 1, 1);
        graph.add_input(n3, n1).expect("add input");
        graph.add_input(n3, n2).expect("add input");
        let node = graph.get_node(n3).expect("node");
        assert_eq!(node.inputs.len(), 2);
    }

    #[test]
    fn set_output() {
        let mut graph = ComputationGraph::new();
        let n = graph.create_node(OpType::Add, 1, 1);
        graph.set_output(n, "(batch, features)", "Float").expect("set output");
        let node = graph.get_node(n).expect("node");
        assert_eq!(node.output_shape, "(batch, features)");
        assert_eq!(node.output_type, "Float");
    }

    #[test]
    fn is_leaf() {
        let mut graph = ComputationGraph::new();
        let param = graph.create_node(OpType::Parameter, 1, 1);
        let const_node = graph.create_node(OpType::Constant, 1, 1);
        let add_node = graph.create_node(OpType::Add, 1, 1);
        
        assert!(graph.get_node(param).unwrap().is_leaf());
        assert!(graph.get_node(const_node).unwrap().is_leaf());
        assert!(!graph.get_node(add_node).unwrap().is_leaf());
    }

    #[test]
    fn simple_topological_sort() {
        let mut graph = ComputationGraph::new();
        let n1 = graph.create_node(OpType::Parameter, 1, 1);
        let n2 = graph.create_node(OpType::Parameter, 1, 1);
        let n3 = graph.create_node(OpType::Add, 1, 1);
        graph.add_input(n3, n1).expect("add input");
        graph.add_input(n3, n2).expect("add input");
        
        graph.compute_reverse_order(n3).expect("compute order");
        let order = graph.get_reverse_order();
        assert_eq!(order.len(), 3);
        // For backward pass, output node comes first
        assert_eq!(order[0], n3); // Output node first in reverse topological order
        // n1 and n2 should come after (order doesn't matter between them)
        assert!(order[1] == n1 || order[1] == n2);
        assert!(order[2] == n1 || order[2] == n2);
    }
}

