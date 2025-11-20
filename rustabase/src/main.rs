use std::io::stdin;

use rustabase::{
    commands::command::CommandResult,
    database::{Database, table::record},
    parser::CommandParser,
};

fn main() {
    let mut db = Database::<i64>::new();

    loop {
        let mut line = String::new();

        match stdin().read_line(&mut line) {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }

        {
            let parse_result = CommandParser::parse_command(&mut db, &line);

            let mut command = match parse_result {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                Ok(command) => command,
            };

            if let Err(e) = command.execute() {
                println!("{}", e);
                continue;
            }

            match command.get_result() {
                CommandResult::Void => (),
                CommandResult::RecordValueList(columns, records) => {
                    print_record_value_list(columns, records);
                }
            }
        }

        println!("Present tables: {:?}", db.get_table_names())
    }
}

fn print_record_value_list(columns: Vec<String>, records: Box<Vec<Vec<record::Value>>>) {
    todo!()
}
