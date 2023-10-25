use std::env;
use std::process;

mod client;

macro_rules! ERROR_CODE {
    () => { 84 };
}

fn usage(err_state: bool) {
    if err_state == true {
        eprintln!("CLIENT USAGE: ./client [url]\n\tcargo run [url]");
        process::exit(ERROR_CODE!());
    }
}

fn error_handling() -> String {
    let args: Vec<String> = env::args().collect();
    let mut url = String::from("");
    let mut err_state: bool = false;

    match args.len() {
        1 => {
            eprintln!("Error: Too few arguments");
            err_state = true;
        }
        2 => {
            url = match env::args().nth(1) {
                Some(url) => url,
                None => {
                    eprintln!("Error: Pass a valid HTTP URL as an argument");
                    process::exit(ERROR_CODE!());
                }
            };
        }
        _ => {
            eprintln!("Error: Too many arguments");
            err_state = true;
        }
    }
    usage(err_state);
    url
}

fn main() {
    let url = error_handling();
    let _ = client::packet_response(&url);
}
