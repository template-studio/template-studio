/**
 * Schema 编辑器 Composable
 * 负责 CodeMirror 编辑器的初始化、更新、格式化、导入/导出等功能
 */
import { ref, nextTick } from 'vue'
import { message } from 'ant-design-vue'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { json } from '@codemirror/lang-json'
import { yaml } from '@codemirror/lang-yaml'
import * as YAML from 'js-yaml'

export function useSchemaEditor(props, emit) {

  // ========== 状态 ==========

  /**
   * 编辑器容器引用
   */
  const editorRef = ref(null)

  /**
   * CodeMirror 编辑器实例
   */
  let editor = null

  /**
   * Schema 格式：'json' | 'yaml'
   */
  const schemaFormat = ref('json')

  // ========== 编辑器管理 ==========

  /**
   * 将 Schema 转换为字符串（根据当前格式）
   */
  const formatSchemaToString = () => {
    try {
      const schemaObj = typeof props.schema === 'string' ? JSON.parse(props.schema) : props.schema

      if (schemaFormat.value === 'yaml') {
        return YAML.dump(schemaObj, {
          indent: 2,
          lineWidth: -1
        })
      } else {
        return JSON.stringify(schemaObj, null, 2)
      }
    } catch (error) {
      console.error('Schema格式转换失败:', error)
      return schemaFormat.value === 'yaml' ? '{}' : '{}'
    }
  }

  /**
   * 初始化编辑器
   */
  const initEditor = () => {
    if (!editorRef.value) return

    // 清除现有编辑器
    if (editor) {
      editor.destroy()
      editor = null
    }

    const extensions = [
      basicSetup,
      EditorView.theme({
        '&': {
          fontSize: '13px',
          fontFamily: 'Monaco, Menlo, "Ubuntu Mono", Consolas, monospace'
        },
        '.cm-content': {
          padding: '12px',
          minHeight: '100%'
        },
        '.cm-editor': {
          height: '100%'
        },
        '.cm-focused': {
          outline: 'none'
        }
      }),
      EditorView.lineWrapping
    ]

    // 根据格式添加语言包
    if (schemaFormat.value === 'yaml') {
      extensions.push(yaml())
    } else {
      extensions.push(json())
    }

    const content = formatSchemaToString()

    editor = new EditorView({
      state: EditorState.create({
        doc: content,
        extensions
      }),
      parent: editorRef.value
    })
  }

  /**
   * 更新编辑器内容
   */
  const updateContent = () => {
    if (!editor) return

    const content = formatSchemaToString()

    // 只有内容真正变化时才更新
    if (editor.state.doc.toString() !== content) {
      const transaction = editor.state.update({
        changes: {
          from: 0,
          to: editor.state.doc.length,
          insert: content
        }
      })

      editor.dispatch(transaction)
    }
  }

  // ========== 编辑器操作 ==========

  /**
   * 格式化 JSON/YAML
   */
  const format = () => {
    try {
      if (!editor) return

      const content = editor.state.doc.toString()
      let formatted

      if (schemaFormat.value === 'yaml') {
        // YAML 格式化
        const parsed = YAML.load(content)
        formatted = YAML.dump(parsed, {
          indent: 2,
          lineWidth: -1
        })
      } else {
        // JSON 格式化
        const parsed = JSON.parse(content)
        formatted = JSON.stringify(parsed, null, 2)
      }

      const transaction = editor.state.update({
        changes: {
          from: 0,
          to: editor.state.doc.length,
          insert: formatted
        }
      })

      editor.dispatch(transaction)
      message.success('格式化成功')
    } catch (error) {
      message.error('格式化失败: ' + error.message)
    }
  }

  /**
   * 复制内容到剪贴板
   */
  const copy = async () => {
    try {
      const content = editor ? editor.state.doc.toString() : props.schema
      await navigator.clipboard.writeText(content)
      message.success(`Schema已复制到剪贴板 (${schemaFormat.value.toUpperCase()}格式)`)
    } catch (error) {
      message.error('复制失败')
    }
  }

  /**
   * 导入 JSON/YAML 文件
   */
  const importSchema = () => {
    return new Promise((resolve, reject) => {
      try {
        const input = document.createElement('input')
        input.type = 'file'
        input.accept = '.json,.yaml,.yml'

        input.onchange = async (e) => {
          const file = e.target.files[0]
          if (!file) {
            reject(new Error('未选择文件'))
            return
          }

          try {
            const text = await file.text()
            let schema

            // 根据文件扩展名判断格式
            if (file.name.endsWith('.yaml') || file.name.endsWith('.yml')) {
              schema = YAML.load(text)
              schemaFormat.value = 'yaml'
            } else {
              schema = JSON.parse(text)
              schemaFormat.value = 'json'
            }

            // 触发导入事件，让父组件处理
            emit('import', schema)

            message.success(`Schema导入成功 (${schemaFormat.value.toUpperCase()}格式)`)
            resolve(schema)
          } catch (error) {
            message.error('导入失败: ' + error.message)
            reject(error)
          }
        }

        input.click()
      } catch (error) {
        message.error('导入失败: ' + error.message)
        reject(error)
      }
    })
  }

  /**
   * 导出 JSON/YAML 文件
   */
  const exportSchema = () => {
    try {
      const content = editor ? editor.state.doc.toString() : props.schema

      // 根据格式确定文件扩展名和MIME类型
      const extension = schemaFormat.value === 'yaml' ? 'yaml' : 'json'
      const mimeType = schemaFormat.value === 'yaml' ? 'text/yaml' : 'application/json'

      const blob = new Blob([content], { type: mimeType })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `schema_${props.templateId}_${Date.now()}.${extension}`
      a.click()
      URL.revokeObjectURL(url)

      message.success(`Schema导出成功 (${schemaFormat.value.toUpperCase()}格式)`)
    } catch (error) {
      message.error('导出失败: ' + error.message)
    }
  }

  /**
   * 同步 Schema 到设计画布
   */
  const syncToCanvas = () => {
    try {
      if (!editor) {
        emit('sync', props.schema)
        return
      }

      const content = editor.state.doc.toString()
      let schema

      // 根据格式解析
      if (schemaFormat.value === 'yaml') {
        schema = YAML.load(content)
      } else {
        schema = JSON.parse(content)
      }

      emit('sync', schema)
      message.success('Schema已同步到设计画布')
    } catch (error) {
      message.error('同步失败: ' + error.message)
    }
  }

  /**
   * 获取编辑器当前内容
   */
  const getContent = () => {
    if (editor) {
      return editor.state.doc.toString()
    }
    return props.schema || '{}'
  }

  /**
   * 销毁编辑器
   */
  const destroy = () => {
    if (editor) {
      editor.destroy()
      editor = null
    }
  }

  return {
    editorRef,
    schemaFormat,
    initEditor,
    updateContent,
    format,
    copy,
    import: importSchema,
    export: exportSchema,
    syncToCanvas,
    getContent,
    destroy
  }
}
