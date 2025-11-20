use crate::{
    commands::command::Command,
    database::{key::DatabaseKey, table::Table},
};

pub struct DeleteCommand<'a, K: DatabaseKey> {
    table: &'a mut Table<K>,
    key: K,
}

impl<'a, K: DatabaseKey> Command<K> for DeleteCommand<'a, K> {
    fn execute(&mut self) -> Result<(), super::command::CommandError<K>> {
        self.table.delete(self.key.clone())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::command::CommandError,
        database::table::{ColumnType, record::Value},
    };

    // Helper: Przygotowanie tabeli z jednym rekordem
    fn prepare_test_table() -> Table<i64> {
        let mut table = Table::new_builder("Products".to_string(), "ProductId".to_string())
            .with_column("Name".to_string(), ColumnType::STRING)
            .build()
            .unwrap();

        table
            .insert(
                vec!["Name".to_string()],
                vec![Value::STRING("Laptop".to_string())],
            )
            .unwrap();

        table
    }

    // Helper: Pobranie klucza pierwszego rekordu (ponieważ generowanie kluczy jest wewnętrzne)
    fn get_existing_key(table: &Table<i64>) -> i64 {
        let records = table.filter(|_| true);
        let record = records.first().expect("Table should have records");
        match record.get_value("ProductId").unwrap() {
            Value::INT(k) => *k,
            _ => panic!("Key column should be INT"),
        }
    }

    #[test]
    fn delete_command_success_test() {
        let mut table = prepare_test_table();
        let key_to_delete = get_existing_key(&table);

        assert_eq!(table.filter(|_| true).len(), 1);

        let mut command = DeleteCommand {
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
        let existing_key = get_existing_key(&table);

        let missing_key = existing_key + 100;

        let mut command = DeleteCommand {
            table: &mut table,
            key: missing_key,
        };

        let result = command.execute();

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            CommandError::<i64>::TableError(crate::database::table::TableError::KeyNotFoundError(
                missing_key
            ))
        );
        // Upewniamy się, że istniejący rekord nie został usunięty
        assert_eq!(table.filter(|_| true).len(), 1);
    }
}
