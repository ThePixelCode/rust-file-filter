use std::env::args;
use rust_file_filter::*;

fn main() {
    match get_operation_handler(args()) {
        Ok(mut op) => {
            match op.run() {
                Ok(_) => (),
                Err(e) => print_error_and_gracefully_exit(e),
            };
        },
        Err(e) => print_error_and_gracefully_exit(e),
    }
}
