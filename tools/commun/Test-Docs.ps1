<#
.SYNOPSIS
Contrôle documentaire unique d'un projet Plop ou du coffre de mémoire.

.DESCRIPTION
Recopié dans tools/commun/ de chaque projet par Sync-Commun.ps1 ; il n'a besoin ni de _modele-projet ni
d'aucun chemin absolu. Appelé par le hook pre-commit : pwsh -File tools/commun/Test-Docs.ps1 -Path .

Mode projet (AGENTS.md présent) :
  1. conformité au manifeste tools/commun/commun.manifest.json : blocs d'AGENTS.md, CLAUDE.md, skills,
     configuration et outils communs (pour .gitignore, .gitattributes, .editorconfig et .codex/config.toml,
     seul le bloc « commun:debut » … « commun:fin » est comparé), sauf écarts déclarés (<!-- commun-ecart: <chemin> — <raison> -->) ;
  2. liens Markdown, par Test-Liens.ps1 placé à côté ;
  3. cohérence fiches et index, par check_docs.mjs placé à côté (non exécuté s'il est absent ou sans node) ;
  4. cohérence roadmap et fiches, par check_roadmap.mjs placé à côté (non exécuté en profil léger, absent ou
     sans node) ; séparé du 3 pour que l'écart de roadmap ne se mêle pas à celui des index.
Mode coffre (INDEX.md sans AGENTS.md, par exemple _modele-projet/memory) :
  liens de toutes les fiches, fiches absentes d'INDEX.md, frontmatter obligatoire (type, themes, projets,
  statut, date, source), doublons de titre.

Chaque contrôle est affiché réussi, échoué ou non exécuté. Code de sortie : 0 aucun échec, 1 au moins un
échec, 2 erreur.
#>
[CmdletBinding()]
param(
    [string]$Path = '.',
    [ValidateSet('auto', 'projet', 'coffre')][string]$Mode = 'auto'
)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false)
$em = [string][char]0x2014
$en = [string][char]0x2013
$maxDetails = 30

function ConvertTo-Lf([string]$s) { return $s.Replace("`r`n", "`n").Replace("`r", "`n") }
function Read-Lf([string]$path) { return ConvertTo-Lf ([IO.File]::ReadAllText($path, $utf8)) }
function Get-Sha([string]$text) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($sha.ComputeHash($utf8.GetBytes($text))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}
function Get-Relative([string]$base, [string]$full) {
    $b = $base.TrimEnd('\', '/')
    if ($full.Length -gt $b.Length -and $full.StartsWith($b, [StringComparison]::OrdinalIgnoreCase)) { return $full.Substring($b.Length + 1).Replace('\', '/') }
    return $full.Replace('\', '/')
}
function Get-MarkdownFiles([string]$dir) {
    $result = [Collections.Generic.List[string]]::new()
    $stack = [Collections.Generic.Stack[string]]::new()
    $stack.Push($dir)
    while ($stack.Count -gt 0) {
        $current = $stack.Pop()
        foreach ($f in [IO.Directory]::GetFiles($current, '*.md')) { $result.Add($f) }
        foreach ($d in [IO.Directory]::GetDirectories($current)) {
            if ([IO.Path]::GetFileName($d) -in '.git', 'node_modules', 'target') { continue }
            $stack.Push($d)
        }
    }
    $sorted = $result.ToArray()
    [Array]::Sort($sorted, [StringComparer]::OrdinalIgnoreCase)
    return $sorted
}

$results = [Collections.Generic.List[object]]::new()
function Add-Result([string]$name, [string]$status, [string[]]$details) {
    $results.Add(@{ Name = $name; Status = $status; Details = @($details | Where-Object { $_ }) })
}

# Écarts déclarés dans AGENTS.md : chemin -> raison.
function Read-Ecarts([string]$text) {
    $ecarts = @{}
    $re = '(?m)^<!--\s*commun-ecart\s*:\s*(\S+)\s+(?:' + $em + '|' + $en + '|--|-)\s+(.+?)\s*-->[ \t]*$'
    foreach ($m in [regex]::Matches($text, $re)) { $ecarts[$m.Groups[1].Value.Replace('\', '/')] = $m.Groups[2].Value }
    return $ecarts
}

function Test-Manifest([string]$root) {
    $name = 'conformité au manifeste'
    $agents = Read-Lf (Join-Path $root 'AGENTS.md')
    $decl = [regex]::Match($agents, '(?m)^<!--\s*commun\s*:\s*(.*?)\s*-->[ \t]*$')
    if (-not $decl.Success) { Add-Result $name 'non exécuté' @('AGENTS.md sans déclaration commun : projet sans commun'); return }
    $manifestPath = $null
    foreach ($candidate in @((Join-Path $root 'tools\commun\commun.manifest.json'), (Join-Path $root 'tool\commun\commun.manifest.json'), (Join-Path $PSScriptRoot 'commun.manifest.json'))) {
        if ([IO.File]::Exists($candidate)) { $manifestPath = $candidate; break }
    }
    if (-not $manifestPath) { Add-Result $name 'échoué' @('manifeste absent : lancer Sync-Commun.ps1 -Apply depuis _modele-projet'); return }
    $manifest = [IO.File]::ReadAllText($manifestPath, $utf8) | ConvertFrom-Json
    $ecarts = Read-Ecarts $agents
    $problems = [Collections.Generic.List[string]]::new()
    $declared = ($decl.Groups[1].Value -replace '\s+', ' ').Trim()
    if ($declared -cne [string]$manifest.declaration) { $problems.Add("AGENTS.md (déclaration modifiée depuis la synchronisation : relancer Sync-Commun.ps1)") }
    foreach ($b in @($manifest.blocs)) {
        if (-not $b) { continue }
        $key = "AGENTS.md#$($b.nom)"
        if ($ecarts.ContainsKey($key)) { continue }
        $m = [regex]::Match($agents, ('(?s)<!-- {0}:debut v\d+ -->.*?<!-- {0}:fin -->' -f [regex]::Escape($b.nom)))
        if (-not $m.Success) { $problems.Add("$key (bloc manquant)") }
        elseif ((Get-Sha $m.Value) -cne $b.sha256) { $problems.Add("$key (bloc modifié à la main)") }
    }
    foreach ($f in @($manifest.fichiers)) {
        if (-not $f -or $ecarts.ContainsKey($f.chemin)) { continue }
        $full = Join-Path $root ($f.chemin.Replace('/', '\'))
        if (-not [IO.File]::Exists($full)) { $problems.Add("$($f.chemin) (absent)"); continue }
        if ($f.PSObject.Properties['bloc']) {
            # Fichier que le projet complète : seul le bloc balisé est comparé, le reste est libre.
            $e = [regex]::Escape([string]$f.bloc)
            $m = [regex]::Match((Read-Lf $full), ('(?ms)^' + $e + ' commun:debut v\d+[ \t]*\n.*?^' + $e + ' commun:fin[ \t]*$'))
            if (-not $m.Success) { $problems.Add("$($f.chemin) (bloc commun manquant)") }
            elseif ((Get-Sha $m.Value) -cne $f.sha256) { $problems.Add("$($f.chemin) (bloc commun modifié à la main)") }
        }
        elseif ((Get-Sha (Read-Lf $full)) -cne $f.sha256) { $problems.Add("$($f.chemin) (modifié à la main)") }
    }
    if ($problems.Count -eq 0) { Add-Result $name 'réussi' @() }
    else { Add-Result $name 'échoué' (@($problems) + 'Corriger dans _modele-projet puis relancer Sync-Commun.ps1, ou déclarer un écart dans AGENTS.md.') }
}

function Test-LiensMarkdown([string]$root, [string]$linkMode) {
    $name = 'liens Markdown'
    $liensScript = Join-Path $PSScriptRoot 'Test-Liens.ps1'
    if (-not [IO.File]::Exists($liensScript)) { Add-Result $name 'non exécuté' @('Test-Liens.ps1 absent à côté de Test-Docs.ps1'); return }
    $output = @(& $liensScript -Path $root -Mode $linkMode)
    $code = $LASTEXITCODE
    $summary = if ($output.Count -gt 0) { [string]$output[$output.Count - 1] } else { '' }
    if ($code -eq 0) { Add-Result $name 'réussi' @($summary) }
    elseif ($code -eq 1) { Add-Result $name 'échoué' @($output | ForEach-Object { [string]$_ }) }
    else { Add-Result $name 'échoué' @("Test-Liens.ps1 a échoué (code $code)") }
}

# Lance un outil Node de tools/commun/ depuis la racine du projet ; les outils lisent le dossier courant.
function Invoke-NodeCheck([string]$root, [string]$scriptName, [string]$name) {
    $js = Join-Path $PSScriptRoot $scriptName
    if (-not [IO.File]::Exists($js)) { Add-Result $name 'non exécuté' @("$scriptName absent à côté de Test-Docs.ps1"); return $null }
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) { Add-Result $name 'non exécuté' @('node introuvable'); return $null }
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    Push-Location $root
    try { $output = @(& node $js 2>&1 | ForEach-Object { [string]$_ }); $code = $LASTEXITCODE }
    finally { Pop-Location; $ErrorActionPreference = $previous }
    return @{ Code = $code; Output = $output }
}

function Test-FichesIndex([string]$root) {
    $name = 'cohérence fiches et index'
    $run = Invoke-NodeCheck $root 'check_docs.mjs' $name
    if (-not $run) { return }
    if ($run.Code -eq 0) { Add-Result $name 'réussi' @() } else { Add-Result $name 'échoué' $run.Output }
}

function Test-Roadmap([string]$root) {
    $name = 'cohérence roadmap et fiches'
    $run = Invoke-NodeCheck $root 'check_roadmap.mjs' $name
    if (-not $run) { return }
    if ($run.Code -eq 0 -and ($run.Output -join "`n") -match '^Non applicable') { Add-Result $name 'non exécuté' @($run.Output); return }
    if ($run.Code -eq 0) { Add-Result $name 'réussi' @() } else { Add-Result $name 'échoué' $run.Output }
}

# --- Coffre -----------------------------------------------------------------------------------------------
function Get-Frontmatter([string]$text) {
    $m = [regex]::Match($text, '(?s)\A---\n(.*?)\n---(?:\n|\z)')
    if (-not $m.Success) { return $null }
    $fields = @{}
    foreach ($line in $m.Groups[1].Value.Split("`n")) {
        $kv = [regex]::Match($line, '^([A-Za-z_][\w-]*):[ \t]*(.*?)[ \t]*$')
        if ($kv.Success) { $fields[$kv.Groups[1].Value] = $kv.Groups[2].Value }
    }
    return @{ Fields = $fields; Body = $text.Substring($m.Length) }
}

function Test-Coffre([string]$root) {
    Test-LiensMarkdown $root 'dossier'

    $all = @(Get-MarkdownFiles $root)
    $fiches = @($all | Where-Object { $n = [IO.Path]::GetFileName($_); $n -ne 'INDEX.md' -and -not $n.StartsWith('_') })
    $parsed = @{}
    foreach ($f in $fiches) { $parsed[$f] = Get-Frontmatter (Read-Lf $f) }

    # Fiches absentes de l'index (une fiche au statut faux sort de l'index).
    $indexText = Read-Lf (Join-Path $root 'INDEX.md')
    $indexed = @{}
    foreach ($m in [regex]::Matches($indexText, '\]\(<?([^)\s>#]+)(?:#[^)\s>]*)?>?\)')) {
        $target = $m.Groups[1].Value
        if ($target -match '^[A-Za-z][A-Za-z0-9+.-]*:') { continue }
        try { $full = [IO.Path]::GetFullPath((Join-Path $root ([Uri]::UnescapeDataString($target).Replace('/', '\')))) } catch { continue }
        $indexed[$full.ToLowerInvariant()] = $true
    }
    $missing = [Collections.Generic.List[string]]::new()
    foreach ($f in $fiches) {
        $fm = $parsed[$f]
        if ($fm -and $fm.Fields['statut'] -eq 'faux') { continue }
        if (-not $indexed.ContainsKey($f.ToLowerInvariant())) { $missing.Add("$(Get-Relative $root $f) (absente de INDEX.md)") }
    }
    if ($missing.Count -eq 0) { Add-Result 'fiches présentes dans INDEX.md' 'réussi' @() } else { Add-Result 'fiches présentes dans INDEX.md' 'échoué' $missing }

    # Frontmatter des fiches de leçon ; les pages de projet (projets/) et les gabarits (_*) en sont exemptés.
    $required = 'type', 'themes', 'projets', 'statut', 'date', 'source'
    $types = 'piege', 'recette', 'regle', 'reference'
    $statuts = 'verifie', 'partiel', 'faux'
    $bad = [Collections.Generic.List[string]]::new()
    foreach ($f in $fiches) {
        $rel = Get-Relative $root $f
        if ($rel.StartsWith('projets/')) { continue }
        $fm = $parsed[$f]
        if (-not $fm) { $bad.Add("$rel (frontmatter absent)"); continue }
        foreach ($key in $required) {
            if (-not $fm.Fields.ContainsKey($key)) { $bad.Add("$rel (champ $key absent)") }
            elseif ($key -ne 'projets' -and -not $fm.Fields[$key]) { $bad.Add("$rel (champ $key vide)") }
        }
        if ($fm.Fields['type'] -and $fm.Fields['type'] -notin $types) { $bad.Add("$rel (type « $($fm.Fields['type']) » inconnu)") }
        if ($fm.Fields['statut'] -and $fm.Fields['statut'] -notin $statuts) { $bad.Add("$rel (statut « $($fm.Fields['statut']) » inconnu)") }
        if ($fm.Fields['date'] -and $fm.Fields['date'] -notmatch '^\d{4}-\d{2}-\d{2}$') { $bad.Add("$rel (date « $($fm.Fields['date']) » hors format AAAA-MM-JJ)") }
    }
    if ($bad.Count -eq 0) { Add-Result 'frontmatter des fiches' 'réussi' @() } else { Add-Result 'frontmatter des fiches' 'échoué' $bad }

    # Doublons de titre (premier titre de niveau 1).
    $titles = @{}
    foreach ($f in $fiches) {
        $fm = $parsed[$f]
        $body = if ($fm) { $fm.Body } else { Read-Lf $f }
        $t = [regex]::Match($body, '(?m)^#[ \t]+(.+?)[ \t]*$')
        if (-not $t.Success) { continue }
        $key = $t.Groups[1].Value.ToLowerInvariant()
        if (-not $titles.ContainsKey($key)) { $titles[$key] = [Collections.Generic.List[string]]::new() }
        $titles[$key].Add((Get-Relative $root $f))
    }
    $dups = @($titles.Keys | Where-Object { $titles[$_].Count -gt 1 } | Sort-Object | ForEach-Object { "« $_ » : $($titles[$_] -join ', ')" })
    if ($dups.Count -eq 0) { Add-Result 'titres uniques' 'réussi' @() } else { Add-Result 'titres uniques' 'échoué' $dups }
}

try {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) { throw "Dossier introuvable : $Path" }
    $root = (Resolve-Path -LiteralPath $Path).Path
    if ($Mode -eq 'auto') {
        if ([IO.File]::Exists((Join-Path $root 'AGENTS.md'))) { $Mode = 'projet' }
        elseif ([IO.File]::Exists((Join-Path $root 'INDEX.md'))) { $Mode = 'coffre' }
        else { throw "Ni AGENTS.md ni INDEX.md dans $root" }
    }
    if ($Mode -eq 'projet') {
        if (-not [IO.File]::Exists((Join-Path $root 'AGENTS.md'))) { throw "AGENTS.md introuvable dans $root" }
        Test-Manifest $root
        Test-LiensMarkdown $root 'projet'
        Test-FichesIndex $root
        Test-Roadmap $root
    }
    else {
        if (-not [IO.File]::Exists((Join-Path $root 'INDEX.md'))) { throw "INDEX.md introuvable dans $root" }
        Test-Coffre $root
    }

    $failed = 0; $passed = 0; $skipped = 0
    foreach ($r in $results) {
        Write-Output ('{0,-12} {1}' -f $r.Status, $r.Name)
        if ($r.Status -eq 'réussi') { $passed++ } elseif ($r.Status -eq 'échoué') { $failed++ } else { $skipped++ }
        $shown = 0
        foreach ($d in $r.Details) {
            if ($r.Status -eq 'réussi' -and $shown -ge 1) { break }
            if ($shown -ge $maxDetails) { Write-Output ('{0,-12} … {1} lignes de plus' -f '', ($r.Details.Count - $shown)); break }
            Write-Output ('{0,-12} {1}' -f '', $d)
            $shown++
        }
    }
    Write-Output ('Bilan : {0} réussi(s), {1} échoué(s), {2} non exécuté(s)' -f $passed, $failed, $skipped)
    exit [int]($failed -gt 0)
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 2
}
