use std::{collections::HashMap, fmt::Display};

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    BOOL(bool),
    STRING(String),
    INT(i64),
    FLOAT(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::BOOL(b) => write!(f, "BOOL {b}"),
            Value::STRING(s) => write!(f, "STRING {s}"),
            Value::INT(i) => write!(f, "INT {i}"),
            Value::FLOAT(fl) => write!(f, "FLOAT {fl}"),
        }?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum RecordError {
    #[error("Record does not contain column: `{0}`")]
    InvalidColumnNameError(String),
    #[error("Column: {column_name} was defined twice as {first_value:?} and {second_value:?}")]
    ColumnDefinedTwiceError {
        column_name: String,
        first_value: Value,
        second_value: Value,
    },
}

#[derive(Debug)]
pub struct Record {
    values_map: HashMap<String, Value>,
}

pub struct RecordBuilder {
    record: Record,
    errors: Vec<RecordError>,
}

impl Value {
    pub fn is_the_same_type_as(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Value::BOOL(_), Value::BOOL(_))
                | (Value::STRING(_), Value::STRING(_))
                | (Value::INT(_), Value::INT(_))
                | (Value::FLOAT(_), Value::FLOAT(_))
        )
    }

    pub fn type_name(&self) -> String {
        match self {
            Value::BOOL(_) => String::from("BOOL"),
            Value::STRING(_) => String::from("STRING"),
            Value::INT(_) => String::from("INT"),
            Value::FLOAT(_) => String::from("FLOAT"),
        }
    }
}

impl Record {
    pub fn new_builder() -> RecordBuilder {
        RecordBuilder {
            record: Record {
                values_map: HashMap::new(),
            },
            errors: Vec::new(),
        }
    }

    pub fn get_value(&self, column_name: &str) -> Result<&Value, RecordError> {
        match self.values_map.get(column_name) {
            Some(value) => Ok(value),
            None => Err(RecordError::InvalidColumnNameError(column_name.to_string())),
        }
    }

    pub fn get_values(&self, column_names: &Vec<&str>) -> Result<Vec<Value>, RecordError> {
        let results: Vec<Result<&Value, RecordError>> = column_names
            .iter()
            .map(|name| self.get_value(name))
            .collect();

        let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        let errors: Vec<RecordError> = errors.into_iter().filter_map(Result::err).collect();

        if let Some(err) = errors.into_iter().next() {
            return Err(err);
        }

        let successes: Vec<Value> = successes
            .into_iter()
            .filter_map(Result::ok)
            .cloned()
            .collect();

        Ok(successes)
    }
}

impl RecordBuilder {
    pub fn with_column(mut self, column_name: String, column_value: Value) -> RecordBuilder {
        if let Some(val) = self.record.values_map.get(&column_name) {
            self.errors.push(RecordError::ColumnDefinedTwiceError {
                column_name: column_name.to_string(),
                first_value: val.clone(),
                second_value: column_value,
            });
            return self;
        }

        self.record.values_map.insert(column_name, column_value);
        self
    }

    pub fn build(self) -> Result<Record, RecordError> {
        if let Some(err) = self.errors.into_iter().next() {
            return Err(err);
        }

        Ok(self.record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_type_name_tests() {
        let v1 = Value::BOOL(false);
        let v2 = Value::INT(14);
        let v3 = Value::FLOAT(2.5f64);
        let v4 = Value::STRING("Test str".to_string());

        assert_eq!(v1.type_name(), String::from("BOOL"));
        assert_eq!(v2.type_name(), String::from("INT"));
        assert_eq!(v3.type_name(), String::from("FLOAT"));
        assert_eq!(v4.type_name(), String::from("STRING"));
    }

    #[test]
    fn value_type_true_comparison_test() {
        let v1 = Value::BOOL(false);
        let v2 = Value::INT(14);
        let v3 = Value::FLOAT(2.5f64);
        let v4 = Value::STRING("Test str".to_string());

        let v5 = Value::BOOL(true);
        let v6 = Value::INT(15);
        let v7 = Value::FLOAT(3.5f64);
        let v8 = Value::STRING("Test str 2".to_string());

        assert!(v1.is_the_same_type_as(&v5));
        assert!(v2.is_the_same_type_as(&v6));
        assert!(v3.is_the_same_type_as(&v7));
        assert!(v4.is_the_same_type_as(&v8));
    }

    #[test]
    fn value_type_false_comparison_test() {
        let v1 = Value::BOOL(false);
        let v2 = Value::INT(14);
        let v3 = Value::FLOAT(2.5f64);
        let v4 = Value::STRING("Test str".to_string());

        let v5 = Value::BOOL(true);
        let v6 = Value::INT(15);
        let v7 = Value::FLOAT(3.5f64);
        let v8 = Value::STRING("Test str 2".to_string());

        assert!(!v1.is_the_same_type_as(&v6));
        assert!(!v1.is_the_same_type_as(&v7));
        assert!(!v1.is_the_same_type_as(&v8));

        assert!(!v2.is_the_same_type_as(&v5));
        assert!(!v2.is_the_same_type_as(&v7));
        assert!(!v2.is_the_same_type_as(&v8));

        assert!(!v3.is_the_same_type_as(&v5));
        assert!(!v3.is_the_same_type_as(&v6));
        assert!(!v3.is_the_same_type_as(&v8));

        assert!(!v4.is_the_same_type_as(&v5));
        assert!(!v4.is_the_same_type_as(&v6));
        assert!(!v4.is_the_same_type_as(&v7));
    }

    #[test]
    fn record_building_test() {
        let record = Record::new_builder()
            .with_column("Name".into(), Value::STRING(String::from("John")))
            .with_column("Age".into(), Value::INT(24))
            .with_column("Married".into(), Value::BOOL(false))
            .with_column("Result".into(), Value::FLOAT(0.75f64))
            .build()
            .unwrap();

        assert_eq!(
            *record.get_value("Name").unwrap(),
            Value::STRING(String::from("John"))
        );

        assert_eq!(*record.get_value("Age").unwrap(), Value::INT(24));
        assert_eq!(*record.get_value("Married").unwrap(), Value::BOOL(false));
        assert_eq!(*record.get_value("Result").unwrap(), Value::FLOAT(0.75f64));

        assert_eq!(
            record.get_value("Surname").err(),
            Some(RecordError::InvalidColumnNameError(String::from("Surname")))
        );
    }

    #[test]
    fn record_building_failure_test() {
        let record = Record::new_builder()
            .with_column("Name".into(), Value::STRING(String::from("John")))
            .with_column("Name".into(), Value::INT(24))
            .with_column("Married".into(), Value::BOOL(false))
            .with_column("Result".into(), Value::FLOAT(0.75f64))
            .build();

        assert!(record.is_err());

        let err = record.unwrap_err();

        assert_eq!(
            err,
            RecordError::ColumnDefinedTwiceError {
                column_name: "Name".to_string(),
                first_value: Value::STRING(String::from("John")),
                second_value: Value::INT(24)
            }
        );
    }

    #[test]
    fn record_get_value_success_test() {
        let record = Record::new_builder()
            .with_column("A".into(), Value::INT(10))
            .with_column("B".into(), Value::STRING("Hello".to_string()))
            .build()
            .unwrap();

        assert_eq!(*record.get_value("A").unwrap(), Value::INT(10));
        assert_eq!(
            *record.get_value("B").unwrap(),
            Value::STRING("Hello".to_string())
        );
    }

    #[test]
    fn record_get_value_failure_test() {
        let record = Record::new_builder().build().unwrap();

        assert_eq!(
            record.get_value("Missing").unwrap_err(),
            RecordError::InvalidColumnNameError("Missing".to_string())
        );
    }

    #[test]
    fn record_get_values_success_test() {
        let record = Record::new_builder()
            .with_column("ID".into(), Value::INT(1))
            .with_column("Status".into(), Value::STRING("Active".to_string()))
            .with_column("Price".into(), Value::FLOAT(99.99f64))
            .build()
            .unwrap();

        let column_names = vec!["Status", "Price"];
        let expected_values = vec![Value::STRING("Active".to_string()), Value::FLOAT(99.99f64)];

        let result = record.get_values(&column_names);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_values);
    }

    #[test]
    fn record_get_values_failure_test() {
        let record = Record::new_builder()
            .with_column("ID".into(), Value::INT(1))
            .with_column("Status".into(), Value::STRING("Active".to_string()))
            .build()
            .unwrap();

        let column_names = vec!["ID", "Missing1", "Status", "Missing2"];

        let result = record.get_values(&column_names);

        assert!(result.is_err());
        let err = result.unwrap_err();

        assert!(matches!(err, RecordError::InvalidColumnNameError(_)));

        let error_name = if let RecordError::InvalidColumnNameError(name) = err {
            name
        } else {
            panic!("Expected InvalidColumnNameError");
        };

        assert!(error_name == "Missing1" || error_name == "Missing2");
    }
}
