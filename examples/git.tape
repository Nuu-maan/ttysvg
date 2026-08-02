output "../docs/examples/git.svg"
theme "nord"
width 88
height 16
window on
title "git"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 45ms
trim-idle 500ms
tail 3s

wait  "$" 15s
type  "git --no-pager log --oneline -5"
enter
sleep 1800ms

type  "git status --short --branch"
enter
sleep 2500ms
