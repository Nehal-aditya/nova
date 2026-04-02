use nova_autodiff::{AutoDiffEngine, ComputationGraph, NodeId, OpType};

/// Test simple gradient computation: z = x + y
#[test]
fn test_simple_addition() {
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

/// Test quadratic: z = x * x
#[test]
fn test_square() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Mul, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, x).unwrap(); // x * x

    engine.set_value(x, 3.0);
    engine.set_value(z, 9.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(x²)/∂x = 2x = 6 at x=3
    assert!((engine.get_gradient(x) - 6.0).abs() < 1e-6);
}

/// Test chain rule: z = sin(x)
#[test]
fn test_sin() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Sin, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 0.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(sin(x))/∂x at x=0 = cos(0) = 1.0
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test chain rule: z = cos(x)
#[test]
fn test_cos() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Cos, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 1.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(cos(x))/∂x at x=0 = -sin(0) = 0.0
    assert!((engine.get_gradient(x) - 0.0).abs() < 1e-6);
}

/// Test exponential: z = exp(x)
#[test]
fn test_exp() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Exp, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 1.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(exp(x))/∂x at x=0 = exp(0) = 1.0
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test logarithm: z = log(x)
#[test]
fn test_log() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Log, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 2.0);
    engine.set_value(z, 2.0_f64.ln());

    engine.backward(z, 1.0).expect("backward");

    // ∂(ln(x))/∂x at x=2 = 1/2 = 0.5
    assert!((engine.get_gradient(x) - 0.5).abs() < 1e-6);
}

/// Test square root: z = sqrt(x)
#[test]
fn test_sqrt() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Sqrt, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 4.0);
    engine.set_value(z, 2.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(√x)/∂x at x=4 = 1/(2*√4) = 0.25
    assert!((engine.get_gradient(x) - 0.25).abs() < 1e-6);
}

/// Test negation: z = -x
#[test]
fn test_negation() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Neg, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 5.0);
    engine.set_value(z, -5.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(-x)/∂x = -1
    assert!((engine.get_gradient(x) - (-1.0)).abs() < 1e-6);
}

/// Test subtraction: z = x - y
#[test]
fn test_subtraction() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Sub, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, y).unwrap();

    engine.set_value(x, 5.0);
    engine.set_value(y, 3.0);
    engine.set_value(z, 2.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(x - y)/∂x = 1
    // ∂(x - y)/∂y = -1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
    assert!((engine.get_gradient(y) - (-1.0)).abs() < 1e-6);
}

/// Test division: z = x / y
#[test]
fn test_division() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Div, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, y).unwrap();

    engine.set_value(x, 6.0);
    engine.set_value(y, 2.0);
    engine.set_value(z, 3.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(x / y)/∂x = 1/y = 0.5
    // ∂(x / y)/∂y = -x/y² = -1.5
    assert!((engine.get_gradient(x) - 0.5).abs() < 1e-6);
    assert!((engine.get_gradient(y) - (-1.5)).abs() < 1e-6);
}

/// Test power: z = x^y
#[test]
fn test_power() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let y = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Pow, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, y).unwrap();

    engine.set_value(x, 2.0);
    engine.set_value(y, 3.0);
    engine.set_value(z, 8.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(x^y)/∂x at x=2, y=3 = y * x^(y-1) = 3 * 4 = 12
    // ∂(x^y)/∂y at x=2, y=3 = x^y * ln(x) = 8 * ln(2) ≈ 5.545
    assert!((engine.get_gradient(x) - 12.0).abs() < 1e-6);
    assert!((engine.get_gradient(y) - (8.0 * 2.0_f64.ln())).abs() < 1e-5);
}

/// Test ReLU activation: z = max(0, x)
#[test]
fn test_relu_positive() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::ReLU, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 3.0);
    engine.set_value(z, 3.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂ReLU(x)/∂x at x=3 = 1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test ReLU with negative input
#[test]
fn test_relu_negative() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::ReLU, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, -2.0);
    engine.set_value(z, 0.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂ReLU(x)/∂x at x=-2 = 0
    assert_eq!(engine.get_gradient(x), 0.0);
}

/// Test sigmoid activation
#[test]
fn test_sigmoid() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Sigmoid, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 0.5);

    engine.backward(z, 1.0).expect("backward");

    // ∂sigmoid(x)/∂x at x=0 = sigmoid(0) * (1 - sigmoid(0)) = 0.5 * 0.5 = 0.25
    assert!((engine.get_gradient(x) - 0.25).abs() < 1e-6);
}

/// Test tanh activation
#[test]
fn test_tanh() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Tanh, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 0.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂tanh(x)/∂x at x=0 = 1 - tanh²(0) = 1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test composite function: z = sin(x) + cos(x)
#[test]
fn test_composite_trig() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let sin_x = engine.graph_mut().create_node(OpType::Sin, 1, 1);
    let cos_x = engine.graph_mut().create_node(OpType::Cos, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Add, 1, 1);

    engine.graph_mut().add_input(sin_x, x).unwrap();
    engine.graph_mut().add_input(cos_x, x).unwrap();
    engine.graph_mut().add_input(z, sin_x).unwrap();
    engine.graph_mut().add_input(z, cos_x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(sin_x, 0.0);
    engine.set_value(cos_x, 1.0);
    engine.set_value(z, 1.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂(sin(x) + cos(x))/∂x at x=0 = cos(0) - sin(0) = 1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test deep network: z = sqrt(exp(x * 2))
#[test]
fn test_deep_network() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let two = engine.graph_mut().create_node(OpType::Constant, 1, 1);
    let x_times_2 = engine.graph_mut().create_node(OpType::Mul, 1, 1);
    let exp_result = engine.graph_mut().create_node(OpType::Exp, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Sqrt, 1, 1);

    engine.graph_mut().add_input(x_times_2, x).unwrap();
    engine.graph_mut().add_input(x_times_2, two).unwrap();
    engine.graph_mut().add_input(exp_result, x_times_2).unwrap();
    engine.graph_mut().add_input(z, exp_result).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(two, 2.0);
    engine.set_value(x_times_2, 0.0);
    engine.set_value(exp_result, 1.0);
    engine.set_value(z, 1.0);

    engine.backward(z, 1.0).expect("backward");

    // Chain rule: ∂z/∂x = (∂z/∂sqrt) * (∂sqrt/∂exp) * (∂exp/∂mul) * (∂mul/∂x)
    // = (1/(2*1)) * (1*1) * (1*2) = 1/2 * 1 * 2 = 1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-5);
}

/// Test multiple gradient flows (fan-out)
#[test]
fn test_fan_out() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let y = engine.graph_mut().create_node(OpType::Mul, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Mul, 1, 1);
    let out = engine.graph_mut().create_node(OpType::Add, 1, 1);

    // y = x * x, z = x * x
    engine.graph_mut().add_input(y, x).unwrap();
    engine.graph_mut().add_input(y, x).unwrap();
    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, x).unwrap();
    // out = y + z = 2x²
    engine.graph_mut().add_input(out, y).unwrap();
    engine.graph_mut().add_input(out, z).unwrap();

    engine.set_value(x, 2.0);
    engine.set_value(y, 4.0);
    engine.set_value(z, 4.0);
    engine.set_value(out, 8.0);

    engine.backward(out, 1.0).expect("backward");

    // ∂(2x²)/∂x at x=2 = 4x = 8
    assert!((engine.get_gradient(x) - 8.0).abs() < 1e-6);
}

/// Test tan function
#[test]
fn test_tan() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Tan, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 0.0);
    engine.set_value(z, 0.0);

    engine.backward(z, 1.0).expect("backward");

    // ∂tan(x)/∂x at x=0 = sec²(0) = 1/cos²(0) = 1
    assert!((engine.get_gradient(x) - 1.0).abs() < 1e-6);
}

/// Test gradient accumulation with higher-order derivatives scaling
#[test]
fn test_gradient_scaling() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    let z = engine.graph_mut().create_node(OpType::Add, 1, 1);

    engine.graph_mut().add_input(z, x).unwrap();
    engine.graph_mut().add_input(z, x).unwrap();

    engine.set_value(x, 1.0);
    engine.set_value(z, 2.0);

    // Backward with scaling factor 2.0
    engine.backward(z, 2.0).expect("backward");

    // ∂(2x)/∂x * 2.0 = 2 * 2 = 4
    assert!((engine.get_gradient(x) - 4.0).abs() < 1e-6);
}

/// Test reset functionality
#[test]
fn test_engine_reset() {
    let mut engine = AutoDiffEngine::new();
    let x = engine.graph_mut().create_node(OpType::Parameter, 1, 1);
    engine.set_value(x, 5.0);

    assert!(engine.graph().has_node(x));
    assert_eq!(engine.get_value(x), Some(5.0));

    engine.reset();

    assert!(!engine.graph().has_node(x));
    assert_eq!(engine.get_value(x), None);
}
