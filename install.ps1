#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$Version = "latest", # Sobreescribe: .\install.ps1 -Version v0.1.0
    [string]$InstallDir = "$env:USERPROFILE\.wn\bin"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$WN_REPO = "cuervolu/wn"
$WN_PACKAGE = "wn-cli"
$LegacyPiolaBin = "$env:USERPROFILE\.piola\bin\piola.exe"

function Write-Info
{
    param($msg) Write-Host "[info] $msg"         -ForegroundColor Green
}
function Write-Warn
{
    param($msg) Write-Host "[advertencia] $msg"  -ForegroundColor Yellow
}
function Write-Section
{
    param($msg) Write-Host "`n==> $msg"          -ForegroundColor Cyan
}
function Write-Fail
{
    param($msg) Write-Error "[error] $msg"; exit 1
}

function Get-LatestVersion
{
    $url = "https://api.github.com/repos/$WN_REPO/releases/latest"
    try
    {
        $response = Invoke-RestMethod -Uri $url -Method Get
        return $response.tag_name
    }
    catch
    {
        Write-Fail "No se pudo obtener la versión más reciente: $_"
    }
}

function Install-Wn
{
    Write-Section "Detectando plataforma"

    if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64" -and $env:PROCESSOR_ARCHITEW6432 -ne "AMD64")
    {
        Write-Fail "Solo Windows 64-bit está soportado actualmente."
    }
    $target = "x86_64-pc-windows-msvc"
    Write-Info "Target: $target"

    Write-Section "Obteniendo versión"
    if ($Version -eq "latest")
    {
        $resolvedVersion = Get-LatestVersion
    }
    else
    {
        $resolvedVersion = $Version
    }
    Write-Info "Versión: $resolvedVersion"

    $archiveUrl = "https://github.com/$WN_REPO/releases/download/$resolvedVersion/$WN_PACKAGE-$target.zip"

    Write-Section "Descargando WN++ $resolvedVersion"
    Write-Info "Desde: $archiveUrl"

    $tmpDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
    New-Item -ItemType Directory -Path $tmpDir | Out-Null

    try
    {
        $archivePath = Join-Path $tmpDir "wn.zip"

        # TLS 1.2+
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

        Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath -UseBasicParsing

        Write-Section "Instalando"
        Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force

        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

        $sourceBin = Join-Path $tmpDir "wn.exe"
        $destBin = Join-Path $InstallDir "wn.exe"
        Move-Item -Path $sourceBin -Destination $destBin -Force
        Write-Info "Binario instalado en: $destBin"

    }
    finally
    {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Write-LegacyPiolaWarning
{
    if (Test-Path $LegacyPiolaBin)
    {
        Write-Warn "Detecté una instalación antigua de Piola en: $LegacyPiolaBin"
        Write-Warn "WN++ usa el comando 'wn' y se instala aparte; 'piola update' no migra al nuevo nombre."
        Write-Warn "Cuando confirmes que 'wn' funciona, puedes eliminar la instalación antigua manualmente."
    }
}

function Add-ToPath
{
    Write-Section "Configurando PATH"

    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

    if ($currentPath -like "*$InstallDir*")
    {
        Write-Info "PATH ya contiene $InstallDir"
        return
    }

    [Environment]::SetEnvironmentVariable(
            "PATH",
            "$currentPath;$InstallDir",
            "User"   # solo para el usuario actual, no requiere admin y que paja manejar esas weas de permisos
    )

    # También actualiza el PATH de la sesión actual
    $env:PATH = "$env:PATH;$InstallDir"

    Write-Info "PATH actualizado. Reinicia tu terminal para que tome efecto."
}

function Test-Installation
{
    $binary = Join-Path $InstallDir "wn.exe"

    if (Test-Path $binary)
    {
        Write-Section "¡Listo!"
        Write-Host ""
        Write-Host "  WN++ instalado exitosamente." -ForegroundColor Green
        Write-Host ""
        Write-Host "  Ejecuta 'wn' para abrir el REPL"
        Write-Host "  o 'wn programa.cl' para ejecutar un archivo"
        Write-Host ""
        Write-Warn "Puede que necesites abrir una nueva terminal para usar 'wn' directamente."
    }
    else
    {
        Write-Fail "La instalación falló. No se encontró el binario en $binary"
    }
}

Write-Host ""
Write-Host "Bienvenido al instalador de WN++" -ForegroundColor Cyan -NoNewline
Write-Host " (Windows)" -ForegroundColor Gray
Write-Host ""

Write-LegacyPiolaWarning
Install-Wn
Add-ToPath
Test-Installation
