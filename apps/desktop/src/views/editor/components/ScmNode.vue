<template>
  <div>
    <div
      v-if="node.dirs.length || node.name"
      class="scm-dir" :style="{ paddingLeft: (8 + depth * 12) + 'px' }"
      @click="node.open = !node.open"
    >
      <span class="scm-chev" :class="{ open: node.open }">›</span>
      <span class="scm-dir-name">{{ node.name || '(根目录)' }}</span>
      <span class="scm-dir-n">{{ node.count }}</span>
    </div>
    <template v-if="node.open">
      <ScmNode
        v-for="d in node.dirs" :key="d.key" :node="d" :depth="depth + 1"
        @open-file="$emit('open-file', $event)" @discard="$emit('discard', $event)"
      />
      <div
        v-for="en in node.files" :key="en.path"
        class="scm-row" :class="en.status" :style="{ paddingLeft: (20 + depth * 12) + 'px' }"
        @click="$emit('open-file', en.path)"
      >
        <span class="scm-name">{{ en.base }}</span>
        <span class="scm-stat" v-if="en.added != null"><i class="a">+{{ en.added }}</i><i v-if="en.removed" class="d">-{{ en.removed }}</i></span>
        <span class="scm-letter" :title="en.status === 'M' ? '已修改' : en.status === 'A' ? '新增' : '已删除'">{{ en.status }}</span>
        <button class="scm-row-act" title="放弃更改" @click.stop="$emit('discard', en)"><UndoOutlined /></button>
      </div>
    </template>
  </div>
</template>

<script setup>
// SCM 树递归节点(目录+文件行);SFC 文件名即自引用名,保证递归解析。
import { UndoOutlined } from '@ant-design/icons-vue'

defineProps({ node: { type: Object, required: true }, depth: { type: Number, default: 0 } })
defineEmits(['open-file', 'discard'])
</script>
