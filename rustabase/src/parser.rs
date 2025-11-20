use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use thiserror::Error;

use crate::{
    commands::{command::Command, create_command::CreateCommand},
    database::{Database, key::DatabaseKey, table::ColumnType},
};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Parsing error occured:\n{0}")]
    Error(String),

    #[error("Unknown rule encountered {0}")]
    UnknownRuleError(String),

    #[error("Missing token: {0}")]
    MissingTokenError(String),
}

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct PestParser {}

pub struct CommandParser {}

impl CommandParser {
    pub fn parse_command<'a, K: DatabaseKey>(
        db: &'a mut Database<K>,
        command: &str,
    ) -> Result<Box<dyn Command<K> + 'a>, ParserError> {
        let result = PestParser::parse(Rule::command, command);

        let pairs = match result {
            Ok(pairs) => pairs,
            Err(err) => return Err(ParserError::Error(err.to_string())),
        };

        for pair in pairs {
            if pair.as_rule() == Rule::create_command {
                return CommandParser::parse_create(pair, db);
            };
        }

        Err(ParserError::Error("Unknown command".into()))
    }

    fn parse_create<'a, K: DatabaseKey>(
        pair: Pair<'_, Rule>,
        db: &'a mut Database<K>,
    ) -> Result<Box<dyn Command<K> + 'a>, ParserError> {
        let mut table_name = None;
        let mut key_name = None;
        let mut fields = Vec::<String>::new();
        let mut types = Vec::<ColumnType>::new();

        for create_pair in pair.into_inner() {
            match create_pair.as_rule() {
                Rule::table_name => table_name = create_pair.as_str().into(),
                Rule::key_name => key_name = create_pair.as_str().into(),
                Rule::field_type_pair => {
                    CommandParser::parse_field_type(&mut fields, &mut types, create_pair)?;
                }
                _ => {
                    return Err(ParserError::UnknownRuleError(create_pair.to_string()));
                }
            };
        }

        if let Some(table_name) = table_name
            && let Some(key_name) = key_name
        {
            let res = CreateCommand {
                database: db,
                table_name: table_name.into(),
                key_name: key_name.into(),
                fields,
                types,
            };

            return Ok(Box::new(res));
        }

        let missing_token = if table_name.is_none() {
            "table_name"
        } else {
            "key_name"
        };

        Err(ParserError::MissingTokenError(missing_token.into()))
    }

    fn parse_field_type(
        fields: &mut Vec<String>,
        types: &mut Vec<ColumnType>,
        pair: Pair<'_, Rule>,
    ) -> Result<(), ParserError> {
        for field_type_pair in pair.into_inner() {
            match field_type_pair.as_rule() {
                Rule::field_name => fields.push(field_type_pair.as_str().into()),
                Rule::r#type => {
                    for type_pair in field_type_pair.into_inner() {
                        match type_pair.as_rule() {
                            Rule::string => types.push(ColumnType::STRING),
                            Rule::int => types.push(ColumnType::INT),
                            Rule::float => types.push(ColumnType::FLOAT),
                            Rule::bool => types.push(ColumnType::BOOL),
                            _ => return Err(ParserError::UnknownRuleError(type_pair.to_string())),
                        };
                    }
                }
                _ => return Err(ParserError::UnknownRuleError(field_type_pair.to_string())),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_command_parsing_test() {
        let statement = "CREATE Users KEY UserId FIELDS Name: STRING, Age: INT";

        let mut db = Database::<i64>::new();
        {
            let mut command = CommandParser::parse_command(&mut db, statement).unwrap();

            command.execute().unwrap();
        }

        assert!(db.get_table("Users".into()).is_ok());
    }
}
