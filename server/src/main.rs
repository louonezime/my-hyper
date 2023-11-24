use std::{env, process};

mod server;

static ERROR_CODE: i32 = 84;

fn server_usage() {
    eprintln!("CLIENT USAGE: ./server\n\tcargo run");
}

fn main() {
    let mut args = env::args_os();
    args.next();

    if let Some(os_mode) = args.next() {
        if let Ok(_) = os_mode.into_string() {
            eprintln!("Error: Too many arguments");
            server_usage();
            process::exit(ERROR_CODE);
        }
    }

    server::create_serv();
}
