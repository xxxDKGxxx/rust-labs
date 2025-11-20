use std::collections::{BTreeMap, HashMap};

use thiserror::Error;

use crate::database::{
    key::DatabaseKey,
    table::record::{Record, RecordBuilder, RecordError, Value},
};

pub mod record;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnType {
    BOOL,
    STRING,
    INT,
    FLOAT,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TableError<T: DatabaseKey> {
    #[error("Column: {column_name} was defined twice as {first_type:?} and {second_type:?}")]
    ColumnDefinedTwiceError {
        column_name: String,
        first_type: ColumnType,
        second_type: ColumnType,
    },

    #[error("Table does not contain column: {0}")]
    InvalidColumnNameError(String),

    #[error("Record by key: {0} not found")]
    KeyNotFoundError(T),

    #[error(
        "Invalid column type while inserting record: {column_name} expected {expected_type:?}, got {got_type:?}"
    )]
    InsertInvalidColumnTypeError {
        column_name: String,
        expected_type: ColumnType,
        got_type: ColumnType,
    },

    #[error("Missing columns on insert: {0:?}")]
    InsertMissingColumnsError(Vec<String>),

    #[error("Record error occured: {0}")]
    RecordError(#[from] RecordError),
}

#[derive(Debug)]
pub struct Table<K: DatabaseKey> {
    name: String,
    records: BTreeMap<K, Record>,
    columns: HashMap<String, ColumnType>,
    last_key: K,
    key_name: String,
}

pub struct TableBuilder<K: DatabaseKey> {
    table: Table<K>,
    errors: Vec<TableError<K>>,
}

impl ColumnType {
    fn is_type_of(&self, other: &Value) -> bool {
        match (self, other) {
            (ColumnType::BOOL, Value::BOOL(_)) => true,
            (ColumnType::STRING, Value::STRING(_)) => true,
            (ColumnType::INT, Value::INT(_)) => true,
            (ColumnType::FLOAT, Value::FLOAT(_)) => true,
            _ => false,
        }
    }

    fn from_value(value: &Value) -> Self {
        match value {
            Value::BOOL(_) => ColumnType::BOOL,
            Value::STRING(_) => ColumnType::STRING,
            Value::INT(_) => ColumnType::INT,
            Value::FLOAT(_) => ColumnType::FLOAT,
        }
    }
}

impl<K: DatabaseKey> Table<K> {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_columns(&self) -> &HashMap<String, ColumnType> {
        &self.columns
    }

    pub fn new(name: String, key_name: String) -> TableBuilder<K> {
        TableBuilder {
            table: Self {
                name,
                records: BTreeMap::new(),
                columns: HashMap::new(),
                last_key: K::next(None),
                key_name,
            },
            errors: Vec::new(),
        }
    }

    fn insert(
        &mut self,
        column_names: Vec<String>,
        column_values: Vec<Value>,
    ) -> Result<(), TableError<K>> {
        let missing_columns: Vec<String> = self
            .columns
            .keys()
            .cloned()
            .filter(|column| !column_names.contains(column))
            .collect();

        if missing_columns.len() > 0 {
            return Err(TableError::InsertMissingColumnsError(missing_columns));
        }

        let mut new_record = Record::new();

        for (name, value) in column_names.into_iter().zip(column_values.into_iter()) {
            let t = match self.columns.get(&name) {
                Some(t) => t,
                None => return Err(TableError::InvalidColumnNameError(name)),
            };

            if !t.is_type_of(&value) {
                return Err(TableError::InsertInvalidColumnTypeError {
                    column_name: name,
                    expected_type: *t,
                    got_type: ColumnType::from_value(&value),
                });
            }

            new_record = new_record.with_column(&name, value);
        }

        self.insert_with_key(new_record)?;

        Ok(())
    }

    fn insert_with_key(&mut self, mut new_record: RecordBuilder) -> Result<(), TableError<K>> {
        new_record =
            new_record.with_column(&self.key_name, K::next(Some(&self.last_key)).to_value());

        let new_record = new_record.build()?;

        self.records
            .insert(K::next(Some(&self.last_key)), new_record);

        self.last_key = K::next(Some(&self.last_key));

        Ok(())
    }

    fn delete(&mut self, key: K) -> Result<(), TableError<K>> {
        match self.records.remove(&key) {
            Some(_) => Ok(()),
            None => Err(TableError::KeyNotFoundError(key)),
        }
    }

    pub fn filter(&self, filter: impl Fn(&Record) -> bool) -> Vec<&Record> {
        self.records
            .values()
            .filter(|record| filter(*record))
            .collect()
    }
}

impl<K: DatabaseKey> TableBuilder<K> {
    pub fn with_column(mut self, column_name: String, column_type: ColumnType) -> Self {
        if let Some(val) = self.table.columns.get(&column_name) {
            self.errors.push(TableError::ColumnDefinedTwiceError {
                column_name,
                first_type: *val,
                second_type: column_type,
            });
            return self;
        }

        self.table.columns.insert(column_name, column_type);

        self
    }

    pub fn build(self) -> Result<Table<K>, TableError<K>> {
        match self.errors.into_iter().next() {
            Some(err) => return Err(err),
            _ => (),
        };

        Ok(self.table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare_test_table() -> Table<i64> {
        let table: Table<i64> = Table::new("Orders".to_string(), "OrderId".to_string())
            .with_column("ClientName".to_string(), ColumnType::STRING)
            .with_column("Capacity".to_string(), ColumnType::INT)
            .build()
            .unwrap();
        table
    }

    #[test]
    fn table_creation_test() {
        let table = prepare_test_table();

        assert_eq!(table.name, "Orders".to_string());
        assert_eq!(table.key_name, "OrderId".to_string());
        assert_eq!(table.records.values().len(), 0);
        assert_eq!(table.last_key, i64::next(None));
        assert_eq!(table.columns.len(), 2);
        assert_eq!(
            table.columns.contains_key("ClientName") && table.columns.contains_key("Capacity"),
            true
        );
        assert_eq!(
            *table.columns.get("ClientName").unwrap(),
            ColumnType::STRING
        );
        assert_eq!(*table.columns.get("Capacity").unwrap(), ColumnType::INT);
    }

    #[test]
    fn table_creation_fail_test() {
        let table = Table::<i64>::new("Orders".to_string(), "OrderId".to_string())
            .with_column("ClientName".to_string(), ColumnType::STRING)
            .with_column("ClientName".to_string(), ColumnType::FLOAT)
            .build();

        assert_eq!(table.is_err(), true);
        assert_eq!(
            table.unwrap_err(),
            TableError::ColumnDefinedTwiceError {
                column_name: "ClientName".to_string(),
                first_type: ColumnType::STRING,
                second_type: ColumnType::FLOAT
            }
        )
    }

    #[test]
    fn table_insert_fail_test() {
        let mut table = prepare_test_table();
        let missing_col_result = table.insert(
            vec!["ClientName".to_string()],
            vec![Value::STRING("Test".to_string())],
        );
        assert!(missing_col_result.is_err());
        assert_eq!(
            missing_col_result.unwrap_err(),
            TableError::InsertMissingColumnsError(vec!["Capacity".to_string()])
        );

        let invalid_type_result = table.insert(
            vec!["ClientName".to_string(), "Capacity".to_string()],
            vec![
                Value::STRING("Test".to_string()),
                Value::STRING("NotAnInt".to_string()),
            ],
        );
        assert!(invalid_type_result.is_err());
        assert_eq!(
            invalid_type_result.unwrap_err(),
            TableError::InsertInvalidColumnTypeError {
                column_name: "Capacity".to_string(),
                expected_type: ColumnType::INT,
                got_type: ColumnType::STRING
            }
        );

        let invalid_name_result = table.insert(
            vec![
                "ClientName".to_string(),
                "NonExistentColumn".to_string(),
                "Capacity".to_string(),
            ],
            vec![
                Value::STRING("A".to_string()),
                Value::INT(1),
                Value::INT(100),
            ],
        );
        assert!(invalid_name_result.is_err());
        assert_eq!(
            invalid_name_result.unwrap_err(),
            TableError::InvalidColumnNameError("NonExistentColumn".to_string())
        );
    }
    #[test]
    fn table_delete_test() {
        let mut table = prepare_test_table();
        let initial_key = table.last_key;

        let insert_result = table.insert(
            vec!["ClientName".to_string(), "Capacity".to_string()],
            vec![Value::STRING("ABC Corp".to_string()), Value::INT(100)],
        );

        assert!(insert_result.is_ok());

        let insert_result_2 = table.insert(
            vec!["Capacity".to_string(), "ClientName".to_string()],
            vec![Value::INT(200), Value::STRING("XYZ Inc".to_string())],
        );
        assert!(insert_result_2.is_ok());

        let first_key = i64::next(Some(&initial_key));
        let second_key = i64::next(Some(&first_key));
        assert_eq!(table.records.len(), 2);
        let delete_result = table.delete(first_key);
        assert!(delete_result.is_ok());
        assert_eq!(table.records.len(), 1);
        assert!(!table.records.contains_key(&first_key));

        let not_found_result = table.delete(first_key);
        assert!(not_found_result.is_err());
        assert_eq!(
            not_found_result.unwrap_err(),
            TableError::KeyNotFoundError(first_key)
        );

        let delete_result_2 = table.delete(second_key);
        assert!(delete_result_2.is_ok());
        assert_eq!(table.records.len(), 0);
        assert!(!table.records.contains_key(&second_key));
    }

    #[test]
    fn table_insert_test() {
        {
            let mut table = prepare_test_table();
            let initial_key = table.last_key;

            let insert_result = table.insert(
                vec!["ClientName".to_string(), "Capacity".to_string()],
                vec![Value::STRING("ABC Corp".to_string()), Value::INT(100)],
            );

            assert!(insert_result.is_ok());
            let first_key = i64::next(Some(&initial_key));
            assert_eq!(table.last_key, first_key);
            assert_eq!(table.records.len(), 1);
            assert!(table.records.contains_key(&first_key));

            let insert_result_2 = table.insert(
                vec!["Capacity".to_string(), "ClientName".to_string()],
                vec![Value::INT(200), Value::STRING("XYZ Inc".to_string())],
            );
            assert!(insert_result_2.is_ok());
            let second_key = i64::next(Some(&first_key));
            assert_eq!(table.last_key, second_key);
            assert_eq!(table.records.len(), 2);
            assert!(table.records.contains_key(&second_key));

            let record = table.records.get(&first_key).unwrap();
            assert_eq!(
                record.get_value("ClientName").unwrap(),
                &Value::STRING("ABC Corp".to_string())
            );
            assert_eq!(record.get_value("Capacity").unwrap(), &Value::INT(100));
        }
    }

    #[test]
    fn table_filter_test() {
        let mut table = prepare_test_table();

        table
            .insert(
                vec!["ClientName".to_string(), "Capacity".to_string()],
                vec![Value::STRING("ABC Corp".to_string()), Value::INT(100)],
            )
            .unwrap();

        table
            .insert(
                vec!["ClientName".to_string(), "Capacity".to_string()],
                vec![Value::STRING("XYZ Inc".to_string()), Value::INT(50)],
            )
            .unwrap();

        table
            .insert(
                vec!["ClientName".to_string(), "Capacity".to_string()],
                vec![Value::STRING("Old Clients".to_string()), Value::INT(100)],
            )
            .unwrap();

        let results_name = table.filter(|record| {
            record
                .get_value("ClientName")
                .map(|v| v == &Value::STRING("XYZ Inc".to_string()))
                .unwrap_or(false)
        });

        assert_eq!(results_name.len(), 1);
        assert_eq!(
            results_name[0].get_value("ClientName").unwrap(),
            &Value::STRING("XYZ Inc".to_string())
        );

        let results_capacity = table.filter(|record| match record.get_value("Capacity") {
            Ok(Value::INT(c)) => *c > 50,
            _ => false,
        });

        assert_eq!(results_capacity.len(), 2);

        assert!(
            results_capacity
                .iter()
                .all(|r| r.get_value("Capacity").unwrap() == &Value::INT(100))
        );

        let results_none = table.filter(|record| {
            record
                .get_value("ClientName")
                .map(|v| v == &Value::STRING("Nonexistent".to_string()))
                .unwrap_or(false)
        });

        assert_eq!(results_none.len(), 0);

        let results_all = table.filter(|_| true);
        assert_eq!(results_all.len(), 3);
    }
}
