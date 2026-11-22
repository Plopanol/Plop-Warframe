<#
.SYNOPSIS
Vérifie les garde-fous du nettoyage dans un dépôt temporaire isolé.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$fixture = Join-Path ([IO.Path]::GetTempPath()) ('plop-clean-test-' + [Guid]::NewGuid().ToString('N'))

function Invoke-Cleanup([int]$Expected, [string[]]$Options = @()) {
    $output = & pwsh -NoProfile -File (Join-Path $fixture 'tools/Nettoyer-Repo.ps1') @Options 2>&1
    if ($LASTEXITCODE -ne $Expected) { throw "Code inattendu : $LASTEXITCODE ; $output" }
}

function Assert-Present([string]$Relative) {
    if (-not (Test-Path -LiteralPath (Join-Path $fixture $Relative))) { throw "Fichier perdu : $Relative" }
}

try {
    [void][IO.Directory]::CreateDirectory((Join-Path $fixture 'tools'))
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Nettoyer-Repo.ps1') -Destination (Join-Path $fixture 'tools')
    & git -C $fixture init --quiet
    if ($LASTEXITCODE -ne 0) { throw 'git init a échoué.' }
    [IO.File]::WriteAllText((Join-Path $fixture '.gitignore'), "/target/`n/data/`n/livrable/`n")
    foreach ($directory in 'target', 'data', 'livrable') {
        [void][IO.Directory]::CreateDirectory((Join-Path $fixture $directory))
        [IO.File]::WriteAllText((Join-Path $fixture "$directory/sentinel.txt"), $directory)
    }
    Invoke-Cleanup 0
    Assert-Present 'target/sentinel.txt'
    Invoke-Cleanup 0 @('-ConserverCacheRust', '-Execute')
    Assert-Present 'target/sentinel.txt'
    Invoke-Cleanup 0 @('-Execute')
    if (Test-Path -LiteralPath (Join-Path $fixture 'target')) { throw 'target/ subsiste après nettoyage.' }
    Assert-Present 'data/sentinel.txt'
    Assert-Present 'livrable/sentinel.txt'

    [void][IO.Directory]::CreateDirectory((Join-Path $fixture 'target'))
    [IO.File]::WriteAllText((Join-Path $fixture 'target/tracked.txt'), 'tracked')
    & git -C $fixture add -f -- target/tracked.txt
    if ($LASTEXITCODE -ne 0) { throw 'git add du témoin suivi a échoué.' }
    Invoke-Cleanup 1 @('-Execute')
    Assert-Present 'target/tracked.txt'
    & git -C $fixture rm --cached -q -- target/tracked.txt
    if ($LASTEXITCODE -ne 0) { throw 'git rm du témoin a échoué.' }

    [void][IO.Directory]::CreateDirectory((Join-Path $fixture 'outside'))
    [IO.File]::WriteAllText((Join-Path $fixture 'outside/sentinel.txt'), 'outside')
    $junction = Join-Path $fixture 'target/link'
    [void](New-Item -ItemType Junction -Path $junction -Target (Join-Path $fixture 'outside'))
    try {
        Invoke-Cleanup 1 @('-Execute')
        Assert-Present 'outside/sentinel.txt'
    } finally {
        # Delete only the junction itself, never its destination.
        if (Test-Path -LiteralPath $junction) { [IO.Directory]::Delete($junction) }
    }

    [void][IO.Directory]::CreateDirectory((Join-Path $fixture 'target/nested/.git'))
    Invoke-Cleanup 1 @('-Execute')
    Assert-Present 'target/tracked.txt'
    [IO.File]::WriteAllText((Join-Path $fixture '.gitignore'), '')
    Invoke-Cleanup 1 @('-Execute')
    Assert-Present 'target/tracked.txt'
    Assert-Present 'data/sentinel.txt'
    Assert-Present 'livrable/sentinel.txt'
    Write-Output 'Réussi : inventaire, conservation Rust, suppression ciblée, fichiers suivis, jonction, dépôt imbriqué, fichiers non ignorés, données et livrables préservés.'
    exit 0
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
} finally {
    $absolute = [IO.Path]::GetFullPath($fixture)
    $tempPrefix = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (-not $absolute.StartsWith($tempPrefix, [StringComparison]::OrdinalIgnoreCase) -or
        [IO.Path]::GetFileName($absolute) -notlike 'plop-clean-test-*') { throw 'Nettoyage de fixture hors temporaire refusé.' }
    if (Test-Path -LiteralPath $absolute) { Remove-Item -LiteralPath $absolute -Recurse -Force }
}
