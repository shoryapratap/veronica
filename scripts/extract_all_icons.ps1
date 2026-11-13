# Script to extract real PNG icons for all installed Win32 & UWP applications
Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;

public class ShellIconExtractor {
    [DllImport("shell32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr SHGetFileInfo(string pszPath, uint dwFileAttributes, ref SHFILEINFO psfi, uint cbFileInfo, uint uFlags);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool DestroyIcon(IntPtr hIcon);

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
    public struct SHFILEINFO {
        public IntPtr hIcon;
        public int iIcon;
        public uint dwAttributes;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)]
        public string szDisplayName;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 80)]
        public string szTypeName;
    }

    public static bool SaveIcon(string filePath, string outPngPath) {
        try {
            SHFILEINFO shinfo = new SHFILEINFO();
            uint SHGFI_ICON = 0x100;
            uint SHGFI_LARGEICON = 0x0;
            SHGetFileInfo(filePath, 0, ref shinfo, (uint)Marshal.SizeOf(shinfo), SHGFI_ICON | SHGFI_LARGEICON);
            if (shinfo.hIcon != IntPtr.Zero) {
                using (Icon icon = Icon.FromHandle(shinfo.hIcon)) {
                    using (Bitmap bmp = icon.ToBitmap()) {
                        bmp.Save(outPngPath, ImageFormat.Png);
                    }
                }
                DestroyIcon(shinfo.hIcon);
                return true;
            }
        } catch {}
        return false;
    }
}
"@ -ReferencedAssemblies System.Drawing

$appDataPath = "$env:APPDATA\Veronica\installed_apps.json"
if (-not (Test-Path $appDataPath)) {
    Write-Error "installed_apps.json not found at $appDataPath"
    exit 1
}

$destDirPublic = "gui\veronica\public\icons"
$destDirAppData = "$env:APPDATA\Veronica\icons"
New-Item -ItemType Directory -Force -Path $destDirPublic | Out-Null
New-Item -ItemType Directory -Force -Path $destDirAppData | Out-Null

$raw = [System.IO.File]::ReadAllText($appDataPath, [System.Text.Encoding]::UTF8)
$apps = $raw | ConvertFrom-Json

# Cache current user's UWP packages
Write-Host "Caching UWP AppX packages..."
$allPackages = Get-AppxPackage -ErrorAction SilentlyContinue

$count = 0
$uwpCount = 0
$win32Count = 0

foreach ($app in $apps) {
    # Sanitize name
    $cleanName = ($app.name -replace '[\\/:*?"<>|]', '_').Trim()
    if ([string]::IsNullOrWhiteSpace($cleanName)) {
        $cleanName = "app_" + $count
    }

    $iconFileName = "$cleanName.png"
    $targetPublicPng = Join-Path $destDirPublic $iconFileName
    $targetAppDataPng = Join-Path $destDirAppData $iconFileName
    $iconSaved = $false

    if ($app.app_type -eq "UWP") {
        # UWP Logo Extraction
        $appId = $app.path
        $pkgPrefix = $appId.Split('!')[0].Split('_')[0]
        
        $matchPkg = $allPackages | Where-Object { $_.Name -like "*$pkgPrefix*" } | Select-Object -First 1
        if ($matchPkg -and (Test-Path $matchPkg.InstallLocation)) {
            $logoCandidate = Get-ChildItem -Path $matchPkg.InstallLocation -Recurse -Filter "*AppList*.png" -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -notlike "*contrast-white*" -and $_.Name -notlike "*contrast-black*" } |
                Sort-Object Length -Descending |
                Select-Object -First 1

            if (-not $logoCandidate) {
                $logoCandidate = Get-ChildItem -Path $matchPkg.InstallLocation -Recurse -Filter "*Logo*.png" -ErrorAction SilentlyContinue |
                    Where-Object { $_.Name -notlike "*contrast-white*" -and $_.Name -notlike "*contrast-black*" } |
                    Sort-Object Length -Descending |
                    Select-Object -First 1
            }

            if ($logoCandidate -and (Test-Path $logoCandidate.FullName)) {
                try {
                    $bytes = [System.IO.File]::ReadAllBytes($logoCandidate.FullName)
                    [System.IO.File]::WriteAllBytes($targetPublicPng, $bytes)
                    [System.IO.File]::WriteAllBytes($targetAppDataPng, $bytes)
                    $iconSaved = $true
                    $uwpCount++
                } catch {}
            }
        }
    } else {
        # Win32 Shortcut / Exe
        if ($app.path -and (Test-Path $app.path)) {
            $extracted = [ShellIconExtractor]::SaveIcon($app.path, $targetPublicPng)
            if ($extracted -and (Test-Path $targetPublicPng)) {
                $bytes = [System.IO.File]::ReadAllBytes($targetPublicPng)
                [System.IO.File]::WriteAllBytes($targetAppDataPng, $bytes)
                $iconSaved = $true
                $win32Count++
            }
        }
    }

    if ($iconSaved) {
        $app.icon_path = "/icons/$iconFileName"
    }
    $count++
}

# Save updated JSON with real icon_path
$updatedJson = $apps | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText($appDataPath, $updatedJson, [System.Text.Encoding]::UTF8)
[System.IO.File]::WriteAllText("gui\veronica\public\installed_apps.json", $updatedJson, [System.Text.Encoding]::UTF8)

Write-Host "✅ Icon extraction complete!"
Write-Host "Total apps processed: $count"
Write-Host "Win32 icons extracted: $win32Count"
Write-Host "UWP icons extracted: $uwpCount"
Write-Host "Total real icons saved: $($win32Count + $uwpCount)"
