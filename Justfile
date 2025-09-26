@default:
    just --list

deploy: build-ru
    cp target/release/bot ../aegl-bot/
build-ru:
    BOT_LANGUAGE=ru cargo build --release
build-en:
    BOT_LANGUAGE=en cargo build --release
test:
    BOT_LANGUAGE=en cargo test
run-ru:
    BOT_LANGUAGE=ru cargo run --bin bot
run:
    BOT_LANGUAGE=en cargo run --bin bot

alias d := deploy
alias b := build-ru
alias t := test
alias r := run-ru
