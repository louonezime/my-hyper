use std::{env, process};

mod hyper_client;
mod reverse_proxy;

static ERROR_CODE: i32 = 84;

fn client_usage() {
    eprintln!("CLIENT USAGE: ./http_proxy_poc client [url]\n\tcargo run client [url]");
}

fn proxy_usage() {
    eprintln!("SERVER USAGE: ./http_proxy_poc proxy\n\tcargo run proxy");
}

fn manage_errors_client(mut args: env::ArgsOs) -> String {
    if let Some(url) = args.next() {
        if args.next().is_none() {
            match url.into_string() {
                Ok(url) => return url,
                Err(_) => eprintln!("Error: Pass a valid HTTP URL as an argument"),
            }
        } else {
            eprintln!("Error: Too many arguments");
        }
    } else {
        eprintln!("Error: Too few arguments");
    }

    client_usage();
    process::exit(ERROR_CODE);
}

fn manage_errors_serv(mut args: env::ArgsOs) {
    if args.next().is_some() {
        eprintln!("Error: Too many arguments");
        proxy_usage();
        process::exit(ERROR_CODE);
    }

    reverse_proxy::create_serv()
}

fn main() {
    let mut args = env::args_os();

    // skip executable name
    args.next();

    if let Some(os_mode) = args.next() {
        if let Ok(mode) = os_mode.into_string() {
            match mode.as_str() {
                "client" => {
                    let url = manage_errors_client(args);
                    let _ = hyper_client::packet_response(&url);
                    return;
                }
                "proxy" => {
                    manage_errors_serv(args);
                    return;
                }
                _ => {}
            }
        }
    }

    client_usage();
    proxy_usage();
    process::exit(ERROR_CODE);
}
