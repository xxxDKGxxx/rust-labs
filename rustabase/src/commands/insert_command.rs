use crate::{
    commands::command::{Command, CommandError},
    database::{
        key::DatabaseKey,
        table::{Table, record::Value},
    },
};

pub struct InsertCommand<'a, K: DatabaseKey> {
    table: &'a mut Table<K>,
    fields: Vec<String>,
    values: Vec<Value>,
}

impl<'a, K: DatabaseKey> Command<K> for InsertCommand<'a, K> {
    fn execute(&mut self) -> Result<(), CommandError<K>> {
        self.table
            .insert(self.fields.clone(), self.values.clone())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::table::ColumnType;

    use super::*;

    fn prepare_test_table() -> Table<i64> {
        Table::new_builder("Orders".to_string(), "OrderId".to_string())
            .with_column("ClientName".to_string(), ColumnType::STRING)
            .with_column("Capacity".to_string(), ColumnType::INT)
            .build()
            .unwrap()
    }

    #[test]
    fn insert_command_success_test() {
        let mut table = prepare_test_table();

        let mut command = InsertCommand {
            table: &mut table,
            fields: vec!["ClientName".to_string(), "Capacity".to_string()],
            values: vec![Value::STRING("Firma ABC".to_string()), Value::INT(100)],
        };

        let result = command.execute();

        assert!(result.is_ok());
        assert_eq!(table.get_columns().len(), 2);

        let records = table.filter(|r| {
            r.get_value("ClientName")
                .map(|v| v == &Value::STRING("Firma ABC".to_string()))
                .unwrap_or(false)
        });

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get_value("Capacity").unwrap(), &Value::INT(100));
    }

    #[test]
    fn insert_command_multiple_success_test() {
        let mut table = prepare_test_table();

        let mut cmd1 = InsertCommand {
            table: &mut table,
            fields: vec!["ClientName".to_string(), "Capacity".to_string()],
            values: vec![Value::STRING("Klient 1".to_string()), Value::INT(50)],
        };
        assert!(cmd1.execute().is_ok());

        let mut cmd2 = InsertCommand {
            table: &mut table,
            fields: vec!["Capacity".to_string(), "ClientName".to_string()],
            values: vec![Value::INT(200), Value::STRING("Klient 2".to_string())],
        };
        assert!(cmd2.execute().is_ok());

        assert_eq!(table.filter(|_| true).len(), 2);
    }

    #[test]
    fn insert_command_missing_columns_error_test() {
        let mut table = prepare_test_table();

        let mut command = InsertCommand {
            table: &mut table,
            fields: vec!["ClientName".to_string()],
            values: vec![Value::STRING("Niekompletny".to_string())],
        };

        let result = command.execute();

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            CommandError::<i64>::TableError(
                crate::database::table::TableError::InsertMissingColumnsError(vec![
                    "Capacity".into()
                ])
            )
        )
    }

    #[test]
    fn insert_command_invalid_column_name_error_test() {
        let mut table = prepare_test_table();

        let mut command = InsertCommand {
            table: &mut table,
            fields: vec![
                "ClientName".to_string(),
                "Capacity".to_string(),
                "Discount".to_string(),
            ],
            values: vec![
                Value::STRING("Test".to_string()),
                Value::INT(10),
                Value::INT(5),
            ],
        };

        let result = command.execute();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::<i64>::TableError(
                crate::database::table::TableError::InvalidColumnNameError("Discount".into())
            )
        )
    }

    #[test]
    fn insert_command_type_mismatch_error_test() {
        let mut table = prepare_test_table();

        let mut command = InsertCommand {
            table: &mut table,
            fields: vec!["ClientName".to_string(), "Capacity".to_string()],
            values: vec![
                Value::STRING("Test".to_string()),
                Value::STRING("Dużo".to_string()),
            ],
        };

        let result = command.execute();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::<i64>::TableError(
                crate::database::table::TableError::InsertInvalidColumnTypeError {
                    column_name: "Capacity".into(),
                    expected_type: ColumnType::INT,
                    got_type: ColumnType::STRING
                }
            )
        )
    }
}
