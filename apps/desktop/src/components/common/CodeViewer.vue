<template>
  <div ref="el" class="code-viewer"></div>
</template>

<script setup>
// 只读代码查看器:与编辑器预览(TemplatePreview)同款 CodeMirror 配置(Dracula/语法高亮/行号)。
// content/filename 变化即重建文档(按扩展名切语言)。
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { EditorView, highlightActiveLine, highlightActiveLineGutter, lineNumbers } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { dracula } from '@uiw/codemirror-theme-dracula'
import { javascript } from '@codemirror/lang-javascript'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { python } from '@codemirror/lang-python'
import { java } from '@codemirror/lang-java'
import { cpp } from '@codemirror/lang-cpp'
import { rust } from '@codemirror/lang-rust'
import { go } from '@codemirror/lang-go'
import { sql } from '@codemirror/lang-sql'
import { xml } from '@codemirror/lang-xml'
import { yaml } from '@codemirror/lang-yaml'
import { vue } from '@codemirror/lang-vue'

const props = defineProps({
  content: { type: String, default: '' },
  filename: { type: String, default: '' },
})

const languageMap = {
  js: javascript(), javascript: javascript(),
  ts: javascript({ typescript: true }), typescript: javascript({ typescript: true }),
  jsx: javascript({ jsx: true }), tsx: javascript({ typescript: true, jsx: true }),
  vue: vue(), html: html(), htm: html(),
  css: css(), scss: css(), sass: css(), less: css(),
  json: json(), md: markdown(), markdown: markdown(),
  py: python(), python: python(), java: java(),
  cpp: cpp(), cc: cpp(), cxx: cpp(), c: cpp(),
  rs: rust(), rust: rust(), go: go(), sql: sql(),
  xml: xml(), yaml: yaml(), yml: yaml(),
}
const getLanguageExtension = (filename) => languageMap[filename?.split('.').pop()?.toLowerCase()] || null

const el = ref(null)
let view = null

const baseExtensions = () => [
  dracula,
  syntaxHighlighting(defaultHighlightStyle),
  lineNumbers(),
  highlightActiveLine(),
  highlightActiveLineGutter(),
  EditorState.readOnly.of(true),
  EditorView.editable.of(false),
  EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
  EditorView.theme({
    '&': { height: '100%' },
    '.cm-scroller': { overflow: 'auto !important', height: '100% !important' },
    '&.cm-focused': { outline: 'none' },
  }),
]

const rebuild = () => {
  if (!view) return
  const lang = getLanguageExtension(props.filename)
  view.setState(EditorState.create({
    doc: props.content || '',
    extensions: lang ? [...baseExtensions(), lang] : baseExtensions(),
  }))
}

onMounted(() => {
  view = new EditorView({ state: EditorState.create({ doc: props.content || '', extensions: baseExtensions() }), parent: el.value })
})
watch(() => [props.content, props.filename], rebuild)
onBeforeUnmount(() => { view?.destroy(); view = null })
</script>

<style scoped>
.code-viewer { width: 100%; height: 100%; min-height: 0; overflow: hidden; background: #282a36; }
.code-viewer :deep(.cm-editor) { background: transparent; }
</style>
