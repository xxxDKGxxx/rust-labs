#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
use std::io::stdin;

use clap::Parser;
use rustabase::{
    commands::command::{Command, CommandResult},
    database::{AnyDatabase, Database, key::DatabaseKey, table::record},
    parser::CommandParser,
};
use thiserror::Error;

#[derive(Clone, Debug)]
enum KeyType {
    String,
    I64,
    Unknown,
}

#[derive(Error, Debug)]
enum ArgsError {
    #[error("Unsupported key type provided for the database.")]
    UnknownKeyTypeError,
}

impl From<String> for KeyType {
    fn from(value: String) -> Self {
        if value == "String" {
            return KeyType::String;
        }

        if value == "I64" {
            return KeyType::I64;
        }

        KeyType::Unknown
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    key_type: KeyType,
}

fn main() {
    let db = match create_db_from_args() {
        Ok(db) => db,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    let mut command_parser = CommandParser::new();

    match db {
        AnyDatabase::StringDatabase(mut database) => loop {
            handle_user_input(&mut database, &mut command_parser);
        },
        AnyDatabase::I64Database(mut database) => loop {
            handle_user_input(&mut database, &mut command_parser);
        },
    }
}

fn create_db_from_args() -> Result<AnyDatabase, ArgsError> {
    let args = Args::parse();

    match args.key_type {
        KeyType::String => Ok(AnyDatabase::StringDatabase(Database::<String>::new())),
        KeyType::I64 => Ok(AnyDatabase::I64Database(Database::<i64>::new())),
        KeyType::Unknown => Err(ArgsError::UnknownKeyTypeError),
    }
}

fn handle_user_input<K: DatabaseKey>(db: &mut Database<K>, command_parser: &mut CommandParser) {
    let mut line = String::new();
    if let Err(e) = stdin().read_line(&mut line) {
        println!("{e}");
        return;
    }
    println!("\n\n");
    let parse_result = command_parser.parse_command(db, &line);
    let command = match parse_result {
        Err(e) => {
            println!("{e}");
            return;
        }
        Ok(command) => command,
    };
    let result = match command.execute() {
        Ok(res) => res,
        Err(e) => {
            println!("{e}");
            command_parser.remove_last_saved_line();
            return;
        }
    };
    match result {
        CommandResult::Void => (),
        CommandResult::RecordValueList(columns, records) => {
            print_record_value_list(&columns, records);
        }
        CommandResult::CommandList(items) => execute_command_list(db, command_parser, items),
    }
}

fn execute_command_list<K: DatabaseKey>(
    db: &mut Database<K>,
    command_parser: &mut CommandParser,
    items: Vec<String>,
) {
    for item in items {
        let parse_result = command_parser.parse_command(db, &item);

        let command = match parse_result {
            Err(e) => {
                println!("{e}");
                break;
            }
            Ok(command) => command,
        };

        let result = match command.execute() {
            Ok(res) => res,
            Err(e) => {
                println!("{e}");
                command_parser.remove_last_saved_line();
                break;
            }
        };

        if let CommandResult::RecordValueList(columns, records) = result {
            print_record_value_list(&columns, records);
        }
    }
}

fn print_record_value_list(columns: &[String], records: Vec<Vec<record::Value>>) {
    let widths = get_column_widths(columns, &records);

    for (i, col) in columns.iter().enumerate() {
        print!("{:width$}  ", col, width = widths[i]);
    }

    println!();

    for w in &widths {
        print!("{}  ", "-".repeat(*w));
    }

    println!();

    for record in records {
        for (i, val) in record.iter().enumerate() {
            let val_str = match val {
                record::Value::BOOL(b) => b.to_string(),
                record::Value::STRING(s) => s.clone(),
                record::Value::INT(i) => i.to_string(),
                record::Value::FLOAT(f) => f.to_string(),
            };

            print!("{:width$}  ", val_str, width = widths[i]);
        }

        println!();
    }
}

fn get_column_widths(columns: &[String], records: &Vec<Vec<record::Value>>) -> Vec<usize> {
    let mut widths = columns.iter().map(|c| c.len()).collect::<Vec<usize>>();

    for record in records {
        for (i, val) in record.iter().enumerate() {
            let len = match val {
                record::Value::BOOL(b) => b.to_string().len(),
                record::Value::STRING(s) => s.len(),
                record::Value::INT(i) => i.to_string().len(),
                record::Value::FLOAT(f) => f.to_string().len(),
            };
            widths[i] = widths[i].max(len);
        }
    }

    widths
}
