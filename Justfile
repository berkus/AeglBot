deploy: build
    cp target/release/bot ../aegl-bot/
build:
    cargo build --release
test:
    cargo test
