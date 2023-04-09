port := "/dev/ttyUSB0"
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
clippy_w := "-W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a
lib := "--package robots-lib"
drv := "--package robots-drv"

check-lib:
    {{check}} {{lib}}

check-esp:
    just robots-esp/check

check-drv:
    {{check}} {{drv}}

clippy-lib:
    {{clippy}} {{lib}}

clippy-esp:
    just robots-esp/clippy

clippy-drv:
    {{clippy}} {{drv}}

test:
    {{test}} {{lib}}

esp:
    just robots-esp/esp

mon:
    just robots-esp/mon

drv:
    cargo run --package robots-drv

dbg:
    serial-monitor -p /dev/ttyUSB1

all: clippy-lib clippy-esp clippy-drv test
