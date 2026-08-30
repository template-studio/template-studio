/// 将本地表同步到远程数据库（在 Rust 端生成 DDL 并执行）
#[derive(serde::Deserialize)]
pub struct PushColumnDef {
    pub name: String,
    pub data_type: String,
    pub length: Option<i64>,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

pub fn generate_create_table_ddl(
    table_name: &str,
    table_engine: Option<&str>,
    table_comment: Option<&str>,
    columns: &[PushColumnDef],
) -> String {
    let name = table_name.trim_matches('`');
    let mut col_defs: Vec<String> = Vec::new();

    for col in columns {
        let col_name = col.name.trim_matches('`');
        let mut def = format!("`{}` {}", col_name, col.data_type);
        if let Some(len) = col.length {
            if len > 0 {
                def += &format!("({})", len);
            }
        }
        if !col.is_nullable {
            def += " NOT NULL";
        }
        if let Some(ref dv) = col.default_value {
            if !dv.is_empty() {
                def += &format!(" DEFAULT {}", dv);
            }
        }
        if let Some(ref c) = col.comment {
            if !c.is_empty() {
                let escaped = c.replace('\'', "''");
                def += &format!(" COMMENT '{}'", escaped);
            }
        }
        col_defs.push(def);
    }

    let pks: Vec<String> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| format!("`{}`", c.name.trim_matches('`')))
        .collect();
    if !pks.is_empty() {
        col_defs.push(format!("PRIMARY KEY ({})", pks.join(", ")));
    }

    let mut sql = format!("CREATE TABLE `{}` (\n  {}\n)", name, col_defs.join(",\n  "));
    if let Some(engine) = table_engine {
        if !engine.is_empty() {
            sql += &format!(" ENGINE={}", engine);
        }
    }
    if let Some(comment) = table_comment {
        if !comment.is_empty() {
            let escaped = comment.replace('\'', "''");
            sql += &format!(" COMMENT='{}'", escaped);
        }
    }
    sql += ";";
    sql
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_col(
        name: &str,
        data_type: &str,
        length: Option<i64>,
        nullable: bool,
        pk: bool,
    ) -> PushColumnDef {
        PushColumnDef {
            name: name.to_string(),
            data_type: data_type.to_string(),
            length,
            is_nullable: nullable,
            is_primary_key: pk,
            default_value: None,
            comment: None,
        }
    }

    #[test]
    fn test_basic_table() {
        let cols = vec![
            make_col("id", "bigint", None, false, true),
            make_col("name", "varchar", Some(50), false, false),
        ];
        let ddl = generate_create_table_ddl("user", None, None, &cols);
        println!("DDL:\n{}", ddl);
        assert!(ddl.contains("`id` bigint NOT NULL"));
        assert!(ddl.contains("`name` varchar(50) NOT NULL"));
        assert!(ddl.contains("PRIMARY KEY (`id`)"));
        assert!(ddl.starts_with("CREATE TABLE `user`"));
    }

    #[test]
    fn test_nullable_and_length_none() {
        let cols = vec![
            make_col("id", "bigint", None, false, true),
            make_col("gender", "tinyint", None, true, false),
            make_col("is_active", "tinyint", None, true, false),
            make_col("score", "decimal", Some(10), true, false),
        ];
        let ddl = generate_create_table_ddl("test_table", Some("InnoDB"), Some("测试表"), &cols);
        println!("DDL:\n{}", ddl);
        assert!(
            ddl.contains("`gender` tinyint"),
            "expected tinyint without length, got: {}",
            ddl
        );
        assert!(
            !ddl.contains("tinyint("),
            "should not have tinyint(length), got: {}",
            ddl
        );
        assert!(ddl.contains("`score` decimal(10)"));
        assert!(ddl.contains("ENGINE=InnoDB"));
        assert!(ddl.contains("COMMENT='测试表'"));
    }

    #[test]
    fn test_backtick_stripping() {
        let cols = vec![PushColumnDef {
            name: "`id`".to_string(),
            data_type: "bigint".to_string(),
            length: None,
            is_nullable: false,
            is_primary_key: true,
            default_value: None,
            comment: None,
        }];
        let ddl = generate_create_table_ddl("`user`", None, None, &cols);
        println!("DDL:\n{}", ddl);
        assert!(ddl.contains("CREATE TABLE `user`"));
        assert!(!ddl.contains("``"));
        assert!(ddl.contains("`id` bigint NOT NULL"));
    }

    #[test]
    fn test_length_some_1() {
        let cols = vec![
            make_col("id", "bigint", None, false, true),
            make_col("status", "tinyint", Some(1), false, false),
        ];
        let ddl = generate_create_table_ddl("item", None, None, &cols);
        println!("DDL:\n{}", ddl);
        assert!(ddl.contains("`status` tinyint(1) NOT NULL"));
    }

    #[test]
    fn test_default_value_and_comment() {
        let cols = vec![
            make_col("id", "bigint", None, false, true),
            PushColumnDef {
                name: "status".to_string(),
                data_type: "int".to_string(),
                length: None,
                is_nullable: true,
                is_primary_key: false,
                default_value: Some("0".to_string()),
                comment: Some("状态".to_string()),
            },
        ];
        let ddl = generate_create_table_ddl("task", None, None, &cols);
        println!("DDL:\n{}", ddl);
        assert!(ddl.contains("DEFAULT 0"));
        assert!(ddl.contains("COMMENT '状态'"));
    }
}
