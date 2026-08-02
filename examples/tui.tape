output "../docs/examples/tui.svg"
theme "catppuccin"
width 96
height 26
window on
title "glowfetch"

sanitize on

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 45ms
trim-idle 500ms
tail 2s

wait  "$" 20s
type  "glowfetch"
enter
sleep 6s
type  "q"
sleep 1200ms
