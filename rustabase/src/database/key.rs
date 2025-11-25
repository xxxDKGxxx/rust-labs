use crate::database::table::{ColumnType, record::Value};

pub trait DatabaseKey: Ord + Clone {
    fn is_equal_to(&self, other: &Self) -> bool;

    fn to_value(self) -> Value;

    fn to_column_type() -> ColumnType;

    fn from_value(value: Value) -> Option<Self>;
}

impl DatabaseKey for i64 {
    fn is_equal_to(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn to_value(self) -> Value {
        Value::INT(self)
    }

    fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::INT(i) => Some(i),
            _ => None,
        }
    }

    fn to_column_type() -> ColumnType {
        ColumnType::INT
    }
}

impl DatabaseKey for String {
    fn is_equal_to(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn to_value(self) -> Value {
        Value::STRING(self)
    }

    fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::STRING(s) => Some(s),
            _ => None,
        }
    }

    fn to_column_type() -> ColumnType {
        ColumnType::STRING
    }
}
