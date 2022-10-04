cargo build --release
wasm-bindgen --target web --out-dir web/generated ./target/wasm32-unknown-unknown/release/pdf_edit.wasm
