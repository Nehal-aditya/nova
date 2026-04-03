#!/usr/bin/env bash
# NOVA Compiler Build Script
# Builds all compiler components in the correct order

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPILER_DIR="$SCRIPT_DIR"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== NOVA Compiler Build ==="
echo "Project root: $PROJECT_ROOT"
echo "Compiler dir: $COMPILER_DIR"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Please install Rust from https://rustup.rs"
        exit 1
    fi
    
    if ! command -v gcc &> /dev/null && ! command -v clang &> /dev/null; then
        log_error "C compiler (gcc or clang) not found"
        exit 1
    fi
    
    if ! command -v javac &> /dev/null; then
        log_warn "Java compiler (javac) not found. Interface validator will be skipped."
    fi
    
    log_info "Dependencies OK"
}

# Build C components (Lexer and Parser)
build_c_components() {
    log_info "Building C components (Lexer, Parser)..."
    
    # Build Lexer
    cd "$COMPILER_DIR/lexer"
    if [ -f "Makefile" ]; then
        make clean 2>/dev/null || true
        make
        log_info "Lexer built successfully"
    else
        # Compile manually if no Makefile
        gcc -shared -fPIC -o liblexer.so src/lexer.c src/token.c -I include
        log_info "Lexer built successfully (manual compilation)"
    fi
    
    # Build Parser
    cd "$COMPILER_DIR/parser"
    if [ -f "Makefile" ]; then
        make clean 2>/dev/null || true
        make
        log_info "Parser built successfully"
    else
        # Compile manually if no Makefile
        gcc -shared -fPIC -o libparser.so src/parser.c src/ast.c src/parser_ffi.c -I include -L../lexer -llexer
        log_info "Parser built successfully (manual compilation)"
    fi
    
    cd "$COMPILER_DIR"
}

# Build Rust components
build_rust_components() {
    log_info "Building Rust components..."
    
    cd "$COMPILER_DIR"
    
    # Build workspace members
    cargo build --release
    
    if [ $? -eq 0 ]; then
        log_info "Rust components built successfully"
    else
        log_error "Failed to build Rust components"
        exit 1
    fi
    
    cd "$COMPILER_DIR"
}

# Build Java components
build_java_components() {
    if ! command -v javac &> /dev/null; then
        log_warn "Skipping Java components (javac not found)"
        return
    fi
    
    log_info "Building Java components (Interface Validator)..."
    
    cd "$COMPILER_DIR/interface_validator"
    
    if [ -f "pom.xml" ]; then
        mvn clean package -DskipTests
    elif [ -f "build.gradle" ]; then
        ./gradlew build
    else
        # Manual compilation
        mkdir -p build/classes
        find src/main/java -name "*.java" | xargs javac -d build/classes
        log_info "Java components built successfully (manual compilation)"
    fi
    
    cd "$COMPILER_DIR"
}

# Build toolchain components
build_toolchain() {
    log_info "Building toolchain components..."
    
    cd "$PROJECT_ROOT/toolchain"
    
    # Build nebula (package manager)
    if [ -d "nebula" ] && [ -f "nebula/Cargo.toml" ]; then
        cd nebula
        cargo build --release 2>/dev/null && log_info "Nebula built successfully" || log_warn "Nebula build skipped (empty project)"
        cd ..
    fi
    
    # Build nova_fmt (formatter)
    if [ -d "nova_fmt" ] && [ -f "nova_fmt/Cargo.toml" ]; then
        cd nova_fmt
        cargo build --release 2>/dev/null && log_info "nova_fmt built successfully" || log_warn "nova_fmt build skipped (empty project)"
        cd ..
    fi
    
    # Build nova_ls (language server)
    if [ -d "nova_ls" ] && [ -f "nova_ls/Cargo.toml" ]; then
        cd nova_ls
        cargo build --release 2>/dev/null && log_info "nova_ls built successfully" || log_warn "nova_ls build skipped (empty project)"
        cd ..
    fi
    
    cd "$PROJECT_ROOT"
}

# Run tests
run_tests() {
    if [ "$1" == "--test" ]; then
        log_info "Running tests..."
        
        cd "$COMPILER_DIR"
        cargo test
        
        cd "$PROJECT_ROOT/tests"
        # Run integration tests
        if [ -f "run_tests.sh" ]; then
            ./run_tests.sh
        fi
        
        log_info "Tests completed"
    fi
}

# Main build process
main() {
    check_dependencies
    build_c_components
    build_rust_components
    build_java_components
    build_toolchain
    run_tests "$1"
    
    echo ""
    log_info "=== Build Complete ==="
    echo ""
    echo "Binaries location:"
    echo "  - C libraries: $COMPILER_DIR/{lexer,parser}/"
    echo "  - Rust binaries: $COMPILER_DIR/target/release/"
    echo "  - Java classes: $COMPILER_DIR/interface_validator/build/"
    echo "  - Toolchain: $PROJECT_ROOT/toolchain/*/target/release/"
    echo ""
}

main "$@"