# NeNe Ledger の窓を、Windows の上で操作して撮る（確かめ用・GitHub Issue #503・gate-proofs 第 86 節と第 102 節）。
# WSL から、Windows の exe を起こした後に呼ぶ。マウス・字・修飾の無いキーは窓へのメッセージで送るので、前面にしなくて
# よく、マウスのカーソルも動かさない（ほかの窓の作業を邪魔しない）。Ctrl などの組み合わせだけは、窓を前面にして送る
# （前面にできなければ、何も送らずに終了コード 3 で止まる。別の窓にキーを打ち込まない）。
#
#   powershell.exe -NoProfile -ExecutionPolicy Bypass -File <この道筋> <exe のプロセスの番号> <命令>...
#
#   rect                 窓と中身の矩形と、窓の名前を書く
#   title                窓の名前を書く（未保存の印 ● を見る）
#   size:w,h             窓の大きさを変える
#   shot:<png の道筋>    窓を PrintWindow で撮る（前面でなくてよい）
#   wait:<ms>
#   hover:x,y            ボタンを押さずに指を動かす（座標は窓の中・物理画素）
#   down:x,y / move:x,y / up:x,y   左のボタンを押す・押したまま動かす・離す
#   ctrldown:x,y         Ctrl を押した左の押し（範囲を足す・枠を掴んで写す）
#   dbl:x,y              2 度目の押し（WM_LBUTTONDBLCLK。down と up の後に送る）
#   uni:<16 進の符号位置,…>  字を送る（WM_CHAR。日本語は引数で壊れるので符号位置で渡す。例 uni:6c0f,540d ＝ 氏名）
#   char:<ASCII の字>    字を送る
#   vk:<仮想キーの番号>   修飾の無いキーを送る（13 Enter・9 Tab・27 Esc・113 F2）
#   keys:<SendKeys の字>  窓を前面にしてキーを送る（^s ＝ Ctrl+S・^z ＝ Ctrl+Z・^+p ＝ Ctrl+Shift+P）
#   dialog:title / dialog:ok / dialog:cancel   このプロセスが出している OS の画面（名前を付けて保存・書き出す先）の
#                        名前を書く・既定のボタンを押す・やめる（WM_COMMAND。前面にしなくてよい）。画面が無ければ終了コード 5
#
# 🔴 見られないもの：マウスのカーソルの形・IME の確定前の文字列と候補の窓・手で触った手応え・画面の倍率ごとの見え。
# これらは施主の実機の確認（QLT-013）のまま。このファイルは BOM 付きの UTF-8（Windows PowerShell 5.1 が BOM の無い
# UTF-8 を CP932 として読み、日本語のコメントで構文が壊れるため）。
param([Parameter(Mandatory=$true)][int]$ProcessId, [Parameter(ValueFromRemainingArguments=$true)][string[]]$Commands)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class NeneWin {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll", EntryPoint="PostMessageW")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hWnd, int x, int y, int w, int h, bool repaint);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int max);
  public static IntPtr DialogOf(uint pid) {
    IntPtr found = IntPtr.Zero;
    while ((found = FindWindowEx(IntPtr.Zero, found, "#32770", null)) != IntPtr.Zero) {
      uint owner; GetWindowThreadProcessId(found, out owner);
      if (owner == pid && IsWindowVisible(found)) { return found; }
    }
    return IntPtr.Zero;
  }
}
"@
[void][NeneWin]::SetProcessDPIAware()
$process = Get-Process -Id $ProcessId -ErrorAction Stop
$hwnd = $process.MainWindowHandle
if ($hwnd -eq [IntPtr]::Zero) { Write-Output "no window"; exit 2 }
function Point([string]$text) { $p = $text.Split(','); return [IntPtr](([int]$p[1] -shl 16) -bor ([int]$p[0] -band 0xFFFF)) }
foreach ($command in $Commands) {
  $name, $value = $command.Split(':', 2)
  switch ($name) {
    'wait' { Start-Sleep -Milliseconds ([int]$value) }
    'rect' {
      $w = New-Object NeneWin+RECT; $c = New-Object NeneWin+RECT
      [void][NeneWin]::GetWindowRect($hwnd, [ref]$w); [void][NeneWin]::GetClientRect($hwnd, [ref]$c)
      Write-Output ("window {0},{1} {2}x{3} client {4}x{5} title {6}" -f $w.Left, $w.Top, ($w.Right-$w.Left), ($w.Bottom-$w.Top), $c.Right, $c.Bottom, $process.MainWindowTitle)
    }
    'size' {
      $w = New-Object NeneWin+RECT; [void][NeneWin]::GetWindowRect($hwnd, [ref]$w)
      $p = $value.Split(','); [void][NeneWin]::MoveWindow($hwnd, $w.Left, $w.Top, [int]$p[0], [int]$p[1], $true)
    }
    'shot' {
      $w = New-Object NeneWin+RECT; [void][NeneWin]::GetWindowRect($hwnd, [ref]$w)
      $bitmap = New-Object System.Drawing.Bitmap(($w.Right-$w.Left), ($w.Bottom-$w.Top))
      $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
      $hdc = $graphics.GetHdc()
      [void][NeneWin]::PrintWindow($hwnd, $hdc, 2)
      $graphics.ReleaseHdc($hdc); $graphics.Dispose()
      $bitmap.Save($value, [System.Drawing.Imaging.ImageFormat]::Png); $bitmap.Dispose()
    }
    'keys' {
      [void][NeneWin]::SetForegroundWindow($hwnd); Start-Sleep -Milliseconds 150
      if ([NeneWin]::GetForegroundWindow() -ne $hwnd) { Write-Output "not foreground: keys skipped ($value)"; exit 3 }
      [System.Windows.Forms.SendKeys]::SendWait($value)
    }
    'down' { [void][NeneWin]::PostMessage($hwnd, 0x0201, [IntPtr]1, (Point $value)) }
    'ctrldown' { [void][NeneWin]::PostMessage($hwnd, 0x0201, [IntPtr]9, (Point $value)) }
    'move' { [void][NeneWin]::PostMessage($hwnd, 0x0200, [IntPtr]1, (Point $value)) }
    'hover' { [void][NeneWin]::PostMessage($hwnd, 0x0200, [IntPtr]0, (Point $value)) }
    'up' { [void][NeneWin]::PostMessage($hwnd, 0x0202, [IntPtr]0, (Point $value)) }
    'char' { foreach ($c in $value.ToCharArray()) { [void][NeneWin]::PostMessage($hwnd, 0x0102, [IntPtr][int]$c, [IntPtr]1); Start-Sleep -Milliseconds 15 } }
    'vk' { [void][NeneWin]::PostMessage($hwnd, 0x0100, [IntPtr][int]$value, [IntPtr]1); [void][NeneWin]::PostMessage($hwnd, 0x0101, [IntPtr][int]$value, [IntPtr]1) }
    'uni' { foreach ($h in $value.Split(',')) { [void][NeneWin]::PostMessage($hwnd, 0x0102, [IntPtr][Convert]::ToInt32($h, 16), [IntPtr]1); Start-Sleep -Milliseconds 15 } }
    'dbl' { [void][NeneWin]::PostMessage($hwnd, 0x0203, [IntPtr]1, (Point $value)) }
    'title' { $process.Refresh(); Write-Output ("title " + $process.MainWindowTitle) }
    'dialog' {
      $dialog = [NeneWin]::DialogOf([uint32]$ProcessId)
      if ($dialog -eq [IntPtr]::Zero) { Write-Output "no dialog"; exit 5 }
      switch ($value) {
        'title' { $text = New-Object System.Text.StringBuilder 256; [void][NeneWin]::GetWindowText($dialog, $text, 256); Write-Output ("dialog " + $text.ToString()) }
        'ok' { [void][NeneWin]::PostMessage($dialog, 0x0111, [IntPtr]1, [IntPtr]::Zero) }
        'cancel' { [void][NeneWin]::PostMessage($dialog, 0x0111, [IntPtr]2, [IntPtr]::Zero) }
        default { Write-Output "unknown: $command"; exit 4 }
      }
    }
    default { Write-Output "unknown: $command"; exit 4 }
  }
}
