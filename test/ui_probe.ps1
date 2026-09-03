param(
  [string]$Action = "shot",
  [int]$X = 0,
  [int]$Y = 0,
  [string]$Out = "shot.png",
  [string]$ProcName = "amni-browse"
)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@
[void][W]::SetProcessDpiAwarenessContext([IntPtr](-4))
$p = Get-Process -Name $ProcName -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $p) { Write-Output "NOWINDOW"; exit 2 }
$h = $p.MainWindowHandle
$r = New-Object W+RECT
[void][W]::GetClientRect($h, [ref]$r)
$o = New-Object W+POINT
[void][W]::ClientToScreen($h, [ref]$o)
$w = $r.Right - $r.Left
$ht = $r.Bottom - $r.Top
Write-Output "CLIENT ${w}x${ht} ORIGIN $($o.X),$($o.Y)"
if ($Action -eq "click" -or $Action -eq "rclick") {
  [void][W]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 350
  [void][W]::SetCursorPos($o.X + $X, $o.Y + $Y)
  Start-Sleep -Milliseconds 250
  if ($Action -eq "rclick") { [W]::mouse_event(0x0008, 0, 0, 0, [IntPtr]::Zero); Start-Sleep -Milliseconds 60; [W]::mouse_event(0x0010, 0, 0, 0, [IntPtr]::Zero) }
  else { [W]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero); Start-Sleep -Milliseconds 60; [W]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero) }
  Write-Output "SENT $Action at client $X,$Y"
  Start-Sleep -Milliseconds 900
}
if ($Action -ne "move") {
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($o.X, $o.Y, 0, 0, (New-Object System.Drawing.Size $w, $ht))
  $bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Write-Output "SHOT $Out"
}
