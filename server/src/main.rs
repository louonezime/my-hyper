use std::env;
use std::process;

mod server;



macro_rules! ERROR_CODE {
    () => { 84 };
}

fn usage(err_state: bool) {
    if err_state == true {
        eprintln!("CLIENT USAGE: ./server\n\tcargo run");
        process::exit(ERROR_CODE!());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut err_state: bool = false;

    match args.len() {
        1 => server::create_serv(),
        _ => {
            eprintln!("Error: Too many arguments");
            err_state = true;
        }
    }
    usage(err_state);
}
