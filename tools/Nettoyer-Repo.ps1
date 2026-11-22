<#
.SYNOPSIS
Inventorie target/ ; supprime uniquement avec -Execute. Préserve data/ et livrable/.
#>
[CmdletBinding()]
param([switch]$Execute, [switch]$ConserverCacheRust)

$ErrorActionPreference = 'Stop'

function Get-SafeTarget {
    $candidate = [IO.Path]::GetFullPath((Join-Path $script:repositoryRoot 'target'))
    $prefix = $script:repositoryRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (-not $candidate.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Chemin hors dépôt refusé : $candidate"
    }
    if (-not (Test-Path -LiteralPath $candidate)) { return $null }

    $tracked = @(& git -C $script:repositoryRoot ls-files --stage -- target)
    if ($LASTEXITCODE -ne 0) { throw 'Échec de la lecture des fichiers suivis.' }
    if ($tracked.Count -gt 0) { throw 'target/ contient des fichiers suivis ou un sous-module : refus.' }
    & git -C $script:repositoryRoot check-ignore -q -- target/
    if ($LASTEXITCODE -ne 0) { throw 'target/ doit être ignoré par Git : refus.' }
    $unignored = @(& git -C $script:repositoryRoot ls-files --others --exclude-standard -- target)
    if ($LASTEXITCODE -ne 0) { throw 'Échec de la lecture des fichiers non suivis.' }
    if ($unignored.Count -gt 0) { throw 'target/ contient des fichiers non ignorés : refus.' }

    $pending = [Collections.Generic.Stack[IO.FileSystemInfo]]::new()
    $pending.Push((Get-Item -LiteralPath $candidate -Force))
    [long]$bytes = 0
    while ($pending.Count -gt 0) {
        $entry = $pending.Pop()
        if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Lien ou jonction refusé : $($entry.FullName)"
        }
        if (-not $entry.FullName.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Entrée hors dépôt refusée : $($entry.FullName)"
        }
        if ($entry.Name -eq '.git') { throw "Dépôt imbriqué refusé : $($entry.FullName)" }
        if ($entry -is [IO.DirectoryInfo]) {
            if ((Test-Path -LiteralPath (Join-Path $entry.FullName 'HEAD') -PathType Leaf) -and
                (Test-Path -LiteralPath (Join-Path $entry.FullName 'objects') -PathType Container)) {
                throw "Dépôt nu imbriqué refusé : $($entry.FullName)"
            }
            foreach ($child in Get-ChildItem -LiteralPath $entry.FullName -Force) {
                $pending.Push($child)
            }
        } else {
            if ($entry.FullName -eq $candidate) { throw 'target doit être un dossier : refus.' }
            $bytes += $entry.Length
        }
    }
    [pscustomobject]@{ Path = $candidate; Bytes = $bytes }
}

try {
    $scriptRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
    $gitRoot = & git -C $scriptRoot rev-parse --show-toplevel
    if ($LASTEXITCODE -ne 0) { throw 'Racine Git introuvable.' }
    $script:repositoryRoot = [IO.Path]::GetFullPath($gitRoot)
    if ($script:repositoryRoot -ne $scriptRoot -or
        ((Get-Item -LiteralPath $script:repositoryRoot -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'Le script doit être dans tools/ à la racine du dépôt, sans lien ni jonction.'
    }
    if ($ConserverCacheRust) {
        Write-Output "Conservé : $(Join-Path $script:repositoryRoot 'target') (cache Rust, debug et release)."
        exit 0
    }
    $target = Get-SafeTarget
    if ($null -eq $target) {
        Write-Output 'Aucun target/ à nettoyer.'
        exit 0
    }
    Write-Output ('Candidat : {0} — {1} octets ({2:N2} Gio).' -f $target.Path, $target.Bytes, ($target.Bytes / 1GB))
    if ($Execute) {
        # Repeat the complete guard immediately before the recursive deletion.
        $verified = Get-SafeTarget
        if ($null -eq $verified) { throw 'Le candidat a disparu depuis son inventaire.' }
        Remove-Item -LiteralPath $verified.Path -Recurse -Force
        Write-Output "Supprimé : $($verified.Path)."
    } else {
        Write-Output 'Inventaire uniquement ; utiliser -Execute pour supprimer.'
    }
    Write-Output 'Données, livrables et caches globaux conservés.'
    exit 0
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
