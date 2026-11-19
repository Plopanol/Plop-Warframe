<#
.SYNOPSIS
Écrit NOTICES.md (licences des composants embarqués dans l'exécutable Windows) ou vérifie qu'il est à jour.

.DESCRIPTION
Lit le graphe Cargo résolu pour la cible du livrable (dépendances normales seulement, sans build ni dev),
recopie les fichiers de licence de chaque crate depuis le registre local, regroupe les textes identiques
et ajoute les composants C embarqués (SQLite, Lua) et l'attribution des données du wiki.
Lancé depuis n'importe où :  pwsh -NoProfile -File tools/New-Notices.ps1 [-Check]
-Check n'écrit rien et échoue si NOTICES.md diffère du résultat. Codes de sortie : 0 réussi, 1 échec.
#>
[CmdletBinding()]
param([switch]$Check)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'

$Target = 'x86_64-pc-windows-msvc'
$LicenseFilePattern = '^(LICEN[CS]E|COPYING|NOTICE|UNLICENSE)'

function Get-ShippedPackages {
    # cargo tree applies the feature resolver for the target: it lists what the executable really links.
    $tree = & cargo tree --locked --target $Target -e normal --prefix none -f '{p}'
    if ($LASTEXITCODE -ne 0) { throw "cargo tree a échoué (code $LASTEXITCODE)." }
    $shipped = [Collections.Generic.HashSet[string]]::new()
    foreach ($line in $tree) {
        if ($line -match '^(?<n>\S+) v(?<v>\S+)') { [void]$shipped.Add("$($Matches.n) $($Matches.v)") }
    }
    $json = & cargo metadata --locked --format-version 1
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata a échoué (code $LASTEXITCODE)." }
    $meta = $json | ConvertFrom-Json -Depth 64
    $script:AllPackages = $meta.packages
    $root = ($meta.packages | Where-Object { $_.id -eq $meta.resolve.root }).name
    $meta.packages |
        Where-Object { $_.name -ne $root -and $shipped.Contains("$($_.name) $($_.version)") } |
        Sort-Object name, version
}

# Lua's notice lives at the end of lua.h; read it from the locked lua-src so it follows upgrades.
function Get-LuaNotice {
    $src = @($script:AllPackages | Where-Object { $_.name -eq 'lua-src' })
    if ($src.Count -ne 1) { throw "lua-src introuvable ou en plusieurs versions dans le graphe Cargo." }
    $header = Get-ChildItem -LiteralPath (Split-Path -Parent $src[0].manifest_path) -Directory -Filter 'lua-5.4*' |
        ForEach-Object { Join-Path $_.FullName 'lua.h' } | Select-Object -First 1
    if (-not $header) { throw 'lua-5.4*/lua.h introuvable dans lua-src.' }
    $m = [regex]::Match([IO.File]::ReadAllText($header), '(?s)\* (Copyright \(C\) 1994-.*?SOFTWARE\.)')
    if (-not $m.Success) { throw "Notice de copyright introuvable dans $header." }
    $version = [regex]::Match($header, 'lua-(5\.4\.\d+)').Groups[1].Value
    $text = ($m.Groups[1].Value -replace "`r`n", "`n" -split "`n" | ForEach-Object { $_ -replace '^\s*\*\s?', '' }) -join "`n"
    [pscustomobject]@{ Version = $version; Text = $text.Trim() }
}

function Get-LicenseTexts($package) {
    $dir = Split-Path -Parent $package.manifest_path
    Get-ChildItem -LiteralPath $dir -File |
        Where-Object { $_.Name -match $LicenseFilePattern } |
        Sort-Object Name |
        ForEach-Object { ([IO.File]::ReadAllText($_.FullName) -replace "`r`n", "`n").Trim() }
}

function Build-Notices {
    $packages = @(Get-ShippedPackages)
    $lua = Get-LuaNotice
    $groups = [ordered]@{}
    $missing = [Collections.Generic.List[string]]::new()
    $rows = foreach ($p in $packages) {
        $label = "$($p.name) $($p.version)"
        $texts = @(Get-LicenseTexts $p)
        if ($texts.Count -eq 0) { $missing.Add($label) }
        foreach ($text in $texts) {
            if (-not $groups.Contains($text)) { $groups[$text] = [Collections.Generic.List[string]]::new() }
            $groups[$text].Add($label)
        }
        $license = if ($p.license) { $p.license } else { 'non déclarée' }
        "| $($p.name) | $($p.version) | $license |"
    }

    $out = [Text.StringBuilder]::new()
    $fence = '```'
    [void]$out.AppendLine('# Notices de licences')
    [void]$out.AppendLine()
    [void]$out.AppendLine("Fichier généré par ``tools/New-Notices.ps1`` pour la cible ``$Target`` ; ne pas le modifier à la main.")
    [void]$out.AppendLine()
    [void]$out.AppendLine('## Données du wiki Warframe')
    [void]$out.AppendLine()
    [void]$out.AppendLine('Les tables de drop importées proviennent des modules du wiki Warframe (<https://wiki.warframe.com/>), écrits par les contributeurs du wiki et publiés sous licence Creative Commons Attribution - Pas d''utilisation commerciale - Partage dans les mêmes conditions 3.0 (CC BY-NC-SA 3.0, <https://creativecommons.org/licenses/by-nc-sa/3.0/>). L''application télécharge ces modules à la demande de l''utilisateur et les réorganise dans une base locale ; aucune donnée du wiki n''est livrée avec elle. Plop-Warframe n''est ni affilié ni approuvé par Digital Extremes ou par le wiki.')
    [void]$out.AppendLine()
    [void]$out.AppendLine('## Composants C embarqués')
    [void]$out.AppendLine()
    [void]$out.AppendLine('- SQLite, compilé par `libsqlite3-sys` (`bundled`) : domaine public (<https://www.sqlite.org/copyright.html>).')
    [void]$out.AppendLine("- Lua $($lua.Version), compilé par ``lua-src`` (``vendored``) : licence MIT, texte ci-dessous.")
    [void]$out.AppendLine()
    [void]$out.AppendLine($fence)
    [void]$out.AppendLine($lua.Text)
    [void]$out.AppendLine($fence)
    [void]$out.AppendLine()
    [void]$out.AppendLine('## Slint')
    [void]$out.AppendLine()
    [void]$out.AppendLine('L''interface utilise Slint sous la licence Slint Royalty-free 2.0 ; l''onglet « À propos » affiche l''attribution requise (`AboutSlint`). Les crates Slint ne contiennent pas le texte de la licence : il est publié sur <https://slint.dev/terms-and-conditions#royalty-free>.')
    [void]$out.AppendLine()
    # Fonts are compiled into the binary (ui/theme.slint); their OFL texts sit next to them.
    [void]$out.AppendLine('## Polices')
    [void]$out.AppendLine()
    [void]$out.AppendLine('Les polices Roboto et Quantico, reprises du wiki Warframe, sont intégrées à l''exécutable ; elles sont publiées sous SIL Open Font License 1.1 (<https://openfontlicense.org>), textes ci-dessous.')
    foreach ($font in 'Roboto', 'Quantico') {
        $text = (Get-Content -Raw -Encoding utf8 (Join-Path $root "ui/fonts/OFL-$font.txt")).Trim() -replace "`r`n", "`n"
        [void]$out.AppendLine()
        [void]$out.AppendLine("### $font")
        [void]$out.AppendLine()
        [void]$out.AppendLine('````text')
        [void]$out.AppendLine($text)
        [void]$out.AppendLine('````')
    }
    [void]$out.AppendLine()
    [void]$out.AppendLine("## Crates Rust ($($packages.Count))")
    [void]$out.AppendLine()
    [void]$out.AppendLine('| Crate | Version | Licence |')
    [void]$out.AppendLine('|---|---|---|')
    foreach ($row in $rows) { [void]$out.AppendLine($row) }
    [void]$out.AppendLine()
    if ($missing.Count -gt 0) {
        [void]$out.AppendLine('Crates sans fichier de licence dans leur paquet (licence déclarée dans le tableau) : ' + ($missing -join ', ') + '.')
        [void]$out.AppendLine()
    }
    [void]$out.AppendLine('## Textes des licences')
    $index = 0
    foreach ($entry in $groups.GetEnumerator()) {
        $index++
        [void]$out.AppendLine()
        [void]$out.AppendLine("### Texte $index")
        [void]$out.AppendLine()
        [void]$out.AppendLine('Crates : ' + ($entry.Value -join ', ') + '.')
        [void]$out.AppendLine()
        [void]$out.AppendLine('````text')
        [void]$out.AppendLine($entry.Key)
        [void]$out.AppendLine('````')
    }
    ($out.ToString() -replace "`r`n", "`n")
}

$root = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $root
try {
    $content = Build-Notices
    $path = Join-Path $root 'NOTICES.md'
    if ($Check) {
        $current = if (Test-Path -LiteralPath $path) { [IO.File]::ReadAllText($path) -replace "`r`n", "`n" } else { '' }
        if ($current -ne $content) { throw 'NOTICES.md n''est pas à jour : lancer pwsh -NoProfile -File tools/New-Notices.ps1.' }
        Write-Output 'NOTICES.md à jour.'
    }
    else {
        [IO.File]::WriteAllText($path, $content, [Text.UTF8Encoding]::new($false))
        Write-Output "NOTICES.md écrit."
    }
    exit 0
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 1
}
finally { Pop-Location }
