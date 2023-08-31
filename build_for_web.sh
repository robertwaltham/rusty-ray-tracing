# todo: output dir
# todo: include --cfg=web_sys_unstable_apis flag manually
cargo build \
    --target wasm32-unknown-unknown \
    --release

wasm-bindgen \
    --out-dir ~/Cargo/wasm32-unknown-unknown/release \
    --target web \
    --no-typescript \
    --weak-refs \
    --reference-types \
    ~/Cargo/wasm32-unknown-unknown/release/rusty-ray-tracing.wasm

cp ~/Cargo/wasm32-unknown-unknown/release/rusty-ray-tracing_bg.wasm ./pages/api/wasm.wasm
cp ~/Cargo/wasm32-unknown-unknown/release/rusty-ray-tracing.js ./pages/api/wasm.js

git status
