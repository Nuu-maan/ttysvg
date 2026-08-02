output "../docs/demo.svg"
theme "tokyonight"
width 78
height 18
font-size 14
padding 20
window on
title "ttysvg"

shell "powershell.exe" "-NoLogo" "-NoProfile" "-NoExit" "-Command" "$env:Path = (Resolve-Path ./target/release).Path + ';' + $env:Path; function prompt { 'ttysvg $ ' }; Clear-Host"

type-delay 55ms
trim-idle 700ms
tail 2500ms

wait "ttysvg $" 20s
type "ttysvg themes"
sleep 400ms
enter
wait "nord" 10s
sleep 900ms

type "ttysvg record --out docs/demo.svg -- glowfetch"
sleep 1200ms
