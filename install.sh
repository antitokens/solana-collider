#!/usr/bin/env bash
set -euo pipefail

# Default versions; skip installation unless specified
LOCALNET_RPC="http://localhost:8899"
WALLET=".config/id.json"
RUST_VERSION="1.83.0"
SOLANA_CLI_VERSION="1.18.26"
ANCHOR_CLI_VERSION="0.29.0"
NODE_VERSION="22.14.0"
YARN_VERSION=""

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
    --localnet-rpc)
        LOCALNET_RPC="$2"
        shift 2
        ;;
    --wallet)
        WALLET="$2"
        shift 2
        ;;
    --rust)
        RUST_VERSION="$2"
        shift 2
        ;;
    --solana-cli)
        SOLANA_CLI_VERSION="$2"
        shift 2
        ;;
    --anchor-cli)
        ANCHOR_CLI_VERSION="$2"
        shift 2
        ;;
    --node)
        NODE_VERSION="$2"
        shift 2
        ;;
    --yarn)
        YARN_VERSION="$2"
        shift 2
        ;;
    *)
        echo "❌ Unknown option: $1"
        exit 1
        ;;
    esac
done

########################################
# Logging Functions
########################################
log_info() {
    printf "ℹ️  [INFO] %s\n" "$1"
}

log_error() {
    printf "❌ [ERROR] %s\n" "$1" >&2
}

########################################
# OS Detection
########################################
detect_os() {
    local os
    os="$(uname)"
    if [[ "$os" == "Linux" ]]; then
        echo "Linux"
    elif [[ "$os" == "Darwin" ]]; then
        echo "Darwin"
    else
        echo "$os"
    fi
}

########################################
# Install OS-Specific Dependencies
########################################
install_dependencies() {
    local os="$1"
    if [[ "$os" == "Linux" ]]; then
        log_info "Detected Linux OS. Updating package list and installing dependencies ⏳"
        SUDO=""
        if command -v sudo >/dev/null 2>&1; then
            SUDO="sudo"
        fi
        $SUDO apt-get update
        $SUDO apt-get install -y \
            build-essential \
            pkg-config \
            libudev-dev \
            llvm \
            libclang-dev \
            protobuf-compiler \
            libssl-dev
    elif [[ "$os" == "Darwin" ]]; then
        log_info "Detected macOS"
        # Check for cargo, install with brew if not found
        if ! command -v cargo >/dev/null 2>&1; then
            log_info "Cargo not found. Installing with Homebrew ⏳"
            if ! command -v brew >/dev/null 2>&1; then
                log_error "Homebrew not installed. Please install Homebrew first"
                exit 1
            fi
            brew install rust
        else
            log_info "Cargo is already installed"
        fi
    else
        log_info "Detected $os"
    fi

    echo ""
}

########################################
# Install Rust via rustup
########################################
install_rust() {
    local target_version="$1"

    if [[ -z "$target_version" ]]; then
        log_info "No Rust version specified. Skipping Rust installation"
        return
    fi

    if ! command -v rustup >/dev/null 2>&1; then
        log_info "Rustup not found. Installing Rustup ⏳"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        log_info "Rustup installation complete"
    fi

    if command -v rustc >/dev/null 2>&1; then
        local current_version
        current_version=$(rustc --version | cut -d' ' -f2)

        if [[ "$current_version" == "$target_version" ]]; then
            log_info "Rust $target_version is already installed. Skipping installation"
        else
            log_info "Installing specific Rust version $target_version ⏳"
            rustup default "$target_version"
        fi
    else
        log_info "Installing Rust ⏳"
        log_info "Installing specific Rust version $target_version ⏳"
        rustup default "$target_version"
    fi

    # Source the Rust environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        . "$HOME/.cargo/env"
    elif [[ -f "$HOME/.cargo/env.fish" ]]; then
        log_info "Sourcing Rust environment for Fish shell ⏳"
        source "$HOME/.cargo/env.fish"
    else
        log_error "Rust environment configuration file not found"
    fi

    if command -v rustc >/dev/null 2>&1; then
        echo "ⓘ  $(rustc --version)"
    else
        log_error "Rust installation failed"
    fi

    echo ""
}

########################################
# Install Solana CLI
########################################
install_solana_cli() {
    local os="$1"
    local target_version="$2"

    if [[ -z "$target_version" ]]; then
        log_info "No Solana CLI version specified. Skipping Solana CLI installation"
        return
    fi

    if command -v solana >/dev/null 2>&1; then
        local current_version
        current_version=$(solana --version | head -n1 | awk '{print $2}')

        if [[ "$current_version" == "$target_version" ]]; then
            log_info "Solana CLI $target_version is already installed. Skipping installation"
        else
            log_info "Installing specific Solana CLI version $target_version ⏳"
            sh -c "$(curl -sSfL https://release.anza.xyz/v$target_version/install)"
        fi
    else
        log_info "Installing Solana CLI version $target_version ⏳"
        sh -c "$(curl -sSfL https://release.anza.xyz/v$target_version/install)"
    fi

    if command -v solana >/dev/null 2>&1; then
        echo "ⓘ  $(solana --version)"
    else
        log_error "Solana CLI installation failed"
    fi

    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

    echo ""
}

########################################
# Install Anchor CLI
########################################
install_anchor_cli() {
    local target_version="$1"

    if [[ -z "$target_version" ]]; then
        log_info "No Anchor CLI version specified. Skipping Anchor CLI installation"
        return
    fi

    if command -v anchor >/dev/null 2>&1; then
        local current_version
        current_version=$(anchor --version | cut -d' ' -f2)

        if [[ "$current_version" == "$target_version" ]]; then
            log_info "Anchor CLI $target_version is already installed. Skipping installation"
        else
            if ! command -v avm >/dev/null 2>&1; then
                log_info "AVM is not installed. Installing AVM ⏳"
                cargo install --force --git https://github.com/coral-xyz/anchor avm
            fi
            log_info "Installing specific Anchor version $target_version ⏳"
            avm install "$target_version"
            avm use "$target_version"
        fi
    else
        log_info "Installing Anchor CLI via AVM ⏳"
        cargo install --git https://github.com/coral-xyz/anchor avm
        log_info "Installing specific Anchor version $target_version ⏳"
        avm install "$target_version"
        avm use "$target_version"
    fi

    if command -v anchor >/dev/null 2>&1; then
        echo "ⓘ  $(anchor --version)"
    else
        log_error "Anchor CLI installation failed"
    fi

    echo ""
}

########################################
# Install nvm and Node.js
########################################
install_nvm_and_node() {
    local target_version="$1"

    if [[ -z "$target_version" ]]; then
        log_info "No Node.js version specified. Skipping Node.js installation"
        return
    fi

    if [ -s "$HOME/.nvm/nvm.sh" ]; then
        log_info "NVM is already installed"
    else
        log_info "Installing NVM ⏳"
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash
    fi

    export NVM_DIR="$HOME/.nvm"
    # Immediately source nvm and bash_completion for the current session
    if [ -s "$NVM_DIR/nvm.sh" ]; then
        if [[ "$os" == "Darwin" ]]; then
            unset PREFIX
        fi
        . "$NVM_DIR/nvm.sh"
    else
        log_error "NVM not found. Ensure it is installed correctly"
        return
    fi

    if [ -s "$NVM_DIR/bash_completion" ]; then
        . "$NVM_DIR/bash_completion"
    fi

    if command -v node >/dev/null 2>&1; then
        local current_node
        current_node=$(node --version)

        if [ "$current_node" = "v$target_version" ]; then
            log_info "Node.js $target_version is already installed. Skipping installation"
        else
            log_info "Installing Node.js version $target_version ⏳"
            nvm install "$target_version"
            nvm alias default "$target_version"
            nvm use default
        fi
    else
        log_info "Installing Node.js version $target_version ⏳"
        nvm install "$target_version"
        nvm alias default "$target_version"
        nvm use default
    fi

    echo ""
}

########################################
# Install Yarn
########################################
install_yarn() {
    local target_version="$1"

    if [[ -z "$target_version" ]]; then
        log_info "No Yarn version specified. Skipping Yarn installation"
        return
    fi

    if command -v yarn >/dev/null 2>&1; then
        local current_version
        current_version=$(yarn --version)

        if [[ "$current_version" == "$target_version" ]]; then
            log_info "Yarn $target_version is already installed. Skipping installation."
        else
            log_info "Installing Yarn version $target_version ⏳"
            npm install --global yarn@"$target_version"
        fi
    else
        log_info "Installing Yarn version $target_version ⏳"
        npm install --global yarn@"$target_version"
    fi

    if command -v yarn >/dev/null 2>&1; then
        yarn --version
    else
        log_error "Yarn installation failed"
    fi

    echo ""
}

########################################
# Print Installed Versions
########################################
print_versions() {
    echo ""
    echo "ℹ️  Installed Versions:"
    echo "ⓘ  Rust: $(rustc --version 2>/dev/null || echo '⚠️  Not installed')"
    echo "ⓘ  Solana CLI: $(solana --version 2>/dev/null || echo '⚠️  Not installed')"
    echo "ⓘ  Anchor CLI: $(anchor --version 2>/dev/null || echo '⚠️  Not installed')"
    echo "ⓘ  Node.js: $(node --version 2>/dev/null || echo '⚠️  Not installed')"
    echo "ⓘ  Yarn: $(yarn --version 2>/dev/null || echo '⚠️  Not installed')"
    echo ""
}

########################################
# Append nvm Initialisation to the Correct Shell RC File
########################################
ensure_nvm_in_shell() {
    if [[ -z "$NODE_VERSION" ]]; then
        # Skip if Node.js is not being installed
        return
    fi

    local shell_rc=""
    local shell_name=""
    if [[ "$SHELL" == *"zsh" ]]; then
        shell_rc="$HOME/.zshrc"
        shell_name="zsh"
    elif [[ "$SHELL" == *"bash" ]]; then
        if [[ -f "$HOME/.bash_profile" ]]; then
            shell_rc="$HOME/.bash_profile"
        else
            shell_rc="$HOME/.bashrc"
        fi
        shell_name="bash"
    else
        shell_rc="$HOME/.profile"
        shell_name="unknown"
    fi

    if [ -f "$shell_rc" ]; then
        if ! grep -q 'export NVM_DIR="$HOME/.nvm"' "$shell_rc"; then
            log_info "Appending nvm initialisation to $shell_name"
            {
                echo ''
                echo 'export NVM_DIR="$HOME/.nvm"'
                echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm'
            } >>"$shell_rc"
        fi
    else
        log_info "$shell_rc does not exist, creating it with nvm initialisation"
        echo 'export NVM_DIR="$HOME/.nvm"' >"$shell_rc"
        echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm' >>"$shell_rc"
    fi
}

########################################
# Main Execution Flow
########################################
main() {
    local os
    os=$(detect_os)

    install_dependencies "$os"
    install_rust "$RUST_VERSION"
    install_solana_cli "$os" "$SOLANA_CLI_VERSION"
    install_anchor_cli "$ANCHOR_CLI_VERSION"
    install_nvm_and_node "$NODE_VERSION"
    install_yarn "$YARN_VERSION"

    ensure_nvm_in_shell

    print_versions

    log_info "Setting .config/id.json as Solana wallet"
    solana config set --url "$LOCALNET_RPC" --keypair "$WALLET"
    echo ""
    echo "✅ Installation complete. Please restart your terminal to apply all changes"
}

main "$@"
