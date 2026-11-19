<#
.SYNOPSIS
Contrôle une copie avant publication : secrets, identités et chemins propres au poste.

.DESCRIPTION
Parcourt les fichiers suivis (ou tous les fichiers hors dépôt Git), leurs noms et, avec -History,
l'historique complet et les identités des commits. Les valeurs confidentielles à rechercher se lisent
dans une liste locale ignorée par Git, pour ne jamais entrer dans le dépôt ; les correspondances sont
signalées sans afficher la valeur trouvée.

Liste des valeurs interdites, une par ligne (# pour un commentaire), cherchée dans cet ordre :
-ForbiddenList, puis <racine contrôlée>/.publication-interdits.txt, puis .publication-interdits.txt dans le
dossier parent du dépôt qui porte ce script (liste commune, hors de tout dépôt).

Dans un dépôt Git, les fichiers non suivis et non ignorés sont contrôlés aussi, puisqu'un `git add -A`
les publierait. Les fichiers au nom sensible (clés, magasins de certificats, .env) sont signalés.

Exceptions qualifiées (attributions tierces, fixtures synthétiques) : fichier versionné
<racine contrôlée>/.publication-exceptions, une ligne « <motif de chemin> | <règle> » ; la règle est le nom
affiché d'un motif générique, « valeur interdite » pour toutes les valeurs de la liste, ou « * ». Les
exceptions ne s'appliquent pas aux identités de commit.

Recopié dans tools/commun/ de chaque projet par Sync-Commun.ps1. Appel depuis la racine du projet :
    pwsh -File tools/commun/Test-Publication.ps1 -Path <copie> -History
Codes de sortie : 0 aucune correspondance, 1 correspondance, 2 erreur.
#>
param(
    [string]$Path = '.',
    [string]$ForbiddenList,
    [switch]$History
)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'
trap { Write-Output "Erreur : $_"; exit 2 }
$root = (Resolve-Path -LiteralPath $Path).Path
if (-not $ForbiddenList) {
    $candidates = @(Join-Path $root '.publication-interdits.txt')
    $scriptRepo = git -C $PSScriptRoot rev-parse --show-toplevel 2>$null
    if ($LASTEXITCODE -eq 0 -and $scriptRepo) {
        # The common list sits next to the repositories, outside any of them.
        $candidates += Join-Path (Split-Path -Parent $scriptRepo) '.publication-interdits.txt'
    }
    $ForbiddenList = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $ForbiddenList) { $ForbiddenList = $candidates[-1] }
}

# Generic patterns; the matched text is never printed.
$patterns = [ordered]@{
    'clé privée'                  = '-----BEGIN [A-Z ]*PRIVATE KEY-----'
    'jeton GitHub'                = '\b(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})'
    'clé AWS'                     = '\bAKIA[0-9A-Z]{16}\b'
    'jeton Slack'                 = '\bxox[abprs]-[A-Za-z0-9-]{10,}'
    'JWT (jeton TMDb ou autre)'   = '\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.'
    'URL avec identifiants'       = '\b[a-z][a-z0-9+.-]*://[^/\s:@''"]+:[^/\s@''"]+@'
    # Octets bounded to 255 and no digit group around, so dotted thousands (10.649.887.816) do not match.
    'adresse IPv4 privée'         = '(?<!\d|\d\.)(10\.(25[0-5]|2[0-4]\d|1?\d?\d)|192\.168|172\.(1[6-9]|2\d|3[01]))(\.(25[0-5]|2[0-4]\d|1?\d?\d)){2}(?!\d|\.\d)'
    'profil utilisateur Windows'  = '\b[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s<]+'
    'dossier personnel Unix'      = '/home/[a-z_][a-z0-9_-]*/'
    'courriel non générique'      = '(?<![A-Za-z0-9._%-])[A-Za-z0-9][A-Za-z0-9._%+-]*@(?!(users\.)?noreply\.|example\.|anthropic\.com)[A-Za-z0-9-]+\.[A-Za-z0-9.-]*[A-Za-z]'
}

$forbidden = @()
if (Test-Path -LiteralPath $ForbiddenList) {
    $forbidden = @(Get-Content -LiteralPath $ForbiddenList -Encoding utf8 |
            ForEach-Object { $_.Trim() } |
            Where-Object { $_ -and -not $_.StartsWith('#') })
}
else {
    Write-Warning "Liste locale des valeurs interdites absente : $ForbiddenList ; seuls les motifs génériques sont contrôlés."
}

$exceptions = @()
$exceptionFile = Join-Path $root '.publication-exceptions'
if (Test-Path -LiteralPath $exceptionFile) {
    $exceptions = @(Get-Content -LiteralPath $exceptionFile -Encoding utf8 |
            ForEach-Object { $_.Trim() } |
            Where-Object { $_ -and -not $_.StartsWith('#') -and $_.Contains('|') } |
            ForEach-Object {
                $parts = $_.Split('|', 2)
                [pscustomobject]@{ Path = $parts[0].Trim(); Rule = $parts[1].Trim() }
            })
}

# Names that usually hold credentials or signing material.
$sensitiveNames = '(^|/)(\.env(\..+)?|key\.properties|local\.properties|id_(rsa|ed25519|ecdsa)|.*\.(jks|keystore|pem|p12|pfx|key))$'

$findings = [System.Collections.Generic.List[string]]::new()

function Add-Finding([string]$File, [string]$Where, [string]$Rule) {
    if ($File) {
        foreach ($e in $exceptions) {
            $ruleMatches = $e.Rule -eq '*' -or $e.Rule -eq $Rule -or ($e.Rule -eq 'valeur interdite' -and $Rule.StartsWith('valeur interdite'))
            if ($ruleMatches -and $File -like $e.Path) { return }
        }
    }
    $findings.Add("$Where : $Rule")
}

function Test-Line([string]$Text, [string]$Where, [string]$File) {
    foreach ($name in $patterns.Keys) {
        if ($Text -match $patterns[$name]) { Add-Finding $File $Where $name }
    }
    for ($i = 0; $i -lt $forbidden.Count; $i++) {
        if ($Text.IndexOf($forbidden[$i], [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            Add-Finding $File $Where "valeur interdite n°$($i + 1)"
        }
    }
}

$isGit = Test-Path -LiteralPath (Join-Path $root '.git')
if ($isGit) {
    $files = @(git -C $root -c core.quotepath=off ls-files -c -o --exclude-standard | Sort-Object -Unique)
    if ($LASTEXITCODE -ne 0) { throw "git ls-files a échoué dans $root" }
}
else {
    $files = @(Get-ChildItem -LiteralPath $root -Recurse -File |
            Where-Object { $_.FullName -notmatch '[\\/](node_modules|target|\.git)[\\/]' } |
            ForEach-Object { $_.FullName.Substring($root.TrimEnd('\', '/').Length + 1) -replace '\\', '/' })
}

if ($files.Count -eq 0) {
    # An empty scan must never pass for a clean copy.
    throw "Aucun fichier à contrôler dans $root (fichiers non suivis ?)."
}

foreach ($file in $files) {
    Test-Line $file "$file (nom)" $file
    if ($file -match $sensitiveNames) { Add-Finding $file "$file (nom)" 'fichier sensible' }
    $full = Join-Path $root $file
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { continue }
    $bytes = [IO.File]::ReadAllBytes($full)
    $head = [Math]::Min($bytes.Length, 8000)
    if ($head -gt 0 -and [Array]::IndexOf($bytes, [byte]0, 0, $head) -ge 0) {
        # Binary file: only forbidden values are searched, and short ones would match random bytes.
        $text = [Text.Encoding]::GetEncoding(28591).GetString($bytes)
        for ($i = 0; $i -lt $forbidden.Count; $i++) {
            if ($forbidden[$i].Length -ge 6 -and $text.IndexOf($forbidden[$i], [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                Add-Finding $file "$file (binaire)" "valeur interdite n°$($i + 1)"
            }
        }
        continue
    }
    $lines = [Text.Encoding]::UTF8.GetString($bytes) -split "`r?`n"
    for ($n = 0; $n -lt $lines.Count; $n++) {
        Test-Line $lines[$n] "${file}:$($n + 1)" $file
    }
}

if ($History) {
    if (-not $isGit) { throw "-History exige un dépôt Git : $root" }
    $identities = @(git -C $root log --all --format='%an <%ae>%n%cn <%ce>' | Sort-Object -Unique)
    for ($k = 0; $k -lt $identities.Count; $k++) { Test-Line $identities[$k] "identité de commit n°$($k + 1)" '' }
    $commit = ''
    $patchFile = ''
    git -C $root -c core.quotepath=off log --all -p --format='commit %H%n%B' | ForEach-Object {
        if ($_ -match '^commit ([0-9a-f]{40})$') { $commit = $Matches[1].Substring(0, 10); $patchFile = ''; return }
        # Patch lines belong to a file, so the qualified exceptions apply to them too.
        if ($_ -match '^diff --git a/.+ b/(.+)$') { $patchFile = $Matches[1] }
        Test-Line $_ "historique $commit" $patchFile
    }
    $refs = git -C $root for-each-ref --format='%(refname)'
    foreach ($ref in $refs) { Test-Line $ref 'référence' '' }
}

$unique = $findings | Sort-Object -Unique
if ($unique) {
    $unique | ForEach-Object { Write-Output $_ }
    Write-Output "Échec : $($unique.Count) correspondance(s) dans $root."
    exit 1
}
Write-Output "Aucune correspondance dans $root ($($files.Count) fichiers$(if ($History) { ', historique inclus' }))."
