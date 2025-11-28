#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::must_use_unit)]

pub mod commands;
pub mod database;
pub mod parser;

#[cfg(test)]
mod integration_tests {
    use crate::commands::command::{Command, CommandResult};
    use crate::database::Database;
    use crate::database::table::record::Value;
    use crate::parser::CommandParser;

    #[test]
    fn full_pipeline_create_insert_select_i64() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let create_cmd = "CREATE Users KEY UserId FIELDS Name:STRING, Age:INT";
        let result = parser
            .parse_command(&mut db, create_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let insert_cmd = "INSERT UserId=1, Name=\"Alice\", Age=30 INTO Users";
        let result = parser
            .parse_command(&mut db, insert_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let select_cmd = "SELECT Name, Age FROM Users";
        let result = parser
            .parse_command(&mut db, select_cmd)
            .unwrap()
            .execute()
            .unwrap();
        if let CommandResult::RecordValueList(columns, records) = result {
            assert_eq!(columns, vec!["Name", "Age"]);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0],
                vec![Value::STRING("Alice".into()), Value::INT(30)]
            );
        } else {
            panic!("Expected RecordValueList");
        }
    }

    #[test]
    fn full_pipeline_create_insert_select_string() {
        let mut parser = CommandParser::new();
        let mut db = Database::<String>::new();

        let create_cmd = "CREATE Users KEY UserId FIELDS Name:STRING, Age:INT";
        let result = parser
            .parse_command(&mut db, create_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let insert_cmd = "INSERT UserId=\"user-1\", Name=\"Bob\", Age=25 INTO Users";
        let result = parser
            .parse_command(&mut db, insert_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let select_cmd = "SELECT Name, Age FROM Users";
        let result = parser
            .parse_command(&mut db, select_cmd)
            .unwrap()
            .execute()
            .unwrap();
        if let CommandResult::RecordValueList(columns, records) = result {
            assert_eq!(columns, vec!["Name", "Age"]);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0],
                vec![Value::STRING("Bob".into()), Value::INT(25)]
            );
        } else {
            panic!("Expected RecordValueList");
        }
    }

    #[test]
    fn full_pipeline_select_with_where_i64() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let create_cmd = "CREATE Users KEY UserId FIELDS Name:STRING, Age:INT";
        parser
            .parse_command(&mut db, create_cmd)
            .unwrap()
            .execute()
            .unwrap();

        let insert1 = "INSERT UserId=1, Name=\"Alice\", Age=30 INTO Users";
        parser
            .parse_command(&mut db, insert1)
            .unwrap()
            .execute()
            .unwrap();
        let insert2 = "INSERT UserId=2, Name=\"Bob\", Age=20 INTO Users";
        parser
            .parse_command(&mut db, insert2)
            .unwrap()
            .execute()
            .unwrap();

        let select_cmd = "SELECT Name FROM Users WHERE Age > 25";
        let result = parser
            .parse_command(&mut db, select_cmd)
            .unwrap()
            .execute()
            .unwrap();
        if let CommandResult::RecordValueList(columns, records) = result {
            assert_eq!(columns, vec!["Name"]);
            assert_eq!(records.len(), 1);
            assert_eq!(records[0], vec![Value::STRING("Alice".into())]);
        } else {
            panic!("Expected RecordValueList");
        }
    }

    #[test]
    fn full_pipeline_delete_i64() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let create_cmd = "CREATE Users KEY UserId FIELDS Name:STRING";
        parser
            .parse_command(&mut db, create_cmd)
            .unwrap()
            .execute()
            .unwrap();

        let insert_cmd = "INSERT UserId=1, Name=\"Alice\" INTO Users";
        parser
            .parse_command(&mut db, insert_cmd)
            .unwrap()
            .execute()
            .unwrap();

        let delete_cmd = "DELETE 1 FROM Users";
        let result = parser
            .parse_command(&mut db, delete_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let select_cmd = "SELECT Name FROM Users";
        let result = parser
            .parse_command(&mut db, select_cmd)
            .unwrap()
            .execute()
            .unwrap();
        if let CommandResult::RecordValueList(_, records) = result {
            assert_eq!(records.len(), 0);
        } else {
            panic!("Expected RecordValueList");
        }
    }

    #[test]
    fn full_pipeline_save_as_read_from_success() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let create_cmd = "CREATE Users KEY UserId FIELDS Name:STRING";
        parser
            .parse_command(&mut db, create_cmd)
            .unwrap()
            .execute()
            .unwrap();
        let insert_cmd = "INSERT UserId=1, Name=\"Alice\" INTO Users";
        parser
            .parse_command(&mut db, insert_cmd)
            .unwrap()
            .execute()
            .unwrap();

        let save_cmd = "SAVE_AS test_commands.txt";
        let result = parser
            .parse_command(&mut db, save_cmd)
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(result, CommandResult::Void));

        let read_cmd = "READ_FROM test_commands.txt";
        let result = parser
            .parse_command(&mut db, read_cmd)
            .unwrap()
            .execute()
            .unwrap();
        if let CommandResult::CommandList(commands) = result {
            println!("{:?}", commands);
            assert!(commands.contains(&"CREATE Users KEY UserId FIELDS Name:STRING".to_string()));
            assert!(commands.contains(&"INSERT UserId=1, Name=\"Alice\" INTO Users".to_string()));
        } else {
            panic!("Expected CommandList");
        }

        std::fs::remove_file("test_commands.txt").unwrap();
    }

    #[test]
    fn full_pipeline_save_as_error() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let save_cmd = "SAVE_AS /invalid/path/commands.txt";
        let result = parser.parse_command(&mut db, save_cmd).unwrap().execute();
        assert!(result.is_err());
    }

    #[test]
    fn full_pipeline_read_from_error() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let read_cmd = "READ_FROM /nonexistent/file.txt";
        let result = parser.parse_command(&mut db, read_cmd).unwrap().execute();
        assert!(result.is_err());
    }

    #[test]
    fn full_pipeline_invalid_command_error() {
        let mut parser = CommandParser::new();
        let mut db = Database::<i64>::new();

        let invalid_cmd = "INVALID COMMAND";
        let result = parser.parse_command(&mut db, invalid_cmd);
        assert!(result.is_err());
    }
}
