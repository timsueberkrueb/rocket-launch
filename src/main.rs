extern crate notify;
extern crate clap;

use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::PathBuf;
use std::fs::metadata;
use std::env;
use std::process::{Command, Child, exit};

use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
use clap::{App, Arg};

// Files with those extensions trigger a relaunch
static WHITELISTED_EXTENSIONS: &'static [&'static str] = &[
    ".rs",      // Rust source file
    ".toml",    // TOML files (e.g. Cargo.toml)
    ".tera",    // Tera templates
    ".hbs",     // Handlebars templates
    ".html",    // HTML files
    ".js",      // JavaScript files
];

// Default interval to check for changes (in seconds)
const DEFAULT_CHECK_INTERVAL: f32 = 0.5;

fn is_whitelisted(path: PathBuf) -> bool {
    let path_str = path.to_str().unwrap_or("");
    WHITELISTED_EXTENSIONS.iter().any(|ext| path_str.ends_with(ext))
}

fn lift_cargo(project_path: &str, args: &[&str]) -> Child {
    println!("⚙ Running cargo {}", args.join(" "));
    // Filter out rustup environment variables to prevent interference
    // in case rocket launch itself is running under cargo:
    let child_env: Vec<(std::ffi::OsString, std::ffi::OsString)> = env::vars_os()
        .filter(|&(ref key, _)| {
            !key.to_str().unwrap_or("").starts_with("RUSTUP")
        }).collect();

    Command::new("cargo")
        .current_dir(project_path)
        .env_clear()
        .envs(child_env)
        .args(args)
        .spawn()
        .expect("Error creating process")
}

fn relaunch_cargo(project_path: &str, cargo_args: &[&str]) -> Child {
    let mut args_vec = vec!["run"];
    args_vec.extend(cargo_args);
    lift_cargo(project_path, &args_vec)
}

fn ignite_boosters(project_path: &str, interval: f32, cargo_args: &[&str]) {
    // Create event channel
    let (tx, rx) = channel();

    // Create filesystem watcher
    let mut watcher = watcher(tx, Duration::from_millis((interval * 1000.0).round() as u64))
        .unwrap_or_else(|e| panic!("Error creating filesystem watcher: {}", e));

    // Watch project path
    watcher.watch(project_path, RecursiveMode::Recursive)
        .unwrap_or_else(|e| panic!("Error watching path: {}", e));

    let mut cargo_p = relaunch_cargo(project_path, cargo_args);

    loop {
        match rx.recv() {
            Ok(event) => {
                let mut trigger = false;
                match event {
                    DebouncedEvent::NoticeWrite(path)
                    | DebouncedEvent::NoticeRemove(path)
                    | DebouncedEvent::Create(path)
                    | DebouncedEvent::Write(path)
                    | DebouncedEvent::Chmod(path)
                    | DebouncedEvent::Remove(path) => {
                        trigger = is_whitelisted(path);
                    }
                    DebouncedEvent::Rename(old, new) => {
                        trigger = is_whitelisted(old)
                            || is_whitelisted(new);
                    }
                    _ => {}
                }
                if trigger {
                    match static_fire(project_path) {
                        Ok(_) => {
                            cargo_p.kill().unwrap_or_else(|e| panic!("Error terminating cargo: {}", e));
                            println!("⚙ Stopped (cargo run)");
                            cargo_p = relaunch_cargo(project_path, cargo_args);
                        },
                        Err(_) => {},
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e)
        }
    }
}

fn static_fire(project_path: &str) -> Result<(), &str> {
    println!("⚙ Running cargo check");
    let status = lift_cargo(project_path, &["check"]).wait()
        .expect("Did not receive an exit status from cargo check");
    println!("⚙ Done (cargo check)");
    if !status.success() {
        Err("Error during cargo check")
    } else {
        Ok(())
    }
}

fn main() {
    let matches = App::new("Rocket Launch")
        .version("0.1.0")
        .author("Tim Süberkrüb <dev@timsueberkrueb.io>")
        .about("Watches your Cargo project for changes and \
                relaunches Cargo when changes are detected.")
        .arg(Arg::with_name("project")
            .help("Path to your project (default: current directory)"))
        .arg(Arg::with_name("seconds")
            .long("interval")
            .short("i")
            .takes_value(true)
            .help("Interval to check for filesystem changes"))
        .arg(Arg::with_name("cargo_args")
            .multiple(true)
            .help("Cargo arguments (usage: <launcher-args> -- <cargo-args>)")
        )
        .get_matches();

    let project_path: String = match matches.value_of("project") {
        Some(path) => path.to_string(),
        None => std::env::current_dir()
            .expect("Error obtaining working directory")
            .to_str().unwrap().to_string()
    };
    let interval: f32 = match matches.value_of("seconds") {
        Some(interval_str) => interval_str.parse::<f32>().expect("Invalid interval"),
        None => DEFAULT_CHECK_INTERVAL
    };
    let cargo_args: Vec<&str> = match matches.value_of("cargo_args") {
        Some(args_str) => args_str.split(" ").collect(),
        None => Vec::new()
    };

    let is_existing_dir = match metadata(&project_path) {
        Ok(m) => m.is_dir(),
        Err(_) => false
    };
    if !is_existing_dir {
        eprintln!("\"{}\" is not a valid directory", &project_path);
        exit(1);
    }

    match static_fire(&project_path) {
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        },
        _ => {}
    }

    ignite_boosters(&project_path, interval, &cargo_args);
}
