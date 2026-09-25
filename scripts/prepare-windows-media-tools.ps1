[CmdletBinding()]
param(
    [string] $TargetTriple = "x86_64-pc-windows-msvc"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($TargetTriple -ne "x86_64-pc-windows-msvc") {
    throw "The bundled FFmpeg distribution currently supports only x86_64-pc-windows-msvc, not $TargetTriple."
}

$version = "9.0.2"
$archiveName = "ffmpeg-$version-essentials_build.zip"
$archiveUrl = "https://www.gyan.dev/ffmpeg/builds/packages/$archiveName"
$expectedArchiveSha256 = "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba"

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$targetRoot = Join-Path $repositoryRoot "target"
$cacheRoot = Join-Path $targetRoot "media-tools"
$archivePath = Join-Path $cacheRoot $archiveName
$extractRoot = Join-Path $cacheRoot "ffmpeg-$version-essentials"
$sidecarRoot = Join-Path $repositoryRoot "src-tauri\binaries"
$noticeRoot = Join-Path $repositoryRoot "src-tauri\generated\ffmpeg"

function Assert-NoReparsePoint {
    param([Parameter(Mandatory = $true)][string] $Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Refusing to use a reparse point in the media-tool build path: $Path"
    }
}

function Assert-NoNestedReparsePoints {
    param([Parameter(Mandatory = $true)][string] $Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $pending.Push($Path)
    while ($pending.Count -gt 0) {
        $current = $pending.Pop()
        Assert-NoReparsePoint -Path $current
        foreach ($item in Get-ChildItem -LiteralPath $current -Force) {
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Refusing to use a nested reparse point in the media-tool build path: $($item.FullName)"
            }
            if ($item.PSIsContainer) {
                $pending.Push($item.FullName)
            }
        }
    }
}

function New-SafeDirectory {
    param([Parameter(Mandatory = $true)][string] $Path)

    New-Item -ItemType Directory -Path $Path -Force | Out-Null
    Assert-NoReparsePoint -Path $Path
}

function Get-LowercaseSha256 {
    param([Parameter(Mandatory = $true)][string] $Path)

    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Resolve-SingleFile {
    param(
        [Parameter(Mandatory = $true)][string] $Root,
        [Parameter(Mandatory = $true)][string] $Name
    )

    $matches = @(Get-ChildItem -LiteralPath $Root -Filter $Name -File -Recurse)
    if ($matches.Count -ne 1) {
        throw "Expected exactly one $Name in the verified FFmpeg archive, found $($matches.Count)."
    }
    return $matches[0]
}

function Assert-NativeTool {
    param(
        [Parameter(Mandatory = $true)][string] $Path,
        [Parameter(Mandatory = $true)][string] $ExpectedPrefix
    )

    $versionOutput = @(& $Path -hide_banner -version)
    $exitCode = $LASTEXITCODE
    $versionLine = if ($versionOutput.Count -gt 0) { [string] $versionOutput[0] } else { "" }
    if ($exitCode -ne 0 -or $versionLine -notlike "$ExpectedPrefix*") {
        throw "The bundled media tool failed its version check: $Path"
    }
    return $versionLine
}

Assert-NoReparsePoint -Path $targetRoot
New-SafeDirectory -Path $cacheRoot
Assert-NoReparsePoint -Path $archivePath

$archiveIsValid = $false
if (Test-Path -LiteralPath $archivePath -PathType Leaf) {
    $archiveIsValid = (Get-LowercaseSha256 -Path $archivePath) -eq $expectedArchiveSha256
}
if (-not $archiveIsValid) {
    Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
    Write-Host "Downloading pinned FFmpeg $version essentials archive..."
    Invoke-WebRequest -Uri $archiveUrl -OutFile $archivePath -UseBasicParsing
}

$actualArchiveSha256 = Get-LowercaseSha256 -Path $archivePath
if ($actualArchiveSha256 -ne $expectedArchiveSha256) {
    Remove-Item -LiteralPath $archivePath -Force
    throw "FFmpeg archive SHA-256 mismatch. Expected $expectedArchiveSha256, received $actualArchiveSha256."
}

New-SafeDirectory -Path $extractRoot
Assert-NoNestedReparsePoints -Path $extractRoot
Expand-Archive -LiteralPath $archivePath -DestinationPath $extractRoot -Force
Assert-NoNestedReparsePoints -Path $extractRoot

$ffmpegSource = Resolve-SingleFile -Root $extractRoot -Name "ffmpeg.exe"
$ffprobeSource = Resolve-SingleFile -Root $extractRoot -Name "ffprobe.exe"
$sourceRoot = $ffmpegSource.Directory.Parent.FullName

New-SafeDirectory -Path $sidecarRoot
Assert-NoNestedReparsePoints -Path $sidecarRoot
$ffmpegDestination = Join-Path $sidecarRoot "ffmpeg-$TargetTriple.exe"
$ffprobeDestination = Join-Path $sidecarRoot "ffprobe-$TargetTriple.exe"
Copy-Item -LiteralPath $ffmpegSource.FullName -Destination $ffmpegDestination -Force
Copy-Item -LiteralPath $ffprobeSource.FullName -Destination $ffprobeDestination -Force

$ffmpegVersionLine = Assert-NativeTool -Path $ffmpegDestination -ExpectedPrefix "ffmpeg version $version"
$ffprobeVersionLine = Assert-NativeTool -Path $ffprobeDestination -ExpectedPrefix "ffprobe version $version"

$encoders = (& $ffmpegDestination -hide_banner -encoders 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $encoders -notmatch "\blibx264\b" -or $encoders -notmatch "\baac\b") {
    throw "The bundled FFmpeg build does not provide the required libx264 and AAC encoders."
}
$filters = (& $ffmpegDestination -hide_banner -filters 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $filters -notmatch "\bsubtitles\b" -or $filters -notmatch "\bass\b") {
    throw "The bundled FFmpeg build does not provide the required subtitle filters."
}

New-SafeDirectory -Path $noticeRoot
Assert-NoNestedReparsePoints -Path $noticeRoot
Get-ChildItem -LiteralPath $noticeRoot -File |
    Where-Object { $_.Name -ne "README.md" } |
    Remove-Item -Force

$requiredNotices = @("LICENSE", "README.txt")
foreach ($notice in $requiredNotices) {
    $sourceNotice = Join-Path $sourceRoot $notice
    if (-not (Test-Path -LiteralPath $sourceNotice -PathType Leaf)) {
        throw "The verified FFmpeg archive is missing required notice: $notice"
    }
    Copy-Item -LiteralPath $sourceNotice -Destination (Join-Path $noticeRoot $notice) -Force
}

Write-Host $ffmpegVersionLine
Write-Host $ffprobeVersionLine
Write-Host "Prepared verified Windows media sidecars and GPL notices for $TargetTriple."
