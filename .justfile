port := "/dev/ttyUSB0"
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
clippy_w := "-W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a
esp := "--package robots-esp --target=riscv32imc-unknown-none-elf"
lib := "--package robots-lib"
drv := "--package robots-drv"

check-lib:
    {{check}} {{lib}}

check-esp:
    {{check}} {{esp}}

check-drv:
    {{check}} {{drv}}

clippy-lib:
    {{clippy}} {{lib}}

clippy-esp:
    {{clippy}} {{esp}}

clippy-drv:
    {{clippy}} {{drv}}

test:
    {{test}} {{lib}}

esp:
    cargo espflash --release {{esp}} {{port}}

mon:
    cargo espflash --release {{esp}} --monitor {{port}}

all: clippy-lib clippy-esp clippy-drv test
