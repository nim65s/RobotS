monport := env("ROBOTS_MON", "/dev/ttyUSB1")
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
lib := "--package robots-lib"
drv := "--package robots-drv"
back := "--package robots-web -F ssr"
front := "--package robots-web -F hydrate --target wasm32-unknown-unknown"
clippy_w := "-W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a

check-lib:
    {{ check }} {{ lib }}

check-esp:
    just robots-esp/check

check-stm:
    just robots-stm/check

check-drv:
    {{ check }} {{ drv }}

check-back:
    {{ check }} {{ back }}

check-front:
    {{ check }} {{ front }}

clippy-lib:
    {{ clippy }} {{ lib }} {{ clippy_args }}

clippy-esp:
    just robots-esp/clippy

clippy-stm:
    just robots-stm/clippy

clippy-drv:
    {{ clippy }} {{ drv }} {{ clippy_args }}

clippy-back:
    {{ clippy }} {{ back }} {{ clippy_args }}

clippy-front:
    {{ clippy }} {{ front }} {{ clippy_args }}

test:
    {{ test }} {{ lib }}

esp:
    just robots-esp/esp

stm:
    just robots-stm/stm

mon:
    python -m serial.tools.miniterm {{ monport }} 115200

esp-mon:
    just esp
    sleep 1
    just mon

drv:
    cargo run --package robots-drv

web:
    cargo leptos watch

all: clippy-lib clippy-esp clippy-stm clippy-drv clippy-back clippy-front test
