use log::info;

use std::io::{self, Write};

fn main() {
    info!(io::stdout(), "hello");
}
