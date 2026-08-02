output "../docs/examples/banner.svg"
theme "tokyonight"
width 78
height 16
padding 20
window on
title "ttysvg"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { 'ttysvg $ ' }; Clear-Host"

type-delay 55ms
trim-idle 700ms
tail 2500ms

wait  "ttysvg $" 20s
type  "ttysvg --help"
enter
wait  "Usage" 10s
sleep 1500ms
