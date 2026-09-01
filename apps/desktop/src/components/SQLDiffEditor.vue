<template>
  <div class="sql-diff-container">
    <div class="diff-panels">
      <!-- 原始 SQL -->
      <div class="diff-panel">
        <div class="panel-header">
          <span>原始版本</span>
          <span class="stats" v-if="stats.removed > 0">-{{ stats.removed }} 行</span>
        </div>
        <div class="panel-content">
          <div ref="originalEditor" class="editor-wrapper"></div>
        </div>
      </div>

      <!-- 修改后的 SQL -->
      <div class="diff-panel">
        <div class="panel-header">
          <span>修改后</span>
          <span class="stats" v-if="stats.added > 0">+{{ stats.added }} 行</span>
        </div>
        <div class="panel-content">
          <div ref="modifiedEditor" class="editor-wrapper"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { ViewPlugin, Decoration } from '@codemirror/view'
import { sql, SQLDialect } from '@codemirror/lang-sql'
import { tags } from '@lezer/highlight'
import { diffLines } from 'diff'

const props = defineProps({
  original: {
    type: String,
    default: ''
  },
  modified: {
    type: String,
    default: ''
  },
  theme: {
    type: String,
    default: 'light'
  }
})

const originalEditor = ref(null)
const modifiedEditor = ref(null)
let originalView = null
let modifiedView = null

// 计算差异统计
const stats = computed(() => {
  const diff = diffLines(props.original, props.modified)

  let added = 0
  let removed = 0

  diff.forEach(part => {
    if (part.added) added += part.count
    if (part.removed) removed += part.count
  })

  return { added, removed }
})

// 创建行装饰器
function createLineDecorations(decorations) {
  return ViewPlugin.fromClass(class {
    decorations = decorations
  }, {
    decorations: () => decorations
  })
}

// 自定义主题
const customTheme = EditorView.theme({
  '&': {
    backgroundColor: props.theme === 'dark' ? '#1e1e1e' : '#ffffff',
    color: props.theme === 'dark' ? '#d4d4d4' : '#24292e',
    fontSize: '13px',
    fontFamily: 'Fira Code, Consolas, Monaco, Courier New, monospace'
  },
  '.cm-gutters': {
    backgroundColor: props.theme === 'dark' ? '#252526' : '#f6f8fa',
    color: props.theme === 'dark' ? '#858585' : '#6e7781',
    border: 'none'
  },
  '.cm-line': {
    padding: '0 12px',
    minHeight: '22px'
  },
  '.cm-content': {
    padding: '0'
  },
  '.cm-focused': {
    outline: 'none'
  },
  // 差异高亮样式
  '.line-deleted': {
    backgroundColor: 'rgba(248, 81, 73, 0.2)'
  },
  '.line-added': {
    backgroundColor: 'rgba(46, 160, 67, 0.2)'
  },
  '.line-modified': {
    backgroundColor: 'rgba(255, 200, 0, 0.15)'
  }
}, {
  dark: props.theme === 'dark'
})

// 语法高亮
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

// 计算差异并创建装饰器
function computeDiffDecorations(isOriginal) {
  const diff = diffLines(
    isOriginal ? props.original : props.modified,
    isOriginal ? props.modified : props.original
  )

  const decorations = []
  let lineNum = 0

  diff.forEach(part => {
    const lines = part.value.split('\n')
    if (lines[lines.length - 1] === '') {
      lines.pop()
    }

    lines.forEach((line, i) => {
      if (part.removed && isOriginal) {
        decorations.push(
          Decoration.line({ class: 'line-deleted' }).range(lineNum)
        )
      } else if (part.added && !isOriginal) {
        decorations.push(
          Decoration.line({ class: 'line-added' }).range(lineNum)
        )
      }

      if (i < lines.length - 1 || line.length > 0) {
        lineNum++
      }
    })
  })

  return Decoration.set(decorations)
}

onMounted(() => {
  const extensions = [
    basicSetup,
    customTheme,
    sqlHighlight,
    EditorState.readOnly.of(true),
    EditorView.lineWrapping
  ]

  // 创建原始版本编辑器
  originalView = new EditorView({
    doc: props.original,
    state: EditorState.create({
      doc: props.original,
      extensions: [
        ...extensions,
        createLineDecorations(
          computeDiffDecorations(true)
        )
      ]
    }),
    parent: originalEditor.value
  })

  // 创建修改版本编辑器
  modifiedView = new EditorView({
    doc: props.modified,
    state: EditorState.create({
      doc: props.modified,
      extensions: [
        ...extensions,
        createLineDecorations(
          computeDiffDecorations(false)
        )
      ]
    }),
    parent: modifiedEditor.value
  })
})

onBeforeUnmount(() => {
  if (originalView) {
    originalView.destroy()
  }
  if (modifiedView) {
    modifiedView.destroy()
  }
})
</script>

<style scoped>
.sql-diff-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.diff-panels {
  display: flex;
  gap: 1px;
  flex: 1;
  background: var(--color-border);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  overflow: hidden;
}

.diff-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-container);
  min-width: 0;
}

.panel-header {
  padding: 8px 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stats {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
}

.panel-header .stats:first-child:nth-last-child(n+2) {
  background: rgba(248, 81, 73, 0.1);
  color: #f85149;
}

.panel-header .stats:last-child:nth-last-child(n+2) {
  background: rgba(46, 160, 67, 0.1);
  color: #2ea043;
}

.panel-content {
  flex: 1;
  overflow: hidden;
}

.editor-wrapper {
  width: 100%;
  height: 100%;
}

.editor-wrapper :deep(.cm-editor) {
  height: 100%;
}

.editor-wrapper :deep(.cm-scroller) {
  overflow: auto;
}
</style>
