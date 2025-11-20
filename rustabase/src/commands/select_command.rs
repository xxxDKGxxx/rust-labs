use crate::{
    commands::command::{Command, CommandError},
    database::{
        key::DatabaseKey,
        table::{
            Table,
            record::{Record, RecordError, Value},
        },
    },
};

pub trait WhereFilter<K: DatabaseKey> {
    fn filter_record(&self, record: &Record) -> bool;

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError<K>>;
}

pub struct SelectCommand<'a, K: DatabaseKey, F: WhereFilter<K>> {
    table: &'a Table<K>,
    selected_columns: Vec<String>,
    where_filter: F,
    select_result: Option<Vec<Vec<Value>>>,
}

impl<'a, K: DatabaseKey, F: WhereFilter<K>> Command<K> for SelectCommand<'a, K, F> {
    fn execute(&mut self) -> Result<(), CommandError<K>> {
        let where_errors: Vec<CommandError<K>> = self
            .table
            .filter(|_r| true)
            .iter()
            .map(|r| self.where_filter.validate_filtering(r))
            .filter_map(Result::err)
            .collect();

        if let Some(err) = where_errors.into_iter().next() {
            return Err(err);
        }

        let results: Vec<Result<Vec<Value>, RecordError>> = self
            .table
            .filter(|record| self.where_filter.filter_record(record))
            .into_iter()
            .map(|record| {
                record.get_values(self.selected_columns.iter().map(|c| c.as_str()).collect())
            })
            .collect();

        let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        let errors = errors.into_iter().filter_map(Result::err);

        if let Some(err) = errors.into_iter().next() {
            return Err(CommandError::RecordError(err));
        };

        let successes = successes.into_iter().filter_map(Result::ok).collect();

        self.select_result = Some(successes);

        Ok(())
    }
}

impl<'a, K: DatabaseKey, F: WhereFilter<K>> SelectCommand<'a, K, F> {
    pub fn new(table: &'a Table<K>, selected_columns: Vec<String>, where_filter: F) -> Self {
        Self {
            table,
            selected_columns,
            where_filter,
            select_result: None,
        }
    }
}

pub struct NoOpWhereFilter {}

impl<K: DatabaseKey> WhereFilter<K> for NoOpWhereFilter {
    fn filter_record(&self, _record: &Record) -> bool {
        true
    }

    fn validate_filtering(&self, _record: &Record) -> Result<(), CommandError<K>> {
        Ok(())
    }
}

pub struct And<'a, K: DatabaseKey> {
    filters: Vec<&'a dyn WhereFilter<K>>,
}

impl<'a, K: DatabaseKey> WhereFilter<K> for And<'a, K> {
    fn filter_record(&self, record: &Record) -> bool {
        self.filters.iter().all(|f| f.filter_record(record))
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError<K>> {
        let validation_results = self.filters.iter().map(|f| f.validate_filtering(record));

        if let Some(err) = validation_results.filter_map(Result::err).next() {
            return Err(err);
        }

        Ok(())
    }
}

pub struct Or<'a, K: DatabaseKey> {
    pub filters: Vec<&'a dyn WhereFilter<K>>,
}

impl<'a, K: DatabaseKey> WhereFilter<K> for Or<'a, K> {
    fn filter_record(&self, record: &Record) -> bool {
        self.filters.iter().any(|f| f.filter_record(record))
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError<K>> {
        let validation_results = self.filters.iter().map(|f| f.validate_filtering(record));

        if let Some(err) = validation_results.filter_map(Result::err).next() {
            return Err(err);
        }

        Ok(())
    }
}

pub struct ValueOperatorFilter {
    pub column_name: String,
    pub op: String,
    pub value: Value,
}

impl<K: DatabaseKey> WhereFilter<K> for ValueOperatorFilter {
    fn filter_record(&self, record: &Record) -> bool {
        let val = match record.get_value(&self.column_name) {
            Ok(val) => val,
            Err(_) => return false,
        };

        if !val.is_the_same_type_as(&self.value) {
            return false;
        }

        match self.op.as_str() {
            ">" => val.gt(&self.value),
            ">=" => val.ge(&self.value),
            "=" => val.eq(&self.value),
            "!=" => val.ne(&self.value),
            "<" => val.lt(&self.value),
            "<=" => val.le(&self.value),
            _ => false,
        }
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError<K>> {
        let val = record.get_value(&self.column_name)?;

        if !val.is_the_same_type_as(&self.value) {
            return Err(CommandError::InvalidValueError {
                column_name: self.column_name.clone(),
                expected_type: val.type_name(),
                got_type: self.value.type_name(),
            });
        }

        if !matches!(self.op.as_str(), ">" | ">=" | "=" | "<" | "<=" | "!=") {
            return Err(CommandError::UnknownOperatorError(self.op.clone()));
        }

        Ok(())
    }
}

pub struct ColumnOperatorFilter {
    pub column_name1: String,
    pub op: String,
    pub column_name2: String,
}

impl<K: DatabaseKey> WhereFilter<K> for ColumnOperatorFilter {
    fn filter_record(&self, record: &Record) -> bool {
        let value = match record.get_value(&self.column_name2) {
            Ok(val) => val,
            Err(_) => return false,
        };

        let value_filter = ValueOperatorFilter {
            column_name: self.column_name1.clone(),
            op: self.op.clone(),
            value: value.clone(),
        };

        WhereFilter::<K>::filter_record(&value_filter, record)
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError<K>> {
        let value = record.get_value(&self.column_name2)?;

        let value_filter = ValueOperatorFilter {
            column_name: self.column_name1.clone(),
            op: self.op.clone(),
            value: value.clone(),
        };

        value_filter.validate_filtering(record)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::database::table::ColumnType;

    use super::*;

    fn setup_test_table() -> Table<i64> {
        let mut table = Table::new_builder("Users".into(), "UserId".into())
            .with_column("Firstname".into(), ColumnType::STRING)
            .with_column("Lastname".into(), ColumnType::STRING)
            .with_column("Age".into(), ColumnType::INT)
            .with_column("Married".into(), ColumnType::BOOL)
            .build()
            .unwrap();

        table
            .insert(
                vec![
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::STRING("Maciej".into()),
                    Value::STRING("Kozłowski".into()),
                    Value::INT(16),
                    Value::BOOL(false),
                ],
            )
            .unwrap();

        table
            .insert(
                vec![
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::STRING("Krzysztof".into()),
                    Value::STRING("Wozniak".into()),
                    Value::INT(24),
                    Value::BOOL(true),
                ],
            )
            .unwrap();

        table
            .insert(
                vec![
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::STRING("Jan".into()),
                    Value::STRING("Kowalski".into()),
                    Value::INT(20),
                    Value::BOOL(false),
                ],
            )
            .unwrap();

        table
    }

    #[test]
    fn select_columns_test() {
        let table = setup_test_table();
        let mut select_command = SelectCommand::new(
            &table,
            vec!["Age".into(), "Firstname".into()],
            NoOpWhereFilter {},
        );

        let _ = select_command.execute();

        let result = select_command.select_result.unwrap();

        assert_eq!(result.len(), 3);

        let ages = vec![16, 24, 20];
        let names = vec!["Maciej", "Krzysztof", "Jan"];

        for (record, (age, name)) in result.iter().zip(ages.iter().zip(names)) {
            assert_eq!(record.len(), 2);

            assert!(record[0].is_the_same_type_as(&Value::INT(0)));
            assert_eq!(record[0], Value::INT(*age));

            assert!(record[1].is_the_same_type_as(&Value::STRING("".into())));
            assert_eq!(record[1], Value::STRING(name.into()));
        }
    }

    #[test]
    fn select_with_simple_filter_test() {
        let table = setup_test_table();

        // Query: SELECT Firstname, Age WHERE Age > 18
        let filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::INT(18),
        };

        let mut select_command =
            SelectCommand::new(&table, vec!["Firstname".into(), "Age".into()], filter);

        let _ = select_command.execute();
        let result = select_command.select_result.unwrap();

        // Oczekujemy Krzysztofa (24) i Jana (20), Maciej (16) odpada
        assert_eq!(result.len(), 2);

        let expected_names = vec!["Krzysztof", "Jan"];
        let expected_ages = vec![24, 20];

        for (record, (name, age)) in result.iter().zip(expected_names.iter().zip(expected_ages)) {
            assert_eq!(record[0], Value::STRING(name.to_string()));
            assert_eq!(record[1], Value::INT(age));
        }
    }

    #[test]
    fn select_with_and_filter_test() {
        let table = setup_test_table();

        // Query: SELECT Firstname WHERE Age > 18 AND Married = false
        let age_filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::INT(18),
        };
        let married_filter = ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::BOOL(false),
        };

        let and_filter = And {
            filters: vec![&age_filter, &married_filter],
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], and_filter);

        let _ = select_command.execute();
        let result = select_command.select_result.unwrap();

        // Maciej: false (za młody), Krzysztof: false (żonaty), Jan: true
        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], Value::STRING("Jan".into()));
    }

    #[test]
    fn select_with_or_filter_test() {
        let table = setup_test_table();

        // Query: SELECT Firstname WHERE Age < 18 OR Married = true
        let age_filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: "<".into(),
            value: Value::INT(18),
        };

        let married_filter = ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::BOOL(true),
        };

        let or_filter = Or {
            filters: vec![&age_filter, &married_filter],
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], or_filter);

        let _ = select_command.execute();
        let result = select_command.select_result.unwrap();

        // Maciej: pasuje (wiek < 18)
        // Krzysztof: pasuje (żonaty)
        // Jan: nie pasuje (wiek > 18 i nieżonaty)
        assert_eq!(result.len(), 2);

        let names: Vec<String> = result
            .iter()
            .map(|r| match &r[0] {
                Value::STRING(s) => s.clone(),
                _ => "".to_string(),
            })
            .collect();

        assert!(names.contains(&"Maciej".to_string()));
        assert!(names.contains(&"Krzysztof".to_string()));
    }

    #[test]
    fn select_with_column_comparison_test() {
        let table = setup_test_table();

        // Query: SELECT Firstname WHERE Age = Age (tautologia, powinna zwrócić wszystkich)
        // Normalnie porównywalibyśmy np. Przychód > Wydatki, ale tu mamy tylko jedną kolumnę INT
        let filter = ColumnOperatorFilter {
            column_name1: "Age".into(),
            op: "=".into(),
            column_name2: "Age".into(),
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let _ = select_command.execute();
        let result = select_command.select_result.unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn select_non_existent_column_fail_test() {
        let table = setup_test_table();

        // Query: SELECT NonExistentColumn
        let mut select_command =
            SelectCommand::new(&table, vec!["NonExistentColumn".into()], NoOpWhereFilter {});

        let result = select_command.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::RecordError(RecordError::InvalidColumnNameError(name)) => {
                assert_eq!(name, "NonExistentColumn");
            }
            _ => panic!("Oczekiwano błędu InvalidColumnNameError"),
        }
    }

    #[test]
    fn select_empty_result_test() {
        let table = setup_test_table();

        // Query: SELECT Firstname WHERE Age > 100
        let filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::INT(100),
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let _ = select_command.execute();
        let result = select_command.select_result.unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn select_filter_validation_missing_column_test() {
        let table = setup_test_table();

        let filter = ValueOperatorFilter {
            column_name: "GhostColumn".into(),
            op: "=".into(),
            value: Value::INT(1),
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let result = select_command.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::RecordError(RecordError::InvalidColumnNameError(name)) => {
                assert_eq!(name, "GhostColumn");
            }
            err => panic!("Oczekiwano błędu braku kolumny, otrzymano: {:?}", err),
        }
    }

    #[test]
    fn select_filter_validation_type_mismatch_test() {
        let table = setup_test_table();

        let filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::STRING("Eighteen".into()),
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let result = select_command.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidValueError {
                column_name,
                expected_type,
                got_type,
            } => {
                assert_eq!(column_name, "Age");
                assert!(expected_type.to_uppercase().contains("INT"));
                assert!(got_type.to_uppercase().contains("STRING"));
            }
            err => panic!("Oczekiwano błędu InvalidValueError, otrzymano: {:?}", err),
        }
    }

    #[test]
    fn select_filter_validation_unknown_operator_test() {
        let table = setup_test_table();

        let filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: "><".into(),
            value: Value::INT(18),
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let result = select_command.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::UnknownOperatorError(op) => {
                assert_eq!(op, "><");
            }
            err => panic!(
                "Oczekiwano błędu UnknownOperatorError, otrzymano: {:?}",
                err
            ),
        }
    }

    #[test]
    fn select_filter_validation_nested_error_test() {
        let table = setup_test_table();

        let valid_filter = ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::INT(10),
        };

        let invalid_filter = ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::INT(1),
        };

        let and_filter = And {
            filters: vec![&valid_filter, &invalid_filter],
        };

        let mut select_command = SelectCommand::new(&table, vec!["Firstname".into()], and_filter);

        let result = select_command.execute();

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidValueError { column_name, .. } => {
                assert_eq!(column_name, "Married");
            }
            err => panic!(
                "Oczekiwano błędu InvalidValueError wewnątrz AND, otrzymano: {:?}",
                err
            ),
        }
    }
}
