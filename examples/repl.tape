output "../docs/examples/repl.svg"
theme "tokyonight"
width 76
height 18
padding 22

shell "python" "-q" "-i"

type-delay 75ms
trim-idle 600ms
tail 3s

wait  ">>>" 20s
type  "import json"
enter
sleep 600ms

type  "json.dumps({'frames': 91, 'seconds': 7.2})"
enter
wait  "frames" 10s
sleep 1200ms

type  "sum(len(w) for w in 'terminal to animated svg'.split())"
enter
sleep 2s
