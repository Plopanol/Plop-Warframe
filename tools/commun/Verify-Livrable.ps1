<#
.SYNOPSIS
Vérifie un livrable : nom, empreinte .sha256, contenu minimal et liste de fichiers attendus.

.DESCRIPTION
Appelé par tools/livrable.ps1 de chaque projet, à la racine du dépôt, après le conditionnement :
  pwsh -File tools/commun/Verify-Livrable.ps1 -Path livrable/app-1.2.3-x86_64-pc-windows-msvc.zip -Project app -Version 1.2.3 -Expected app.exe,LICENSE
Règles de modules/livrable.md :
  - nom <projet>-<version>-<cible>.<ext> ; -Version, obligatoire, et -Project, s'il est donné, doivent correspondre
    et servent à découper le nom, parce qu'une cible à tiret (windows-x64) est indiscernable d'une préversion ;
  - <nom>.sha256 à côté : empreinte SHA-256 recalculée égale (format « <hash> » ou « <hash>  <nom> ») ;
  - CHANGELOG.md, et NOTICES.md sauf -NoNotices, à la racine de l'archive ou à côté du livrable ;
  - aucune donnée utilisateur : entrées data/, .git/, target/, node_modules/ et motifs de -Forbidden refusés ;
    -AllowPath (expressions régulières) exempte de ce refus par défaut les entrées qui y correspondent, pour une
    ressource d'exécution au nom réservé (data/ de Flutter Windows) ; -Forbidden s'applique toujours.
-Expected accepte une liste séparée par des virgules (« pwsh -File » ne construit pas de tableau) ; -ExpectedHashes
n'est utilisable que par un appel direct (& tools/commun/Verify-Livrable.ps1 ...) depuis livrable.ps1.
Pour une archive .zip : sans -Expected, seul le minimum est exigé ; avec -Expected (noms exacts relatifs à la
racine de l'archive, jokers * admis), le contenu doit lui correspondre exactement (-AllowExtra tolère les
fichiers en plus). -ExpectedHashes (table nom vers SHA-256) contrôle l'empreinte d'entrées précises.
Une entrée vide, dupliquée ou à chemin absolu ou remontant (..) est refusée.

Pourquoi un script commun : chaque projet recopiait la même vérification d'archive ; la règle du livrable ne
doit changer qu'à un endroit. Codes de sortie : 0 conforme, 1 écart, 2 erreur d'usage ou fichier illisible.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Path,
    [string]$Project,
    [string]$Version,
    [string[]]$Expected,
    [hashtable]$ExpectedHashes,
    [string[]]$Forbidden = @(),
    [string[]]$AllowPath = @(),
    [switch]$AllowExtra,
    [switch]$NoNotices
)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'

function Resolve-Input([string]$path) {
    if ([IO.Path]::IsPathRooted($path)) { return [IO.Path]::GetFullPath($path) }
    return [IO.Path]::GetFullPath((Join-Path $PWD.ProviderPath $path))
}

function Get-Sha256([IO.Stream]$stream) {
    $digest = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($digest.ComputeHash($stream)).Replace('-', '').ToLowerInvariant() }
    finally { $digest.Dispose() }
}

$Expected = @($Expected | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$AllowPath = @($AllowPath | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$problems = [Collections.Generic.List[string]]::new()
$checked = [Collections.Generic.List[string]]::new()

try {
    $archive = Resolve-Input $Path
    if (-not [IO.File]::Exists($archive)) { throw "Livrable introuvable : $Path" }
    $name = [IO.Path]::GetFileName($archive)
    $directory = [IO.Path]::GetDirectoryName($archive)

    # Name: -Version is the authority, because a hyphenated target (windows-x64) cannot be told apart from a
    # pre-release by reading the name alone.
    # Checked here rather than by Mandatory, so a missing value exits with code 2 instead of prompting.
    if (-not $Version) { throw '-Version est obligatoire' }
    if ($Version -notmatch'^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$') { throw "-Version n'est pas une version SemVer : $Version" }
    $projectPart = if ($Project) { [regex]::Escape($Project) } else { '.+?' }
    $parts = [regex]::Match($name, ('^(?<projet>' + $projectPart + ')-(?<version>' + [regex]::Escape($Version) + ')-(?<cible>[A-Za-z0-9_][A-Za-z0-9_.-]*?)\.(?<ext>[A-Za-z0-9]+)$'))
    if (-not $parts.Success) {
        # Mismatch: re-read the name by deduction so the report says which part differs; a pre-release is
        # left to the target there, since only -Version can tell them apart.
        $parts = [regex]::Match($name, '^(?<projet>.+?)-(?<version>\d+\.\d+\.\d+)-(?<cible>[A-Za-z0-9_][A-Za-z0-9_.-]*?)\.(?<ext>[A-Za-z0-9]+)$')
    }
    if (-not $parts.Success) { $problems.Add("Nom non conforme à <projet>-<version>-<cible>.<ext> : $name") }
    else {
        if ($Project -and $parts.Groups['projet'].Value -cne $Project) { $problems.Add("Projet du nom « $($parts.Groups['projet'].Value) » au lieu de « $Project »") }
        if ($Version -and $parts.Groups['version'].Value -cne $Version) { $problems.Add("Version du nom « $($parts.Groups['version'].Value) » au lieu de « $Version »") }
        $checked.Add('nom')
    }

    # Hash: the .sha256 sits next to the deliverable; a file name it cites must be the deliverable's.
    $shaFile = "$archive.sha256"
    $stream = [IO.File]::OpenRead($archive)
    try { $actual = Get-Sha256 $stream } finally { $stream.Dispose() }
    if (-not [IO.File]::Exists($shaFile)) { $problems.Add("Empreinte absente : $name.sha256") }
    else {
        $line = ([IO.File]::ReadAllText($shaFile)).Trim()
        $m = [regex]::Match($line, '^(?<hash>[0-9A-Fa-f]{64})(?:\s+\*?(?<file>.+))?$')
        if (-not $m.Success) { $problems.Add("$name.sha256 illisible : « <hash> » ou « <hash>  <nom> » attendu") }
        else {
            if ($m.Groups['hash'].Value.ToLowerInvariant() -cne $actual) { $problems.Add("Empreinte divergente : $name.sha256 indique $($m.Groups['hash'].Value.ToLowerInvariant()), le fichier vaut $actual") }
            if ($m.Groups['file'].Success -and $m.Groups['file'].Value.Trim() -cne $name) { $problems.Add("$name.sha256 cite un autre fichier : $($m.Groups['file'].Value.Trim())") }
            $checked.Add('empreinte')
        }
    }

    # Archive content.
    $isZip = $name.EndsWith('.zip', [StringComparison]::OrdinalIgnoreCase)
    $entries = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::Ordinal)
    if ($isZip) {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $zip = [IO.Compression.ZipFile]::OpenRead($archive)
        try {
            foreach ($entry in $zip.Entries) {
                $full = $entry.FullName
                if ($full.EndsWith('/')) { continue }
                if ($full -match '(^|/)\.\.?(/|$)' -or $full.StartsWith('/') -or $full -match '^[A-Za-z]:' -or $full.Contains('\')) { $problems.Add("Chemin d'entrée interdit : $full"); continue }
                if ($entries.ContainsKey($full)) { $problems.Add("Entrée dupliquée : $full"); continue }
                if ($entry.Length -eq 0) { $problems.Add("Entrée vide : $full") }
                $allowed = [bool]($AllowPath | Where-Object { $full -match $_ })
                if (-not $allowed -and $full -match '(^|/)(data|\.git|target|node_modules)/') { $problems.Add("Donnée ou dossier de travail dans le paquet : $full") }
                foreach ($pattern in $Forbidden) { if ($full -match $pattern) { $problems.Add("Entrée interdite ($pattern) : $full") } }
                $entryStream = $entry.Open()
                try { $entries[$full] = Get-Sha256 $entryStream } finally { $entryStream.Dispose() }
            }
        } finally { $zip.Dispose() }
        if ($entries.Count -eq 0) { $problems.Add('Archive vide') }
        $checked.Add('archive')
    }
    elseif ($Expected.Count -gt 0) { $problems.Add("-Expected ne s'applique qu'à une archive .zip : $name") }

    # CHANGELOG.md and notices: at the archive root or next to the deliverable.
    $required = [Collections.Generic.List[string]]::new()
    $required.Add('CHANGELOG.md')
    if (-not $NoNotices) { $required.Add('NOTICES.md') }
    foreach ($doc in $required) {
        $inside = $entries.ContainsKey($doc)
        $beside = [IO.File]::Exists((Join-Path $directory $doc))
        if (-not ($inside -or $beside)) { $problems.Add("$doc absent : ni à la racine de l'archive ni à côté du livrable" + $(if ($doc -eq 'NOTICES.md') { " (-NoNotices si aucune dépendance tierce n'est embarquée)" } else { '' })) }
    }
    $checked.Add('contenu minimal')

    # Exact expected list, including the minimum files that are inside the archive.
    if ($isZip -and $Expected.Count -gt 0) {
        $patterns = @($Expected) + @($required | Where-Object { $entries.ContainsKey($_) })
        foreach ($pattern in $patterns) {
            if (-not ($entries.Keys | Where-Object { $_ -clike $pattern })) { $problems.Add("Entrée attendue absente : $pattern") }
        }
        if (-not $AllowExtra) {
            foreach ($key in $entries.Keys) {
                if (-not ($patterns | Where-Object { $key -clike $_ })) { $problems.Add("Entrée inattendue : $key") }
            }
        }
        $checked.Add('liste attendue')
    }
    if ($isZip -and $ExpectedHashes) {
        foreach ($key in $ExpectedHashes.Keys) {
            if (-not $entries.ContainsKey($key)) { $problems.Add("Entrée à empreinte imposée absente : $key") }
            elseif ($entries[$key] -cne ([string]$ExpectedHashes[$key]).ToLowerInvariant()) { $problems.Add("Empreinte divergente de la référence : $key") }
        }
        $checked.Add('empreintes d''entrées')
    }

    if ($problems.Count -eq 0) {
        Write-Output "Livrable conforme : $name"
        Write-Output "SHA-256 : $actual"
        Write-Output ('Contrôles : ' + ($checked -join ', ') + $(if ($isZip) { " ; $($entries.Count) entrée(s)" } else { '' }))
        exit 0
    }
    foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
    [Console]::Error.WriteLine("Livrable non conforme : $($problems.Count) écart(s) dans $name")
    exit 1
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 2
}
