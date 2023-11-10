#![feature(noop_waker)]

mod new_runtime;
mod pinned;
mod runtime;
mod use_tokio;

use new_runtime::runtime_main;
use pinned::{pin_test, pin_test_simple};
use use_tokio::{read_file, write_file};

#[tokio::main]
async fn main() {
    if let Err(e) = write_file("foo.txt").await {
        println!("{e}");
    } else {
        let s = read_file("foo.txt").await.unwrap();

        println!("File is: {s}");
    }
}
