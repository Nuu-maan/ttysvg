output "../docs/examples/secrets.svg"
theme "gruvbox"
width 92
height 14
window on
title "deploy"

sanitize on
redact "sk-[A-Za-z0-9]+"
redact "ghp_[A-Za-z0-9]+"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 45ms
trim-idle 600ms
tail 3s

wait  "$" 15s
type  "Write-Host \"key=sk-Ab3xQ9zz token=ghp_9dK2vLm4 home=$env:USERPROFILE\""
enter
wait  "key=" 15s
sleep 2500ms
