<#
.SYNOPSIS
Vérification rapide (par défaut) ou contrôle complet (-Complet) du projet ; autorité : AGENTS.md.
Lancé depuis n'importe où : pwsh -NoProfile -File tools/Test-Projet.ps1 [-Complet]
Codes de sortie : 0 réussi, 1 échec.
#>
[CmdletBinding()]
param([switch]$Complet)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'

function Invoke-Check {
    param([string]$Command, [string[]]$Arguments)
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Échec du contrôle : $Command $($Arguments -join ' ') (code $LASTEXITCODE)"
    }
}

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    Invoke-Check -Command cargo -Arguments @('fmt', '--all', '--check')
    Invoke-Check -Command cargo -Arguments @('clippy', '--locked', '--all-targets', '--', '-D', 'warnings')
    Invoke-Check -Command cargo -Arguments @('test', '--locked')
    if ($Complet) {
        Invoke-Check -Command cargo -Arguments @('build', '--release', '--locked')
        Invoke-Check -Command cargo -Arguments @('deny', '--locked', 'check', 'licenses')
        Invoke-Check -Command pwsh -Arguments @('-NoProfile', '-File', 'tools/commun/Test-Docs.ps1', '-Path', '.')
    }
}
catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
finally { Pop-Location }
