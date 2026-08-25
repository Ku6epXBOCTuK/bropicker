param(
    [string]$ProcName = "bropicker",
    [int]$ProcId = 0,
    [string]$Out = "tools/out/app.png"
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
$w = $rect.Right - $rect.Left
$h = $rect.Bottom - $rect.Top

if ($w -le 0 -or $h -le 0) {
    Write-Error "invalid window size ${w}x${h}"
    exit 1
}

Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
$g.Dispose()

$outDir = Split-Path -Parent (Join-Path $PWD $Out)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$outAbs = [System.IO.Path]::GetFullPath((Join-Path $PWD $Out))
$bmp.Save($outAbs, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Host "OK $OutAbs (${w}x${h})"
