#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: ./install_global.sh [--no-sudo] [--bin-dir <directory>]

Compile Galois en mode release et installe le binaire globalement.

Options:
  --no-sudo            N'utilise pas sudo pendant l'installation.
  --bin-dir <dir>      Répertoire de destination (défaut: /usr/local/bin).
  -h, --help           Affiche cette aide.
EOF
}

use_sudo=1
bin_dir="/usr/local/bin"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-sudo)
            use_sudo=0
            shift
            ;;
        --bin-dir)
            if [[ $# -lt 2 ]]; then
                echo "Erreur: --bin-dir requiert un argument." >&2
                usage
                exit 1
            fi
            bin_dir="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Option inconnue: $1" >&2
            usage
            exit 1
            ;;
    esac
done

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source_bin="$repo_dir/target/release/galois"
dest_bin="$bin_dir/galois"

cd "$repo_dir"
echo "Compilation release..."
cargo build --release

if [[ ! -x "$source_bin" ]]; then
    echo "Erreur: binaire introuvable après compilation: $source_bin" >&2
    exit 1
fi

install_cmd=(install -m 755 "$source_bin" "$dest_bin")

if [[ "$use_sudo" -eq 1 && "$EUID" -ne 0 ]]; then
    echo "Installation dans $dest_bin (sudo)..."
    sudo mkdir -p "$bin_dir"
    sudo "${install_cmd[@]}"
else
    echo "Installation dans $dest_bin..."
    mkdir -p "$bin_dir"
    "${install_cmd[@]}"
fi

echo "Installation terminée: $dest_bin"
echo "Test:"
"$dest_bin" --help | head -n 1 || true
