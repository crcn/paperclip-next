#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

need_cmd() { command -v "$1" >/dev/null 2>&1; }

quick_check() {
  local missing=()
  need_cmd cargo || missing+=("rust")
  need_cmd node || missing+=("node")
  need_cmd protoc || missing+=("protoc")

  if [ ${#missing[@]} -gt 0 ]; then
    echo "⚠️  Missing: ${missing[*]}"
    echo "   Run './dev.sh setup' to install"
    echo ""
  fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Editor Detection
# ─────────────────────────────────────────────────────────────────────────────

detect_editors() {
  local editors=()
  need_cmd code || [ -d "/Applications/Visual Studio Code.app" ] && editors+=("vscode")
  need_cmd cursor || [ -d "/Applications/Cursor.app" ] && editors+=("cursor")
  need_cmd antigravity || [ -d "/Applications/Antigravity.app" ] && editors+=("antigravity")
  echo "${editors[*]:-}"
}

get_editor_cmd() {
  case "$1" in
    vscode) need_cmd code && echo "code" || echo "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";;
    cursor) need_cmd cursor && echo "cursor" || echo "/Applications/Cursor.app/Contents/Resources/app/bin/cursor";;
    antigravity) need_cmd antigravity && echo "antigravity" || echo "/Applications/Antigravity.app/Contents/Resources/app/bin/antigravity";;
  esac
}

get_ext_dir() {
  case "$1" in
    vscode) echo "$HOME/.vscode/extensions";;
    cursor) echo "$HOME/.cursor/extensions";;
    antigravity) echo "$HOME/.antigravity/extensions";;
  esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────────────────────

cmd_setup() {
  echo ""
  echo "Paperclip Setup"
  echo "───────────────"
  echo ""

  # Check deps
  echo "Checking dependencies..."
  echo ""
  need_cmd cargo && echo "  ✓ Rust" || echo "  ✗ Rust"
  need_cmd node && echo "  ✓ Node.js" || echo "  ✗ Node.js"
  need_cmd protoc && echo "  ✓ protoc" || echo "  ✗ protoc"
  need_cmd fzf && echo "  ✓ fzf" || echo "  ○ fzf (optional)"
  need_cmd wasm-pack && echo "  ✓ wasm-pack" || echo "  ○ wasm-pack (optional)"

  local editors=$(detect_editors)
  echo ""
  echo "Editors: ${editors:-none detected}"
  echo ""

  read -p "Install missing dependencies? (y/n) " -n 1 -r
  echo ""
  [[ ! $REPLY =~ ^[Yy]$ ]] && return 0

  # Install Rust
  if ! need_cmd cargo; then
    echo "→ Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  fi

  # Install protoc (macOS)
  if ! need_cmd protoc && need_cmd brew; then
    echo "→ Installing protoc..."
    brew install protobuf
  fi

  # Install fzf (macOS)
  if ! need_cmd fzf && need_cmd brew; then
    echo "→ Installing fzf..."
    brew install fzf
  fi

  # Install wasm-pack
  if ! need_cmd wasm-pack; then
    echo "→ Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  fi

  # Install JS deps (yarn workspaces)
  yarn install

  echo ""
  echo "✓ Setup complete"
}

# ─────────────────────────────────────────────────────────────────────────────
# Commands
# ─────────────────────────────────────────────────────────────────────────────

cmd_init() {
  echo ""
  echo "Paperclip Init"
  echo "──────────────"
  echo ""

  # 1. Check/install dependencies
  echo "① Checking dependencies..."
  local missing=()
  need_cmd cargo || missing+=("rust")
  need_cmd node || missing+=("node")
  need_cmd protoc || missing+=("protoc")

  if [ ${#missing[@]} -gt 0 ]; then
    echo "   Missing: ${missing[*]}"
    echo "   Running setup..."
    cmd_setup
  else
    echo "   ✓ All dependencies installed"
  fi
  echo ""

  # 2. Build Rust (needed for WASM)
  echo "② Building Rust workspace..."
  cargo build --workspace --release 2>&1 | sed 's/^/   /'
  echo ""

  # 3. Build WASM (needed before yarn install for loader/plugin deps)
  if need_cmd wasm-pack && [ -d "packages/wasm" ]; then
    echo "③ Building WASM..."
    (cd packages/wasm && wasm-pack build --target bundler --out-dir pkg 2>&1 | sed 's/^/   /')
    (cd packages/wasm && wasm-pack build --target nodejs --out-dir pkg-node 2>&1 | sed 's/^/   /')
    echo ""
  fi

  # 4. Install all JS dependencies (yarn workspaces)
  echo "④ Installing JS dependencies..."
  yarn install 2>&1 | sed 's/^/   /'
  echo ""

  # 5. Build proto package (others depend on it)
  if [ -d "packages/proto" ]; then
    echo "⑤ Building proto package..."
    (cd packages/proto && npm run build) 2>&1 | sed 's/^/   /'
    echo ""
  fi

  # 6. Build VS Code extension
  if [ -d "packages/vscode-extension" ]; then
    echo "⑥ Building VS Code extension..."
    (cd packages/vscode-extension && npm run compile) 2>&1 | sed 's/^/   /'
    echo ""
  fi

  echo "✓ Init complete!"
  echo ""
  echo "Next steps:"
  echo "  ./dev.sh server      - Start gRPC server"
  echo "  ./dev.sh install-ext - Install VS Code extension"
  echo "  ./dev.sh demo        - Start demo client"
  echo ""
}

cmd_build() {
  echo "→ Building Rust (release)..."
  cargo build --workspace --release

  if need_cmd wasm-pack && [ -f "build-loaders.sh" ]; then
    echo "→ Building WASM..."
    ./build-loaders.sh
  fi

  if [ -d "packages/vscode-extension" ]; then
    echo "→ Building extension..."
    (cd packages/vscode-extension && npm run compile)
  fi

  echo "✓ Build complete"
}

cmd_server() {
  echo "→ Starting gRPC server on :50051"
  cargo run --bin paperclip-server examples
}

cmd_demo() {
  echo "→ Starting client at http://localhost:3000"
  (cd packages/client && npm run dev)
}

cmd_test() {
  echo "→ Running Rust tests..."
  cargo test --workspace
  echo ""
  echo "→ Running VS Code extension tests..."
  (cd packages/vscode-extension && npx vitest run)
  echo ""
  echo "✓ All tests passed"
}

cmd_check() {
  echo "→ check"
  cargo check --workspace
  echo "→ clippy"
  cargo clippy --workspace -- -D warnings
  echo "→ fmt"
  cargo fmt --all -- --check
  echo "→ test"
  cargo test --workspace
  echo "✓ All checks passed"
}

cmd_clippy() { cargo clippy --workspace -- -D warnings; }
cmd_format() { cargo fmt --all; }
cmd_bench() { cargo bench --workspace; }
cmd_clean() {
  if ! need_cmd fzf; then
    # No fzf - clean everything
    echo "→ Cleaning all build artifacts and dependencies..."
    git clean -fXd
    echo "✓ Clean complete"
    return
  fi

  local choices
  choices=$(cat <<'EOF' | fzf --multi --height=40% --layout=reverse --border=none --prompt="Select items to clean (TAB to multi-select): " --pointer="›"
All (git clean -fXd)
Rust target/
Node node_modules/
VSCode extension out/
WASM pkg/
Proto generated/
EOF
)

  [ -z "$choices" ] && return 0

  echo ""
  while IFS= read -r choice; do
    case "$choice" in
      "All (git clean -fXd)")
        echo "→ Cleaning all (git clean -fXd)..."
        git clean -fXd
        ;;
      "Rust target/")
        echo "→ Cleaning Rust target/..."
        cargo clean
        ;;
      "Node node_modules/")
        echo "→ Cleaning node_modules/..."
        find . -name "node_modules" -type d -prune -exec rm -rf {} + 2>/dev/null || true
        ;;
      "VSCode extension out/")
        echo "→ Cleaning VSCode extension out/..."
        rm -rf packages/vscode-extension/out
        ;;
      "WASM pkg/")
        echo "→ Cleaning WASM pkg/..."
        rm -rf packages/wasm/pkg packages/wasm/pkg-node
        ;;
      "Proto generated/")
        echo "→ Cleaning proto generated/..."
        rm -rf packages/proto/src/generated packages/proto/lib
        ;;
    esac
  done <<< "$choices"

  echo "✓ Clean complete"
}

cmd_install_ext() {
  local editors=$(detect_editors)

  if [ -z "$editors" ]; then
    echo "No supported editors found (VSCode, Cursor, Antigravity)"
    return 1
  fi

  local editor_array=($editors)
  local selected

  if [ ${#editor_array[@]} -eq 1 ]; then
    selected="${editor_array[0]}"
  else
    # Multiple editors - use fzf if available
    if need_cmd fzf; then
      selected=$(printf '%s\n' "${editor_array[@]}" | fzf --height=10 --layout=reverse --prompt="Select editor: ")
    else
      echo "Select editor:"
      select e in "${editor_array[@]}"; do selected="$e"; break; done
    fi
  fi

  [ -z "$selected" ] && return 0

  # Build if needed
  if [ ! -f "packages/vscode-extension/out/extension.js" ]; then
    echo "→ Building extension..."
    (cd packages/vscode-extension && npm run compile)
  fi

  # Symlink extension
  local ext_dir=$(get_ext_dir "$selected")
  mkdir -p "$ext_dir"
  local target="$ext_dir/paperclip-vscode"

  rm -rf "$target" 2>/dev/null || true
  ln -s "$SCRIPT_DIR/packages/vscode-extension" "$target"

  echo "✓ Installed to $selected"
  echo "  Restart editor to activate"
}

cmd_status() {
  echo ""
  echo "Paperclip Status"
  echo "────────────────"
  echo ""
  need_cmd cargo && echo "  ✓ Rust" || echo "  ✗ Rust"
  need_cmd node && echo "  ✓ Node $(node --version 2>/dev/null)" || echo "  ✗ Node"
  need_cmd protoc && echo "  ✓ protoc" || echo "  ✗ protoc"
  need_cmd fzf && echo "  ✓ fzf" || echo "  ○ fzf"
  need_cmd wasm-pack && echo "  ✓ wasm-pack" || echo "  ○ wasm-pack"
  echo ""
  local editors=$(detect_editors)
  echo "Editors: ${editors:-none}"
  echo ""
}

# ─────────────────────────────────────────────────────────────────────────────
# Interactive Menu
# ─────────────────────────────────────────────────────────────────────────────

show_menu() {
  if ! need_cmd fzf; then
    echo "fzf required for interactive mode"
    echo "  brew install fzf"
    echo ""
    echo "Or use: ./dev.sh <command>"
    echo "  build, server, demo, test, check, install-ext, setup, status"
    return 1
  fi

  local choice
  choice=$(cat <<'EOF' | fzf --height=50% --layout=reverse --border=none --prompt="› " --pointer="›"
Init (install & build all)
Build everything
Start server
Start demo
Install extension
Run tests
Run checks
Run clippy
Format code
Run benchmarks
Clean
Setup
Status
EOF
)

  [ -z "$choice" ] && return 0
  echo ""

  case "$choice" in
    "Init (install & build all)") cmd_init;;
    "Build everything") cmd_build;;
    "Start server") cmd_server;;
    "Start demo") cmd_demo;;
    "Install extension") cmd_install_ext;;
    "Run tests") cmd_test;;
    "Run checks") cmd_check;;
    "Run clippy") cmd_clippy;;
    "Format code") cmd_format;;
    "Run benchmarks") cmd_bench;;
    "Clean") cmd_clean;;
    "Setup") cmd_setup;;
    "Status") cmd_status;;
  esac
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

# Show warning for missing deps on interactive mode
[ $# -eq 0 ] && quick_check

case "${1:-}" in
  init) cmd_init;;
  build) cmd_build;;
  server|serve) cmd_server;;
  demo|client) cmd_demo;;
  install-ext|ext) cmd_install_ext;;
  test|tests) cmd_test;;
  check|checks) cmd_check;;
  clippy) cmd_clippy;;
  format|fmt) cmd_format;;
  bench) cmd_bench;;
  clean) cmd_clean;;
  setup) cmd_setup;;
  status) cmd_status;;
  -h|--help|help)
    echo "Usage: ./dev.sh [command]"
    echo ""
    echo "Commands: init, build, server, demo, install-ext, test, check, clippy, format, bench, clean, setup, status"
    echo ""
    echo "Run without args for interactive menu"
    ;;
  "") show_menu;;
  *) echo "Unknown: $1"; exit 1;;
esac
