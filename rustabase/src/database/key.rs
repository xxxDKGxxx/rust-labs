use crate::database::table::record::Value;

pub trait DatabaseKey: Ord + Clone {
    fn is_equal_to(&self, other: &Self) -> bool;

    fn next(previous_key: Option<&Self>) -> Self;

    fn to_value(self) -> Value;
}

impl DatabaseKey for i64 {
    fn is_equal_to(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn next(previous_key: Option<&Self>) -> Self {
        match previous_key {
            Some(previous) => *previous + 1,
            None => 0,
        }
    }

    fn to_value(self) -> Value {
        Value::INT(self)
    }
}
