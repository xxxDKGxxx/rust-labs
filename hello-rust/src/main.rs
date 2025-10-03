use rand::Rng;
use std::{fs::File, io::{self, Write}};

fn powers(x: u64) -> [u64; 10] {
    let mut arr = [0u64; 10];

    arr[0] = x;

    for i in 1..arr.len() {
        arr[i] = arr[i - 1] * x;
    }

    arr
}

fn collatz_holds(mut n: u64) -> bool {
    for _ in 0..100 {
        if n == 1 {
            return true;
        }
        if n.is_multiple_of(2) {
            n /=  2;
        } else {
            n = 3 * n + 1;
        }
    }

    false
}

fn check_collatz(arr: &[u64; 10]) -> [bool; 10] {
    let mut result = [false; 10];
    for i in 0..10 {
        result[i] = collatz_holds(arr[i]);
    }
    result
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

        let file_str = format!("{:?}", collatz_results);

        if file.write_all(file_str.as_bytes()).is_err() {
            break true
        };
    };

    if finished {
        println!("Wystąpił błąd");
        return;
    }

    println!("Wyjście z woli użytkownika")
}
