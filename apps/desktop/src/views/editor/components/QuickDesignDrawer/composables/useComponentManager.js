/**
 * 组件管理 Composable
 * 负责组件的增删改查、Schema转换、容器组件管理等核心业务逻辑
 */
import { ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import {
  generateFieldName,
  generateDefaultSchema,
  generateComponentTitle,
  COMPONENT_CATEGORIES
} from '../utils/componentTemplates'

export function useComponentManager(schemaStore) {

  // ========== 状态 ==========

  /**
   * 组件列表
   */
  const components = ref([])

  /**
   * 当前选中的组件ID
   */
  const selectedComponentId = ref(null)

  /**
   * 容器组件展开状态管理
   */
  const expandedComponents = ref(new Set())

  // ========== 计算属性 ==========

  /**
   * 当前选中的组件对象（递归查找子节点）
   */
  const selectedComponent = computed(() => {
    if (!selectedComponentId.value) {
      return null
    }

    // 先在根级别查找
    let found = components.value.find(c => c.id === selectedComponentId.value)
    if (found) return found

    // 递归在子节点中查找
    const searchInChildren = (list) => {
      for (const comp of list) {
        if (comp.children && comp.children.length > 0) {
          const foundInChildren = comp.children.find(c => c.id === selectedComponentId.value)
          if (foundInChildren) return foundInChildren

          const foundInNested = searchInChildren(comp.children)
          if (foundInNested) return foundInNested
        }
      }
      return null
    }

    return searchInChildren(components.value)
  })

  /**
   * 组件总数
   */
  const componentCount = computed(() => components.value.length)

  /**
   * 是否有未保存的更改
   */
  const hasUnsavedChanges = computed(() => {
    return schemaStore.hasUnsavedDraft
  })

  /**
   * 格式化的 Schema 字符串（用于编辑器显示）
   */
  const formattedSchema = computed(() => {
    const schema = {}
    components.value.forEach(comp => {
      schema[comp.fieldName] = componentToSchema(comp)
    })
    return JSON.stringify(schema, null, 2)
  })

  /**
   * 表单预览用的 Schema（对象格式）
   */
  const currentSchemaForPreview = computed(() => {
    const schema = {}
    components.value.forEach(comp => {
      schema[comp.fieldName] = componentToSchema(comp)
    })
    return schema
  })

  // ========== 组件操作 ==========

  /**
   * 添加组件
   * @param {Object} componentTemplate - 组件模板
   * @param {Object} targetContainer - 目标容器（可选，如果提供则添加到容器内）
   */
  const addComponent = (componentTemplate, targetContainer = null) => {
    // 收集已存在的字段名（用于避免重复）
    const existingFields = {}
    components.value.forEach(comp => {
      existingFields[comp.fieldName] = true
    })

    // 生成唯一且有意义的字段名
    const fieldName = generateFieldName(componentTemplate.type, existingFields)
    const schema = generateDefaultSchema(componentTemplate.type, fieldName)

    // 生成友好的中文标题
    const typeCount = components.value.filter(c => c.type === componentTemplate.type).length + 1
    schema.title = generateComponentTitle(componentTemplate.type, typeCount)

    const newComponent = {
      id: `comp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      fieldName,
      type: componentTemplate.type,
      schema,
      template: componentTemplate,
      children: componentTemplate.isContainer ? [] : undefined
    }

    // 如果提供了目标容器，添加到容器内
    if (targetContainer) {
      addChildToContainer(targetContainer, newComponent)
      selectedComponentId.value = newComponent.id
      message.success(`已添加 ${componentTemplate.name} 到 ${targetContainer.schema.title || targetContainer.fieldName}`)
      return newComponent
    }

    // 如果当前选中的是容器组件，添加到该容器内
    if (selectedComponentId.value) {
      const selected = selectedComponent.value
      if (selected && isContainerComponent(selected)) {
        addChildToContainer(selected, newComponent)
        selectedComponentId.value = newComponent.id
        message.success(`已添加 ${componentTemplate.name} 到 ${selected.schema.title || selected.fieldName}`)
        return newComponent
      }
    }

    // 添加到根级别
    components.value.push(newComponent)

    // 自动选中新组件
    selectedComponentId.value = newComponent.id

    message.success(`已添加 ${componentTemplate.name}`)

    return newComponent
  }

  /**
   * 删除组件
   * @param {number} index - 组件在列表中的索引
   * @param {Object} parentContainer - 父容器（可选，用于删除容器内的子组件）
   */
  const removeComponent = (index, parentContainer = null) => {
    let removed
    let targetList = parentContainer ? parentContainer.children : components.value

    removed = targetList.splice(index, 1)[0]

    if (removed.id === selectedComponentId.value) {
      selectedComponentId.value = null
    }

    // 如果是从容器中删除，需要同步更新 Schema
    if (parentContainer) {
      if (parentContainer.type === 'object' && parentContainer.schema.properties) {
        delete parentContainer.schema.properties[removed.fieldName]
      } else if (parentContainer.type === 'object_arr' && parentContainer.schema.items?.properties) {
        delete parentContainer.schema.items.properties[removed.fieldName]
      }
    }

    message.info('已删除组件')
  }

  /**
   * 选择组件
   * @param {Object} component - 要选择的组件
   */
  const selectComponent = (component) => {
    selectedComponentId.value = component.id
  }

  /**
   * 更新组件 Schema
   * @param {string} componentId - 组件ID
   * @param {Object} schemaUpdates - Schema 更新内容
   */
  const updateComponentSchema = (componentId, schemaUpdates) => {
    const component = findComponentById(componentId)
    if (!component) return

    Object.assign(component.schema, schemaUpdates)
  }

  /**
   * 更新组件类型
   * @param {string} componentId - 组件ID
   * @param {string} newType - 新类型
   */
  const updateComponentType = (componentId, newType) => {
    const component = findComponentById(componentId)
    if (!component) return

    component.type = newType

    // 查找对应的模板
    for (const category of COMPONENT_CATEGORIES) {
      const template = category.components.find(c => c.type === newType)
      if (template) {
        component.template = template
        break
      }
    }
  }

  /**
   * 更新组件字段名
   * @param {string} componentId - 组件ID
   * @param {string} newFieldName - 新字段名
   */
  const updateComponentFieldName = (componentId, newFieldName) => {
    const component = findComponentById(componentId)
    if (!component) return

    const oldFieldName = component.fieldName
    component.fieldName = newFieldName

    // 更新 insertText
    component.schema.insertText = `{{ ${newFieldName} }}`

    // 如果是容器组件，需要递归更新子字段的引用路径
    if (isContainerComponent(component) && component.children) {
      updateChildrenFields(component, newFieldName)
    }

    // 如果组件在容器内，需要更新容器的 properties
    updateParentContainerProperties(component, oldFieldName, newFieldName)
  }

  // ========== 容器组件操作 ==========

  /**
   * 判断是否是容器组件
   * @param {Object} component - 组件对象
   * @returns {boolean}
   */
  const isContainerComponent = (component) => {
    if (!component || !component.template) return false
    return component.template.isContainer === true
  }

  /**
   * 判断组件是否展开
   * @param {string} componentId - 组件ID
   * @returns {boolean}
   */
  const isComponentExpanded = (componentId) => {
    return expandedComponents.value.has(componentId)
  }

  /**
   * 切换组件展开/折叠状态
   * @param {string} componentId - 组件ID
   */
  const toggleComponentExpansion = (componentId) => {
    if (expandedComponents.value.has(componentId)) {
      expandedComponents.value.delete(componentId)
    } else {
      expandedComponents.value.add(componentId)
    }
  }

  /**
   * 向容器添加子组件
   * @param {Object} container - 容器组件
   * @param {Object} child - 子组件
   */
  const addChildToContainer = (container, child) => {
    if (!container.children) {
      container.children = []
    }
    container.children.push(child)

    // 同时更新 Schema
    if (container.type === 'object') {
      if (!container.schema.properties) {
        container.schema.properties = {}
      }
      container.schema.properties[child.fieldName] = child.schema
    } else if (container.type === 'object_arr') {
      if (!container.schema.items) {
        container.schema.items = { type: 'object', properties: {} }
      }
      if (!container.schema.items.properties) {
        container.schema.items.properties = {}
      }
      container.schema.items.properties[child.fieldName] = child.schema
    }
  }

  /**
   * 从容器移除子组件
   * @param {Object} container - 容器组件
   * @param {string} childId - 子组件ID
   */
  const removeChildFromContainer = (container, childId) => {
    if (!container.children) return

    const index = container.children.findIndex(c => c.id === childId)
    if (index > -1) {
      const removed = container.children.splice(index, 1)[0]

      // 同时从 Schema 中移除
      if (container.type === 'object' && container.schema.properties) {
        delete container.schema.properties[removed.fieldName]
      } else if (container.type === 'object_arr' && container.schema.items?.properties) {
        delete container.schema.items.properties[removed.fieldName]
      }
    }
  }

  /**
   * 获取容器组件的子字段数量
   * @param {Object} component - 组件对象
   * @returns {number}
   */
  const getChildrenCount = (component) => {
    if (component.type === 'object') {
      return Object.keys(component.schema.properties || {}).length
    } else if (component.type === 'array') {
      return 0 // 简单数组没有子字段
    } else if (component.type === 'object_arr') {
      return Object.keys(component.schema.properties || {}).length
    }
    return 0
  }

  // ========== Schema 转换 ==========

  /**
   * 将组件转换为 Schema（递归处理容器组件）
   * @param {Object} component - 组件对象
   * @returns {Object} Schema 对象
   */
  const componentToSchema = (component) => {
    const schema = { ...component.schema }

    // 处理容器组件的子字段
    if (component.type === 'object' && component.children && component.children.length > 0) {
      schema.properties = {}
      component.children.forEach(child => {
        schema.properties[child.fieldName] = componentToSchema(child)
      })
    } else if (component.type === 'object_arr' && component.children && component.children.length > 0) {
      // object_arr: 子字段定义在 items.properties 中
      if (!schema.items) {
        schema.items = { type: 'object', properties: {} }
      }
      component.children.forEach(child => {
        schema.items.properties[child.fieldName] = componentToSchema(child)
      })
    }
    // array 类型暂时不支持子字段（简单数组）

    return schema
  }

  /**
   * 从 Schema 创建组件（递归处理容器组件）
   * @param {string} fieldName - 字段名
   * @param {Object} fieldSchema - Schema 对象
   * @returns {Object} 组件对象
   */
  const createComponentFromSchema = (fieldName, fieldSchema) => {
    const component = {
      id: `comp_${fieldName}_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      fieldName,
      type: fieldSchema.type,
      schema: { ...fieldSchema },
      children: undefined
    }

    // 查找对应的模板信息（用于图标、颜色等）
    COMPONENT_CATEGORIES.forEach(category => {
      const found = category.components.find(c => c.type === fieldSchema.type)
      if (found) {
        component.template = found
      }
    })

    // 处理容器组件的子字段
    if (fieldSchema.type === 'object' && fieldSchema.properties) {
      component.children = []
      Object.entries(fieldSchema.properties).forEach(([childName, childSchema]) => {
        component.children.push(createComponentFromSchema(childName, childSchema))
      })
    } else if (fieldSchema.type === 'object_arr' && fieldSchema.items && fieldSchema.items.properties) {
      component.children = []
      Object.entries(fieldSchema.items.properties).forEach(([childName, childSchema]) => {
        component.children.push(createComponentFromSchema(childName, childSchema))
      })
    }

    return component
  }

  /**
   * 将整个 Schema 转换为组件列表
   * @param {Object} schema - Schema 对象
   */
  const schemaToComponents = (schema) => {
    // 先构建完整的组件数组，避免多次触发 watch
    const newComponents = []
    Object.entries(schema).forEach(([fieldName, fieldSchema]) => {
      newComponents.push(createComponentFromSchema(fieldName, fieldSchema))
    })
    // 一次性赋值，只触发一次 watch
    components.value = newComponents
  }

  /**
   * 将组件列表转换为 Schema
   * @returns {Object} Schema 对象
   */
  const componentsToSchema = () => {
    const schema = {}
    components.value.forEach(comp => {
      schema[comp.fieldName] = componentToSchema(comp)
    })
    return schema
  }

  // ========== 工具方法 ==========

  /**
   * 根据 ID 查找组件（递归搜索容器内的组件）
   * @param {string} componentId - 组件ID
   * @returns {Object|null} 组件对象
   */
  const findComponentById = (componentId) => {
    // 在根级别查找
    let found = components.value.find(c => c.id === componentId)
    if (found) return found

    // 在容器的 children 中递归查找
    const searchInContainer = (container) => {
      if (!container.children) return null

      for (const child of container.children) {
        if (child.id === componentId) return child

        if (child.children) {
          const foundInChildren = searchInContainer(child)
          if (foundInChildren) return foundInChildren
        }
      }
      return null
    }

    for (const comp of components.value) {
      if (comp.children) {
        const foundInChildren = searchInContainer(comp)
        if (foundInChildren) return foundInChildren
      }
    }

    return null
  }

  /**
   * 更新容器子字段的引用路径
   * @param {Object} container - 容器组件
   * @param {string} newParentFieldName - 新的父字段名
   */
  const updateChildrenFields = (container, newParentFieldName) => {
    if (!container.children) return

    container.children.forEach(child => {
      // 更新 insertText 中的父字段引用
      if (child.schema.insertText) {
        child.schema.insertText = child.schema.insertText.replace(
          /\{\{[\s\S]*?\}\}/,
          `{{ ${newParentFieldName}.${child.fieldName} }}`
        )
      }

      // 递归更新嵌套容器
      if (child.children) {
        updateChildrenFields(child, `${newParentFieldName}.${child.fieldName}`)
      }
    })
  }

  /**
   * 更新父容器的 properties（当子组件字段名改变时）
   * @param {Object} component - 子组件
   * @param {string} oldFieldName - 旧字段名
   * @param {string} newFieldName - 新字段名
   */
  const updateParentContainerProperties = (component, oldFieldName, newFieldName) => {
    // 查找该组件所在的容器
    const findParent = (searchList, parent = null) => {
      for (const item of searchList) {
        if (item.id === component.id) return parent

        if (item.children) {
          const found = findParent(item.children, item)
          if (found) return found
        }
      }
      return null
    }

    const parent = findParent(components.value)
    if (!parent) return

    // 更新父容器的 properties
    if (parent.type === 'object' && parent.schema.properties) {
      const oldSchema = parent.schema.properties[oldFieldName]
      if (oldSchema) {
        parent.schema.properties[newFieldName] = oldSchema
        delete parent.schema.properties[oldFieldName]
      }
    } else if (parent.type === 'object_arr' && parent.schema.items?.properties) {
      const oldSchema = parent.schema.items.properties[oldFieldName]
      if (oldSchema) {
        parent.schema.items.properties[newFieldName] = oldSchema
        delete parent.schema.items.properties[oldFieldName]
      }
    }
  }

  /**
   * 重置所有组件
   */
  const resetComponents = () => {
    components.value = []
    selectedComponentId.value = null
    expandedComponents.value.clear()
  }

  return {
    // 状态
    components,
    selectedComponentId,
    selectedComponent,
    componentCount,
    hasUnsavedChanges,
    expandedComponents,
    formattedSchema,
    currentSchemaForPreview,

    // 组件操作
    addComponent,
    removeComponent,
    selectComponent,
    updateComponentSchema,
    updateComponentType,
    updateComponentFieldName,
    resetComponents,

    // 容器操作
    isContainerComponent,
    isComponentExpanded,
    toggleComponentExpansion,
    addChildToContainer,
    removeChildFromContainer,
    getChildrenCount,

    // Schema 转换
    componentToSchema,
    createComponentFromSchema,
    schemaToComponents,
    componentsToSchema,

    // 工具方法
    findComponentById
  }
}
