use std::{
    env,
    process,
    ffi::OsString
};

mod client;

static ERROR_CODE: i32 = 84;

fn client_usage() {
    eprintln!("CLIENT USAGE: ./client [url]\n\tcargo run [url]");
}

fn error_handling(url: OsString, mut args: env::ArgsOs) -> String {
    if args.next().is_none() {
        match url.into_string() {
            Ok(url) => return url,
            Err(_) => eprintln!("Error: Pass a valid HTTP URL as an argument"),
        }
    } else {
        eprintln!("Error: Too many arguments");
    }

    client_usage();
    process::exit(ERROR_CODE);
}

#[tokio::main]
async fn main() {
    let mut args = env::args_os();

    args.next();

    if let Some(res) = args.next() {
        let url = error_handling(res, args);
        if let Err(err) = client::packet_response(&url).await {
            eprintln!("Error: {}", err);
            return;
        }
        return;
    } else {
        eprintln!("Error: Too few arguments");
    }

    client_usage();
    process::exit(ERROR_CODE);
}
