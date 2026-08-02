output "../docs/examples/prompt.svg"
theme "nord"
width 84
height 18
window on
title "setup"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 60ms
trim-idle 600ms
tail 3s

wait  "$" 15s
type  "$name = Read-Host 'Project name'"
enter
wait  "Project name" 10s
sleep 900ms
type  "my-app"
enter
sleep 800ms

type  "$stack = Read-Host 'Framework'"
enter
wait  "Framework" 10s
sleep 900ms
type  "vite"
enter
sleep 800ms

type  "Write-Host \"scaffolding $name with $stack\""
enter
wait  "scaffolding" 10s
sleep 1500ms
