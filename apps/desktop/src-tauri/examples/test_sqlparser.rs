use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn main() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(50))";
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    println!("{:#?}", ast);
}
