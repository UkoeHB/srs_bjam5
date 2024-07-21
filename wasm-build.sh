# Building for WASM
cp -r assets wasm/ && \
cargo build --target wasm32-unknown-unknown --release && \
wasm-bindgen --no-typescript --out-name srs_bjam5 --out-dir wasm --target web target/wasm32-unknown-unknown/release/srs_bjam5.wasm && \
wasm-opt -Os wasm/srs_bjam5_bg.wasm -o wasm/srs_bjam5_bg.wasm && \
zip -r srs_bjam5.zip wasm
