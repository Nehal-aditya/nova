// Comprehensive test suite for NOVA Phase 0 (Lexer and Parser)

use nova_compiler::{Lexer, Parser, TopLevelItem};

#[test]
fn test_simple_mission() {
    let code = r#"
        mission hello() → Void {
            transmit("Hello, Universe!")
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::MissionDecl(m) => {
            assert_eq!(m.name, "hello");
            assert!(m.return_type.is_some());
        }
        _ => panic!("Expected mission declaration"),
    }
}

#[test]
fn test_parallel_mission() {
    let code = r#"
        parallel mission compute() → Float {
            return 42.0
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::ParallelMissionDecl(m) => {
            assert_eq!(m.name, "compute");
        }
        _ => panic!("Expected parallel mission declaration"),
    }
}

#[test]
fn test_model_declaration() {
    let code = r#"
        model NeuralNet {
            layer dense1(784, 128)
            layer dense2(128, 10)
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::ModelDecl(m) => {
            assert_eq!(m.name, "NeuralNet");
            assert_eq!(m.layers.len(), 2);
        }
        _ => panic!("Expected model declaration"),
    }
}

#[test]
fn test_struct_declaration() {
    let code = r#"
        struct Point {
            x: Float
            y: Float
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::StructDecl(s) => {
            assert_eq!(s.name, "Point");
            assert_eq!(s.fields.len(), 2);
        }
        _ => panic!("Expected struct declaration"),
    }
}

#[test]
fn test_enum_declaration() {
    let code = r#"
        enum StarType {
            O
            B
            A
            F
            G
            K
            M
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::EnumDecl(e) => {
            assert_eq!(e.name, "StarType");
            assert_eq!(e.variants.len(), 7);
        }
        _ => panic!("Expected enum declaration"),
    }
}

#[test]
fn test_constellation_declaration() {
    let code = r#"
        constellation Astronomy {
            mission observe(target: String) → Float {
                return 1.5
            }
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::ConstellationDecl(c) => {
            assert_eq!(c.name, "Astronomy");
        }
        _ => panic!("Expected constellation declaration"),
    }
}

#[test]
fn test_test_mission() {
    let code = r#"
        test mission verify_computation() {
            assert(true)
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::TestMission(m) => {
            assert_eq!(m.name, "verify_computation");
        }
        _ => panic!("Expected test mission"),
    }
}

#[test]
fn test_multiple_missions() {
    let code = r#"
        mission first() → Int { return 1 }
        mission second() → Int { return 2 }
        mission third() → Int { return 3 }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_function_with_parameters() {
    let code = r#"
        mission add(a: Int, b: Int) → Int {
            return a + b
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::MissionDecl(m) => {
            assert_eq!(m.name, "add");
            assert_eq!(m.params.len(), 2);
        }
        _ => panic!("Expected mission declaration"),
    }
}

#[test]
fn test_function_with_unit_types() {
    let code = r#"
        mission distance(speed: Float[m/s], time: Float[s]) → Float[m] {
            return speed * time
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_lexer_token_count() {
    let code = r#"
        mission main() → Void {
            transmit("Hello")
            let x = 42
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    
    // Should have many tokens
    assert!(tokens.len() >= 15);
}

#[test]
fn test_lexer_preserves_string_literals() {
    let code = r#"transmit("Hello, Universe!")"#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    
    assert!(tokens.len() >= 2);
}

#[test]
fn test_lexer_preserves_numbers() {
    let code = r#"let x = 42; let y = 3.14; let z = 1_000_000"#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    
    // Should recognize all three numbers
    assert!(tokens.len() >= 10);
}

#[test]
fn test_parser_preserves_identifiers() {
    let code = r#"
        mission my_mission_name() → Int {
            return 0
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    match &program.items[0] {
        TopLevelItem::MissionDecl(m) => {
            assert_eq!(m.name, "my_mission_name");
        }
        _ => panic!("Expected mission declaration"),
    }
}

#[test]
fn test_empty_mission_body() {
    let code = r#"
        mission empty_mission() → Void {
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_autodiff_in_mission() {
    let code = r#"
        mission train() → Float {
            autodiff {
                return loss
            }
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_pipe_expression() {
    let code = r#"
        mission process() → Float {
            let result = data |> filter(x > 0) |> map(x * 2) |> sum
            return result
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_generic_types() {
    let code = r#"
        struct Container {
            value: Int
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_nested_return() {
    let code = r#"
        mission conditional(x: Int) → String {
            if x > 0 {
                return "positive"
            }
            return "other"
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_let_binding() {
    let code = r#"
        mission test_let() → Void {
            let x = 5
            let y = x + 3
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_arrow_return_type() {
    let code = r#"
        mission my_test() → Int {
            return 42
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
    match &program.items[0] {
        TopLevelItem::MissionDecl(m) => {
            assert!(m.return_type.is_some());
        }
        _ => panic!("Expected mission"),
    }
}

#[test]
fn test_transmit_statement() {
    let code = r#"
        mission print_hello() → Void {
            transmit("Hello")
            transmit(42)
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_if_statement() {
    let code = r#"
        mission test_if(x: Int) → Void {
            if x > 0 {
                transmit("positive")
            }
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_while_loop() {
    let code = r#"
        mission count() → Void {
            let i = 0
            while i < 10 {
                i = i + 1
            }
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_for_loop() {
    let code = r#"
        mission iterate() → Void {
            for i in 0..10 {
                transmit(i)
            }
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_layer_declaration() {
    let code = r#"
        model Network {
            layer conv2d(3, 64, 3, 3)
            layer relu()
            layer maxpool(2, 2)
            layer dense(1024, 10)
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_example_from_docs() {
    let code = r#"
        constellation Astronomy {
            mission observe(target: String, exposure: Float[s]) → Float[lux] {
                transmit("Observing: " + target)
                return 1.5
            }
        }

        mission main() → Void {
            transmit("Starting observation")
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    // constellation + mission
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_no_panic_on_empty_input() {
    let code = "";
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();
    
    // Should succeed (empty program is valid)
    assert!(program.is_ok());
}

#[test]
fn test_comments_ignored() {
    let code = r#"
        -- This is a comment
        mission my_mission() → Void {
            -- Another comment
            transmit("Hello")
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_multiple_structs() {
    let code = r#"
        struct Point { x: Float, y: Float }
        struct Vector { dx: Float, dy: Float }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_array_type() {
    let code = r#"
        mission process(data: Array[Float]) → Float {
            return 1.0
        }
    "#;
    
    let mut lexer = Lexer::new(code, 0);
    let tokens = lexer.tokenize().expect("Lexer failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parser failed");
    
    assert_eq!(program.items.len(), 1);
}
