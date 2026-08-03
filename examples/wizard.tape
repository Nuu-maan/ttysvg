output "../docs/examples/wizard.svg"
theme "tokyonight"
width 100
height 26
padding 20
window on
title "ttysvg"

sanitize on

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

type-delay 55ms
trim-idle 500ms
tail 3s

wait  "$" 20s
type  "ttysvg"
enter
sleep 2s

down
sleep 700ms
down
sleep 1200ms
enter
sleep 600ms

type  "cargo --version"
enter
enter
sleep 900ms

down
sleep 1000ms
down
sleep 1400ms
enter
sleep 800ms

down
down
sleep 900ms
space
sleep 1400ms
enter
sleep 1600ms

enter
wait  "recorded" 90s
sleep 2500ms

right
sleep 1800ms
right
sleep 1800ms
right
sleep 2500ms
