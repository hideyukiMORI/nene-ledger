# Windows App Certification Kit（WACK）で、tools/msix.sh が作った MSIX をストアの認定と同じ検査にかける
# （ADR 0035・GitHub Issue #342）。🔴 管理者の PowerShell で走らせる（appcert.exe は昇格が要る）。
#
#   powershell -ExecutionPolicy Bypass -File tools\wack.ps1 -Package <MSIX の道筋>
#
# 結果の XML と、要約（合否と落ちた検査）を MSIX と同じフォルダに書く。検査は 10〜20 分かかる。
param([Parameter(Mandatory = $true)][string]$Package)
$ErrorActionPreference = 'Stop'
$appcert = 'C:\Program Files (x86)\Windows Kits\10\App Certification Kit\appcert.exe'
if (-not (Test-Path $appcert)) { throw "appcert.exe が無い: $appcert（Windows SDK の App Certification Kit を入れる）" }
$package = (Resolve-Path $Package).Path
$report = [IO.Path]::ChangeExtension($package, '.wack.xml')
& $appcert reset | Out-Null
& $appcert test -appxpackagepath $package -reportoutputpath $report
[xml]$xml = Get-Content $report
$result = $xml.REPORT.OVERALL_RESULT
"総合: $result"
$xml.REPORT.REQUIREMENTS.REQUIREMENT.TEST | Where-Object { $_.RESULT -ne 'PASS' } |
  ForEach-Object { "  $($_.RESULT): $($_.NAME)" }
"報告: $report"
