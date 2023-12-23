port := "/dev/ttyUSB1"
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
lib := "--package robots-lib"
drv := "--package robots-drv"
back := "--package robots-web -F ssr"
front := "--package robots-web -F hydrate --target wasm32-unknown-unknown"

check-lib:
    {{check}} {{lib}}

check-esp:
    just robots-esp/check

check-stm:
    just robots-stm/check

check-drv:
    {{check}} {{drv}}

check-back:
    {{check}} {{back}}

check-front:
    {{check}} {{front}}

clippy-lib:
    {{clippy}} {{lib}}

clippy-esp:
    just robots-esp/clippy

clippy-stm:
    just robots-stm/clippy

clippy-drv:
    {{clippy}} {{drv}}

clippy-back:
    {{clippy}} {{back}}

clippy-front:
    {{clippy}} {{front}}

test:
    {{test}} {{lib}}

esp:
    just robots-esp/esp

stm:
    just robots-stm/stm

mon:
    python -m serial.tools.miniterm {{port}} 115200

esp-mon:
    just esp
    sleep 1
    just mon

drv:
    cargo run --package robots-drv

web:
    cargo leptos watch


all: clippy-lib clippy-esp clippy-stm clippy-drv test
