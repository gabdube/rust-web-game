mod shared;
mod sprites;
mod packing;

mod generate_objects_sprites;
mod generate_characters_sprites;
mod generate_fonts;

fn command_name() -> Option<String> {
    let position = ::std::env::args().position(|arg| arg.as_str() == "-c");
    position.and_then(|p| ::std::env::args().skip(p+1).next() )
}

fn main() {
    let cmd = match command_name() {
        Some(cmd) => cmd,
        None => {
            println!("Missing command. Usage:");
            println!("cargo run -p tools -- -c *command_name* *arguments*");
            return;
        }
    };

    match cmd.as_str() {
        "generate_characters_sprites" => {
            generate_characters_sprites::generate_sprites();
        },
        "generate_objects_sprites" => {
            generate_objects_sprites::generate_sprites();
        },
        "generate_fonts" => {
            generate_fonts::generate_fonts();
        },
        _ => {
            eprintln!("Unknown command name {:?}", cmd);
        }
    }
}
