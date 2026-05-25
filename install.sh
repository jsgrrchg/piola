#!/usr/bin/env bash

set -e

WN_REPO="cuervolu/wn"
WN_PACKAGE="wn-cli"
WN_BIN_DIR="${WN_HOME:-$HOME/.wn}/bin"
WN_VERSION="${WN_VERSION:-latest}"
LEGACY_PIOLA_BIN="${PIOLA_HOME:-$HOME/.piola}/bin/piola"

if [ -t 1 ]; then
    BOLD="$(printf '\033[1m')"
    GREEN="$(printf '\033[32m')"
    YELLOW="$(printf '\033[33m')"
    RED="$(printf '\033[31m')"
    RESET="$(printf '\033[0m')"
else
    BOLD="" GREEN="" YELLOW="" RED="" RESET=""
fi

info()    { printf "%s[info]%s %s\n"    "$GREEN"  "$RESET" "$1"; }
warn()    { printf "%s[advertencia]%s %s\n" "$YELLOW" "$RESET" "$1"; }
error()   { printf "%s[error]%s %s\n"   "$RED"    "$RESET" "$1" >&2; exit 1; }
section() { printf "\n%s==> %s%s\n"     "$BOLD"   "$1" "$RESET"; }

detect_target() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)  echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                armv7l)  echo "armv7-unknown-linux-gnueabihf" ;;
                *)       error "Arquitectura Linux no soportada: $arch" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64)  echo "aarch64-apple-darwin" ;;   # ← M1/M2/M3
                *)      error "Arquitectura macOS no soportada: $arch" ;;
            esac
            ;;
        *)
            error "Sistema operativo no soportado: $os. Usa Windows con install.ps1"
            ;;
    esac
}

check_dependencies() {
    for cmd in curl tar; do
        if ! command -v "$cmd" > /dev/null 2>&1; then
            error "Necesitas '$cmd' instalado para continuar."
        fi
    done
}

get_latest_version() {
    if ! command -v curl > /dev/null 2>&1; then
        error "curl no encontrado"
    fi

    local url="https://api.github.com/repos/${WN_REPO}/releases/latest"
    local version

    version=$(curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        error "No se pudo obtener la versión más reciente de GitHub. ¿Existe un release?"
    fi

    echo "$version"
}


install_wn() {
    local target version archive_url tmp_dir

    section "Detectando plataforma"
    target="$(detect_target)"
    info "Target: $target"

    section "Obteniendo versión"
    if [ "$WN_VERSION" = "latest" ]; then
        version="$(get_latest_version)"
    else
        version="$WN_VERSION"
    fi
    info "Versión: $version"

    archive_url="https://github.com/${WN_REPO}/releases/download/${version}/${WN_PACKAGE}-${target}.tar.gz"

    section "Descargando WN++ $version"
    info "Desde: $archive_url"

    tmp_dir="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmp_dir'" EXIT

    if ! curl -fsSL --progress-bar "$archive_url" -o "$tmp_dir/wn.tar.gz"; then
        error "No se pudo descargar $archive_url\n  ¿Existe este release para $target?"
    fi

    section "Instalando"
    tar -xzf "$tmp_dir/wn.tar.gz" -C "$tmp_dir"
    mkdir -p "$WN_BIN_DIR"
    mv "$tmp_dir/wn" "$WN_BIN_DIR/wn"
    chmod +x "$WN_BIN_DIR/wn"
    info "Binario instalado en: $WN_BIN_DIR/wn"

    section "Configurando PATH"
    configure_path
}

configure_path() {
    local shell_config export_line

    export_line="export PATH=\"\$PATH:$WN_BIN_DIR\""

    case "${SHELL:-}" in
        */zsh)  shell_config="$HOME/.zshrc" ;;
        */bash) shell_config="$HOME/.bashrc" ;;
        */fish) shell_config="$HOME/.config/fish/config.fish"
                export_line="fish_add_path $WN_BIN_DIR" ;;
        *)      shell_config="$HOME/.profile" ;;
    esac

    if ! grep -qF "$WN_BIN_DIR" "$shell_config" 2>/dev/null; then
        printf "\n# WN++\n%s\n" "$export_line" >> "$shell_config"
        info "PATH actualizado en $shell_config"
    else
        info "PATH ya configurado en $shell_config"
    fi

    warn "Reinicia tu terminal o ejecuta: source $shell_config"
}

warn_legacy_piola() {
    if [ -x "$LEGACY_PIOLA_BIN" ]; then
        warn "Detecté una instalación antigua de Piola en: $LEGACY_PIOLA_BIN"
        warn "WN++ usa el comando 'wn' y se instala aparte; 'piola update' no migra al nuevo nombre."
        warn "Cuando confirmes que 'wn' funciona, puedes eliminar la instalación antigua manualmente."
    fi
}

verify_installation() {
    if "$WN_BIN_DIR/wn" --version > /dev/null 2>&1; then
        section "¡Listo!"
        printf "\n  %sWN++ instalado exitosamente.%s\n\n" "$GREEN$BOLD" "$RESET"
        printf "  Ejecuta %swn%s para abrir el REPL\n"     "$BOLD" "$RESET"
        printf "  o %swn programa.cl%s para ejecutar un archivo\n\n" "$BOLD" "$RESET"
    else
        warn "El binario se instaló pero no respondió a --version."
        warn "Puede que necesites reiniciar tu terminal."
    fi
}

main() {
    printf "\n%sBienvenido al instalador de WN++%s\n" "$BOLD" "$RESET"

    check_dependencies
    warn_legacy_piola
    install_wn
    verify_installation
}

main "$@"
