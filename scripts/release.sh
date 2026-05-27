#!/usr/bin/env bash
# scripts/release.sh — Publica tags de releases YA PREPARADAS para WN++
#
# Flujo correcto antes de ejecutar create:
#   1. Actualiza version = "X.Y.Z" en Cargo.toml
#   2. Actualiza CHANGELOG.md
#   3. Commitea y pushea a main
#
# Recién ahí:
#   ./scripts/release.sh create vX.Y.Z

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

info()    { printf "${GREEN}[info]${RESET}  %s\n" "$1"; }
warn()    { printf "${YELLOW}[warn]${RESET}  %s\n" "$1"; }
error()   { printf "${RED}[error]${RESET} %s\n" "$1" >&2; exit 1; }
section() { printf "\n${BOLD}==> %s${RESET}\n" "$1"; }

check_tag_format() {
    local tag="$1"
    if ! echo "$tag" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'; then
        error "El tag '$tag' no tiene formato válido. Usa vX.Y.Z o vX.Y.Z-prerelease"
    fi
}

check_clean_workdir() {
    if [ -n "$(git status --porcelain)" ]; then
        error "Hay cambios sin commitear. No se puede crear una release."
    fi
}

check_main_branch() {
    local branch
    branch=$(git branch --show-current)
    [ "$branch" = "main" ] ||
        error "Debes crear releases desde la branch main (estás en '$branch')."
}

check_in_sync_with_origin() {
    git fetch origin main --quiet

    local head origin_main
    head=$(git rev-parse HEAD)
    origin_main=$(git rev-parse origin/main)

    [ "$head" = "$origin_main" ] ||
        error "Tu main local no coincide con origin/main. Haz pull antes de continuar."
}

check_version_matches_cargo() {
    local tag="$1"
    local version="${tag#v}"

    local workspace_version
    workspace_version=$(
        awk '
            /^\[workspace\.package\]$/ { in_section = 1; next }
            /^\[/ && in_section        { exit }
            in_section && /^version[[:space:]]*=/ {
                gsub(/"/, "", $3)
                print $3
                exit
            }
        ' Cargo.toml
    )

    [ -n "$workspace_version" ] ||
        error "No se pudo leer [workspace.package].version en Cargo.toml."

    [ "$version" = "$workspace_version" ] ||
        error "El tag $tag no coincide con la versión en Cargo.toml ($workspace_version)."
}

check_remote_exists() {
    git remote get-url origin > /dev/null 2>&1 ||
        error "No hay remote 'origin' configurado."
}

tag_exists_local()  { git tag --list "$1" | grep -q "$1"; }
tag_exists_remote() { git ls-remote --tags origin "refs/tags/$1" | grep -q "$1"; }

cmd_create() {
    local tag="$1"

    check_tag_format       "$tag"
    check_remote_exists
    check_clean_workdir
    check_main_branch
    check_in_sync_with_origin
    check_version_matches_cargo "$tag"

    section "Creando tag $tag"

    if tag_exists_local "$tag"; then
        error "El tag '$tag' ya existe localmente. Usa 'recreate' para reemplazarlo."
    fi
    if tag_exists_remote "$tag"; then
        error "El tag '$tag' ya existe en el remoto. Usa 'recreate' para reemplazarlo."
    fi

    local commit msg
    commit=$(git rev-parse --short HEAD)
    msg=$(git log -1 --format="%s" HEAD)
    info "Commit: $commit — $msg"

    git tag -a "$tag" -m "WN++ $tag"
    info "Tag creado localmente"

    # ← solo el tag, no main; no publicamos commits accidentalmente
    git push origin "$tag"
    info "Tag pusheado a origin"

    section "¡Listo!"
    info "Revisa el pipeline: https://github.com/cuervolu/wn/actions"
}

cmd_delete() {
    local tag="$1"

    check_tag_format "$tag"
    check_remote_exists

    section "Borrando tag $tag"

    local deleted=false

    if tag_exists_local "$tag"; then
        git tag -d "$tag"
        info "Tag borrado localmente"
        deleted=true
    else
        warn "El tag '$tag' no existe localmente, saltando..."
    fi

    if tag_exists_remote "$tag"; then
        git push origin --delete "$tag"
        info "Tag borrado del remoto"
        deleted=true
    else
        warn "El tag '$tag' no existe en el remoto, saltando..."
    fi

    if [ "$deleted" = false ]; then
        warn "El tag '$tag' no existía en ningún lado."
    else
        section "¡Listo!"
        warn "Si había un Release en GitHub, bórralo manualmente:"
        info "https://github.com/cuervolu/wn/releases/tag/$tag"
    fi
}

cmd_recreate() {
    local tag="$1"

    check_tag_format "$tag"

    section "Recreando tag $tag"
    warn "Usa esto SOLO si el workflow falló antes de publicar una GitHub Release."
    warn "Si la versión ya fue publicada o descargada, crea una nueva versión patch."
    printf "\n¿Confirmas que NO existe una release publicada para %s? [s/N] " "$tag"
    read -r respuesta

    if ! echo "$respuesta" | grep -qiE '^s$'; then
        error "Abortado."
    fi

    cmd_delete "$tag"
    cmd_create "$tag"
}

cmd_list() {
    section "Tags locales (últimos 10)"
    git tag --sort=-version:refname | head -10

    section "Último tag"
    local last
    last=$(git describe --tags --abbrev=0 2>/dev/null || echo "ninguno")
    info "Último tag: $last"
}

usage() {
    cat <<EOF

Uso:
  $0 <comando> [tag]

Comandos:
  create   <vX.Y.Z>   Crea y pushea el tag
  delete   <vX.Y.Z>   Borra el tag local y remoto
  recreate <vX.Y.Z>   Borra y vuelve a crear (para fixes de CI)
  list                Muestra los últimos tags

Ejemplos:
  $0 create v0.2.0
  $0 recreate v0.1.0
  $0 delete v0.1.0-beta

EOF
}

CMD="${1:-}"
TAG="${2:-}"

case "$CMD" in
    create)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 create v0.2.0"; }
        cmd_create "$TAG"
        ;;
    delete)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 delete v0.1.0"; }
        cmd_delete "$TAG"
        ;;
    recreate)
        [ -z "$TAG" ] && { usage; error "Falta el tag. Ej: $0 recreate v0.1.0"; }
        cmd_recreate "$TAG"
        ;;
    list)
        cmd_list
        ;;
    *)
        usage
        exit 1
        ;;
esac