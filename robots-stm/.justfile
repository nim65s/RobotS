set fallback

check := "cargo check --color always"
clippy := "cargo clippy --color always"
test := "cargo test --color always"
clippy_w := "-W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used"
clippy_a := "-A clippy::missing-errors-doc -A clippy::missing-panics-doc"
clippy_args := "-- " + clippy_w + " " + clippy_a

check:
    {{check}}

clippy:
    {{clippy}} {{clippy_args}}

stm:
    cargo run --release
