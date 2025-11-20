use std::io::stdin;

use rustabase::{database::Database, parser::CommandParser};

fn main() {
    let mut db = Database::<i64>::new();

    loop {
        let mut line = String::new();

        match stdin().read_line(&mut line) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }

        {
            let parse_result = CommandParser::parse_command(&mut db, &line);

            match parse_result {
                Err(e) => println!("{}", e),
                Ok(mut command) => {
                    if let Err(e) = command.execute() {
                        println!("{}", e)
                    }
                }
            }
        }

        println!("Present tables: {:?}", db.get_table_names())
    }
}
