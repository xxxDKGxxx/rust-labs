use thiserror::Error;

use crate::database::{
    key::DatabaseKey,
    table::{ColumnType, Table, TableError},
};

pub mod key;
pub mod table;

pub enum AnyDatabase {
    StringDatabase(Database<String>),
    I64Database(Database<i64>),
}

#[derive(Error, Debug, PartialEq)]
pub enum DatabaseError {
    #[error("Table named: {0} already exists")]
    TableAlreadyExistsError(String),

    #[error("Table error occured: {0}")]
    TableError(#[from] TableError),

    #[error("Table {0} not found")]
    TableNotFoundError(String),
}

pub struct Database<K: DatabaseKey> {
    tables: Vec<Table<K>>,
}

impl<K: DatabaseKey> Default for Database<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: DatabaseKey> Database<K> {
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }
    pub fn create_table(
        &mut self,
        table_name: String,
        key_name: String,
        fields: Vec<String>,
        types: Vec<ColumnType>,
    ) -> Result<(), DatabaseError> {
        if self
            .tables
            .iter()
            .filter(|t| t.get_name() == table_name)
            .count()
            > 0
        {
            return Err(DatabaseError::TableAlreadyExistsError(table_name));
        }

        let mut new_table = Table::<K>::new_builder(table_name, key_name);

        for (field, t) in fields.into_iter().zip(types.into_iter()) {
            new_table = new_table.with_column(field, t);
        }

        let new_table = new_table.build()?;

        self.tables.push(new_table);
        Ok(())
    }

    pub fn get_table(&mut self, table_name: &str) -> Result<&mut Table<K>, DatabaseError> {
        match self
            .tables
            .iter_mut()
            .find(|tab| tab.get_name() == table_name)
        {
            Some(tab) => Ok(tab),
            None => Err(DatabaseError::TableNotFoundError(table_name.into())),
        }
    }

    pub fn get_table_names(&self) -> Vec<&str> {
        self.tables.iter().map(Table::get_name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare_database() -> Database<i64> {
        Database::new()
    }

    fn prepare_populated_database() -> Database<i64> {
        let mut db = prepare_database();
        db.create_table(
            "Users".to_string(),
            "UserId".to_string(),
            vec!["Name".to_string(), "Age".to_string()],
            vec![ColumnType::STRING, ColumnType::INT],
        )
        .unwrap();
        db
    }

    #[test]
    fn database_creation_test() {
        let db = Database::<i64>::new();
        assert_eq!(db.tables.len(), 0);
    }

    #[test]
    fn create_table_success_test() {
        let mut db = prepare_database();
        let result = db.create_table(
            "Items".to_string(),
            "ItemId".to_string(),
            vec!["Name".to_string()],
            vec![ColumnType::STRING],
        );

        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables[0].get_name(), "Items".to_string());
        assert_eq!(db.tables[0].get_columns().len(), 2);
    }

    #[test]
    fn create_table_already_exists_test() {
        let mut db = prepare_populated_database();

        let result = db.create_table("Users".to_string(), "Key".to_string(), vec![], vec![]);

        assert!(result.is_err());
        assert_eq!(db.tables.len(), 1);
        assert_eq!(
            result.unwrap_err(),
            DatabaseError::TableAlreadyExistsError("Users".to_string())
        );
    }

    #[test]
    fn get_table_success_test() {
        let mut db = prepare_populated_database();
        let table_name = "Users".to_string();

        let result = db.get_table(&table_name);

        assert!(result.is_ok());
        let table = result.unwrap();
        assert_eq!(table.get_name(), table_name);
    }

    #[test]
    fn get_table_not_found_test() {
        let mut db = prepare_populated_database();
        let missing_name = "Orders".to_string();

        let result = db.get_table(&missing_name);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            DatabaseError::TableNotFoundError(missing_name)
        );
    }
}
