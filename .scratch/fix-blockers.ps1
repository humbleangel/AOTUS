$root = "C:\Users\SSDGO\Desktop\AOTUS"
Set-Location $root

# Build reverse map: issue → ticket, and ticket → issue
$ticket2issue = @{
  1=1; 2=2; 3=3; 4=4; 5=5; 6=6; 7=7
  8=10; 9=8; 10=11; 11=12; 12=13
}
for ($i = 13; $i -le 55; $i++) { $ticket2issue[$i] = $i + 1 }

# Script block for match evaluator
$eval = {
  $n = [int]$_.Groups[1].Value
  if ($ticket2issue.ContainsKey($n)) { "#$($ticket2issue[$n])" } else { $_.Value }
}

for ($issue = 14; $issue -le 56; $issue++) {
  $body = gh issue view $issue --json body --jq '.body'
  if ($LASTEXITCODE -ne 0 -or !$body) { Write-Host "FAIL read #$issue" -ForegroundColor Red; continue }

  $newBody = [regex]::Replace($body, '(?<=Blocked by:.*)#(\d+)', $eval)
  if ($newBody -ne $body) {
    $body | Set-Content ".scratch\fix-body.md" -Encoding UTF8
    $newBody | Set-Content ".scratch\fix-body.md" -Encoding UTF8
    gh issue edit $issue --body-file ".scratch\fix-body.md" 2>&1 | Out-Null
    Remove-Item ".scratch\fix-body.md" -Force
    Write-Host "Fixed #$issue"
  } else {
    Write-Host "OK #$issue"
  }
}
Write-Host "DONE"
