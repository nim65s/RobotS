set fallback := true

port := env("ROBOTS_PORT", "/dev/ttyUSB0")
check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
clippy_w := "-W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a

check:
    {{ check }}

clippy:
    {{ clippy }} {{ clippy_args }}

esp:
    cargo espflash flash --release --port {{ port }}

all: check clippy
