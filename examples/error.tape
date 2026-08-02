output "../docs/examples/error.svg"
theme "catppuccin"
width 88
height 18
window on
title "the fix"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 50ms
trim-idle 800ms
tail 4s

wait  "$" 15s
type  "ttysvg render missing.json"
enter
wait  "Caused by" 20s
sleep 2500ms

type  "ttysvg themes"
enter
wait  "nord" 10s
sleep 2s
