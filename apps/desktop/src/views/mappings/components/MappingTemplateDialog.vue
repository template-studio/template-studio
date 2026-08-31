<template>
  <a-modal
    :open="open"
    title="映射模板"
    width="600px"
    :footer="null"
    @update:open="(val) => !val && emit('update:open', false)"
  >
    <div class="template-list">
      <div
        v-for="template in mappingTemplates"
        :key="template.id"
        class="template-card"
        @click="emit('apply', template)"
      >
        <div class="template-header">
          <h4 class="template-name">{{ template.name }}</h4>
          <a-tag :color="template.dbType === 'mysql' ? 'blue' : template.dbType === 'postgresql' ? 'green' : 'orange'">
            {{ template.dbType.toUpperCase() }}
          </a-tag>
        </div>
        <p class="template-desc">{{ template.description }}</p>
        <div class="template-preview">
          <span v-for="(item, idx) in template.mappings.slice(0, 4)" :key="idx" class="preview-item">
            <code>{{ item.pattern }}</code> → <code>{{ item.targetType }}</code>
          </span>
          <span v-if="template.mappings.length > 4" class="preview-more">+{{ template.mappings.length - 4 }} 更多...</span>
        </div>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
defineProps({ open: { type: Boolean, default: false } })
const emit = defineEmits(['update:open', 'apply'])

const mappingTemplates = [
  {
    id: 'mysql-java-mybatis',
    name: 'MySQL → Java (MyBatis)',
    description: '适用于 MyBatis/MyBatis-Plus 的常用 MySQL 到 Java 类型映射',
    dbType: 'mysql', langType: 'java',
    mappings: [
      { pattern: 'VARCHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'CHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'TEXT', targetType: 'String', priority: 10 },
      { pattern: 'LONGTEXT', targetType: 'String', priority: 10 },
      { pattern: 'INT', targetType: 'Integer', priority: 10 },
      { pattern: 'BIGINT', targetType: 'Long', priority: 10 },
      { pattern: 'SMALLINT', targetType: 'Integer', priority: 10 },
      { pattern: 'TINYINT(1)', targetType: 'Boolean', priority: 20 },
      { pattern: 'TINYINT(%)', targetType: 'Integer', priority: 10 },
      { pattern: 'DECIMAL(%,%)', targetType: 'BigDecimal', priority: 10 },
      { pattern: 'FLOAT', targetType: 'Float', priority: 10 },
      { pattern: 'DOUBLE', targetType: 'Double', priority: 10 },
      { pattern: 'BOOLEAN', targetType: 'Boolean', priority: 10 },
      { pattern: 'DATE', targetType: 'LocalDate', priority: 10 },
      { pattern: 'TIMESTAMP', targetType: 'LocalDateTime', priority: 10 },
      { pattern: 'DATETIME', targetType: 'LocalDateTime', priority: 10 },
      { pattern: 'TIME', targetType: 'LocalTime', priority: 10 },
      { pattern: 'BLOB', targetType: 'byte[]', priority: 10 },
      { pattern: 'JSON', targetType: 'String', priority: 10 }
    ]
  },
  {
    id: 'mysql-java-jpa',
    name: 'MySQL → Java (JPA)',
    description: '适用于 JPA/Hibernate 的 MySQL 到 Java 类型映射',
    dbType: 'mysql', langType: 'java',
    mappings: [
      { pattern: 'VARCHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'CHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'TEXT', targetType: 'String', priority: 10 },
      { pattern: 'LONGTEXT', targetType: 'String', priority: 10 },
      { pattern: 'INT', targetType: 'Integer', priority: 10 },
      { pattern: 'BIGINT', targetType: 'Long', priority: 10 },
      { pattern: 'SMALLINT', targetType: 'Short', priority: 10 },
      { pattern: 'TINYINT(1)', targetType: 'Boolean', priority: 20 },
      { pattern: 'TINYINT(%)', targetType: 'Byte', priority: 10 },
      { pattern: 'DECIMAL(%,%)', targetType: 'BigDecimal', priority: 10 },
      { pattern: 'FLOAT', targetType: 'Float', priority: 10 },
      { pattern: 'DOUBLE', targetType: 'Double', priority: 10 },
      { pattern: 'BOOLEAN', targetType: 'Boolean', priority: 10 },
      { pattern: 'DATE', targetType: 'LocalDate', priority: 10 },
      { pattern: 'TIMESTAMP', targetType: 'Instant', priority: 10 },
      { pattern: 'DATETIME', targetType: 'LocalDateTime', priority: 10 },
      { pattern: 'TIME', targetType: 'LocalTime', priority: 10 },
      { pattern: 'BLOB', targetType: 'byte[]', priority: 10 },
      { pattern: 'JSON', targetType: 'String', priority: 10 }
    ]
  },
  {
    id: 'postgresql-java-mybatis',
    name: 'PostgreSQL → Java (MyBatis)',
    description: '适用于 MyBatis 的 PostgreSQL 到 Java 类型映射',
    dbType: 'postgresql', langType: 'java',
    mappings: [
      { pattern: 'VARCHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'CHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'TEXT', targetType: 'String', priority: 10 },
      { pattern: 'INTEGER', targetType: 'Integer', priority: 10 },
      { pattern: 'BIGINT', targetType: 'Long', priority: 10 },
      { pattern: 'SMALLINT', targetType: 'Integer', priority: 10 },
      { pattern: 'BOOLEAN', targetType: 'Boolean', priority: 10 },
      { pattern: 'DECIMAL(%,%)', targetType: 'BigDecimal', priority: 10 },
      { pattern: 'NUMERIC(%,%)', targetType: 'BigDecimal', priority: 10 },
      { pattern: 'REAL', targetType: 'Float', priority: 10 },
      { pattern: 'DOUBLE PRECISION', targetType: 'Double', priority: 10 },
      { pattern: 'DATE', targetType: 'LocalDate', priority: 10 },
      { pattern: 'TIMESTAMP', targetType: 'LocalDateTime', priority: 10 },
      { pattern: 'TIMESTAMPTZ', targetType: 'OffsetDateTime', priority: 10 },
      { pattern: 'TIME', targetType: 'LocalTime', priority: 10 },
      { pattern: 'BYTEA', targetType: 'byte[]', priority: 10 },
      { pattern: 'JSON', targetType: 'String', priority: 10 },
      { pattern: 'JSONB', targetType: 'String', priority: 10 },
      { pattern: 'UUID', targetType: 'String', priority: 10 },
      { pattern: 'SERIAL', targetType: 'Integer', priority: 10 },
      { pattern: 'BIGSERIAL', targetType: 'Long', priority: 10 }
    ]
  },
  {
    id: 'mysql-python-sqlalchemy',
    name: 'MySQL → Python (SQLAlchemy)',
    description: '适用于 SQLAlchemy 的 MySQL 到 Python 类型映射',
    dbType: 'mysql', langType: 'python',
    mappings: [
      { pattern: 'VARCHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'CHAR(%)', targetType: 'String', priority: 10 },
      { pattern: 'TEXT', targetType: 'Text', priority: 10 },
      { pattern: 'LONGTEXT', targetType: 'Text', priority: 10 },
      { pattern: 'INT', targetType: 'Integer', priority: 10 },
      { pattern: 'BIGINT', targetType: 'BigInteger', priority: 10 },
      { pattern: 'SMALLINT', targetType: 'SmallInteger', priority: 10 },
      { pattern: 'TINYINT(1)', targetType: 'Boolean', priority: 20 },
      { pattern: 'TINYINT(%)', targetType: 'SmallInteger', priority: 10 },
      { pattern: 'DECIMAL(%,%)', targetType: 'Numeric', priority: 10 },
      { pattern: 'FLOAT', targetType: 'Float', priority: 10 },
      { pattern: 'DOUBLE', targetType: 'Float', priority: 10 },
      { pattern: 'BOOLEAN', targetType: 'Boolean', priority: 10 },
      { pattern: 'DATE', targetType: 'Date', priority: 10 },
      { pattern: 'TIMESTAMP', targetType: 'DateTime', priority: 10 },
      { pattern: 'DATETIME', targetType: 'DateTime', priority: 10 },
      { pattern: 'TIME', targetType: 'Time', priority: 10 },
      { pattern: 'BLOB', targetType: 'LargeBinary', priority: 10 },
      { pattern: 'JSON', targetType: 'JSON', priority: 10 }
    ]
  },
  {
    id: 'mysql-typescript-prisma',
    name: 'MySQL → TypeScript (Prisma)',
    description: '适用于 Prisma 的 MySQL 到 TypeScript 类型映射',
    dbType: 'mysql', langType: 'typescript',
    mappings: [
      { pattern: 'VARCHAR(%)', targetType: 'string', priority: 10 },
      { pattern: 'CHAR(%)', targetType: 'string', priority: 10 },
      { pattern: 'TEXT', targetType: 'string', priority: 10 },
      { pattern: 'LONGTEXT', targetType: 'string', priority: 10 },
      { pattern: 'INT', targetType: 'number', priority: 10 },
      { pattern: 'BIGINT', targetType: 'bigint', priority: 10 },
      { pattern: 'SMALLINT', targetType: 'number', priority: 10 },
      { pattern: 'TINYINT(1)', targetType: 'boolean', priority: 20 },
      { pattern: 'TINYINT(%)', targetType: 'number', priority: 10 },
      { pattern: 'DECIMAL(%,%)', targetType: 'Decimal', priority: 10 },
      { pattern: 'FLOAT', targetType: 'number', priority: 10 },
      { pattern: 'DOUBLE', targetType: 'number', priority: 10 },
      { pattern: 'BOOLEAN', targetType: 'boolean', priority: 10 },
      { pattern: 'DATE', targetType: 'Date', priority: 10 },
      { pattern: 'TIMESTAMP', targetType: 'Date', priority: 10 },
      { pattern: 'DATETIME', targetType: 'Date', priority: 10 },
      { pattern: 'TIME', targetType: 'string', priority: 10 },
      { pattern: 'BLOB', targetType: 'Buffer', priority: 10 },
      { pattern: 'JSON', targetType: 'JsonValue', priority: 10 }
    ]
  }
]
</script>

<style scoped>
.template-list { display: flex; flex-direction: column; gap: 12px; max-height: 60vh; overflow-y: auto; }
.template-card { border: 1px solid var(--color-border); border-radius: var(--border-radius-md); padding: 16px; cursor: pointer; transition: all 0.2s; }
.template-card:hover { border-color: var(--color-border-strong); background: var(--color-hover); }
.template-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
.template-name { margin: 0; font-size: 15px; font-weight: 600; color: var(--color-text); }
.template-desc { margin: 0 0 12px; font-size: 13px; color: var(--color-text-secondary); }
.template-preview { display: flex; flex-wrap: wrap; gap: 8px; }
.preview-item { font-size: 12px; color: var(--color-text-secondary); }
.preview-item code { font-family: 'Fira Code', 'Consolas', monospace; font-size: 11px; padding: 1px 4px; background: var(--color-bg-secondary); border-radius: 3px; color: var(--color-primary); }
.preview-more { font-size: 12px; color: var(--color-text-muted); }
</style>
