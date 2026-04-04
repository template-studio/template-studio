<template>
  <div ref="editorContainer" class="sql-editor-container"></div>
</template>

<script setup>
import { ref, onMounted, watch, onBeforeUnmount } from 'vue'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { sql, SQLDialect } from '@codemirror/lang-sql'
import { tags } from '@lezer/highlight'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  readonly: {
    type: Boolean,
    default: false
  },
  theme: {
    type: String,
    default: 'light' // 'light' or 'dark'
  }
})

const emit = defineEmits(['update:modelValue'])

const editorContainer = ref(null)
let editorView = null

// 自定义高亮主题
const customHighlighting = EditorView.theme({
  '&': {
    backgroundColor: props.theme === 'dark' ? '#1e1e1e' : '#ffffff',
    color: props.theme === 'dark' ? '#d4d4d4' : '#24292e'
  },
  '.cm-gutters': {
    backgroundColor: props.theme === 'dark' ? '#252526' : '#f6f8fa',
    color: props.theme === 'dark' ? '#858585' : '#6e7781',
    border: 'none'
  },
  '.cm-activeLineGutter': {
    backgroundColor: props.theme === 'dark' ? '#2a2d2e' : '#f0f0f0',
    color: props.theme === 'dark' ? '#c6c6c6' : '#24292e'
  },
  '.cm-line': {
    padding: '0 0',
    minHeight: '22px'
  },
  '.cm-content': {
    padding: '8px 0'
  },
  '.cm-focused': {
    outline: 'none'
  },
  '.cm-selectionBackground': {
    background: props.theme === 'dark' ? '#264f78' : '#b4d5fe'
  },
  // 语法高亮颜色（优化后）
  '&.cm-editor .cm-scroller': {
    fontFamily: "'Fira Code', 'Consolas', 'Monaco', 'Courier New', monospace",
    fontSize: '13px'
  }
}, {
  dark: props.theme === 'dark'
})

// 自定义语法高亮样式
const sqlHighlight = sql({
  dialect: SQLDialect.define({
    keywords: 'SELECT INSERT UPDATE DELETE CREATE TABLE ALTER DROP INDEX PRIMARY KEY FOREIGN KEY UNIQUE NOT NULL DEFAULT CHECK CONSTRAINT REFERENCES CASCADE RESTRICT ASC DESC ORDER BY GROUP BY HAVING WHERE FROM JOIN INNER LEFT RIGHT OUTER FULL CROSS UNION INTERSECT EXCEPT AS ON DISTINCT EXISTS IN BETWEEN LIKE IS NULL TRUE FALSE AND OR NOT',
    types: 'INT INTEGER VARCHAR CHAR TEXT DATE TIME DATETIME TIMESTAMP DECIMAL NUMERIC FLOAT DOUBLE BOOLEAN BLOB JSON',
    operators: '= != < > <= >= <> LIKE IN IS AND OR NOT BETWEEN EXISTS',
    builtin: 'COUNT SUM AVG MIN MAX CONCAT COALESCE NULLIF CAST CONVERT UPPER LOWER TRIM SUBSTRING LENGTH NOW CURRENT_DATE CURRENT_TIME CURRENT_TIMESTAMP',
    schemaTables: () => [],
    schema: () => []
  }),
  highlightStyle: [
    { tag: tags.keyword, color: props.theme === 'dark' ? '#569cd6' : '#cf222e', fontWeight: '600' },
    { tag: tags.string, color: props.theme === 'dark' ? '#ce9178' : '#0a3069' },
    { tag: tags.number, color: props.theme === 'dark' ? '#b5cea8' : '#0550ae' },
    { tag: tags.comment, color: props.theme === 'dark' ? '#6a9955' : '#6e7781', fontStyle: 'italic' },
    { tag: tags.variableName, color: props.theme === 'dark' ? '#9cdcfe' : '#953800' },
    { tag: tags.typeName, color: props.theme === 'dark' ? '#4ec9b0' : '#953800' },
    { tag: tags.function(tags.variableName), color: props.theme === 'dark' ? '#dcdcaa' : '#8250df' },
    { tag: tags.operator, color: props.theme === 'dark' ? '#d4d4d4' : '#24292e' },
    { tag: tags.punctuation, color: props.theme === 'dark' ? '#d4d4d4' : '#24292e' },
    { tag: tags.propertyName, color: props.theme === 'dark' ? '#9cdcfe' : '#0550ae' }
  ]
})

onMounted(() => {
  const extensions = [
    basicSetup,
    customHighlighting,
    sqlHighlight
  ]

  editorView = new EditorView({
    doc: props.modelValue,
    extensions: extensions,
    parent: editorContainer.value
  })
})

watch(() => props.modelValue, (newValue) => {
  if (editorView && newValue !== editorView.state.doc.toString()) {
    const transaction = editorView.state.update({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: newValue || ''
      }
    })
    editorView.dispatch(transaction)
  }
})

onBeforeUnmount(() => {
  if (editorView) {
    editorView.destroy()
  }
})
</script>

<style scoped>
.sql-editor-container {
  width: 100%;
  height: 100%;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.sql-editor-container :deep(.cm-editor) {
  height: 100%;
}

.sql-editor-container :deep(.cm-scroller) {
  overflow: auto;
}

.sql-editor-container :deep(.cm-content) {
  padding: 8px 12px;
}
</style>
