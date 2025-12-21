#[macro_export]
macro_rules! string {
    ($arg: expr) => {
        $arg.to_string()
    };
    ($arg: ident) => {
        String::from($arg)
    };
}

pub trait StateMachine<S> {
    fn step(&self, state: S) -> Option<S>;
}

#[macro_export]
macro_rules! impl_state_machine {
    ($machine_name: ident, [ $($numFrom: tt -> $numTo: tt)* ]) => {
        struct $machine_name {
            map: HashMap<i32, i32>,
        }

        impl $machine_name {
            pub fn new() -> Self {
                let mut map = HashMap::new();

                const END: i32 = -1;

                $(
                    let target = { impl_state_machine!(@val $numTo) };

                    if (target != END) {
                        map.insert($numFrom, target);
                    }
                )*

                Self {
                    map
                }
            }
        }

        impl Default for $machine_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl StateMachine<i32> for $machine_name {
            fn step(&self, state: i32) -> Option<i32> {
                self.map.get(&state).cloned()
            }
        }
    };
    (@val END) => { -1 };
    (@val $expr: expr) => { $expr };
}

impl<S, H: ::std::hash::BuildHasher> StateMachine<S> for std::collections::HashMap<S, S, H>
where
    S: Clone + Eq + std::hash::Hash,
{
    fn step(&self, state: S) -> Option<S> {
        self.get(&state).cloned()
    }
}

pub fn join_machines<'a, S, M1, M2>(x: M1, y: M2) -> Vec<Box<dyn StateMachine<S> + 'a>>
where
    M1: StateMachine<S> + 'a,
    M2: StateMachine<S> + 'a,
{
    vec![Box::new(x), Box::new(y)]
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // --- 1. Testy makra string! ---

    #[test]
    fn test_string_literals_and_variables() {
        // Test dla literału string
        assert_eq!(string!("hello"), String::from("hello"));

        // Test dla zmiennej (&str)
        let var = "world";
        assert_eq!(string!(var), String::from("world"));

        // Test dla String (własność)
        let string_owned = String::from("owned");
        assert_eq!(string!(string_owned), String::from("owned"));
    }

    #[test]
    fn test_string_numbers() {
        // Test dla liczb (wymaga, aby makro użyło .to_string())
        // UWAGA: Zadziała tylko, jeśli w makrze reguła ($arg: expr) jest PIERWSZA.
        assert_eq!(string!(123), "123");
        assert_eq!(string!(42.5), "42.5");
    }

    // --- 2. Testy generycznej implementacji dla HashMap ---

    #[test]
    fn test_hashmap_implementation() {
        let mut map = HashMap::new();
        map.insert(10, 20);
        map.insert(20, 30);

        // Sprawdzamy czy HashMap<i32, i32> działa jako StateMachine
        assert_eq!(map.step(10), Some(20));
        assert_eq!(map.step(20), Some(30));
        // Brak klucza = koniec automatu
        assert_eq!(map.step(30), None);
    }

    // --- 3. Testy makra impl_state_machine! ---

    #[test]
    fn test_generated_machine_logic() {
        // Definiujemy maszynę.
        // Zgodnie z Twoim wzorcem ($($numFrom: tt -> $numTo: tt)*) nie używamy średników.
        impl_state_machine!(Workflow, [
            1 -> 2
            2 -> 3
            3 -> END
        ]);

        // Twoje makro implementuje Default, który woła new()
        let machine = Workflow::default();

        // Krok 1 -> 2
        assert_eq!(machine.step(1), Some(2));

        // Krok 2 -> 3
        assert_eq!(machine.step(2), Some(3));

        // Krok 3 -> END
        // Logika Twojego kodu:
        // if target != END { map.insert(...) }
        // Skoro target == END (-1), to nic nie wstawiono dla klucza 3.
        // Dlatego map.get(3) zwraca None.
        assert_eq!(machine.step(3), None);

        // Stan spoza definicji też zwraca None
        assert_eq!(machine.step(99), None);
    }

    #[test]
    fn test_empty_machine() {
        // Test dla pustej definicji
        impl_state_machine!(EmptyMachine, []);
        let machine = EmptyMachine::default();
        assert_eq!(machine.step(1), None);
    }

    // --- 4. Testy funkcji join_machines ---

    #[test]
    fn test_join_machines_heterogeneous() {
        // Testujemy łączenie dwóch RÓŻNYCH typów, które implementują ten sam trait.

        // Typ 1: HashMapa
        let mut map_machine = HashMap::new();
        map_machine.insert("Start", "Middle");

        // Typ 2: Wygenerowana struktura (musimy dostosować typy do &str, ale Twoje makro jest hardcoded na i32)
        // Więc przetestujmy łączenie dwóch HashMap dla uproszczenia typów S
        let mut map_machine2 = HashMap::new();
        map_machine2.insert("Middle", "End");

        let combined = join_machines(map_machine, map_machine2);

        assert_eq!(combined.len(), 2);

        // Sprawdzamy pierwszą maszynę
        assert_eq!(combined[0].step("Start"), Some("Middle"));
        // Sprawdzamy drugą maszynę
        assert_eq!(combined[1].step("Middle"), Some("End"));
    }
}
