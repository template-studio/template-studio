"""
数据库迁移脚本
读取 config/config.toml 中的数据库连接信息，按顺序执行 migrations/ 目录下的 SQL 文件。

用法:
    python scripts/migrate.py              # 执行所有未执行的迁移
    python scripts/migrate.py --dry-run    # 仅显示待执行的迁移，不实际执行
    python scripts/migrate.py --only 008 009  # 只执行指定编号的迁移（忽略已执行记录）
    python scripts/migrate.py --force 008  # 强制重新执行指定编号的迁移
"""

import argparse
import os
import re
import sys
import tomllib
from pathlib import Path

import pymysql

# 项目根目录（脚本所在目录的上一级）
ROOT_DIR = Path(__file__).resolve().parent.parent
CONFIG_PATH = ROOT_DIR / "config" / "config.toml"
MIGRATIONS_DIR = ROOT_DIR / "migrations"
MIGRATION_TABLE = "schema_migrations"


def load_db_config() -> dict:
    """从 config.toml 读取数据库连接信息并解析 URL"""
    with open(CONFIG_PATH, "rb") as f:
        config = tomllib.load(f)

    url = config["database"]["url"]
    # mysql://user:password@host:port/database
    pattern = r"mysql://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)"
    m = re.match(pattern, url)
    if not m:
        print(f"无法解析数据库 URL: {url}")
        sys.exit(1)

    return {
        "host": m.group(3),
        "port": int(m.group(4)),
        "user": m.group(1),
        "password": m.group(2),
        "database": m.group(5),
    }


def ensure_migration_table(cursor):
    """确保迁移记录表存在"""
    cursor.execute(f"""
        CREATE TABLE IF NOT EXISTS {MIGRATION_TABLE} (
            version VARCHAR(10) PRIMARY KEY,
            filename VARCHAR(255) NOT NULL,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)


def get_applied_versions(cursor) -> set:
    """获取已执行的迁移版本号"""
    try:
        cursor.execute(f"SELECT version FROM {MIGRATION_TABLE}")
        return {row[0] for row in cursor.fetchall()}
    except pymysql.err.ProgrammingError:
        return set()


def get_pending_migrations(applied: set) -> list:
    """获取待执行的迁移文件列表（按文件名排序）"""
    if not MIGRATIONS_DIR.exists():
        print(f"迁移目录不存在: {MIGRATIONS_DIR}")
        sys.exit(1)

    files = []
    for f in sorted(MIGRATIONS_DIR.glob("*.sql")):
        version = f.name.split("_")[0]
        if version not in applied:
            files.append((version, f))
    return files


def run_migration(cursor, version, filepath):
    """执行单个迁移文件"""
    sql = filepath.read_text(encoding="utf-8")
    # 按分号拆分多条语句，跳过空行
    statements = [s.strip() for s in sql.split(";") if s.strip()]

    for stmt in statements:
        cursor.execute(stmt)

    cursor.execute(
        f"INSERT INTO {MIGRATION_TABLE} (version, filename) VALUES (%s, %s)",
        (version, filepath.name),
    )


def main():
    parser = argparse.ArgumentParser(description="数据库迁移工具")
    parser.add_argument("--dry-run", action="store_true", help="仅显示待执行的迁移")
    parser.add_argument("--force", type=str, help="强制重新执行指定编号的迁移")
    parser.add_argument("--only", nargs="+", help="只执行指定编号的迁移（忽略已执行记录）")
    args = parser.parse_args()

    db = load_db_config()
    print(f"连接数据库: {db['host']}:{db['port']}/{db['database']}")

    conn = pymysql.connect(**db, charset="utf8mb4")
    try:
        cursor = conn.cursor()

        if args.force:
            version = args.force
            files = sorted(MIGRATIONS_DIR.glob(f"{version}_*.sql"))
            if not files:
                print(f"未找到编号 {version} 的迁移文件")
                sys.exit(1)

            filepath = files[0]
            print(f"强制重新执行: {filepath.name}")
            sql = filepath.read_text(encoding="utf-8")
            statements = [s.strip() for s in sql.split(";") if s.strip()]
            for stmt in statements:
                cursor.execute(stmt)
            conn.commit()
            print("完成")
            return

        if args.only:
            ensure_migration_table(cursor)
            print(f"仅执行指定迁移 ({len(args.only)} 个):")
            for version in args.only:
                files = sorted(MIGRATIONS_DIR.glob(f"{version}_*.sql"))
                if not files:
                    print(f"  未找到编号 {version} 的迁移文件，跳过")
                    continue
                filepath = files[0]
                print(f"  执行: {filepath.name} ...", end=" ")
                run_migration(cursor, version, filepath)
                conn.commit()
                print("OK")
            print("\n完成")
            return

        ensure_migration_table(cursor)
        applied = get_applied_versions(cursor)
        pending = get_pending_migrations(applied)

        if not pending:
            print("所有迁移已执行，无需操作")
            return

        print(f"待执行迁移 ({len(pending)} 个):")
        for version, filepath in pending:
            print(f"  {filepath.name}")

        if args.dry_run:
            print("\n(dry-run 模式，未实际执行)")
            return

        print()
        for version, filepath in pending:
            print(f"执行: {filepath.name} ...", end=" ")
            run_migration(cursor, version, filepath)
            conn.commit()
            print("OK")

        print(f"\n完成，共执行 {len(pending)} 个迁移")

    except Exception as e:
        conn.rollback()
        print(f"\n迁移失败: {e}")
        sys.exit(1)
    finally:
        cursor.close()
        conn.close()


if __name__ == "__main__":
    main()
