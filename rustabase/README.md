# Rustabase

Rustabase is a simple database written in Rust, designed to handle basic SQL operations such as creating tables, inserting data, selection with filtering, deleting records, and command persistence. The project uses the Pest parser for syntactic analysis of commands entered by the user in an interactive console.

## Project Structure

The project is organized into a hierarchy of directories and files, which facilitates navigation and understanding of the architecture. Below is a detailed description of the structure along with the roles of individual files and directories.

### Root Directory (`/`)

- **Cargo.toml**: Cargo configuration file, containing project metadata and a list of external dependencies, such as `clap` (for command-line argument handling), `pest` and `pest_derive` (for grammar parsing), `pest_ascii_tree` (for parser debugging), and `thiserror` (for error handling). It also specifies the Rust edition (2024) and the package name ("rustabase").

- **clippy.toml**: Configuration file for Clippy, a tool for static code analysis to find potential issues and violations of Rust best practices.

- **test.txt**, **test3.txt**, **testqueries.txt**, **text2.txt**: Test files containing sample input data, SQL queries, and test results. They serve to verify the functionality of the parser and commands during development.

### `src/` Directory

Main source directory containing the application code.

- **lib.rs**: Main library file, exporting the `commands`, `database`, and `parser` modules. It also contains integration tests checking full workflows (e.g., CREATE → INSERT → SELECT).

- **main.rs**: Application entry point. Handles command-line arguments (e.g., `--key-type String` or `--key-type I64` to specify the database primary key type). Launches an interactive loop where the user can enter commands, parse them, and execute them on a database instance.

- **parser.rs**: Module responsible for parsing commands. Uses the Pest library for syntactic analysis based on the grammar defined in `grammar.pest`. Parses commands such as CREATE, INSERT, SELECT, DELETE, SAVE_AS, and READ_FROM, converting them into command structures.

- **commands.rs**: Module containing common structures and enumerations for commands, such as `CommandResult` and `CommandError`.

- **database.rs**: Module defining database data structures, including `Database`, `Table`, and `Record`. Supports different key types (i64 or String).

- **grammar.pest**: Grammar file for the Pest parser, defining parsing rules for SQL-like commands, including WHERE expressions with operators and column comparisons.

#### `commands/` Subdirectory

Contains implementations of individual commands.

- **command.rs**: Base for all commands. Defines the `Command` trait with the `execute()` method, the `AnyCommand` enum for polymorphism, and common result and error types.

- **create_command.rs**: Implementation of the `CREATE TABLE` command, allowing definition of tables with primary keys and fields of various types (STRING, INT, FLOAT, BOOL).

- **insert_command.rs**: Implementation of the `INSERT` command, enabling addition of new records to the table with data type validation.

- **select_command.rs**: Implementation of the `SELECT` command, supporting column selection and optional filtering using the `WHERE` clause. Supports complex conditions with logical operators (AND, OR) and comparisons between values or columns.

- **delete_command.rs**: Implementation of the `DELETE` command, allowing deletion of records based on the primary key value.

- **save_as_command.rs**: Implementation of the `SAVE_AS` command, saving the history of executed commands to a text file for persistence purposes.

- **read_from_command.rs**: Implementation of the `READ_FROM` command, loading commands from a file and executing them sequentially.

#### `database/` Subdirectory

Contains data structures representing the database.

- **key.rs**: Definitions of the `DatabaseKey` trait and implementations for key types (i64 and String), enabling abstraction over different primary key types.

- **table.rs**: Implementation of the `Table` structure, managing records, columns, and keys. Supports insertion, filtering, and validation operations.

- **table/record.rs**: Definitions of the `Record` and `Value` structures, representing individual records and values of various types (STRING, INT, FLOAT, BOOL). Contains methods for comparisons and conversions.

## Features

Rustabase supports the following operations:

- **CREATE TABLE**: Creating tables with primary key and field definitions.
- **INSERT**: Inserting data into the table.
- **SELECT**: Selecting data from the table with optional filtering using `WHERE`.
- **DELETE**: Deleting records based on the key.
- **SAVE_AS**: Saving command history to a file.
- **READ_FROM**: Loading and executing commands from a file.

### ColumnOperatorFilter: Column Comparison in WHERE Clause

One of the advanced features not described in standard SQL commands is `ColumnOperatorFilter`, which enables dynamic data filtering by comparing the value of one column with the value of another column in the same table. This is particularly useful in business scenarios where we want to filter records based on relationships between columns, for example, to check if an employee's age is greater than the minimum age required for a position, or if revenues exceed costs.

#### How It Works

`ColumnOperatorFilter` allows expressions like `column1 > column2`, where both sides are column names. For each record in the table, the filter retrieves values from both columns and compares them using the selected comparison operator (>, >=, =, !=, <, <=). If the condition is met, the record is included in the query result.

#### Usage Example

Consider a table `Employees` with columns `Age` (age) and `MinAge` (minimum age for the position):

```sql
CREATE Employees KEY EmployeeId FIELDS Age:INT, MinAge:INT;
INSERT EmployeeId=1, Age=25, MinAge=21 INTO Employees;
INSERT EmployeeId=2, Age=19, MinAge=21 INTO Employees;
SELECT Name FROM Employees WHERE Age > MinAge;
```

In this example, the first `SELECT` query will return only the employee with `EmployeeId=1`, because 25 > 21, while for the second, 19 > 21 is false.

In the actual Rust implementation, `ColumnOperatorFilter` delegates the comparison to `ValueOperatorFilter`, retrieving the value from the second column for each record and performing the comparison with the first column. This ensures consistency with other filters and data type handling.

This feature extends filtering capabilities, enabling more complex data analyses without the need for preliminary processing or adding additional computed columns.

## Favourite module

My favourite module is the commands module. Especially the select command with where filters: and, or and brackets. I really like the tree-like structure that I use to filter records, which is also very simple considering the presence of the filter metod on the struct Table that receives a closure as its' argument. I only regret some commands being generic by the DatabaseKey as I should have used AnyDatabase enum from the beginnig, and I also dislike using the AnyCommand enum for execution in the main.rs file - I would prefer to use dyn, but since the challenge was not to use any smart pointers that is the workaround I have come up with. On the other hand - I cannot stand the parser module. Although using pest was easier, the 30 line function length limit made some functions very modular and hard to read at times.
