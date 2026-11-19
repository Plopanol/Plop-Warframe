<#
.SYNOPSIS
Contrôle les liens Markdown relatifs : fichier cible, ancre de titre, alias explicites.

.DESCRIPTION
-Path désigne un dossier. Si ce dossier contient AGENTS.md (mode projet), sont contrôlés AGENTS.md,
CLAUDE.md, docs/ et .agents/skills/. Sinon (mode dossier), tous les .md du dossier, récursivement
(coffre memory/ par exemple).

Ancres reconnues : slug des titres calculé comme GitHub (minuscules, ponctuation retirée hors tiret et
souligné, espaces en tirets, suffixe -1, -2 pour les doublons) et alias explicites
<a id="..."></a> ou <a name="...">. Les blocs de code et les commentaires HTML sont ignorés.
Les gabarits de fiche (TASK-XXX-*.md, BUG-XXX*.md, ADR-XXX*.md, nom avec XXX littéral) sont ignorés.

Code de sortie : 0 aucun lien cassé, 1 au moins un lien cassé, 2 erreur.
#>
[CmdletBinding()]
param(
    [string]$Path = '.',
    [ValidateSet('auto', 'projet', 'dossier')][string]$Mode = 'auto'
)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false)

# Cible d'un lien en ligne : [texte](cible "titre") ou [texte](<cible avec espaces>) ; les images sont exclues.
$linkRegex = [regex]::new('(?<!!)\[[^\]]*\]\((<[^>]+>|[^)\s]+)(?:\s+"[^"]*")?\)', 'Compiled')
$codeSpanRegex = [regex]::new('`[^`]*`', 'Compiled')
$commentRegex = [regex]::new('<!--.*?-->', 'Compiled')
$headingRegex = [regex]::new('^ {0,3}(?:>\s*)*#{1,6}[ \t]+(.+?)(?:[ \t]+#+)?[ \t]*$', 'Compiled')
$aliasRegex = [regex]::new('<[A-Za-z][^>]*?\b(?:id|name)\s*=\s*["'']([^"'']+)["'']', 'Compiled')
$templateRegex = [regex]::new('^(?:TASK|BUG|ADR)-XXX(?:[-.]|$)', 'Compiled')

# Algorithme de GitHub : le texte rendu du titre, en minuscules, sans ponctuation hors tiret et souligné.
function Get-Slug([string]$heading) {
    $t = [regex]::Replace($heading, '!?\[([^\]]*)\]\([^)]*\)', '$1')
    $t = [regex]::Replace($t, '<[^>]+>', '')
    $t = [Net.WebUtility]::HtmlDecode($t)
    $t = $t.Replace('`', '')
    $t = $t.Replace('*', '')
    # Les soulignés d'emphase disparaissent ; ceux d'un identifiant (item_bases) restent.
    $t = [regex]::Replace($t, '(?<![\p{L}\p{Nd}])_+|_+(?![\p{L}\p{Nd}])', '')
    $t = $t.Trim().ToLowerInvariant()
    $t = [regex]::Replace($t, '[^\p{L}\p{M}\p{Nd}\p{Nl}\p{Pc}\- ]', '')
    return $t.Replace(' ', '-')
}

# Lignes du fichier avec blocs de code et commentaires HTML vidés (numéros de ligne conservés).
$maskCache = @{}
function Get-MaskedLines([string]$file) {
    if ($maskCache.ContainsKey($file)) { return , $maskCache[$file] }
    $lines = [IO.File]::ReadAllLines($file, $utf8)
    $fence = $false
    $comment = $false
    for ($i = 0; $i -lt $lines.Length; $i++) {
        $line = $lines[$i]
        $trimmed = $line.TrimStart()
        if ($trimmed.StartsWith('```') -or $trimmed.StartsWith('~~~')) { $fence = -not $fence; $lines[$i] = ''; continue }
        if ($fence) { $lines[$i] = ''; continue }
        if ($comment) {
            $end = $line.IndexOf('-->')
            if ($end -lt 0) { $lines[$i] = ''; continue }
            $line = $line.Substring($end + 3)
            $comment = $false
        }
        if ($line.IndexOf('<!--') -ge 0) {
            $line = $commentRegex.Replace($line, '')
            $start = $line.IndexOf('<!--')
            if ($start -ge 0) { $line = $line.Substring(0, $start); $comment = $true }
        }
        $lines[$i] = $line
    }
    $maskCache[$file] = $lines
    return , $lines
}

$anchorCache = @{}
function Get-Anchors([string]$file) {
    if (-not $anchorCache.ContainsKey($file)) {
        $slugs = [Collections.Generic.HashSet[string]]::new()
        $aliases = [Collections.Generic.HashSet[string]]::new()
        foreach ($line in (Get-MaskedLines $file)) {
            if ($line.Length -eq 0) { continue }
            if ($line.IndexOf('<') -ge 0) {
                foreach ($m in $aliasRegex.Matches($line)) { [void]$aliases.Add($m.Groups[1].Value) }
            }
            $m = $headingRegex.Match($line)
            if (-not $m.Success) { continue }
            $slug = Get-Slug $m.Groups[1].Value
            $candidate = $slug
            for ($n = 1; -not $slugs.Add($candidate); $n++) { $candidate = "$slug-$n" }
        }
        $anchorCache[$file] = @{ Slugs = $slugs; Aliases = $aliases }
    }
    return $anchorCache[$file]
}

function Get-Relative([string]$base, [string]$full) {
    $b = $base.TrimEnd('\', '/')
    if ($full.Length -gt $b.Length -and $full.StartsWith($b, [StringComparison]::OrdinalIgnoreCase)) { return $full.Substring($b.Length + 1).Replace([string][char]92, '/') }
    return $full.Replace([string][char]92, '/')
}

function Get-MarkdownFiles([string]$dir) {
    $result = [Collections.Generic.List[string]]::new()
    $stack = [Collections.Generic.Stack[string]]::new()
    $stack.Push($dir)
    while ($stack.Count -gt 0) {
        $current = $stack.Pop()
        foreach ($f in [IO.Directory]::GetFiles($current, '*.md')) { $result.Add($f) }
        foreach ($d in [IO.Directory]::GetDirectories($current)) {
            $leaf = [IO.Path]::GetFileName($d)
            if ($leaf -in '.git', 'node_modules', 'target') { continue }
            $stack.Push($d)
        }
    }
    $sorted = $result.ToArray()
    [Array]::Sort($sorted, [StringComparer]::OrdinalIgnoreCase)
    return $sorted
}

try {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) { throw "Dossier introuvable : $Path" }
    $root = (Resolve-Path -LiteralPath $Path).Path
    if ($Mode -eq 'auto') { $Mode = if (Test-Path -LiteralPath (Join-Path $root 'AGENTS.md')) { 'projet' } else { 'dossier' } }

    $files = [Collections.Generic.List[string]]::new()
    if ($Mode -eq 'projet') {
        foreach ($name in 'AGENTS.md', 'CLAUDE.md') {
            $candidate = Join-Path $root $name
            if (Test-Path -LiteralPath $candidate -PathType Leaf) { $files.Add($candidate) }
        }
        foreach ($sub in 'docs', '.agents\skills') {
            $dir = Join-Path $root $sub
            if (Test-Path -LiteralPath $dir -PathType Container) { foreach ($f in (Get-MarkdownFiles $dir)) { $files.Add($f) } }
        }
    }
    else {
        foreach ($f in (Get-MarkdownFiles $root)) { $files.Add($f) }
    }
    $files = @($files | Where-Object { -not $templateRegex.IsMatch([IO.Path]::GetFileName($_)) })

    $broken = 0
    $count = 0
    $existsCache = @{}
    foreach ($file in $files) {
        $relative = Get-Relative $root $file
        $fileDir = [IO.Path]::GetDirectoryName($file)
        $lines = Get-MaskedLines $file
        for ($i = 0; $i -lt $lines.Length; $i++) {
            $line = $lines[$i]
            if ($line.IndexOf('](') -lt 0) { continue }
            if ($line.IndexOf('`') -ge 0) { $line = $codeSpanRegex.Replace($line, '') }
            foreach ($match in $linkRegex.Matches($line)) {
                $target = $match.Groups[1].Value.Trim('<', '>')
                if ($target -match '^[A-Za-z][A-Za-z0-9+.-]*:' -or $target.StartsWith('//')) { continue }
                $count++
                $hash = $target.IndexOf('#')
                $pathPart = if ($hash -ge 0) { $target.Substring(0, $hash) } else { $target }
                $anchor = if ($hash -ge 0) { $target.Substring($hash + 1) } else { '' }
                try { $pathPart = [Uri]::UnescapeDataString($pathPart); $anchor = [Uri]::UnescapeDataString($anchor) } catch { }
                if ($pathPart.Length -eq 0) { $resolved = $file }
                elseif ($pathPart.StartsWith('/')) { $resolved = [IO.Path]::GetFullPath((Join-Path $root $pathPart.TrimStart('/'))) }
                else { $resolved = [IO.Path]::GetFullPath((Join-Path $fileDir $pathPart)) }
                if (-not $existsCache.ContainsKey($resolved)) {
                    $existsCache[$resolved] = if ([IO.File]::Exists($resolved)) { 'file' } elseif ([IO.Directory]::Exists($resolved)) { 'dir' } else { 'none' }
                }
                $kind = $existsCache[$resolved]
                $reason = $null
                if ($kind -eq 'none') { $reason = 'cible absente' }
                elseif ($anchor.Length -gt 0 -and $kind -eq 'file' -and $resolved.EndsWith('.md', [StringComparison]::OrdinalIgnoreCase)) {
                    $anchors = Get-Anchors $resolved
                    if (-not ($anchors.Aliases.Contains($anchor) -or $anchors.Slugs.Contains($anchor.ToLowerInvariant()))) { $reason = 'ancre absente' }
                }
                if ($reason) { Write-Output ('{0}:{1} → {2} ({3})' -f $relative, ($i + 1), $target, $reason); $broken++ }
            }
        }
    }
    Write-Output ('{0} fichiers, {1} liens, {2} cassés' -f $files.Count, $count, $broken)
    exit [int]($broken -gt 0)
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 2
}
