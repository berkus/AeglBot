deploy: build
    cp target/release/bot ../aegl-bot/
build:
    cargo build --release
test:
    cargo test

alias d := deploy
alias b := build
alias t := test
