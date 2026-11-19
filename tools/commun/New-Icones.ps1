<#
.SYNOPSIS
Génère l'icône Windows (.ico multi-tailles) et les mipmaps Android d'un projet à partir d'un PNG source.

.DESCRIPTION
Recopié dans tools/commun/ ; chemins relatifs au dossier courant, à lancer depuis la racine du projet :
  pwsh -File tools/commun/New-Icones.ps1 -Source assets/icon.png -Ico assets/icon.ico
  pwsh -File tools/commun/New-Icones.ps1 -Source assets/icon.png -AndroidRes android/app/src/main/res -Adaptive -Background '#F5A22B'
  -Png assets/icon-64.png -PngSize 64 écrit en plus un PNG redimensionné (icône de fenêtre).
Au moins une sortie parmi -Ico, -AndroidRes, -Png est requise. -Sizes change les tailles de l'.ico
(16, 24, 32, 48, 64, 128, 256 par défaut). Le PNG source est carré, de 256 px ou plus, de préférence 1024.

Android : mipmap-<densité>/ic_launcher.png et ic_launcher_round.png (48 à 192 px). Avec -Adaptive :
ic_launcher_foreground.png et ic_launcher_monochrome.png (108 à 432 px, motif réduit aux 72/108 centraux,
zone sûre d'Android), mipmap-anydpi-v26/ic_launcher*.xml et values/ic_launcher_background.xml (couleur
-Background). Aucun fichier existant n'est supprimé ; les fichiers de même nom sont remplacés.

Pourquoi System.Drawing et pas Python/Pillow : aucune dépendance à installer, Windows PowerShell 5.1 et pwsh 7.
Une icône dessinée (vecteur, motif) reste produite par le projet, qui fournit alors le PNG source.
Codes de sortie : 0 réussi, 2 erreur.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Source,
    [string]$Ico,
    [string]$AndroidRes,
    [string]$Png,
    [int]$PngSize = 64,
    [int[]]$Sizes = @(16, 24, 32, 48, 64, 128, 256),
    [switch]$Adaptive,
    [string]$Background = '#FFFFFF'
)

try { [Console]::OutputEncoding = [Text.UTF8Encoding]::new() } catch { }
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# [IO.File] resolves paths from the process directory, not from PowerShell's current location.
function Resolve-Output([string]$path) {
    if ([IO.Path]::IsPathRooted($path)) { return [IO.Path]::GetFullPath($path) }
    return [IO.Path]::GetFullPath((Join-Path $PWD.ProviderPath $path))
}

# Halve repeatedly: a single bicubic pass aliases badly from 1024 down to 16.
function New-Resized([System.Drawing.Bitmap]$image, [int]$size, [int]$canvas = 0, [switch]$Round) {
    $current = $image
    while ($current.Width -ge $size * 2) {
        $half = [Math]::Max([int]($current.Width / 2), 1)
        $next = [System.Drawing.Bitmap]::new($half, $half, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $g = [System.Drawing.Graphics]::FromImage($next)
        try {
            $g.InterpolationMode = 'HighQualityBicubic'; $g.PixelOffsetMode = 'HighQuality'; $g.SmoothingMode = 'HighQuality'
            $g.DrawImage($current, 0, 0, $half, $half)
        } finally { $g.Dispose() }
        if ($current -ne $image) { $current.Dispose() }
        $current = $next
    }
    if ($canvas -eq 0) { $canvas = $size }
    $out = [System.Drawing.Bitmap]::new($canvas, $canvas, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($out)
    try {
        $g.InterpolationMode = 'HighQualityBicubic'; $g.PixelOffsetMode = 'HighQuality'; $g.SmoothingMode = 'HighQuality'
        if ($Round) {
            $path = [System.Drawing.Drawing2D.GraphicsPath]::new()
            $path.AddEllipse(0, 0, $canvas - 1, $canvas - 1)
            $g.SetClip($path)
            $path.Dispose()
        }
        $offset = [int](($canvas - $size) / 2)
        $g.DrawImage($current, $offset, $offset, $size, $size)
    } finally { $g.Dispose(); if ($current -ne $image) { $current.Dispose() } }
    return $out
}

# White silhouette: colour channels go to 1, alpha is kept (Android 13 themed icon).
function New-Monochrome([System.Drawing.Bitmap]$image) {
    $matrix = [System.Drawing.Imaging.ColorMatrix]::new([float[][]]@(
        [float[]]@(0, 0, 0, 0, 0), [float[]]@(0, 0, 0, 0, 0), [float[]]@(0, 0, 0, 0, 0),
        [float[]]@(0, 0, 0, 1, 0), [float[]]@(1, 1, 1, 0, 1)))
    $attributes = [System.Drawing.Imaging.ImageAttributes]::new()
    $attributes.SetColorMatrix($matrix)
    $out = [System.Drawing.Bitmap]::new($image.Width, $image.Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($out)
    try {
        $rect = [System.Drawing.Rectangle]::new(0, 0, $image.Width, $image.Height)
        $g.DrawImage($image, $rect, 0, 0, $image.Width, $image.Height, [System.Drawing.GraphicsUnit]::Pixel, $attributes)
    } finally { $g.Dispose(); $attributes.Dispose() }
    return $out
}

function Save-Png([System.Drawing.Bitmap]$bitmap, [string]$path) {
    $dir = [IO.Path]::GetDirectoryName($path)
    if (-not [IO.Directory]::Exists($dir)) { [void][IO.Directory]::CreateDirectory($dir) }
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function Get-PngBytes([System.Drawing.Bitmap]$bitmap) {
    $stream = [IO.MemoryStream]::new()
    try { $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png); return , $stream.ToArray() }
    finally { $stream.Dispose() }
}

# An .ico may hold PNG frames; 256 is written as 0 in the header.
function Write-Ico([System.Drawing.Bitmap]$image, [int[]]$sizes, [string]$path) {
    $frames = foreach ($size in ($sizes | Sort-Object -Unique)) {
        $resized = New-Resized $image $size
        try { @{ Size = $size; Bytes = (Get-PngBytes $resized) } } finally { $resized.Dispose() }
    }
    $frames = @($frames)
    $dir = [IO.Path]::GetDirectoryName($path)
    if (-not [IO.Directory]::Exists($dir)) { [void][IO.Directory]::CreateDirectory($dir) }
    $file = [IO.File]::Create($path)
    $writer = [IO.BinaryWriter]::new($file)
    try {
        $writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]$frames.Count)
        $offset = 6 + 16 * $frames.Count
        foreach ($frame in $frames) {
            $dimension = if ($frame.Size -ge 256) { 0 } else { $frame.Size }
            $writer.Write([byte]$dimension); $writer.Write([byte]$dimension)
            $writer.Write([byte]0); $writer.Write([byte]0)
            $writer.Write([uint16]1); $writer.Write([uint16]32)
            $writer.Write([uint32]$frame.Bytes.Length); $writer.Write([uint32]$offset)
            $offset += $frame.Bytes.Length
        }
        foreach ($frame in $frames) { $writer.Write([byte[]]$frame.Bytes) }
    } finally { $writer.Dispose() }
    return ($frames | ForEach-Object { $_.Size })
}

# Read the header back: a malformed .ico only shows up when Explorer opens it.
function Read-IcoSizes([string]$path) {
    $bytes = [IO.File]::ReadAllBytes($path)
    if ($bytes.Length -lt 6 -or [BitConverter]::ToUInt16($bytes, 2) -ne 1) { throw "En-tête .ico invalide : $path" }
    $count = [BitConverter]::ToUInt16($bytes, 4)
    $found = for ($i = 0; $i -lt $count; $i++) {
        $width = $bytes[6 + 16 * $i]
        if ($width -eq 0) { 256 } else { [int]$width }
    }
    return @($found)
}

function Write-Text([string]$path, [string]$text) {
    $dir = [IO.Path]::GetDirectoryName($path)
    if (-not [IO.Directory]::Exists($dir)) { [void][IO.Directory]::CreateDirectory($dir) }
    [IO.File]::WriteAllText($path, $text, [Text.UTF8Encoding]::new($false))
}

try {
    if (-not ($Ico -or $AndroidRes -or $Png)) { throw 'Aucune sortie demandée : indiquer -Ico, -AndroidRes ou -Png.' }
    if ($Background -notmatch '^#[0-9A-Fa-f]{6}$') { throw "-Background attend une couleur #RRGGBB : $Background" }
    $sourcePath = Resolve-Output $Source
    if (-not [IO.File]::Exists($sourcePath)) { throw "PNG source introuvable : $Source" }
    $bytes = [IO.File]::ReadAllBytes($sourcePath)
    $stream = [IO.MemoryStream]::new($bytes)
    $loaded = [System.Drawing.Image]::FromStream($stream)
    $image = [System.Drawing.Bitmap]::new($loaded.Width, $loaded.Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($image)
    try { $g.DrawImage($loaded, 0, 0, $loaded.Width, $loaded.Height) } finally { $g.Dispose(); $loaded.Dispose(); $stream.Dispose() }
    try {
        if ($image.Width -ne $image.Height) { throw "Le PNG source doit être carré ($($image.Width)x$($image.Height))." }
        $largest = [Math]::Max([int]($Sizes | Measure-Object -Maximum).Maximum, 192)
        if ($image.Width -lt 256) { throw "PNG source trop petit ($($image.Width) px) : 256 px au minimum, 1024 conseillé." }
        if ($image.Width -lt $largest -and $AndroidRes) { Write-Warning "PNG source de $($image.Width) px : les plus grandes densités sont agrandies." }

        if ($Ico) {
            $icoPath = Resolve-Output $Ico
            [void](Write-Ico $image $Sizes $icoPath)
            $written = Read-IcoSizes $icoPath
            $expected = @($Sizes | Sort-Object -Unique)
            if (($written -join ',') -ne ($expected -join ',')) { throw "Tailles de l'.ico inattendues : $($written -join ', ')" }
            Write-Output "Écrit : $Ico ($($written -join ', ') px)"
        }
        if ($Png) {
            $resized = New-Resized $image $PngSize
            try { Save-Png $resized (Resolve-Output $Png) } finally { $resized.Dispose() }
            Write-Output "Écrit : $Png (${PngSize} px)"
        }
        if ($AndroidRes) {
            $res = Resolve-Output $AndroidRes
            $legacy = [ordered]@{ 'mipmap-mdpi' = 48; 'mipmap-hdpi' = 72; 'mipmap-xhdpi' = 96; 'mipmap-xxhdpi' = 144; 'mipmap-xxxhdpi' = 192 }
            $count = 0
            foreach ($density in $legacy.Keys) {
                $size = $legacy[$density]
                $square = New-Resized $image $size
                $round = New-Resized $image $size -Round
                try {
                    Save-Png $square (Join-Path $res "$density\ic_launcher.png")
                    Save-Png $round (Join-Path $res "$density\ic_launcher_round.png")
                    $count += 2
                } finally { $square.Dispose(); $round.Dispose() }
            }
            if ($Adaptive) {
                $layers = [ordered]@{ 'mipmap-mdpi' = 108; 'mipmap-hdpi' = 162; 'mipmap-xhdpi' = 216; 'mipmap-xxhdpi' = 324; 'mipmap-xxxhdpi' = 432 }
                foreach ($density in $layers.Keys) {
                    $canvas = $layers[$density]
                    $foreground = New-Resized $image ([int]($canvas * 72 / 108)) $canvas
                    $mono = New-Monochrome $foreground
                    try {
                        Save-Png $foreground (Join-Path $res "$density\ic_launcher_foreground.png")
                        Save-Png $mono (Join-Path $res "$density\ic_launcher_monochrome.png")
                        $count += 2
                    } finally { $foreground.Dispose(); $mono.Dispose() }
                }
                $xml = "<?xml version=`"1.0`" encoding=`"utf-8`"?>`n<adaptive-icon xmlns:android=`"http://schemas.android.com/apk/res/android`">`n" +
                    "    <background android:drawable=`"@color/ic_launcher_background`"/>`n" +
                    "    <foreground android:drawable=`"@mipmap/ic_launcher_foreground`"/>`n" +
                    "    <monochrome android:drawable=`"@mipmap/ic_launcher_monochrome`"/>`n</adaptive-icon>`n"
                Write-Text (Join-Path $res 'mipmap-anydpi-v26\ic_launcher.xml') $xml
                Write-Text (Join-Path $res 'mipmap-anydpi-v26\ic_launcher_round.xml') $xml
                Write-Text (Join-Path $res 'values\ic_launcher_background.xml') "<?xml version=`"1.0`" encoding=`"utf-8`"?>`n<resources>`n    <color name=`"ic_launcher_background`">$($Background.ToUpperInvariant())</color>`n</resources>`n"
                $count += 3
            }
            Write-Output "Écrit : $AndroidRes ($count fichier(s))"
        }
    } finally { $image.Dispose() }
    exit 0
}
catch {
    [Console]::Error.WriteLine("Erreur : $($_.Exception.Message)")
    exit 2
}
