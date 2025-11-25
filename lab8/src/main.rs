use std::{
    borrow::Cow,
    cell::{Cell, LazyCell, OnceCell, RefCell},
    collections::VecDeque,
    fs::read_to_string,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    str::FromStr,
};

enum HeapOrStack<T> {
    Stack(T),
    Heap(Box<T>),
}

impl<T> Deref for HeapOrStack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            HeapOrStack::Stack(el) => el,
            HeapOrStack::Heap(el) => el,
        }
    }
}

impl<T> DerefMut for HeapOrStack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            HeapOrStack::Stack(el) => el,
            HeapOrStack::Heap(el) => el,
        }
    }
}

struct AustroHungarianGreeter {
    idx: Cell<usize>,
}

impl AustroHungarianGreeter {
    fn new() -> Self {
        Self { idx: Cell::new(0) }
    }

    fn greet(&self) -> &'static str {
        const MESSAGES: [&str; 3] = [
            "Es lebe der Kaiser!",
            "Möge uns der Kaiser schützen!",
            "Éljen Ferenc József császár!",
        ];

        let curr_idx = self.idx.get();
        let return_msg = MESSAGES[curr_idx % MESSAGES.len()];

        self.idx.set(curr_idx + 1);

        return_msg
    }
}

impl Drop for AustroHungarianGreeter {
    fn drop(&mut self) {
        println!("Ich habe {} mal gegrüßt", self.idx.get());
    }
}

fn canon_head<'a>(xs: &'a VecDeque<i32>) -> Option<Cow<'a, VecDeque<i32>>> {
    if xs.is_empty() {
        return Some(Cow::Borrowed(xs));
    }

    match xs.iter().position(|&x| x % 2 != 0) {
        Some(0) => Some(Cow::Borrowed(xs)),
        Some(index) => {
            let mut owned_deque = xs.clone();
            owned_deque.rotate_left(index);
            Some(Cow::Owned(owned_deque))
        }
        None => None,
    }
}

struct CachedFile {
    cache: OnceCell<String>,
}

impl CachedFile {
    fn new() -> Self {
        Self {
            cache: OnceCell::new(),
        }
    }

    fn get(&self, path: &Path) -> &str {
        let current_file = self.cache.get_or_init(|| match read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(e) => panic!("{}", e),
        });

        current_file
    }

    fn try_get(&self) -> Option<&str> {
        self.cache.get().map(|s| s.as_str())
    }
}

#[derive(Clone)]
struct SharedFile {
    file: Rc<LazyCell<String, Box<dyn FnOnce() -> String>>>,
}

impl SharedFile {
    fn new(path: PathBuf) -> Self {
        Self {
            file: Rc::new(LazyCell::<_, Box<dyn FnOnce() -> String>>::new(Box::new(
                || match read_to_string(path) {
                    Ok(file_contents) => file_contents,
                    Err(e) => panic!("{}", e),
                },
            ))),
        }
    }

    fn get(&self) -> &str {
        &self.file
    }
}

struct Vertex {
    out_edges_owned: Vec<Rc<RefCell<Vertex>>>,
    out_edges: Vec<Weak<RefCell<Vertex>>>,
    data: i32,
}

impl Vertex {
    fn new() -> Self {
        Self {
            out_edges_owned: Vec::new(),
            out_edges: Vec::new(),
            data: i32::default(),
        }
    }

    fn create_neighbor(&mut self) -> Rc<RefCell<Vertex>> {
        let new_neighbor = Rc::new(RefCell::new(Vertex::new()));

        self.out_edges_owned.push(new_neighbor.clone());

        new_neighbor
    }

    fn link_to(&mut self, other: &Rc<RefCell<Vertex>>) {
        self.out_edges.push(Rc::downgrade(other));
    }

    fn all_neighbours(&self) -> Vec<Weak<RefCell<Vertex>>> {
        self.out_edges_owned
            .iter()
            .map(Rc::downgrade)
            .chain(self.out_edges.clone())
            .collect()
    }
}

fn cycle(n: usize) -> Rc<RefCell<Vertex>> {
    let first_vertex = Rc::new(RefCell::new(Vertex::new()));

    let mut current_neighbor = Rc::downgrade(&first_vertex);

    for i in 1..n {
        match current_neighbor.upgrade() {
            Some(val) => {
                let new_neighbor = val.borrow_mut().create_neighbor();

                new_neighbor.borrow_mut().data = i as i32;

                current_neighbor = Rc::downgrade(&new_neighbor);
            }
            None => panic!(),
        }
    }

    let last_vertex_upgraded = match current_neighbor.upgrade() {
        Some(val) => val,
        None => panic!(),
    };

    last_vertex_upgraded.borrow_mut().link_to(&first_vertex);

    first_vertex
}

fn main() {
    let greeter = AustroHungarianGreeter::new();

    for _ in 0..=10 {
        println!("{}", greeter.greet())
    }
    println!("Hello, world!");

    let stack = HeapOrStack::Heap(Box::new(2));
    let heap = HeapOrStack::Stack(2);

    println!("{}", *stack);
    println!("{}", *heap);

    let mut que = VecDeque::<i32>::new();

    que.push_back(2);
    que.push_back(4);
    que.push_back(6);
    que.push_back(3);
    que.push_back(2);

    let modified_que = canon_head(&que);

    assert!(modified_que.is_some());
    println!("{:?}", modified_que.unwrap());

    let cached_file = CachedFile::new();
    let path = Path::new("./src/test.txt");

    assert!(cached_file.try_get().is_none());

    println!("{}", cached_file.get(path));

    assert!(cached_file.try_get().is_some());

    let path_buf = PathBuf::from_str("./src/test.txt").unwrap();
    let shared_file = SharedFile::new(path_buf);
    let shared_file2 = shared_file.clone();
    let shared_file3 = shared_file.clone();

    println!("{}", shared_file2.get());
    println!("{}", shared_file.get());
    println!("{}", shared_file3.get());

    let mut current_node = cycle(5);

    for _ in 1..10 {
        println!("{}", current_node.borrow().data);

        current_node = current_node
            .clone()
            .borrow()
            .all_neighbours()
            .first()
            .unwrap()
            .upgrade()
            .unwrap()
    }
}
