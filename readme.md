## Project Structure

* engine - Typescript graphics engine
* server - Simple rust web server for development
* game - Rust game client. Compiles to wasm
* build - Build folder for `game`
* tools - Build utilities to preprocess assets

## Building instruction

Requires Rust, wasm-pack (https://github.com/rustwasm/wasm-pack), and node

### Server

```
cargo run -p server --release
```

### Webdemo typescript client

```
npm install
npx rollup --config rollup.config.mjs  --watch
```

### Webdemo wasm application

```
cd game

wasm-pack build --out-dir ../build/game --target web
```

### Tools
```
cargo run -p tools --release -- -c *command_name* -f *filters*
```

