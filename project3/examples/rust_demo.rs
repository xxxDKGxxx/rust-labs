use red_black_tree_dict::{dict, Dictionary, MyString};

fn main() {
    println!("--- Rust Dictionary Demo ---");

    // 1. Create a new dictionary
    println!("\n1. Creating a new, empty dictionary.");
    let mut dict = Dictionary::new();

    // 2. Insert elements
    println!("\n2. Inserting key-value pairs:");
    println!("   - Inserting (10, 'ten')");
    dict.insert(10, MyString::from_str("ten"));
    println!("   - Inserting (20, 'twenty')");
    dict.insert(20, MyString::from_str("twenty"));
    println!("   - Inserting (5, 'five')");
    dict.insert(5, MyString::from_str("five"));

    // 3. Check for keys
    println!("\n3. Checking for keys:");
    println!("   - Contains key 10? {}", dict.contains_key(10));
    println!("   - Contains key 15? {}", dict.contains_key(15));

    // 4. Get values
    println!("\n4. Getting values:");
    if let Some(val) = dict.get(10) {
        // This is unsafe because we are converting a raw pointer to a slice.
        // We know it's safe because MyString stores the length.
        let val_str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(val.as_ptr() as *const u8, val.len()))
        };
        println!("   - Value for key 10: {}", val_str);
    }
    if dict.get(15).is_none() {
        println!("   - Value for key 15: Not found (as expected)");
    }

    // 5. Remove an element
    println!("\n5. Removing an element:");
    println!("   - Removing key 20...");
    dict.remove(20);
    println!("   - Contains key 20 after removal? {}", dict.contains_key(20));
    println!("   - Contains key 10 after removal? {}", dict.contains_key(10));

    // 6. Use the dict! macro
    println!("\n6. Creating a new dictionary with the dict! macro:");
    let macro_dict = dict! {
        100 => "one hundred",
        200 => "two hundred",
    };
    println!("   - Macro-dict contains key 100? {}", macro_dict.contains_key(100));
    println!("   - Macro-dict contains key 300? {}", macro_dict.contains_key(300));
    if let Some(val) = macro_dict.get(200) {
        let val_str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(val.as_ptr() as *const u8, val.len()))
        };
        println!("   - Value for key 200 from macro-dict: {}", val_str);
    }
    
    println!("\n--- Demo Complete ---");
}
