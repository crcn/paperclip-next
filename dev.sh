#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

info() { echo "→ $*"; }
success() { echo "✓ $*"; }
error() { echo "✗ $*" >&2; }
warn() { echo "⚠ $*" >&2; }

# Check for fzf and offer to install
ensure_fzf() {
  if command -v fzf >/dev/null 2>&1; then
    return 0
  fi

  warn "fzf not found - needed for interactive menu"
  echo ""
  echo "Install fzf:"
  echo "  macOS:   brew install fzf"
  echo "  Linux:   sudo apt install fzf or sudo yum install fzf"
  echo ""
  read -p "Install with Homebrew now? (y/n) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v brew >/dev/null 2>&1; then
      brew install fzf
      success "fzf installed!"
    else
      error "Homebrew not found. Please install fzf manually."
      exit 1
    fi
  else
    error "fzf required for interactive mode. Use: ./dev.sh <command>"
    exit 1
  fi
}

run_tests() {
  info "Running Rust tests..."
  cargo test --workspace
  success "Tests complete"
}

run_benchmarks() {
  info "Running benchmarks..."
  cargo bench --workspace
  success "Benchmarks complete"
}

start_server() {
  info "Starting gRPC server..."
  info "Server will listen on 127.0.0.1:50051"
  info "Watching: examples/"
  cargo run --bin paperclip-server examples
}

open_designer() {
  info "Starting TypeScript client demo..."
  cd packages/client
  if [ ! -d "node_modules" ]; then
    info "Installing dependencies..."
    npm install
  fi
  info "Starting dev server at http://localhost:3000"
  npm run dev
}

run_checks() {
  info "Running all checks..."

  info "1. Cargo check..."
  cargo check --workspace

  info "2. Cargo clippy..."
  cargo clippy --workspace -- -D warnings

  info "3. Cargo format check..."
  cargo fmt --all -- --check

  info "4. Running tests..."
  cargo test --workspace

  success "All checks passed!"
}

build_all() {
  info "Building all packages..."
  cargo build --workspace --release
  success "Build complete"
}

clean_all() {
  info "Cleaning build artifacts..."
  cargo clean
  if [ -d "packages/client/node_modules" ]; then
    info "Cleaning node_modules..."
    rm -rf packages/client/node_modules
  fi
  success "Clean complete"
}

format_code() {
  info "Formatting code..."
  cargo fmt --all
  success "Format complete"
}

run_clippy() {
  info "Running clippy..."
  cargo clippy --workspace -- -D warnings
  success "Clippy complete"
}

show_menu() {
  ensure_fzf

  # Define menu items
  local options=(
    "test|Run all Rust tests (cargo test --workspace)"
    "bench|Run performance benchmarks"
    "server|Start gRPC server on :50051"
    "demo|Open TypeScript client demo (localhost:3000)"
    "check|Run all checks (check/clippy/fmt/test)"
    "build|Build all packages in release mode"
    "clippy|Run clippy linter"
    "format|Format all code with rustfmt"
    "clean|Clean all build artifacts"
  )

  # Use fzf for selection with constrained height
  local selected
  selected=$(printf '%s\n' "${options[@]}" | \
    fzf --height=40% \
        --layout=reverse \
        --border=rounded \
        --prompt="❯ " \
        --header="Paperclip Development Menu" \
        --header-first \
        --info=inline)

  if [ -z "$selected" ]; then
    echo "Cancelled."
    exit 0
  fi

  # Extract command from selection (format: "command|description")
  local cmd
  cmd=$(echo "$selected" | sed -E 's/^([^|]+)\|.*/\1/')

  echo ""
  case "$cmd" in
    test) run_tests;;
    bench) run_benchmarks;;
    server) start_server;;
    demo) open_designer;;
    check) run_checks;;
    build) build_all;;
    clippy) run_clippy;;
    format) format_code;;
    clean) clean_all;;
    *) error "Unknown command: $cmd"; exit 1;;
  esac
}

# If arguments provided, run specific command
if [ $# -gt 0 ]; then
  case "$1" in
    test|tests) run_tests;;
    bench|benchmarks) run_benchmarks;;
    server|serve) start_server;;
    demo|designer|client) open_designer;;
    check|checks) run_checks;;
    build) build_all;;
    clippy) run_clippy;;
    format|fmt) format_code;;
    clean) clean_all;;
    *)
      error "Unknown command: $1"
      echo ""
      echo "Available commands:"
      echo "  test       - Run tests"
      echo "  bench      - Run benchmarks"
      echo "  server     - Start gRPC server"
      echo "  demo       - Open client demo"
      echo "  check      - Run all checks"
      echo "  build      - Build all packages"
      echo "  clippy     - Run clippy linter"
      echo "  format     - Format code"
      echo "  clean      - Clean build artifacts"
      exit 1
      ;;
  esac
else
  # Interactive mode
  show_menu
fi
