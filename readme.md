## Project Structure

* engine - Typescript game engine
* server - Simple rust web server for development
* game - Rust game client
* build - Build folder for `game`
* tools - Build utilities to preprocess assets

## Building instruction

Requires Rust, wasm-pack (https://github.com/rustwasm/wasm-pack), and node


### Typescript engine

```
npm install
npx rollup --config rollup.config.mjs  --watch
```

### WASM game client

```
cd game

wasm-pack build --out-dir ../build/game --target web
```

### Dev Server

```
cargo run -p server --release
```

### Tools
```
cargo run -p tools --release -- -c *command_name* -f *filters*
```


## Credits

Tinysword - https://pixelfrog-assets.itch.io/tiny-swords 
Delaunator - https://github.com/mourner/delaunator-rs