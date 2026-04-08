<template>
  <div
    class="template-card"
    :class="{ selected: isSelected }"
    @click="$emit('select', template)"
  >
    <div class="card-visual-area">
      <div class="visual-bg">
        <div class="code-snippet-preview">{{ getCodeSnippet(template) }}</div>
      </div>
      <div v-if="template.isFeatured" class="template-badge">
        <span>推荐</span>
      </div>
    </div>

    <div class="card-content-area">
      <h4 class="template-name">{{ template.name }}</h4>
      <p class="template-description">{{ template.description }}</p>

      <div class="template-languages">
        <n-tag
          v-if="template.language"
          :color="{ color: '#f0f0f0', textColor: '#666' }"
          size="small"
        >
          {{ template.language }}
        </n-tag>
      </div>

      <div class="card-footer">
        <div class="card-author">
          <div class="author-avatar"></div>
          <span class="author-name">Template Studio</span>
        </div>
        <div class="template-type">
          <span class="type-badge">{{ template.templateType }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { NTag } from 'naive-ui'

defineProps({
  template: {
    type: Object,
    required: true
  },
  isSelected: {
    type: Boolean,
    default: false
  }
})

defineEmits(['select'])

const getCodeSnippet = (template) => {
  const lang = template.language?.toLowerCase() || ''
  const name = template.name || 'Template'

  if (lang.includes('rust')) {
    `fn main() {
    println!("Hello, ${name}!");
}`
  }

  if (lang.includes('go') || lang.includes('golang')) {
    return `package main

import "fmt"

func main() {
    fmt.Printf("Hello, ${name}!\\n")
}`
  }

  if (lang.includes('python')) {
    return `def main():
    print("Hello, ${name}!")

if __name__ == "__main__":
    main()`
  }

  if (lang.includes('javascript') || lang.includes('typescript')) {
    return `function main() {
    console.log('Hello, ${name}!');
}

main();`
  }

  return `// ${name}
class Application {
  constructor() {
    this.name = '${name}';
  }

  run() {
    console.log('Running', this.name);
  }
}

const app = new Application();
app.run();`
}
</script>

<style scoped>
.template-card {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(12px);
  border-radius: 16px;
  box-shadow: 0 1px 2px 0 rgba(60, 64, 67, 0.1), 0 1px 3px 1px rgba(60, 64, 67, 0.05);
  overflow: hidden;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  cursor: pointer;
  position: relative;
  border: 2px solid transparent;
}

.template-card:hover {
  transform: translateY(-6px) scale(1.02);
  box-shadow: 0 12px 40px rgba(66, 133, 244, 0.15);
  background: rgba(255, 255, 255, 0.95);
}

.template-card.selected {
  border-color: #4285f4;
  box-shadow: 0 0 0 4px rgba(66, 133, 244, 0.1);
}

.card-visual-area {
  width: 100%;
  height: 160px;
  position: relative;
  overflow: hidden;
}

.visual-bg {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.visual-bg::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    45deg,
    transparent 30%,
    rgba(255, 255, 255, 0.2) 50%,
    transparent 70%
  );
  animation: shimmer 3s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.code-snippet-preview {
  font-family: 'Courier New', 'Consolas', 'Monaco', monospace;
  font-size: 9px;
  line-height: 1.4;
  color: rgba(255, 255, 255, 0.4);
  white-space: pre;
  overflow: hidden;
  padding: 16px;
  text-align: left;
  position: relative;
  z-index: 1;
}

.template-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: #4285f4;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 2;
}

.card-content-area {
  padding: 16px;
  background: #fff;
}

.template-name {
  font-size: 16px;
  font-weight: 600;
  color: #202124;
  margin: 0 0 8px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.template-description {
  font-size: 13px;
  color: #5f6368;
  margin: 0 0 12px 0;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  min-height: 42px;
}

.template-languages {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
}

.card-author {
  display: flex;
  align-items: center;
  gap: 8px;
}

.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
}

.author-name {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.template-type {
  display: flex;
  align-items: center;
}

.type-badge {
  font-size: 11px;
  color: #999;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
