output "../docs/examples/bash.svg"
theme "tokyonight"
width 84
height 18
window on
title "bash"

shell "bash" "--norc" "-i"

type-delay 55ms
trim-idle 700ms
tail 2500ms

wait  "$" 15s
type  "export PS1='demo $ ' && clear"
enter
sleep 800ms

type  "ttysvg themes | tr '\n' ' '"
enter
wait  "nord" 15s
sleep 1500ms
