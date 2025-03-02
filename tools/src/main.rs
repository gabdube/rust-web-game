mod shared;
mod generate_static_sprites;
mod generate_animated_sprites;
mod generate_effects_sprites;

fn filters() -> Option<Vec<String>> {
    let index = ::std::env::args().position(|arg| arg.as_str() == "-f" || arg.as_str() == "--filters" )?;
    let filters = ::std::env::args().skip(index + 1).next()?;
    Some(filters.split(',').map(|v| v.to_string() ).collect())
}

fn command_name() -> Option<String> {
    let position = ::std::env::args().position(|arg| arg.as_str() == "-c");
    position.and_then(|p| ::std::env::args().skip(p+1).next() )
}

fn main() {
    let filters = filters().unwrap_or_default();
    let cmd = match command_name() {
        Some(cmd) => cmd,
        None => {
            println!("Missing command. Usage:");
            println!("cargo run -p tools -- -c *command_name* *arguments*");
            return;
        }
    };

    match cmd.as_str() {
        "generate_animated_sprites" => {
            generate_animated_sprites::generate_sprites(&filters);
        },
        "generate_static_sprites" => {
            generate_static_sprites::generate_sprites();
        },
        "generate_effects_sprites" => {
            generate_effects_sprites::generate_effects();
        },
        _ => {
            eprintln!("Unknown command name {:?}", cmd);
        }
    }
}
