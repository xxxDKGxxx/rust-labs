use crate::{
    commands::command::{Command, CommandError},
    database::{Database, key::DatabaseKey, table::ColumnType},
};

pub struct CreateCommand<'a, K: DatabaseKey> {
    pub database: &'a mut Database<K>,
    pub table_name: String,
    pub key_name: String,
    pub fields: Vec<String>,
    pub types: Vec<ColumnType>,
}

impl<'a, K: DatabaseKey> Command<K> for CreateCommand<'a, K> {
    fn execute(&mut self) -> Result<(), CommandError<K>> {
        self.database.create_table(
            self.table_name.clone(),
            self.key_name.clone(),
            self.fields.clone(),
            self.types.clone(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_table_basic() {
        let mut db = Database::<i64>::new();

        let table_name = String::from("Users");
        let key_name = String::from("UserId");
        let fields = vec![String::from("Name"), String::from("Age")];
        let types = vec![ColumnType::STRING, ColumnType::INT];

        let mut command = CreateCommand {
            database: &mut db,
            table_name: table_name.clone(),
            key_name: key_name.clone(),
            fields: fields.clone(),
            types: types.clone(),
        };

        command.execute().unwrap();

        let table = db.get_table("Users".into()).unwrap();

        let table_columns = table.get_columns();

        assert!(table_columns.keys().into_iter().all(|c| fields.contains(c)));

        assert!(fields.iter().all(|f| table_columns.contains_key(f)));

        assert!(
            fields
                .iter()
                .zip(types)
                .all(|(f, t)| *table_columns.get(f).unwrap() == t)
        );

        assert_eq!(table.get_name(), table_name);
        assert_eq!(table.get_key_name(), key_name);
    }

    #[test]
    fn create_second_table_with_the_same_name() {
        let mut db = Database::<i64>::new();

        let table_name = String::from("Users");
        let key_name = String::from("UserId");
        let fields = vec![String::from("Name"), String::from("Age")];
        let types = vec![ColumnType::STRING, ColumnType::INT];

        let mut command1 = CreateCommand {
            database: &mut db,
            table_name: table_name.clone(),
            key_name: key_name.clone(),
            fields: fields.clone(),
            types: types.clone(),
        };

        command1.execute().unwrap();

        let mut command2 = CreateCommand {
            database: &mut db,
            table_name: table_name.clone(),
            key_name: key_name.clone(),
            fields: fields.clone(),
            types: types.clone(),
        };

        let result = command2.execute();

        let err = result.unwrap_err();

        assert_eq!(
            err,
            CommandError::<i64>::DatabaseError(
                crate::database::DatabaseError::TableAlreadyExistsError(table_name)
            )
        );
    }

    #[test]
    fn create_table_with_two_same_columns() {
        let mut db = Database::<i64>::new();

        let table_name = String::from("Users");
        let key_name = String::from("UserId");
        let fields = vec![String::from("Age"), String::from("Age")];
        let types = vec![ColumnType::FLOAT, ColumnType::INT];

        let mut command = CreateCommand {
            database: &mut db,
            table_name,
            key_name,
            fields,
            types,
        };

        let result = command.execute();

        let err = result.unwrap_err();

        assert_eq!(
            err,
            CommandError::<i64>::DatabaseError(crate::database::DatabaseError::TableError(
                crate::database::table::TableError::ColumnDefinedTwiceError {
                    column_name: "Age".into(),
                    first_type: ColumnType::FLOAT,
                    second_type: ColumnType::INT
                }
            ))
        );
    }
}
