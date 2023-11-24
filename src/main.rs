use chip8::emulator::Chip8;
use macroquad::prelude::*;
use std::io;
use std::{env, process::exit};

fn conf() -> Conf {
    Conf {
        window_title: String::from("Chip8 Emulator"),
        window_width: 64 * 24,
        window_height: 32 * 24,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("ROM file not specified in the arguements");
        exit(1);
    }

    let mut e = Chip8::new();
    let res = e.load_from_file(&args[1]);

    if let Err(e) = res {
        match e.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("No such file exists");
            }
            _ => {
                eprintln!("Error reading the file");
            }
        }
        exit(1);
    }

    loop {
        println!("Framerate : {}", get_fps());
        e.run();
        next_frame().await;
    }
}
