param(
    [string]$ProcName = "bropicker",
    [int]$ProcId = 0,
    [string]$Out = "tools/out/app.png",
    [switch]$Card
)

$ErrorActionPreference = "Stop"

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class BpWin {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hWnd);
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
"@

[BpWin]::SetProcessDPIAware() | Out-Null

if ($ProcId -eq 0) {
    $proc = Get-Process -Name $ProcName -ErrorAction Stop | Select-Object -First 1
    $ProcId = $proc.Id
}

$found = [IntPtr]::Zero
$callback = [BpWin+EnumWindowsProc]{
    param($hWnd, $lParam)
    $winPid = 0
    [BpWin]::GetWindowThreadProcessId($hWnd, [ref]$winPid) | Out-Null
    if ($winPid -eq $ProcId -and [BpWin]::IsWindowVisible($hWnd)) {
        $script:found = $hWnd
        return $false
    }
    return $true
}
[BpWin]::EnumWindows($callback, [IntPtr]::Zero) | Out-Null

if ($found -eq [IntPtr]::Zero) {
    Write-Error "visible window for process $ProcId not found"
    exit 1
}

$rect = New-Object BpWin+RECT
[BpWin]::GetWindowRect($found, [ref]$rect) | Out-Null

[BpWin]::SetWindowPos($found, [IntPtr](-1), 0, 0, 0, 0, 0x0003) | Out-Null
[BpWin]::SetForegroundWindow($found) | Out-Null
Start-Sleep -Milliseconds 250

$w = $rect.Right - $rect.Left
$h = $rect.Bottom - $rect.Top

if ($w -le 0 -or $h -le 0) {
    Write-Error "invalid window size ${w}x${h}"
    exit 1
}

Add-Type -AssemblyName System.Drawing
$full = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($full)
$g.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
$g.Dispose()

if ($Card) {
    $dpi = [BpWin]::GetDpiForWindow($found)
    $scale = if ($dpi -gt 0) { $dpi / 96.0 } else { 1.0 }
    $inset = [int][Math]::Round(20 * $scale)
    $radius = [int][Math]::Round(16 * $scale)

    $clipInset = 2
    $cw = $w - 2 * $inset - 2 * $clipInset
    $ch = $h - 2 * $inset - 2 * $clipInset
    $srcX = $inset + $clipInset
    $srcY = $inset + $clipInset
    $radius = [Math]::Max(1, $radius - $clipInset)

    $bmp = New-Object System.Drawing.Bitmap($cw, $ch)
    $g2 = [System.Drawing.Graphics]::FromImage($bmp)
    $g2.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

    $cardPath = New-Object System.Drawing.Drawing2D.GraphicsPath
    $cardPath.AddArc(0, 0, $radius, $radius, 180, 90)
    $cardPath.AddArc($cw - $radius, 0, $radius, $radius, 270, 90)
    $cardPath.AddArc($cw - $radius, $ch - $radius, $radius, $radius, 0, 90)
    $cardPath.AddArc(0, $ch - $radius, $radius, $radius, 90, 90)
    $cardPath.CloseFigure()

    $g2.SetClip($cardPath)
    $srcRect = New-Object System.Drawing.Rectangle($srcX, $srcY, $cw, $ch)
    $dstRect = New-Object System.Drawing.Rectangle(0, 0, $cw, $ch)
    $g2.DrawImage($full, $dstRect, $srcRect, [System.Drawing.GraphicsUnit]::Pixel)
    $g2.Dispose()
    $cardPath.Dispose()
    $full.Dispose()

    $w = $cw
    $h = $ch
} else {
    $bmp = $full
}

$outDir = Split-Path -Parent (Join-Path $PWD $Out)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$outAbs = [System.IO.Path]::GetFullPath((Join-Path $PWD $Out))
$bmp.Save($outAbs, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Host "OK $OutAbs (${w}x${h} @ $($rect.Left),$($rect.Top))"
