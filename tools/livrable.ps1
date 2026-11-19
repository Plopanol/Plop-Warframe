<#
.SYNOPSIS
Produit le livrable du projet dans livrable/ : construction, archive, empreinte, vérification, copie.

.DESCRIPTION
Squelette commun (gabarit de _modele-projet) : les lignes marquées « À COMPLÉTER » sont les seules propres
au projet. Lancé depuis la racine du dépôt :  pwsh -File tools/livrable.ps1
Étapes (modules/livrable.md) :
  1. lit la version dans $VersionFile (le livrable ne la modifie jamais) ;
  2. délègue la construction à l'outil du projet (Invoke-ProjectBuild) ;
  3. nomme <projet>-<version>-<cible>.<ext> et écrit <nom>.sha256 ;
  4. appelle tools/commun/Verify-Livrable.ps1 ;
  5. copie les deux fichiers dans livrable/ (hors zip, avec CHANGELOG.md et NOTICES.md à côté du binaire), dossier ignoré par Git ; livrable/data/ n'est jamais touché.
Windows PowerShell 5.1 et PowerShell 7 ; UTF-8 avec BOM. Codes de sortie : 0 réussi, 1 échec.
#>
[CmdletBinding()]
param()

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'

# --- À COMPLÉTER : identité du livrable ---------------------------------------------------------------
$Project = 'plop-warframe'
$VersionFile = 'Cargo.toml'                                  # même fichier que « Version : » dans AGENTS.md
$VersionPattern = '(?m)^version\s*=\s*"(?<v>\d+\.\d+\.\d+)"'  # groupe nommé v ; adapter au format du fichier
$Target = 'x86_64-pc-windows-msvc'                      # cible nommée en triplet, ou avec un tiret bas : « windows-x64 » ferait lire « 0.1.0-windows » comme version
$Extension = 'zip'                                           # zip, ou autre pour un fichier unique
$Expected = @('plop-warframe.exe', 'CHANGELOG.md', 'NOTICES.md') # noms exacts attendus dans l'archive (Verify-Livrable -Expected)
$Notices = $true                                         # $true si des dépendances tierces sont embarquées (NOTICES.md)
$AllowPath = @()                                             # entrées exemptées du refus data/ (Verify-Livrable -AllowPath), justifiées ici

# Construction release sur l'hôte Windows x64 (= $Target). NOTICES.md doit refléter le graphe Cargo construit ;
# seul l'exécutable est déposé : data/ se crée à côté de lui au premier lancement, jamais dans le paquet.
function Invoke-ProjectBuild([string]$Staging, [string]$Version) {
    & pwsh -NoProfile -File (Join-Path $root 'tools/New-Notices.ps1') -Check
    if ($LASTEXITCODE -ne 0) { throw 'NOTICES.md n''est pas à jour : lancer tools/New-Notices.ps1.' }
    & cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "cargo build a échoué (code $LASTEXITCODE)." }
    $exe = Join-Path $root 'target/release/plop-warframe.exe'
    if (-not (Test-Path -LiteralPath $exe)) { throw "Exécutable introuvable : $exe" }
    Copy-Item -LiteralPath $exe -Destination $Staging
}

# --- Socle commun : ne pas modifier ---------------------------------------------------------------------
$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

function Find-CommonTool([string]$name) {
    foreach ($dir in 'tools', 'tool') {
        $candidate = Join-Path $root "$dir\commun\$name"
        if (Test-Path -LiteralPath $candidate) { return $candidate }
    }
    throw "$name introuvable dans tools/commun : lancer Sync-Commun.ps1 -Apply depuis _modele-projet."
}

function Get-ProjectVersion {
    $path = Join-Path $root $VersionFile
    if (-not (Test-Path -LiteralPath $path)) { throw "Fichier de version introuvable : $VersionFile" }
    $m = [regex]::Match([IO.File]::ReadAllText($path, [Text.Encoding]::UTF8), $VersionPattern)
    if (-not $m.Success) { throw "Version introuvable dans $VersionFile (motif $VersionPattern)" }
    return $m.Groups['v'].Value
}

$work = Join-Path ([IO.Path]::GetTempPath()) ("livrable-" + [Guid]::NewGuid().ToString('N'))
try {
    $version = Get-ProjectVersion
    $name = "$Project-$version-$Target.$Extension"
    $staging = Join-Path $work 'staging'
    [void][IO.Directory]::CreateDirectory($staging)

    Invoke-ProjectBuild $staging $version

    $changelog = Join-Path $root 'CHANGELOG.md'
    if (-not (Test-Path -LiteralPath $changelog)) { throw 'CHANGELOG.md introuvable à la racine.' }
    if ($Extension -eq 'zip') {
        Copy-Item -LiteralPath $changelog -Destination $staging
        if ($Notices) { Copy-Item -LiteralPath (Join-Path $root 'NOTICES.md') -Destination $staging }
    }

    $artifact = Join-Path $work $name
    if ($Extension -eq 'zip') {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [IO.Compression.ZipFile]::CreateFromDirectory($staging, $artifact, [IO.Compression.CompressionLevel]::Optimal, $false)
    }
    else {
        $files = @(Get-ChildItem -LiteralPath $staging -File)
        if ($files.Count -ne 1) { throw "Un fichier unique attendu dans $staging pour l'extension .$Extension, trouvé $($files.Count)." }
        Copy-Item -LiteralPath $files[0].FullName -Destination $artifact
        # Fichier unique : CHANGELOG.md et NOTICES.md (s'il existe) l'accompagnent, à côté, pour Verify-Livrable.
        Copy-Item -LiteralPath $changelog -Destination $work
        if (Test-Path -LiteralPath (Join-Path $root 'NOTICES.md')) { Copy-Item -LiteralPath (Join-Path $root 'NOTICES.md') -Destination $work }
    }

    $hash = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$artifact.sha256", "$hash  $name`n", [Text.UTF8Encoding]::new($false))

    $verifyArgs = @{ Path = $artifact; Project = $Project; Version = $version }
    if ($Expected.Count -gt 0) { $verifyArgs.Expected = $Expected }
    if ($AllowPath.Count -gt 0) { $verifyArgs.AllowPath = $AllowPath }
    if (-not $Notices) { $verifyArgs.NoNotices = $true }
    & (Find-CommonTool 'Verify-Livrable.ps1') @verifyArgs
    if ($LASTEXITCODE -ne 0) { throw "Verify-Livrable.ps1 a refusé le livrable (code $LASTEXITCODE)." }

    $out = Join-Path $root 'livrable'
    [void][IO.Directory]::CreateDirectory($out)
    Copy-Item -LiteralPath $artifact -Destination $out -Force
    Copy-Item -LiteralPath "$artifact.sha256" -Destination $out -Force
    if ($Extension -ne 'zip') {
        foreach ($doc in 'CHANGELOG.md', 'NOTICES.md') {
            if (Test-Path -LiteralPath (Join-Path $work $doc)) { Copy-Item -LiteralPath (Join-Path $work $doc) -Destination $out -Force }
        }
    }
    Write-Output "Livrable : livrable/$name"
    Write-Output "SHA-256 : $hash"
    exit 0
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 1
}
finally {
    if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue }
}
