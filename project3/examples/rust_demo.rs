use red_black_tree_dict::{dict, CustomString, NumberStringDictionary};

fn main() {
    println!("--- Rust Dictionary Demo ---");

    // create a new dictionary
    println!("\n1. Creating a new, empty dictionary.");
    let mut dict = NumberStringDictionary::new();

    // insert elements
    println!("\n2. Inserting key-value pairs:");
    println!("   - Inserting (10, 'ten')");
    dict.insert(10, CustomString::from_str("ten"));
    println!("   - Inserting (20, 'twenty')");
    dict.insert(20, CustomString::from_str("twenty"));
    println!("   - Inserting (5, 'five')");
    dict.insert(5, CustomString::from_str("five"));

    // check for keys
    println!("\n3. Checking for keys:");
    println!("   - Contains key 10? {}", dict.contains_key(10));
    println!("   - Contains key 15? {}", dict.contains_key(15));

    // get values
    println!("\n4. Getting values:");
    if let Some(val) = dict.get(10) {
        println!("   - Value for key 10: {}", val.as_str());
    }
    if dict.get(15).is_none() {
        println!("   - Value for key 15: Not found (as expected)");
    }

    // remove an element
    println!("\n5. Removing an element:");
    println!("   - Removing key 20...");
    dict.remove(20);
    println!(
        "   - Contains key 20 after removal? {}",
        dict.contains_key(20)
    );
    println!(
        "   - Contains key 10 after removal? {}",
        dict.contains_key(10)
    );

    // use the dict! macro
    println!("\n6. Creating a new dictionary with the dict! macro:");
    let macro_dict = dict! {
        100 => "one hundred",
        200 => "two hundred",
    };
    println!(
        "   - Macro-dict contains key 100? {}",
        macro_dict.contains_key(100)
    );
    println!(
        "   - Macro-dict contains key 300? {}",
        macro_dict.contains_key(300)
    );
    if let Some(val) = macro_dict.get(200) {
        println!("   - Value for key 200 from macro-dict: {}", val.as_str());
    }

    println!("\n--- Demo Complete ---");
}
