use std::collections::VecDeque;

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

trait WhereFilter {
    fn filter_record(&self, record: &Record) -> bool;
}

struct SelectCommand<'a, K: DatabaseKey, F: WhereFilter> {
    table: &'a Table<K>,
    selected_columns: Vec<String>,
    where_filter: F,
    select_result: Option<Vec<Vec<Value>>>,
}

impl<'a, K: DatabaseKey, F: WhereFilter> Command for SelectCommand<'a, K, F> {
    fn execute(&mut self) -> Result<(), CommandError> {
        let results: Vec<Result<Vec<Value>, RecordError>> = self
            .table
            .filter(|record| self.where_filter.filter_record(record))
            .into_iter()
            .map(|record| {
                record.get_values(self.selected_columns.iter().map(|c| c.as_str()).collect())
            })
            .collect();

        let (successes, errors): (
            Vec<Result<Vec<Value>, RecordError>>,
            Vec<Result<Vec<Value>, RecordError>>,
        ) = results.into_iter().partition(Result::is_ok);

        let errors = errors.into_iter().filter_map(Result::err);

        match errors.into_iter().next() {
            Some(err) => return Err(CommandError::RecordError(err)),
            _ => (),
        };

        let successes = successes.into_iter().filter_map(Result::ok).collect();

        self.select_result = Some(successes);

        Ok(())
    }
}

impl<'a, K: DatabaseKey, F: WhereFilter> SelectCommand<'a, K, F> {
    fn new(table: &'a Table<K>, selected_columns: Vec<String>, where_filter: F) -> Self {
        Self {
            table,
            selected_columns,
            where_filter,
            select_result: None,
        }
    }
}

struct AndWhereFilter<'a> {
    filters: Vec<&'a dyn WhereFilter>,
}

impl<'a> WhereFilter for AndWhereFilter<'a> {
    fn filter_record(&self, record: &Record) -> bool {
        return self.filters.iter().all(|f| f.filter_record(record));
    }
}

struct OrWhereFilter<'a> {
    filters: Vec<&'a dyn WhereFilter>,
}

impl<'a> WhereFilter for OrWhereFilter<'a> {
    fn filter_record(&self, record: &Record) -> bool {
        return self.filters.iter().any(|f| f.filter_record(record));
    }
}
