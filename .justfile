port := "/dev/ttyUSB1"
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
clippy_w := "-W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a
lib := "--package robots-lib"
drv := "--package robots-drv"
back := "--package robots-web -F ssr"
front := "--package robots-web -F hydrate --target wasm32-unknown-unknown"

check-lib:
    {{check}} {{lib}}

check-esp:
    just robots-esp/check

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

mon:
    serial-monitor -p {{port}}

esp-mon:
    just esp
    sleep 1
    just mon

drv:
    cargo run --package robots-drv

web:
    cargo leptos watch


all: clippy-lib clippy-esp clippy-drv test
