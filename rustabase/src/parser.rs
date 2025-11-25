use pest::{Parser, iterators::Pair};
use pest_ascii_tree::print_ascii_tree;
use pest_derive::Parser;
use thiserror::Error;

use crate::{
    commands::{
        command::AnyCommand,
        create_command::CreateCommand,
        delete_command::DeleteCommand,
        insert_command::InsertCommand,
        read_from_command::ReadFromCommand,
        save_as_command::SaveAsCommand,
        select_command::{
            And, AnyFilter, AnyWhereFilter, ColumnOperatorFilter, NoOpWhereFilter, Or,
            SelectCommand, ValueOperatorFilter,
        },
    },
    database::{
        Database, DatabaseError,
        key::DatabaseKey,
        table::{ColumnType, record::Value},
    },
};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Parsing error occured:\n{0}")]
    Error(String),

    #[error("Unknown rule encountered {0}")]
    UnknownRuleError(String),

    #[error("Missing token: {0}")]
    MissingTokenError(String),

    #[error("Database error occured: {0}")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct PestParser {}

pub struct CommandParser {
    commands_parsed: Vec<String>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            commands_parsed: Vec::new(),
        }
    }

    pub fn parse_command<'a, K: DatabaseKey>(
        &'a mut self,
        db: &'a mut Database<K>,
        command: &str,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let result = PestParser::parse(Rule::command, command);

        print_ascii_tree(result.clone());

        let pairs = match result {
            Ok(pairs) => pairs,
            Err(err) => return Err(ParserError::Error(err.to_string())),
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::create_command => return self.parse_create(pair, db),
                Rule::insert_command => return self.parse_insert(pair, db),
                Rule::select_query => return self.parse_select(pair, db),
                Rule::delete_command => return self.parse_delete(pair, db),
                Rule::save_as_command => return self.parse_save_as(pair),
                Rule::read_from_command => return self.parse_read_from(pair),
                _ => (),
            }
        }

        Err(ParserError::Error("Unknown command".into()))
    }

    fn parse_create<'a, K: DatabaseKey>(
        &mut self,
        pair: Pair<'_, Rule>,
        db: &'a mut Database<K>,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let mut table_name = None;
        let mut key_name = None;
        let mut fields = Vec::<String>::new();
        let mut types = Vec::<ColumnType>::new();

        let command_str = pair.as_str().to_string();

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
            }
        }

        let Some(table_name) = table_name else {
            return Err(ParserError::MissingTokenError("table_name".into()));
        };

        let Some(key_name) = key_name else {
            return Err(ParserError::MissingTokenError("key_name".into()));
        };

        let res = CreateCommand {
            database: db,
            table_name: table_name.into(),
            key_name: key_name.into(),
            fields,
            types,
        };

        self.commands_parsed.push(command_str);

        return Ok(res.into());
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
                        }
                    }
                }
                _ => return Err(ParserError::UnknownRuleError(field_type_pair.to_string())),
            }
        }

        Ok(())
    }

    fn parse_insert<'a, K: DatabaseKey>(
        &mut self,
        pair: Pair<'_, Rule>,
        db: &'a mut Database<K>,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let mut fields = Vec::<String>::new();
        let mut values = Vec::<Value>::new();
        let mut table_name = None;

        let command_str = pair.as_str().to_string();

        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::field_value_pair => {
                    CommandParser::parse_field_value_pair(&mut fields, &mut values, token)?;
                }
                Rule::table_name => table_name = Some(token.as_str()),
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }
        let Some(table_name) = table_name else {
            return Err(ParserError::MissingTokenError("table_name".into()));
        };

        let table = db.get_table(table_name)?;

        let res = InsertCommand {
            table,
            fields,
            values,
        };

        self.commands_parsed.push(command_str);

        Ok(res.into())
    }

    fn parse_value(token: &Pair<'_, Rule>) -> Result<Option<Value>, ParserError> {
        match token.as_rule() {
            Rule::int_value => {
                let value = match token.as_str().trim().parse::<i64>() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(ParserError::Error(e.to_string()));
                    }
                };

                Ok(Some(Value::INT(value)))
            }
            Rule::float_value => {
                let value = match token.as_str().trim().parse::<f64>() {
                    Ok(f) => f,
                    Err(e) => return Err(ParserError::Error(e.to_string())),
                };

                Ok(Some(Value::FLOAT(value)))
            }
            Rule::string_value => Ok(Some(Value::STRING(token.as_str().into()))),
            Rule::bool_value => {
                let value = match token.as_str().trim().parse::<bool>() {
                    Ok(v) => v,
                    Err(e) => return Err(ParserError::Error(e.to_string())),
                };

                Ok(Some(Value::BOOL(value)))
            }
            _ => Ok(None),
        }
    }

    fn parse_field_value_pair(
        fields: &mut Vec<String>,
        values: &mut Vec<Value>,
        field_value_pair: Pair<'_, Rule>,
    ) -> Result<(), ParserError> {
        for token in field_value_pair.into_inner() {
            if let Some(value) = CommandParser::parse_value(&token)? {
                values.push(value);
            } else {
                match token.as_rule() {
                    Rule::field_name => fields.push(token.as_str().into()),
                    _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
                }
            }
        }

        Ok(())
    }

    fn parse_select<'a, K: DatabaseKey>(
        &mut self,
        pair: Pair<'_, Rule>,
        db: &'a mut Database<K>,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let mut selected_columns = Vec::<String>::new();
        let mut table_name = None;
        let mut where_filter = NoOpWhereFilter {}.to_enum();

        let command_str = pair.as_str().to_string();

        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::column_names => {
                    CommandParser::parse_column_names(&mut selected_columns, token)?;
                }
                Rule::table_name => table_name = Some(token.as_str()),
                Rule::where_clause => {
                    if let Some(where_token) = token.into_inner().next() {
                        where_filter = CommandParser::construct_where_filter(where_token)?;
                    } else {
                        return Err(ParserError::MissingTokenError("or_expr".into()));
                    }
                }
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }

        let Some(table_name) = table_name else {
            return Err(ParserError::MissingTokenError("table_name".into()));
        };
        let table = db.get_table(table_name)?;

        let command = SelectCommand {
            table,
            selected_columns,
            where_filter,
        };

        self.commands_parsed.push(command_str);

        return Ok(command.into());
    }

    fn parse_column_names(
        column_names: &mut Vec<String>,
        column_names_token: Pair<'_, Rule>,
    ) -> Result<(), ParserError> {
        for token in column_names_token.into_inner() {
            match token.as_rule() {
                Rule::column_name => column_names.push(token.as_str().into()),
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }
        Ok(())
    }

    fn construct_where_filter(token: Pair<'_, Rule>) -> Result<AnyWhereFilter, ParserError> {
        let result = match token.as_rule() {
            Rule::or_expr => {
                let or_filter = CommandParser::construct_or(token)?;

                or_filter.to_enum()
            }
            Rule::and_expr => {
                let and_filter = CommandParser::construct_and(token)?;

                and_filter.to_enum()
            }
            Rule::operator_expr => CommandParser::construct_operator_filter(token)?,
            _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
        };

        Ok(result)
    }

    fn construct_operator_filter(token: Pair<'_, Rule>) -> Result<AnyWhereFilter, ParserError> {
        let mut column_name: Option<&str> = None;
        let mut op: Option<&str> = None;
        let mut value = None;

        for operator_token in token.into_inner() {
            if let Some(val) = CommandParser::parse_value(&operator_token)? {
                value = Some(val);
            } else {
                match operator_token.as_rule() {
                    Rule::column_name => match column_name {
                        Some(name) => {
                            let Some(op) = op else {
                                return Err(ParserError::MissingTokenError("op".into()));
                            };

                            let result = ColumnOperatorFilter {
                                column_name1: name.into(),
                                op: op.into(),
                                column_name2: operator_token.as_str().into(),
                            };

                            return Ok(result.to_enum());
                        }
                        None => column_name = Some(operator_token.as_str()),
                    },
                    Rule::op => {
                        op = Some(operator_token.as_str());
                    }
                    _ => {
                        return Err(ParserError::UnknownRuleError(
                            operator_token.as_str().into(),
                        ));
                    }
                }
            }
        }

        if let Some(column_name) = column_name
            && let Some(op) = op
            && let Some(value) = value
        {
            let result = ValueOperatorFilter {
                column_name: column_name.into(),
                op: op.into(),
                value,
            };

            return Ok(result.to_enum());
        }

        let missing_token = if column_name.is_none() {
            "column_name"
        } else if op.is_none() {
            "op"
        } else {
            "value"
        };

        Err(ParserError::MissingTokenError(missing_token.into()))
    }

    fn construct_and(token: Pair<'_, Rule>) -> Result<And, ParserError> {
        let mut and_filter = And {
            filters: Vec::new(),
        };

        for inner_and_token in token.into_inner() {
            and_filter
                .filters
                .push(CommandParser::construct_where_filter(inner_and_token)?.to_box());
        }
        Ok(and_filter)
    }

    fn construct_or(token: Pair<'_, Rule>) -> Result<Or, ParserError> {
        let mut or_filter = Or {
            filters: Vec::new(),
        };

        for inner_or_token in token.into_inner() {
            or_filter
                .filters
                .push(CommandParser::construct_where_filter(inner_or_token)?.to_box());
        }
        Ok(or_filter)
    }

    fn parse_delete<'a, K: DatabaseKey>(
        &mut self,
        pair: Pair<'_, Rule>,
        db: &'a mut Database<K>,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let mut table_name: Option<&str> = None;
        let mut key_value: Option<K> = None;

        let command_str = pair.as_str().to_string();

        for token in pair.into_inner() {
            if let Some(value) = CommandParser::parse_value(&token)?
                && let Some(key) = K::from_value(value)
            {
                key_value = Some(key);
                continue;
            }

            match token.as_rule() {
                Rule::table_name => {
                    table_name = Some(token.as_str());
                }
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }

        let Some(table_name) = table_name else {
            return Err(ParserError::MissingTokenError("table_name".into()));
        };

        let Some(key_value) = key_value else {
            return Err(ParserError::MissingTokenError("key_value".into()));
        };

        let table = db.get_table(table_name)?;

        self.commands_parsed.push(command_str);

        Ok(DeleteCommand {
            table,
            key: key_value,
        }
        .into())
    }

    fn parse_save_as<'a, K: DatabaseKey>(
        &'a self,
        pair: Pair<'_, Rule>,
    ) -> Result<AnyCommand<'a, K>, ParserError> {
        let mut file_name: Option<String> = None;

        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::file_name => {
                    file_name = Some(token.as_str().to_string());
                }
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }

        let Some(file_name) = file_name else {
            return Err(ParserError::MissingTokenError("file_name".into()));
        };

        let result = SaveAsCommand {
            file_name,
            lines: &self.commands_parsed,
        };

        Ok(result.into())
    }

    fn parse_read_from<K: DatabaseKey>(
        &self,
        pair: Pair<'_, Rule>,
    ) -> Result<AnyCommand<'_, K>, ParserError> {
        let mut file_name: Option<String> = None;

        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::file_name => {
                    file_name = Some(token.as_str().to_string());
                }
                _ => return Err(ParserError::UnknownRuleError(token.as_str().into())),
            }
        }

        let Some(file_name) = file_name else {
            return Err(ParserError::MissingTokenError("file_name".into()));
        };

        let result = ReadFromCommand { file_name };

        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::command::{AnyCommand, Command};
    use crate::database::table::ColumnType;

    use super::*;

    fn prepare_parser() -> CommandParser {
        CommandParser::new()
    }

    fn prepare_db() -> Database<i64> {
        Database::<i64>::new()
    }

    #[test]
    fn parse_create_command_basic() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "CREATE Users KEY UserId FIELDS Name: STRING, Age: INT";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::CreateCommand(create_cmd) => {
                assert_eq!(create_cmd.table_name, "Users");
                assert_eq!(create_cmd.key_name, "UserId");
                assert_eq!(create_cmd.fields, vec!["Name", "Age"]);
                assert_eq!(create_cmd.types, vec![ColumnType::STRING, ColumnType::INT]);
            }
            _ => panic!("Expected CreateCommand"),
        }
    }

    #[test]
    fn parse_create_command_without_fields() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "CREATE Users KEY UserId";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::CreateCommand(create_cmd) => {
                assert_eq!(create_cmd.table_name, "Users");
                assert_eq!(create_cmd.key_name, "UserId");
                assert!(create_cmd.fields.is_empty());
                assert!(create_cmd.types.is_empty());
            }
            _ => panic!("Expected CreateCommand"),
        }
    }

    #[test]
    fn parse_insert_command_basic() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let create_str = "CREATE Users KEY UserId FIELDS Name: STRING, Age: INT";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "INSERT Name=\"John\", Age=25 INTO Users";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::InsertCommand(insert_cmd) => {
                assert_eq!(insert_cmd.fields, vec!["Name", "Age"]);
                assert_eq!(
                    insert_cmd.values,
                    vec![Value::STRING("John".into()), Value::INT(25)]
                );
            }
            _ => panic!("Expected InsertCommand"),
        }
    }

    #[test]
    fn parse_select_command_basic() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let create_str = "CREATE Users KEY UserId FIELDS Name: STRING";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Name FROM Users";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(select_cmd) => {
                assert_eq!(select_cmd.selected_columns, vec!["Name"]);
                assert_eq!(select_cmd.table.get_name(), "Users");
            }
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Age FROM Users WHERE Age > 18";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_and() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT, Name: STRING";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Name FROM Users WHERE Age > 18 AND Name = \"John\"";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_or() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT, Status: STRING";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Age FROM Users WHERE Age < 30 OR Status = \"VIP\"";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_parentheses() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT, Status: STRING";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Age FROM Users WHERE (Age > 18 AND Age < 65) OR Status = \"VIP\"";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_operators() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT, Score: FLOAT, Active: BOOL";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str =
            "SELECT Age FROM Users WHERE Age >= 21 AND Score <= 100.5 AND Active = true";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_column_comparison() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Age: INT, MinAge: INT";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Age FROM Users WHERE Age > MinAge";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_select_command_with_where_not_equal() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        // Najpierw stworzyć tabelę
        let create_str = "CREATE Users KEY UserId FIELDS Status: STRING";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "SELECT Status FROM Users WHERE Status != \"Inactive\"";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SelectCommand(_) => {}
            _ => panic!("Expected SelectCommand"),
        }
    }

    #[test]
    fn parse_delete_command_basic() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let create_str = "CREATE Users KEY UserId";
        parser
            .parse_command(&mut db, create_str)
            .unwrap()
            .execute()
            .unwrap();

        let command_str = "DELETE 1 FROM Users";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::DeleteCommand(delete_cmd) => {
                assert_eq!(delete_cmd.key, 1i64);
                assert_eq!(delete_cmd.table.get_name(), "Users");
            }
            _ => panic!("Expected DeleteCommand"),
        }
    }

    #[test]
    fn parse_save_as_command() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "SAVE_AS commands.txt";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::SaveAsCommand(save_cmd) => {
                assert_eq!(save_cmd.file_name, "commands.txt");
            }
            _ => panic!("Expected SaveAsCommand"),
        }
    }

    #[test]
    fn parse_read_from_command() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "READ_FROM commands.txt";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_ok());
        let command = result.unwrap();
        match command {
            AnyCommand::ReadFromCommand(read_cmd) => {
                assert_eq!(read_cmd.file_name, "commands.txt");
            }
            _ => panic!("Expected ReadFromCommand"),
        }
    }

    #[test]
    fn parse_unknown_command_error() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "UNKNOWN COMMAND";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_syntax_error() {
        let mut parser = prepare_parser();
        let mut db = prepare_db();

        let command_str = "CREATE INVALID SYNTAX";

        let result = parser.parse_command(&mut db, command_str);

        assert!(result.is_err());
    }
}
