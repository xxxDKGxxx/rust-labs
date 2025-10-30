use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::num::NonZero;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::str::{FromStr, from_utf8};
use std::time::Instant;
use std::{fs, hint, io};

fn divisors(n: NonZero<u32>) -> BTreeSet<NonZero<u32>> {
    let mut result = BTreeSet::<NonZero<u32>>::new();
    let value = n.get();

    for i in 2..n.get() {
        if !value.is_multiple_of(i) {
            continue;
        }

        let Some(val) = NonZero::<u32>::new(i) else {
            continue;
        };

        result.insert(val);
    }

    result
}

fn assert_sorted(buf: &[i32]) {
    let vector = Vec::<i32>::from(buf);

    for window in vector.windows(2) {
        if window[0] <= window[1] {
            continue;
        }

        panic!();
    }
}

fn divisors_benchmark(iter: u16) {
    let start = Instant::now();

    for i in 1..iter {
        let val = NonZero::<u32>::new(i.into()).expect("");

        hint::black_box(divisors(hint::black_box(val)));
    }

    let elapsed = Instant::now() - start;
    let mean_time = elapsed.as_secs_f64() / iter as f64 * 1000f64;

    println!("Mean time: {}", mean_time);
}

fn bulk_read(stream: &mut TcpStream, size: usize) -> io::Result<Vec<u8>> {
    let mut result: Vec<u8> = vec![0; size];
    let mut count = 0;

    loop {
        if count == size {
            break;
        }

        let read_bytes = stream.read(&mut result[count..size])?;

        if read_bytes == 0 {
            break;
        }

        count += read_bytes;
    }

    if count < result.len() {
        result.resize(count, 0);
    }

    Ok(result)
}

fn bulk_write(stream: &mut TcpStream, buf: &[u8]) -> io::Result<()> {
    let mut count = 0;

    loop {
        if count == buf.len() {
            break;
        }

        let written_bytes = stream.write(&buf[count..buf.len()])?;

        if written_bytes == 0 {
            break;
        }

        count += written_bytes;
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    loop {
        let mut path: Vec<u8> = Vec::new();

        loop {
            let path_len = bulk_read(&mut stream, 4)?;
            let path_len_str = from_utf8(&path_len[..]).unwrap_or("0000");
            let path_len_usize: usize = path_len_str.trim_start_matches('0').parse().unwrap_or(0);
            let mut read_bytes = bulk_read(&mut stream, path_len_usize)?;

            if read_bytes.len() < path_len_usize {
                continue;
            }

            path.append(&mut read_bytes);

            break;
        }

        let path_str = match String::from_utf8(path) {
            Err(_) => {
                bulk_write(&mut stream, "Conversion error\n".as_bytes())?;
                return Ok(());
            }
            Ok(v) => v,
        };

        let path_buf = match PathBuf::from_str(path_str.trim()) {
            Ok(path) => path,
            Err(_) => {
                bulk_write(&mut stream, "Bad path\n".as_bytes())?;
                return Ok(());
            }
        };

        println!("Valid path: {:?}", path_str);
        let mut response = Vec::<u8>::new();
        let read_dir = match fs::read_dir(path_buf) {
            Ok(read_dir) => read_dir,
            Err(e) => {
                bulk_write(&mut stream, "Bad dir\n".as_bytes())?;
                println!("Error occured while reading dir: {}\n", e);
                return Ok(());
            }
        };

        for direntry in read_dir {
            let direntry = match direntry {
                Err(e) => {
                    bulk_write(&mut stream, "Bad dir\n".as_bytes())?;
                    println!("Error occured while reading direntry: {}\n", e);
                    return Ok(());
                }

                Ok(direntry) => direntry,
            };
            response.append(&mut Vec::<u8>::from(direntry.file_name().as_bytes()));
            response.append(&mut Vec::<u8>::from("\n".as_bytes()));
        }

        bulk_write(&mut stream, &response)?;

        println!("Response written successfully");
    }
}

fn main() {
    divisors_benchmark(10);

    let arr = [1, 2, 3, 4, 5];
    assert_sorted(&arr);

    let val = NonZero::new(12).unwrap();
    let divs = divisors(val);
    println!("Divisors of {}: {:?}", val, divs);

    let listener = TcpListener::bind("localhost:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("New client");

        if let Err(e) = handle_client(stream) {
            println!("Error occured in handle_client: {}", e)
        };
    }
}
