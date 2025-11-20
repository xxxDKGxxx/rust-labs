use itertools::{Iterate, Itertools};
use std::collections::{BTreeSet, HashSet};

// Nie zmieniaj ciała tej funkcji — jedynie typy.
pub fn wrap_call(f1: impl Fn(u32) -> u32, f2: impl FnOnce(u32, u32) -> u32) -> u32 {
    let f1_rename = f1;
    f2(f1_rename(1), f1_rename(2))
}

pub fn sum_squares_odd_loop(list: &[u32]) -> u32 {
    let mut sum = 0;

    for el in list {
        if el % 2 == 1 {
            sum += el * el;
        }
    }

    sum
}

pub fn sum_squares_odd(list: &[u32]) -> u32 {
    list.iter()
        .copied()
        .filter(|el| el % 2 == 1)
        .map(|el| el * el)
        .sum()
}

pub fn vertices_loop(edges: &[(u32, u32)]) -> Vec<u32> {
    let mut result = Vec::<u32>::new();

    for edge in edges {
        if !result.contains(&edge.0) {
            result.push(edge.0);
        }
        if !result.contains(&edge.1) {
            result.push(edge.1);
        }
    }

    result.sort();
    result
}

pub fn vertices(edges: &[(u32, u32)]) -> Vec<u32> {
    edges
        .iter()
        .flat_map(|edge| vec![edge.0, edge.1])
        .unique()
        .sorted()
        .collect::<Vec<u32>>()
}

// Zwraca posortowany rosnąco wektor wierzchołków uczestniczących w jakimkolwiek
// cyklu długości 2 (u->v oraz v->u, u!=v), bez duplikatów.
pub fn cycles_2_loop(edges: &[(u32, u32)]) -> Vec<u32> {
    let mut set = HashSet::<(u32, u32)>::new();
    let mut result = Vec::<u32>::new();

    for edge in edges {
        if set.contains(&(edge.1, edge.0)) && edge.0 != edge.1 {
            if !result.contains(&edge.0) {
                result.push(edge.0);
            }

            if !result.contains(&edge.1) {
                result.push(edge.1);
            }

            continue;
        }

        set.insert(*edge);
    }

    result.sort();
    result
}

pub fn cycles_2(edges: &[(u32, u32)]) -> Vec<u32> {
    edges
        .iter()
        .cartesian_product(edges)
        .filter(|pair| pair.0.0 != pair.0.1)
        .filter(|pair| pair.0.0 == pair.1.1)
        .filter(|pair| pair.0.1 == pair.1.0)
        .flat_map(|pair| vec![pair.0.0, pair.0.1])
        .unique()
        .sorted()
        .collect::<Vec<u32>>()
}

pub fn primes_loop(n: u32) -> Vec<u32> {
    let mut sito = Vec::<u32>::with_capacity(n as usize);
    let mut result = Vec::<u32>::new();

    for _ in 1..=n + 1 {
        sito.push(0);
    }

    for i in 2..n {
        if sito[i as usize] == 0 {
            result.push(i);
        }

        let mut curr_num = i;

        loop {
            if curr_num > n {
                break;
            }

            sito[curr_num as usize] = 1;

            curr_num += curr_num
        }
    }

    result
}

pub fn primes(n: u32) -> Vec<u32> {
    let mut sito = (1..=n + 1).map(|x| 0).collect::<Vec<u32>>();

    (2..n).fold(Vec::new(), |acc, el| {
        if sito[el as usize] == 0 {
            acc.push(el);
            sito = sito.iter().enumerate().map(|el| {
                if el.0 
            })
        }
    })
}

pub fn run_length_encode_loop(list: &[u32]) -> Vec<(u32, usize)> {
    todo!()
}

pub fn run_length_encode(list: &[u32]) -> Vec<(u32, usize)> {
    todo!()
}

pub fn compose_all_loop(fns: &[fn(i32) -> i32]) -> impl Fn(i32) -> i32 {
    |x| x
}

pub fn compose_all(fns: &[fn(i32) -> i32]) -> impl Fn(i32) -> i32 {
    |x| x
}

fn make_counter(start: i64) -> impl FnMut() -> i64 {
    let mut start = start - 1;

    move || -> i64 {
        start += 1;
        start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nasty_test() {
        let f1 = |x| x * 100;
        let mut vec = Vec::new();
        let f2 = move |v1, v2| {
            vec.push(v1 + v2);
            let val = vec[0];
            std::mem::drop(vec);
            val
        };
        let val = super::wrap_call(f1, f2);
        assert_eq!(val, 300);
    }

    #[test]
    fn counter_basic() {
        let mut c = make_counter(10);
        assert_eq!(c(), 10);
        assert_eq!(c(), 11);
        assert_eq!(c(), 12);
        let mut c2 = make_counter(-3);
        assert_eq!(c2(), -3);
        assert_eq!(c2(), -2);
        assert_eq!(c(), 13); // niezależne liczniki
    }

    #[test]
    fn sum_squares_odd_cases() {
        let empty: &[u32] = &[];
        assert_eq!(sum_squares_odd_loop(empty), 0);
        assert_eq!(sum_squares_odd(empty), 0);
        let evens = [2, 4, 6];
        assert_eq!(sum_squares_odd_loop(&evens), 0);
        assert_eq!(sum_squares_odd(&evens), 0);
        let nums = [1, 2, 3, 4, 5];
        assert_eq!(sum_squares_odd_loop(&nums), 35);
        assert_eq!(sum_squares_odd(&nums), 35);
    }

    #[test]
    fn vertices_and_cycles() {
        let edges = [(1, 2), (2, 1), (3, 4), (4, 3), (5, 5), (2, 3)];
        let v_loop = vertices_loop(&edges);
        let v_iter = vertices(&edges);
        assert_eq!(v_loop, v_iter);
        assert_eq!(v_loop, vec![1, 2, 3, 4, 5]);
        let c_loop = cycles_2_loop(&edges);
        let c_iter = cycles_2(&edges);
        assert_eq!(c_loop, c_iter);
        assert_eq!(c_loop, vec![1, 2, 3, 4]);
    }

    #[test]
    fn cycles_2_duplicates() {
        let edges = [(1, 2), (2, 1), (1, 2), (2, 1), (2, 2)];
        assert_eq!(cycles_2_loop(&edges), vec![1, 2]);
        assert_eq!(cycles_2(&edges), vec![1, 2]);
    }

    #[test]
    fn empty_graph() {
        let edges: [(u32, u32); 0] = [];
        assert_eq!(vertices_loop(&edges), Vec::<u32>::new());
        assert_eq!(vertices(&edges), Vec::<u32>::new());
        assert_eq!(cycles_2_loop(&edges), Vec::<u32>::new());
        assert_eq!(cycles_2(&edges), Vec::<u32>::new());
    }

    #[test]
    fn primes_examples() {
        assert_eq!(primes_loop(0), Vec::<u32>::new());
        assert_eq!(primes(0), Vec::<u32>::new());
        assert_eq!(primes_loop(2), Vec::<u32>::new());
        assert_eq!(primes(2), Vec::<u32>::new());
        assert_eq!(primes_loop(3), vec![2]);
        assert_eq!(primes(3), vec![2]);
        assert_eq!(primes_loop(10), vec![2, 3, 5, 7]);
        assert_eq!(primes(10), vec![2, 3, 5, 7]);
        assert_eq!(primes_loop(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert_eq!(primes(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn primes_large_count() {
        let p100 = primes(100);
        assert_eq!(p100.len(), 25);
        assert_eq!(p100.last(), Some(&97));
        assert_eq!(p100, primes_loop(100));
    }

    #[test]
    fn wrap_call_fn_ptr() {
        fn times2(x: u32) -> u32 {
            x * 2
        }
        let val = wrap_call(times2, |a, b| a + b);
        assert_eq!(val, 6); // 2*1 + 2*2 = 2 + 4 = 6
    }

    #[test]
    fn rle_basic_and_edges() {
        assert_eq!(run_length_encode_loop(&[]), Vec::<(u32, usize)>::new());
        assert_eq!(run_length_encode(&[]), Vec::<(u32, usize)>::new());
        assert_eq!(run_length_encode_loop(&[7]), vec![(7, 1)]);
        assert_eq!(run_length_encode(&[7]), vec![(7, 1)]);
        let data = [1, 1, 2, 2, 2, 1];
        let expect = vec![(1, 2), (2, 3), (1, 1)];
        assert_eq!(run_length_encode_loop(&data), expect);
        assert_eq!(run_length_encode(&data), expect);
    }

    #[test]
    fn rle_varied_runs() {
        let data = [3, 3, 3, 3, 2, 2, 9, 9, 9, 1, 1, 1, 1, 1];
        let expect = vec![(3, 4), (2, 2), (9, 3), (1, 5)];
        assert_eq!(run_length_encode_loop(&data), expect);
        assert_eq!(run_length_encode(&data), expect);
    }

    #[test]
    fn compose_all_identity_and_order() {
        fn add1(x: i32) -> i32 {
            x + 1
        }
        fn times2(x: i32) -> i32 {
            x * 2
        }
        fn square(x: i32) -> i32 {
            x * x
        }

        let id_iter = compose_all(&[]);
        let id_loop = compose_all_loop(&[]);
        assert_eq!(id_iter(42), 42);
        assert_eq!(id_loop(42), 42);

        // Zastosowanie w kolejności: add1, times2, square
        let f_iter = compose_all(&[add1, times2, square]);
        let f_loop = compose_all_loop(&[add1, times2, square]);
        // (((3 + 1) * 2) ^2) = (4*2)^2 = 8^2 = 64
        assert_eq!(f_iter(3), 64);
        assert_eq!(f_loop(3), 64);

        // Odwrócenie kolejności daje inny wynik
        let g_iter = compose_all(&[square, times2, add1]);
        assert_eq!(g_iter(3), ((3 * 3) * 2) + 1);
    }

    #[test]
    fn compose_all_matches_loop() {
        fn f1(x: i32) -> i32 {
            x - 5
        }
        fn f2(x: i32) -> i32 {
            x * 3
        }
        fn f3(x: i32) -> i32 {
            x + 10
        }
        let funcs = [f1, f2, f3];
        let c1 = compose_all(&funcs);
        let c2 = compose_all_loop(&funcs);
        for x in [-10, -1, 0, 1, 7, 20] {
            assert_eq!(c1(x), c2(x));
        }
    }
}
