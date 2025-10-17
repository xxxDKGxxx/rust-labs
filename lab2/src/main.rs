#[derive(Debug, Clone, Default)]
struct NumberWithUnit {
    unit: String,
    value: f64,
}

// constructors
impl NumberWithUnit {
    fn unitless(value: f64) -> Self {
        Self {
            unit: String::new(),
            value,
        }
    }

    fn with_unit(value: f64, unit: String) -> Self {
        Self { value, unit }
    }

    fn with_unit_from(other: Self, value: f64) -> Self {
        Self {
            unit: other.unit,
            value,
        }
    }
}

// methods
impl NumberWithUnit {
    fn add(self, other: Self) -> Self {
        if self.unit != other.unit {
            panic!();
        }

        Self {
            unit: self.unit,
            value: self.value + other.value,
        }
    }

    fn mul(self, other: Self) -> Self {
        Self {
            unit: format!("{}*{}", self.unit, other.unit),
            value: self.value * other.value,
        }
    }

    fn div(self, other: Self) -> Self {
        Self {
            unit: format!("{}/{}", self.unit, other.unit),
            value: self.value / other.value,
        }
    }
}

// in place methods
impl NumberWithUnit {
    fn add_in_place(&mut self, other: &Self) {
        if self.unit != other.unit {
            panic!();
        }

        self.value += other.value
    }
    fn mul_in_place(&mut self, other: &Self) {
        self.unit = format!("{}*{}", self.unit, other.unit);
        self.value *= other.value;
    }
    fn div_in_place(&mut self, other: &Self) {
        self.unit = format!("{}/{}", self.unit, other.unit);
        self.value /= other.value;
    }
}

fn mul_vals(vals: &[NumberWithUnit]) -> NumberWithUnit {
    let mut result = NumberWithUnit::with_unit_from(vals[0].clone(), 1f64);

    for val in vals.iter() {
        result.mul_in_place(val);
    }

    result
}

fn mul_vals_vec(vals: Vec<NumberWithUnit>) -> NumberWithUnit {
    let mut result = NumberWithUnit::unitless(1.0f64);

    for val in vals.iter() {
        result.mul_in_place(val);
    }

    result
}

struct DoubleString(String, String);

// constructors
impl DoubleString {
    fn from_strs(str_1: &str, str_2: &str) -> Self {
        Self(String::from(str_1), String::from(str_2))
    }
    fn from_strings(str_1: &String, str_2: &String) -> Self {
        Self(String::from(str_1), String::from(str_2))
    }
}

// methods
impl DoubleString {
    fn show(&self) {
        println!("({}, {})", self.0, self.1);
    }
}

fn main() {
    let mut kg_unit = NumberWithUnit::with_unit(2f64, String::from("kg"));
    let unitless = NumberWithUnit::unitless(1f64);
    let another_kg_unit = NumberWithUnit::with_unit_from(kg_unit.clone(), 5f64);
    println!("{:?} {:?} {:?}", kg_unit, unitless, another_kg_unit);

    let result = kg_unit.clone().add(another_kg_unit.clone());
    kg_unit.add_in_place(&another_kg_unit);
    println!("Adding: {:?} {:?}", kg_unit, result);

    let mut distance = NumberWithUnit::with_unit(100f64, String::from("m"));
    let mut time = NumberWithUnit::with_unit(50f64, String::from("s"));

    // kg_unit.add_in_place(&time.clone());
    // should panic

    let result = distance.clone().div(time.clone());
    distance.div_in_place(&time);
    println!("Division: {:?} {:?}", distance, result);

    let result = time.clone().mul(time.clone());
    time.mul_in_place(&time.clone());
    println!("Multiplication: {:?} {:?}", time, result);

    let vals = [
        kg_unit.clone(),
        unitless.clone(),
        another_kg_unit.clone(),
        distance.clone(),
        result.clone(),
    ];
    let vals_vec = vec![
        kg_unit.clone(),
        unitless.clone(),
        another_kg_unit.clone(),
        distance.clone(),
        result.clone(),
    ];
    println!("Mul vals: {:?}", mul_vals(&vals));
    println!("Mul vals 2: {:?}", mul_vals(&vals));
    println!("Mul vals vec: {:?}", mul_vals_vec(vals_vec.clone()));
    println!("Mul vals vec 2: {:?}", mul_vals_vec(vals_vec.clone()));

    let string: String = String::from("Tekst1");
    let str_slice: &str = "Test";

    // let db1 = DoubleString::from_strings(&string, &str_slice);
    let db1 = DoubleString::from_strings(&string, &string);
    let db2 = DoubleString::from_strs(&string, str_slice);

    db1.show();
    db2.show();
}
