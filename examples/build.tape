output "../docs/examples/build.svg"
theme "gruvbox"
width 96
height 24
window on
title "cargo test"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "function prompt { '$ ' }; Clear-Host"

speed 3.0
trim-idle 400ms
tail 3s

wait  "$" 15s
type  "cargo test"
enter
wait  "test result" 300s
sleep 2500ms
