use libc::{c_char, free, malloc, strcpy};
use std::ptr;

// A custom string type that manages its own memory.
#[derive(Debug)]
pub struct CustomString {
    ptr: *mut c_char,
    len: usize,
}

impl CustomString {
    pub fn from_str(s: &str) -> Self {
        let len = s.len();
        // Allocate memory for the string plus a null terminator.
        let c_str_ptr = unsafe { malloc(len + 1) as *mut c_char };

        if c_str_ptr.is_null() {
            // In a real-world scenario, we would handle this error more gracefully.
            // For this project, we'll follow the no-panic rule.
            // Returning an empty string is a possible safe fallback.
            return Self {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }

        unsafe {
            // Copy the string content.
            ptr::copy_nonoverlapping(s.as_ptr() as *const c_char, c_str_ptr, len);
            // Null-terminate the string.
            *c_str_ptr.add(len) = 0;
        }

        Self {
            ptr: c_str_ptr,
            len,
        }
    }

    // Returns the length of the string.
    pub fn len(&self) -> usize {
        self.len
    }

    // Returns a raw pointer to the underlying C-style string.
    pub fn as_ptr(&self) -> *const c_char {
        self.ptr as *const c_char
    }
}

impl Drop for CustomString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                free(self.ptr as *mut _);
            }
        }
    }
}

impl Clone for CustomString {
    fn clone(&self) -> Self {
        if self.ptr.is_null() {
            return Self {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
        let new_ptr = unsafe { malloc(self.len + 1) as *mut c_char };
        if new_ptr.is_null() {
            return Self {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
        unsafe {
            strcpy(new_ptr, self.ptr);
        }
        Self {
            ptr: new_ptr,
            len: self.len,
        }
    }
}

impl PartialEq for CustomString {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        if self.ptr.is_null() && other.ptr.is_null() {
            return true;
        }
        if self.ptr.is_null() || other.ptr.is_null() {
            return false;
        }
        unsafe {
            std::slice::from_raw_parts(self.ptr as *const u8, self.len)
                == std::slice::from_raw_parts(other.ptr as *const u8, other.len)
        }
    }
}

impl Eq for CustomString {}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    Red,
    Black,
}

struct Node {
    key: u64,
    value: CustomString,
    color: Color,
    parent: *mut Node,
    left: *mut Node,
    right: *mut Node,
}

impl Node {
    fn new(key: u64, value: CustomString) -> *mut Self {
        unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let node_ptr = malloc(layout.size()) as *mut Self;
            if node_ptr.is_null() {
                return ptr::null_mut();
            }
            ptr::write(
                node_ptr,
                Self {
                    key,
                    value,
                    color: Color::Red, // easiest to fix
                    parent: ptr::null_mut(),
                    left: ptr::null_mut(),
                    right: ptr::null_mut(),
                },
            );
            node_ptr
        }
    }
}

pub struct NumberStringDictionary {
    root: *mut Node,
}

impl NumberStringDictionary {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
        }
    }

    fn find_node(&self, key: u64) -> *mut Node {
        let mut current = self.root;
        while !current.is_null() {
            let current_key = unsafe { (*current).key };
            if key < current_key {
                current = unsafe { (*current).left };
            } else if key > current_key {
                current = unsafe { (*current).right };
            } else {
                return current;
            }
        }
        ptr::null_mut()
    }

    pub fn get(&self, key: u64) -> Option<&CustomString> {
        let node = self.find_node(key);
        if node.is_null() {
            None
        } else {
            unsafe { Some(&(*node).value) }
        }
    }

    pub fn contains_key(&self, key: u64) -> bool {
        !self.find_node(key).is_null()
    }

    // 34
    pub fn insert(&mut self, key: u64, value: CustomString) {
        let new_node = Node::new(key, value);
        if new_node.is_null() {
            return;
        }
        let mut y = ptr::null_mut();
        let mut x = self.root;
        while !x.is_null() {
            y = x;
            unsafe {
                if (*new_node).key < (*x).key {
                    x = (*x).left;
                } else if (*new_node).key > (*x).key {
                    x = (*x).right;
                } else {
                    let new_value =
                        std::mem::replace(&mut (*new_node).value, CustomString::from_str(""));
                    (*x).value = new_value;
                    Self::drop_node(new_node);
                    return;
                }
            }
        }
        unsafe {
            (*new_node).parent = y;
            if y.is_null() {
                self.root = new_node;
            } else if (*new_node).key < (*y).key {
                (*y).left = new_node;
            } else {
                (*y).right = new_node;
            }
            self.insert_fixup(new_node);
        }
    }

    unsafe fn insert_fixup(&mut self, mut z: *mut Node) {
        while !(*z).parent.is_null() && (*(*z).parent).color == Color::Red {
            let parent = (*z).parent;
            let grandparent = (*parent).parent;

            if grandparent.is_null() {
                break;
            }

            if parent == (*grandparent).left {
                z = self.insert_fixup_left_case(z, parent, grandparent);
            } else {
                z = self.insert_fixup_right_case(z, parent, grandparent);
            }
        }

        if !self.root.is_null() {
            (*self.root).color = Color::Black;
        }
    }

    unsafe fn insert_fixup_left_case(
        &mut self,
        mut z: *mut Node,
        parent: *mut Node,
        grandparent: *mut Node,
    ) -> *mut Node {
        let y = (*grandparent).right;
        if !y.is_null() && (*y).color == Color::Red {
            // Case 1: Uncle is Red
            (*parent).color = Color::Black;
            (*y).color = Color::Black;
            (*grandparent).color = Color::Red;
            z = grandparent;
        } else {
            if z == (*parent).right {
                // Case 2: Triangle -> transform to line
                z = parent;
                self.left_rotate(z);
            }
            // Case 3: Line
            (*(*z).parent).color = Color::Black;
            (*(*(*z).parent).parent).color = Color::Red;
            self.right_rotate((*(*z).parent).parent);
        }
        z
    }

    unsafe fn insert_fixup_right_case(
        &mut self,
        mut z: *mut Node,
        parent: *mut Node,
        grandparent: *mut Node,
    ) -> *mut Node {
        let y = (*grandparent).left;
        if !y.is_null() && (*y).color == Color::Red {
            // Case 1
            (*parent).color = Color::Black;
            (*y).color = Color::Black;
            (*grandparent).color = Color::Red;
            z = grandparent;
        } else {
            if z == (*parent).left {
                // Case 2
                z = parent;
                self.right_rotate(z);
            }
            // Case 3
            (*(*z).parent).color = Color::Black;
            (*(*(*z).parent).parent).color = Color::Red;
            self.left_rotate((*(*z).parent).parent);
        }
        z
    }

    unsafe fn left_rotate(&mut self, x: *mut Node) {
        let y = (*x).right;
        (*x).right = (*y).left;

        if !(*y).left.is_null() {
            (*(*y).left).parent = x;
        }

        (*y).parent = (*x).parent;

        if (*x).parent.is_null() {
            self.root = y;
        } else if x == (*(*x).parent).left {
            (*(*x).parent).left = y;
        } else {
            (*(*x).parent).right = y;
        }

        (*y).left = x;
        (*x).parent = y;
    }

    unsafe fn right_rotate(&mut self, y: *mut Node) {
        let x = (*y).left;
        (*y).left = (*x).right;

        if !(*x).right.is_null() {
            (*(*x).right).parent = y;
        }

        (*x).parent = (*y).parent;

        if (*y).parent.is_null() {
            self.root = x;
        } else if y == (*(*y).parent).right {
            (*(*y).parent).right = x;
        } else {
            (*(*y).parent).left = x;
        }

        (*x).right = y;
        (*y).parent = x;
    }

    // 40
    pub fn remove(&mut self, key: u64) {
        let z = self.find_node(key);
        if z.is_null() {
            return; // Key not found
        }
        unsafe {
            let mut y = z;
            let mut _y_original_color = (*y).color;
            let x: *mut Node;
            let x_parent: *mut Node;
            if (*z).left.is_null() {
                x = (*z).right;
                x_parent = (*z).parent;
                self.transplant(z, (*z).right);
            } else if (*z).right.is_null() {
                x = (*z).left;
                x_parent = (*z).parent;
                self.transplant(z, (*z).left);
            } else {
                y = self.minimum((*z).right);
                _y_original_color = (*y).color;
                x = (*y).right;
                if (*y).parent == z {
                    x_parent = y;
                } else {
                    x_parent = (*y).parent;
                    self.transplant(y, (*y).right);
                    (*y).right = (*z).right;
                    (*(*y).right).parent = y;
                }
                self.transplant(z, y);
                (*y).left = (*z).left;
                (*(*y).left).parent = y;
                (*y).color = (*z).color;
            }
            if _y_original_color == Color::Black {
                self.remove_fixup(x, x_parent);
            }
            Self::free_node(z);
        }
    }

    unsafe fn remove_two_children(&mut self, z: *mut Node) -> Color {
        let y = self.minimum((*z).right);
        let _y_original_color = (*y).color;
        let x = (*y).right;
        if (*y).parent == z {
            x_parent = y;
        } else {
            x_parent = (*y).parent;
            self.transplant(y, (*y).right);
            (*y).right = (*z).right;
            (*(*y).right).parent = y;
        }
        self.transplant(z, y);
        (*y).left = (*z).left;
        (*(*y).left).parent = y;
        (*y).color = (*z).color;
        _y_original_color
    }

    unsafe fn minimum(&self, mut node: *mut Node) -> *mut Node {
        while !(*node).left.is_null() {
            node = (*node).left;
        }
        node
    }

    unsafe fn transplant(&mut self, u: *mut Node, v: *mut Node) {
        if (*u).parent.is_null() {
            self.root = v;
        } else if u == (*(*u).parent).left {
            (*(*u).parent).left = v;
        } else {
            (*(*u).parent).right = v;
        }
        if !v.is_null() {
            (*v).parent = (*u).parent;
        }
    }

    unsafe fn remove_fixup(&mut self, mut x: *mut Node, mut parent: *mut Node) {
        while x != self.root && (x.is_null() || (*x).color == Color::Black) {
            if parent.is_null() {
                break;
            }

            if x == (*parent).left {
                let (new_x, new_parent) = self.remove_fixup_left_case(x, parent);
                x = new_x;
                parent = new_parent;
            } else {
                let (new_x, new_parent) = self.remove_fixup_right_case(x, parent);
                x = new_x;
                parent = new_parent;
            }
        }
        if !x.is_null() {
            (*x).color = Color::Black;
        }
    }

    unsafe fn remove_fixup_left_case(
        &mut self,
        mut x: *mut Node,
        mut parent: *mut Node,
    ) -> (*mut Node, *mut Node) {
        let mut w = (*parent).right;
        if w.is_null() {
            return (x, parent);
        }

        if (*w).color == Color::Red {
            (*w).color = Color::Black;
            (*parent).color = Color::Red;
            self.left_rotate(parent);
            w = (*parent).right;
        }
        if w.is_null() {
            return (x, parent);
        }

        if ((*w).left.is_null() || (*(*w).left).color == Color::Black)
            && ((*w).right.is_null() || (*(*w).right).color == Color::Black)
        {
            (*w).color = Color::Red;
            x = parent;
            parent = (*x).parent;
        } else {
            if (*w).right.is_null() || (*(*w).right).color == Color::Black {
                if !(*w).left.is_null() {
                    (*(*w).left).color = Color::Black;
                }
                (*w).color = Color::Red;
                self.right_rotate(w);
                w = (*parent).right;
            }
            if w.is_null() {
                return (self.root, ptr::null_mut());
            }
            (*w).color = (*parent).color;
            (*parent).color = Color::Black;
            if !(*w).right.is_null() {
                (*(*w).right).color = Color::Black;
            }
            self.left_rotate(parent);
            x = self.root;
        }
        (x, parent)
    }

    unsafe fn remove_fixup_right_case(
        &mut self,
        mut x: *mut Node,
        mut parent: *mut Node,
    ) -> (*mut Node, *mut Node) {
        let mut w = (*parent).left;
        if w.is_null() {
            return (x, parent);
        }

        if (*w).color == Color::Red {
            (*w).color = Color::Black;
            (*parent).color = Color::Red;
            self.right_rotate(parent);
            w = (*parent).left;
        }
        if w.is_null() {
            return (x, parent);
        }

        if ((*w).right.is_null() || (*(*w).right).color == Color::Black)
            && ((*w).left.is_null() || (*(*w).left).color == Color::Black)
        {
            (*w).color = Color::Red;
            x = parent;
            parent = (*x).parent;
        } else {
            if (*w).left.is_null() || (*(*w).left).color == Color::Black {
                if !(*w).right.is_null() {
                    (*(*w).right).color = Color::Black;
                }
                (*w).color = Color::Red;
                self.left_rotate(w);
                w = (*parent).left;
            }
            if w.is_null() {
                return (self.root, ptr::null_mut());
            }
            (*w).color = (*parent).color;
            (*parent).color = Color::Black;
            if !(*w).left.is_null() {
                (*(*w).left).color = Color::Black;
            }
            self.right_rotate(parent);
            x = self.root;
        }
        (x, parent)
    }

    unsafe fn drop_node(node_ptr: *mut Node) {
        if node_ptr.is_null() {
            return;
        }
        Self::drop_node((*node_ptr).left);
        Self::drop_node((*node_ptr).right);

        Self::free_node(node_ptr);
    }

    unsafe fn free_node(node_ptr: *mut Node) {
        if node_ptr.is_null() {
            return;
        }

        ptr::drop_in_place(&mut (*node_ptr).value);

        free(node_ptr as *mut _);
    }
}

impl Default for NumberStringDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NumberStringDictionary {
    fn drop(&mut self) {
        if !self.root.is_null() {
            unsafe {
                Self::drop_node(self.root);
            }
        }
    }
}

#[macro_export]
macro_rules! dict {
    // Handle trailing comma
    ($($key:expr => $value:expr,)+) => {
        dict!($($key => $value),+)
    };
    // Main macro logic
    ($($key:expr => $value:expr),*) => {
        {
            let mut temp_dict = NumberStringDictionary::new();
            $(
                temp_dict.insert($key, CustomString::from_str($value));
            )*
            temp_dict
        }
    };
}

pub mod ffi {
    use super::{CustomString, NumberStringDictionary};
    use libc::{c_char, free, malloc};
    use std::ffi::CStr;
    use std::ptr;

    #[no_mangle]
    pub extern "C" fn dict_new() -> *mut NumberStringDictionary {
        unsafe {
            let dict_ptr = malloc(std::mem::size_of::<NumberStringDictionary>())
                as *mut NumberStringDictionary;
            if dict_ptr.is_null() {
                return ptr::null_mut();
            }
            ptr::write(dict_ptr, NumberStringDictionary::new());
            dict_ptr
        }
    }

    #[no_mangle]
    pub extern "C" fn dict_free(dict: *mut NumberStringDictionary) {
        if !dict.is_null() {
            unsafe {
                ptr::drop_in_place(dict);
                free(dict as *mut _);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn dict_insert(
        dict: *mut NumberStringDictionary,
        key: u64,
        value: *const c_char,
    ) {
        if dict.is_null() || value.is_null() {
            return;
        }
        let dict = unsafe { &mut *dict };
        let c_str = unsafe { CStr::from_ptr(value) };
        if let Ok(rust_str) = c_str.to_str() {
            dict.insert(key, CustomString::from_str(rust_str));
        }
    }

    #[no_mangle]
    pub extern "C" fn dict_get(dict: *const NumberStringDictionary, key: u64) -> *const c_char {
        if dict.is_null() {
            return ptr::null();
        }
        let dict = unsafe { &*dict };
        match dict.get(key) {
            Some(s) => s.as_ptr(),
            None => ptr::null(),
        }
    }

    #[no_mangle]
    pub extern "C" fn dict_contains_key(dict: *const NumberStringDictionary, key: u64) -> bool {
        if dict.is_null() {
            return false;
        }
        let dict = unsafe { &*dict };
        dict.contains_key(key)
    }

    #[no_mangle]
    pub extern "C" fn dict_remove(dict: *mut NumberStringDictionary, key: u64) {
        if dict.is_null() {
            return;
        }
        let dict = unsafe { &mut *dict };
        dict.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mystring() {
        let s1 = CustomString::from_str("hello");
        let s2 = s1.clone();
        let s3 = CustomString::from_str("world");

        assert_eq!(s1.len(), 5);
        assert_eq!(s2.len(), 5);
        assert_eq!(s3.len(), 5);

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_dict_insert_and_get() {
        let mut dict = NumberStringDictionary::new();
        dict.insert(10, CustomString::from_str("ten"));
        dict.insert(20, CustomString::from_str("twenty"));
        dict.insert(5, CustomString::from_str("five"));

        assert!(dict.contains_key(10));
        assert!(dict.contains_key(20));
        assert!(dict.contains_key(5));
        assert!(!dict.contains_key(15));

        assert_eq!(dict.get(10), Some(&CustomString::from_str("ten")));
        assert_eq!(dict.get(20), Some(&CustomString::from_str("twenty")));
        assert_eq!(dict.get(5), Some(&CustomString::from_str("five")));
        assert_eq!(dict.get(15), None);

        // Test updating a key
        dict.insert(10, CustomString::from_str("ten-updated"));
        assert_eq!(dict.get(10), Some(&CustomString::from_str("ten-updated")));
    }

    #[test]
    fn test_dict_remove() {
        let mut dict = NumberStringDictionary::new();
        let keys = [10, 20, 5, 15, 25, 3, 8, 1, 4, 7, 9];
        for &key in &keys {
            dict.insert(key, CustomString::from_str(&key.to_string()));
        }

        // Remove a leaf node
        dict.remove(1);
        assert!(!dict.contains_key(1));
        assert!(dict.contains_key(10));

        // Remove a node with one child
        dict.remove(8);
        assert!(!dict.contains_key(8));
        assert!(dict.contains_key(7));
        assert!(dict.contains_key(9));

        // Remove a node with two children
        dict.remove(5);
        assert!(!dict.contains_key(5));
        assert!(dict.contains_key(3));
        assert!(dict.contains_key(4));
        assert!(dict.contains_key(7));

        // Remove the root
        dict.remove(10);
        assert!(!dict.contains_key(10));
        assert!(dict.contains_key(15));
        assert!(dict.contains_key(20));

        // Remove non-existent key
        dict.remove(100);

        // Check remaining keys
        let remaining_keys = [20, 15, 25, 3, 4, 7, 9];
        for &key in &remaining_keys {
            assert!(dict.contains_key(key));
            assert_eq!(
                dict.get(key),
                Some(&CustomString::from_str(&key.to_string()))
            );
        }
    }

    #[test]
    fn test_dict_macro() {
        let dict = dict! {
            1 => "one",
            2 => "two",
            3 => "three",
        };

        assert!(dict.contains_key(1));
        assert!(dict.contains_key(2));
        assert!(dict.contains_key(3));
        assert!(!dict.contains_key(4));

        assert_eq!(dict.get(1), Some(&CustomString::from_str("one")));
        assert_eq!(dict.get(2), Some(&CustomString::from_str("two")));
        assert_eq!(dict.get(3), Some(&CustomString::from_str("three")));
    }
}
