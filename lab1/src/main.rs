use rand::Rng;
use std::{
    fs::File,
    io::{self, Write},
};

const COLLATZ_ITER_NUM: u8 = 100;
const NUM_ARR_SIZE: usize = 10;

fn powers(x: u64) -> [u64; NUM_ARR_SIZE] {
    let mut arr = [0u64; NUM_ARR_SIZE];

    arr[0] = x;

    for i in 1..arr.len() {
        arr[i] = arr[i - 1] * x;
    }

    arr
}

fn collatz_single(mut n: u64) -> bool {
    for _ in 0..COLLATZ_ITER_NUM {
        if n == 1 {
            return true;
        }
        if n.is_multiple_of(2) {
            n /= 2;
        } else {
            n = 3 * n + 1;
        }
    }

    false
}

fn check_collatz(arr: &[u64; NUM_ARR_SIZE]) -> [bool; 10] {
    let mut result = [false; NUM_ARR_SIZE];
    for i in 0..10 {
        result[i] = collatz_single(arr[i]);
    }
    result
}

fn double_loop_tuple_returner(prob: f64) -> (usize, [u8; 10]) {
    let mut idx: usize = 0;
    let mut arr: [u8; NUM_ARR_SIZE] = [0u8; NUM_ARR_SIZE];
    let mut rng = rand::rng();
    let mut break_count = 0;

    'outer: loop {
        if idx == arr.len() {
            break 'outer;
        }
        arr[idx] = 1;
        idx += 1;
        'inner: loop {
            if rng.random_bool(prob) {
                break_count += 1;
                break 'inner;
            }
            if idx == arr.len() {
                break 'outer;
            }
            arr[idx] = 0;
            idx += 1;
        }
    }
    (break_count, arr)
}

fn main() {
    let finished: bool = loop {
        println!("Podaj liczbę:");

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Błąd odczytu");

        let x: u64 = match input.trim().parse() {
            Ok(0) => break false,
            Err(_) => break true,
            Ok(num) => num,
        };

        let mut rng = rand::rng();
        let r: u64 = rng.random_range(0..=5);
        let new_x = x + r;

        println!("Nowa wartość x = {}", new_x);

        let potegi = powers(new_x);

        println!("Potęgi x: {:?}", potegi);

        let collatz_results = check_collatz(&potegi);

        println!("Hipoteza Collatza (true/false): {:?}", collatz_results);

        let mut file = match File::create("xyz.txt") {
            Err(_) => break true,
            Ok(file) => file,
        };

        let collatz_results = format!("{:?}", collatz_results);

        if file.write_all(collatz_results.as_bytes()).is_err() {
            break true;
        };
    };

    if finished {
        println!("Wystąpił błąd");
        return;
    }

    println!("Wyjście z woli użytkownika");

    let (idx, arr) = double_loop_tuple_returner(0.2);

    println!("Liczba break'ów: {}, stan tablicy {:?}", idx, arr);
}
