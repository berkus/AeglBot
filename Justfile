@default:
    just --list

deploy: build
    cp target/release/bot ../aegl-bot/
build:
    cargo build --release
test:
    cargo test
run:
    cargo run --bin bot

alias d := deploy
alias b := build
alias t := test
alias r := run
