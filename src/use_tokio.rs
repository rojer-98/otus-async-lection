use std::{thread::sleep, time::Duration};

use rand::random;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, Result as TResult},
    runtime::Builder,
};

pub async fn random_print(num: i32) {
    let r = random::<u64>() % 100;

    println!("Get random {r}");

    sleep(Duration::from_millis(r));

    println!("{num} is complete");
}

pub fn test_tokio() {
    let rt = Builder::new_multi_thread()
        .worker_threads(8)
        .enable_io()
        .build()
        .unwrap();

    for j in 0..10 {
        rt.spawn(random_print(j));
    }

    sleep(Duration::from_millis(5000));
}

pub async fn read_file(file_name: &str) -> TResult<String> {
    let mut f = File::open(file_name).await?;
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).await?;

    Ok(String::from_utf8(buffer).unwrap())
}

pub async fn write_file(file_name: &str) -> TResult<()> {
    let mut f = File::create(file_name).await?;
    let n = f.write(b"Hello World!").await?;

    println!("Wrote bytes: {n}");

    Ok(())
}
