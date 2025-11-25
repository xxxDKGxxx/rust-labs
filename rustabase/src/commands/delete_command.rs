use crate::{
    commands::command::{AnyCommand, Command, CommandError, CommandResult},
    database::{key::DatabaseKey, table::Table},
};

pub struct DeleteCommand<'a, K: DatabaseKey> {
    pub table: &'a mut Table<K>,
    pub key: K,
}

impl<K: DatabaseKey> Command for DeleteCommand<'_, K> {
    fn execute(self) -> Result<CommandResult, CommandError> {
        self.table.delete(self.key.clone())?;
        Ok(CommandResult::Void)
    }
}

impl<'a, K: DatabaseKey> From<DeleteCommand<'a, K>> for AnyCommand<'a, K> {
    fn from(value: DeleteCommand<'a, K>) -> Self {
        Self::DeleteCommand(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::command::CommandError,
        database::table::{ColumnType, record::Value},
    };

    fn prepare_test_table() -> Table<i64> {
        let mut table = Table::new_builder("Products".to_string(), "ProductId".to_string())
            .with_column("Name".to_string(), ColumnType::STRING)
            .build()
            .unwrap();

        table
            .insert(
                vec!["ProductId".to_string(), "Name".to_string()],
                vec![Value::INT(1), Value::STRING("Laptop".to_string())],
            )
            .unwrap();

        table
    }

    #[test]
    fn delete_command_success_test() {
        let mut table = prepare_test_table();
        let key_to_delete = 1;

        assert_eq!(table.filter(|_| true).len(), 1);

        let command = DeleteCommand {
            table: &mut table,
            key: key_to_delete,
        };

        let result = command.execute();

        assert!(result.is_ok());

        assert_eq!(table.filter(|_| true).len(), 0);
    }

    #[test]
    fn delete_command_key_not_found_error_test() {
        let mut table = prepare_test_table();

        let missing_key = 101;

        let command = DeleteCommand {
            table: &mut table,
            key: missing_key,
        };

        let result = command.execute();

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            CommandError::TableError(crate::database::table::TableError::KeyNotFoundError(
                missing_key.to_value()
            ))
        );
        assert_eq!(table.filter(|_| true).len(), 1);
    }
}
