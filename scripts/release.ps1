<#
.SYNOPSIS
    Gestiona tags de release para WN++.
.PARAMETER Command
    Acción a ejecutar: create | delete | recreate | list
.PARAMETER Tag
    Tag en formato vX.Y.Z (ej: v0.2.0). Requerido para create, delete y recreate.
.EXAMPLE
    .\release.ps1 create v0.2.0
.EXAMPLE
    .\release.ps1 recreate v0.1.0
.EXAMPLE
    .\release.ps1 list
#>

[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet('create', 'delete', 'recreate', 'list')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Tag
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Info    ([string]$msg) { Write-Host "[info]  $msg" -ForegroundColor Green }
function Write-Warn    ([string]$msg) { Write-Host "[warn]  $msg" -ForegroundColor Yellow }
function Write-Section ([string]$msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Fatal   ([string]$msg) {
    Write-Host "[error] $msg" -ForegroundColor Red
    exit 1
}

function Invoke-Git ([string[]]$GitArgs) {
    $output = & git @GitArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "git $($GitArgs -join ' ') falló con código $LASTEXITCODE`n$output"
    }
    return $output
}


function Test-TagExistsLocal ([string]$tag) {
    $result = & git tag --list $tag 2>&1
    return ($LASTEXITCODE -eq 0) -and ($result -match [regex]::Escape($tag))
}

function Test-TagExistsRemote ([string]$tag) {
    $result = & git ls-remote --tags origin "refs/tags/$tag" 2>&1
    return ($LASTEXITCODE -eq 0) -and ($result -match [regex]::Escape($tag))
}

function Assert-TagFormat ([string]$tag) {
    # ← regex anclada al final; acepta prereleases (v0.2.0-beta)
    if ($tag -notmatch '^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$') {
        Write-Fatal "El tag '$tag' no tiene formato válido. Usa vX.Y.Z o vX.Y.Z-prerelease"
    }
}

function Assert-CleanWorkdir {
    $dirty = & git status --porcelain 2>&1
    if ($dirty) {
        Write-Fatal "Hay cambios sin commitear. No se puede crear una release."
    }
}

function Assert-MainBranch {
    $branch = (Invoke-Git 'branch', '--show-current').Trim()
    if ($branch -ne 'main') {
        Write-Fatal "Debes crear releases desde la branch main (estás en '$branch')."
    }
}

function Assert-InSyncWithOrigin {
    Invoke-Git 'fetch', 'origin', 'main', '--quiet' | Out-Null

    $head       = (Invoke-Git 'rev-parse', 'HEAD').Trim()
    $originMain = (Invoke-Git 'rev-parse', 'origin/main').Trim()

    if ($head -ne $originMain) {
        Write-Fatal "Tu main local no coincide con origin/main. Haz pull antes de continuar."
    }
}

function Assert-VersionMatchesCargo ([string]$tag) {
    $version = $tag -replace '^v', ''  # ← elimina el prefijo "v"

    $cargoContent = Get-Content Cargo.toml -Raw

    # Intenta primero [workspace.package]
    $workspaceMatch = [regex]::Match(
        $cargoContent,
        '(?s)\[workspace\.package\].*?version\s*=\s*"([^"]+)"'
    )

    $cargoVersion = if ($workspaceMatch.Success) {
        $workspaceMatch.Groups[1].Value
    } else {
        # fallback: primer version = "..." del archivo
        $m = [regex]::Match($cargoContent, 'version\s*=\s*"([^"]+)"')
        if ($m.Success) { $m.Groups[1].Value } else { $null }
    }

    if (-not $cargoVersion) {
        Write-Fatal "No se pudo leer la versión de Cargo.toml."
    }

    if ($version -ne $cargoVersion) {
        Write-Fatal "El tag $tag no coincide con la versión en Cargo.toml ($cargoVersion)."
    }
}

function Assert-RemoteExists {
    $null = & git remote get-url origin 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Fatal "No hay remote 'origin' configurado."
    }
}


function Invoke-Create ([string]$tag) {
    Assert-TagFormat           $tag
    Assert-RemoteExists
    Assert-CleanWorkdir
    Assert-MainBranch
    Assert-InSyncWithOrigin
    Assert-VersionMatchesCargo $tag

    Write-Section "Creando tag $tag"

    if (Test-TagExistsLocal $tag)  { Write-Fatal "El tag '$tag' ya existe localmente. Usa 'recreate' para reemplazarlo." }
    if (Test-TagExistsRemote $tag) { Write-Fatal "El tag '$tag' ya existe en el remoto. Usa 'recreate' para reemplazarlo." }

    $commit = (Invoke-Git 'rev-parse', '--short', 'HEAD').Trim()
    $msg    = (Invoke-Git 'log', '-1', '--format=%s', 'HEAD').Trim()
    Write-Info "Commit: $commit — $msg"

    Invoke-Git 'tag', '-a', $tag, '-m', "WN++ $tag" | Out-Null
    Write-Info "Tag creado localmente"

    Invoke-Git 'push', 'origin', $tag | Out-Null
    Write-Info "Tag pusheado a origin"

    Write-Section "¡Listo!"
    Write-Info "Revisa el pipeline: https://github.com/cuervolu/wn/actions"
}

function Invoke-Delete ([string]$tag) {
    Assert-TagFormat $tag
    Assert-RemoteExists

    Write-Section "Borrando tag $tag"
    $deleted = $false

    if (Test-TagExistsLocal $tag) {
        Invoke-Git 'tag', '-d', $tag | Out-Null
        Write-Info "Tag borrado localmente"
        $deleted = $true
    } else {
        Write-Warn "El tag '$tag' no existe localmente, saltando..."
    }

    if (Test-TagExistsRemote $tag) {
        Invoke-Git 'push', 'origin', '--delete', $tag | Out-Null
        Write-Info "Tag borrado del remoto"
        $deleted = $true
    } else {
        Write-Warn "El tag '$tag' no existe en el remoto, saltando..."
    }

    if (-not $deleted) {
        Write-Warn "El tag '$tag' no existía en ningún lado."
    } else {
        Write-Section "¡Listo!"
        Write-Warn "Si había un Release en GitHub, bórralo manualmente:"
        Write-Info "https://github.com/cuervolu/wn/releases/tag/$tag"
    }
}

function Invoke-Recreate ([string]$tag) {
    Assert-TagFormat $tag

    Write-Section "Recreando tag $tag"
    Write-Warn "Usa esto SOLO si el workflow falló antes de publicar una GitHub Release."
    Write-Warn "Si la versión ya fue publicada o descargada, crea una nueva versión patch."

    $resp = Read-Host "`n¿Confirmas que NO existe una release publicada para $tag? [s/N]"
    if ($resp -notmatch '^[sS]$') {
        Write-Fatal "Abortado."
    }

    Invoke-Delete $tag
    Invoke-Create $tag
}

function Invoke-List {
    Write-Section "Tags locales (últimos 10)"
    & git tag --sort=-version:refname | Select-Object -First 10

    Write-Section "Último tag"
    $last = & git describe --tags --abbrev=0 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $last) { $last = "ninguno" }
    Write-Info "Último tag: $last"
}

function Show-Usage {
    Write-Host @"

Uso:
  .\release.ps1 <comando> [tag]

Comandos:
  create   <vX.Y.Z>   Crea y pushea el tag
  delete   <vX.Y.Z>   Borra el tag local y remoto
  recreate <vX.Y.Z>   Borra y vuelve a crear (para fixes de CI)
  list                Muestra los últimos tags

Ejemplos:
  .\release.ps1 create v0.2.0
  .\release.ps1 recreate v0.1.0
  .\release.ps1 delete v0.1.0-beta

"@
}

if (-not $Command) { Show-Usage; exit 1 }

switch ($Command) {
    'create'   {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 create v0.2.0" }
        Invoke-Create $Tag
    }
    'delete'   {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 delete v0.1.0" }
        Invoke-Delete $Tag
    }
    'recreate' {
        if (-not $Tag) { Show-Usage; Write-Fatal "Falta el tag. Ej: .\release.ps1 recreate v0.1.0" }
        Invoke-Recreate $Tag
    }
    'list'     { Invoke-List }
}