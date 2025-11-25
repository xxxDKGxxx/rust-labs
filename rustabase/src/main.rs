#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
use std::io::stdin;

use rustabase::{
    commands::command::{Command, CommandResult},
    database::{Database, table::record},
    parser::CommandParser,
};

fn main() {
    let mut db = Database::<i64>::new();
    let mut command_parser = CommandParser::new();

    loop {
        handle_user_input(&mut db, &mut command_parser);
    }
}

fn handle_user_input(db: &mut Database<i64>, command_parser: &mut CommandParser) {
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

    println!("Present tables: {:?}", db.get_table_names());
}

fn execute_command_list(
    db: &mut Database<i64>,
    command_parser: &mut CommandParser,
    items: Vec<String>,
) {
    for item in items {
        let parse_result = command_parser.parse_command(db, &item);

        let command = match parse_result {
            Err(e) => {
                println!("{e}");
                continue;
            }
            Ok(command) => command,
        };

        let result = match command.execute() {
            Ok(res) => res,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };

        match result {
            CommandResult::RecordValueList(columns, records) => {
                print_record_value_list(&columns, records);
            }
            _ => (),
        }
    }
}

fn print_record_value_list(columns: &[String], records: Vec<Vec<record::Value>>) {
    println!(
        "{}",
        columns.iter().fold(String::new(), |mut acc, col| {
            acc.push_str(&format!("{col}\t"));
            acc
        })
    );

    for record in records {
        println!(
            "{}",
            record.iter().fold(String::new(), |mut acc, val| {
                let val_str = match val {
                    record::Value::BOOL(b) => b.to_string(),
                    record::Value::STRING(s) => s.to_string(),
                    record::Value::INT(i) => i.to_string(),
                    record::Value::FLOAT(f) => f.to_string(),
                };
                acc.push_str(&format!("{val_str}\t"));
                acc
            })
        );
    }
}
