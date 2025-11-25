use crate::{
    commands::command::{AnyCommand, Command, CommandError, CommandResult},
    database::{
        key::DatabaseKey,
        table::{
            Table,
            record::{Record, RecordError, Value},
        },
    },
};

pub trait WhereFilter {
    fn filter_record(&self, record: &Record) -> bool;

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError>;
}

pub trait AnyFilter {
    fn to_enum(self) -> AnyWhereFilter;
}

#[derive(Debug)]
pub enum AnyWhereFilter {
    NoOp(NoOpWhereFilter),
    And(And),
    Or(Or),
    ValueOperator(ValueOperatorFilter),
    ColumnOperator(ColumnOperatorFilter),
}

impl AnyWhereFilter {
    pub fn to_box(self) -> Box<Self> {
        Box::new(self)
    }
}

impl WhereFilter for AnyWhereFilter {
    fn filter_record(&self, record: &Record) -> bool {
        match self {
            AnyWhereFilter::NoOp(no_op_where_filter) => no_op_where_filter.filter_record(record),
            AnyWhereFilter::And(and) => and.filter_record(record),
            AnyWhereFilter::Or(or) => or.filter_record(record),
            AnyWhereFilter::ValueOperator(value_operator_filter) => {
                value_operator_filter.filter_record(record)
            }
            AnyWhereFilter::ColumnOperator(column_operator_filter) => {
                column_operator_filter.filter_record(record)
            }
        }
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError> {
        match self {
            AnyWhereFilter::NoOp(no_op_where_filter) => {
                no_op_where_filter.validate_filtering(record)
            }
            AnyWhereFilter::And(and) => and.validate_filtering(record),
            AnyWhereFilter::Or(or) => or.validate_filtering(record),
            AnyWhereFilter::ValueOperator(value_operator_filter) => {
                value_operator_filter.validate_filtering(record)
            }
            AnyWhereFilter::ColumnOperator(column_operator_filter) => {
                column_operator_filter.validate_filtering(record)
            }
        }
    }
}

pub struct SelectCommand<'a, K: DatabaseKey> {
    pub table: &'a Table<K>,
    pub selected_columns: Vec<String>,
    pub where_filter: AnyWhereFilter,
}

impl<K: DatabaseKey> Command for SelectCommand<'_, K> {
    fn execute(self) -> Result<CommandResult, CommandError> {
        let where_errors = self.validate_where();

        if let Some(err) = where_errors.into_iter().next() {
            return Err(err);
        }

        let results = self.select_records();
        let (successes, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        let errors = errors.into_iter().filter_map(Result::err);

        if let Some(err) = errors.into_iter().next() {
            return Err(CommandError::RecordError(err));
        }

        let successes = successes.into_iter().filter_map(Result::ok).collect();

        Ok(CommandResult::RecordValueList(
            self.selected_columns,
            successes,
        ))
    }
}

impl<'a, K: DatabaseKey> SelectCommand<'a, K> {
    pub fn new(
        table: &'a Table<K>,
        selected_columns: Vec<String>,
        where_filter: AnyWhereFilter,
    ) -> Self {
        Self {
            table,
            selected_columns,
            where_filter,
        }
    }

    fn validate_where(&self) -> Vec<CommandError> {
        let where_errors: Vec<CommandError> = self
            .table
            .filter(|_r| true)
            .iter()
            .map(|r| self.where_filter.validate_filtering(r))
            .filter_map(Result::err)
            .collect();
        where_errors
    }

    fn select_records(&self) -> Vec<Result<Vec<Value>, RecordError>> {
        let results: Vec<Result<Vec<Value>, RecordError>> = self
            .table
            .filter(|record| self.where_filter.filter_record(record))
            .into_iter()
            .map(|record| {
                record.get_values(&self.selected_columns.iter().map(String::as_str).collect())
            })
            .collect();
        results
    }
}

#[derive(Debug)]
pub struct NoOpWhereFilter {}

impl WhereFilter for NoOpWhereFilter {
    fn filter_record(&self, _record: &Record) -> bool {
        true
    }

    fn validate_filtering(&self, _record: &Record) -> Result<(), CommandError> {
        Ok(())
    }
}

impl AnyFilter for NoOpWhereFilter {
    fn to_enum(self) -> AnyWhereFilter {
        AnyWhereFilter::NoOp(self)
    }
}

#[derive(Debug)]
pub struct And {
    pub filters: Vec<Box<AnyWhereFilter>>,
}

impl WhereFilter for And {
    fn filter_record(&self, record: &Record) -> bool {
        self.filters.iter().all(|f| f.filter_record(record))
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError> {
        let mut validation_results = self.filters.iter().map(|f| f.validate_filtering(record));

        if let Some(err) = validation_results.find_map(Result::err) {
            return Err(err);
        }

        Ok(())
    }
}

impl AnyFilter for And {
    fn to_enum(self) -> AnyWhereFilter {
        AnyWhereFilter::And(self)
    }
}

#[derive(Debug)]
pub struct Or {
    pub filters: Vec<Box<AnyWhereFilter>>,
}

impl WhereFilter for Or {
    fn filter_record(&self, record: &Record) -> bool {
        self.filters.iter().any(|f| f.filter_record(record))
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError> {
        let mut validation_results = self.filters.iter().map(|f| f.validate_filtering(record));

        if let Some(err) = validation_results.find_map(Result::err) {
            return Err(err);
        }

        Ok(())
    }
}

impl AnyFilter for Or {
    fn to_enum(self) -> AnyWhereFilter {
        AnyWhereFilter::Or(self)
    }
}

#[derive(Debug)]
pub struct ValueOperatorFilter {
    pub column_name: String,
    pub op: String,
    pub value: Value,
}

impl WhereFilter for ValueOperatorFilter {
    fn filter_record(&self, record: &Record) -> bool {
        let Ok(val) = record.get_value(&self.column_name) else {
            return false;
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

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError> {
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

impl AnyFilter for ValueOperatorFilter {
    fn to_enum(self) -> AnyWhereFilter {
        AnyWhereFilter::ValueOperator(self)
    }
}

#[derive(Debug)]
pub struct ColumnOperatorFilter {
    pub column_name1: String,
    pub op: String,
    pub column_name2: String,
}

impl WhereFilter for ColumnOperatorFilter {
    fn filter_record(&self, record: &Record) -> bool {
        let Ok(value) = record.get_value(&self.column_name2) else {
            return false;
        };

        let value_filter = ValueOperatorFilter {
            column_name: self.column_name1.clone(),
            op: self.op.clone(),
            value: value.clone(),
        };

        WhereFilter::filter_record(&value_filter, record)
    }

    fn validate_filtering(&self, record: &Record) -> Result<(), CommandError> {
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

impl AnyFilter for ColumnOperatorFilter {
    fn to_enum(self) -> AnyWhereFilter {
        AnyWhereFilter::ColumnOperator(self)
    }
}

impl<'a, K: DatabaseKey> From<SelectCommand<'a, K>> for AnyCommand<'a, K> {
    fn from(value: SelectCommand<'a, K>) -> Self {
        Self::SelectCommand(value)
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
                // Dodano UserId
                vec![
                    "UserId".into(),
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::INT(1),
                    Value::STRING("Maciej".into()),
                    Value::STRING("Kozłowski".into()),
                    Value::INT(16),
                    Value::BOOL(false),
                ],
            )
            .unwrap();

        table
            .insert(
                // Dodano UserId
                vec![
                    "UserId".into(),
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::INT(2),
                    Value::STRING("Krzysztof".into()),
                    Value::STRING("Wozniak".into()),
                    Value::INT(24),
                    Value::BOOL(true),
                ],
            )
            .unwrap();

        table
            .insert(
                // Dodano UserId
                vec![
                    "UserId".into(),
                    "Firstname".into(),
                    "Lastname".into(),
                    "Age".into(),
                    "Married".into(),
                ],
                vec![
                    Value::INT(3),
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
        let selected_columns = vec!["Age".into(), "Firstname".into()];
        let select_command = SelectCommand::new(
            &table,
            selected_columns.clone(),
            NoOpWhereFilter {}.to_enum(),
        );

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(column_names, result) = result else {
            unreachable!("Checked above");
        };

        assert!(selected_columns.iter().all(|c| column_names.contains(c)));
        assert!(column_names.iter().all(|c| selected_columns.contains(c)));

        assert_eq!(result.len(), 3);

        let ages = [16, 24, 20];
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
        }
        .to_enum();

        let selected_columns = vec!["Firstname".into(), "Age".into()];
        let select_command = SelectCommand::new(&table, selected_columns.clone(), filter);

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(column_names, result) = result else {
            unreachable!("Checked above");
        };

        assert!(selected_columns.iter().all(|c| column_names.contains(c)));
        assert!(column_names.iter().all(|c| selected_columns.contains(c)));

        // Oczekujemy Krzysztofa (24) i Jana (20), Maciej (16) odpada
        assert_eq!(result.len(), 2);

        let expected_names = ["Krzysztof", "Jan"];
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
        let age_filter = AnyWhereFilter::ValueOperator(ValueOperatorFilter {
            column_name: "Age".into(),
            op: ">".into(),
            value: Value::INT(18),
        });

        let married_filter = AnyWhereFilter::ValueOperator(ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::BOOL(false),
        });

        let and_filter = AnyWhereFilter::And(And {
            filters: vec![age_filter.to_box(), married_filter.to_box()],
        });

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], and_filter);

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(_, result) = result else {
            unreachable!("Checked above");
        };

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
        }
        .to_enum()
        .to_box();

        let married_filter = ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::BOOL(true),
        }
        .to_enum()
        .to_box();

        let or_filter = Or {
            filters: vec![age_filter, married_filter],
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], or_filter);

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(_, result) = result else {
            unreachable!("Checked above");
        };

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
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(_, result) = result else {
            unreachable!("Checked above");
        };

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn select_non_existent_column_fail_test() {
        let table = setup_test_table();

        // Query: SELECT NonExistentColumn
        let select_command = SelectCommand::new(
            &table,
            vec!["NonExistentColumn".into()],
            NoOpWhereFilter {}.to_enum(),
        );

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
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

        let result = select_command.execute().unwrap();

        assert!(matches!(result, CommandResult::RecordValueList(_, _)));

        let CommandResult::RecordValueList(_, result) = result else {
            unreachable!("Checked above");
        };

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn select_filter_validation_missing_column_test() {
        let table = setup_test_table();

        let filter = ValueOperatorFilter {
            column_name: "GhostColumn".into(),
            op: "=".into(),
            value: Value::INT(1),
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

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
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

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
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], filter);

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
        }
        .to_enum();

        let invalid_filter = ValueOperatorFilter {
            column_name: "Married".into(),
            op: "=".into(),
            value: Value::INT(1),
        }
        .to_enum();

        let and_filter = And {
            filters: vec![valid_filter.to_box(), invalid_filter.to_box()],
        }
        .to_enum();

        let select_command = SelectCommand::new(&table, vec!["Firstname".into()], and_filter);

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
