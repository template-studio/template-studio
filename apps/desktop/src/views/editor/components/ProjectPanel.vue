<template>
  <div class="proj-panel">
    <!-- VSCode 式标题行 -->
    <div class="pp-head">
      <span class="pp-title">项目区</span>
      <div class="pp-head-actions">
        <button class="pp-act" title="重新渲染" @click="renderAll"><RedoOutlined :spin="rendering" /></button>
      </div>
    </div>

    <div class="pp-tip">渲染产物实例化到本地并构建冒烟——验证模板真实可用</div>

    <!-- 变量/技术栈 -->
    <div class="pp-form">
      <div class="pp-field">
        <div class="pp-label">测试数据</div>
        <a-select
          v-model:value="dataSource"
          size="small"
          style="width: 100%"
          :options="[
            { label: '已保存的测试数据', value: 'saved' },
            { label: '变量默认值', value: 'defaults' },
          ]"
        />
      </div>
      <div class="pp-field">
        <div class="pp-label">技术栈(构建命令)</div>
        <a-select v-model:value="packId" size="small" style="width: 100%" :options="packOptions" />
      </div>
      <a-button block size="small" type="primary" :loading="rendering" style="margin-top: 4px" @click="renderAll">
        <template #icon><RocketOutlined /></template>渲染到本地
      </a-button>
    </div>

    <!-- 渲染结果 -->
    <div v-if="files.length" class="pp-section">
      <div class="pp-lhead">渲染产物（{{ files.length }} 文件）<span v-if="lastDir" class="pp-dir" :title="lastDir">{{ shortDir }}</span></div>
      <div class="pp-files">
        <div v-for="f in files" :key="f.path" class="pp-file" :class="{ err: f.error }">
          <span class="pp-fname" :title="f.path">{{ f.path }}</span>
          <span v-if="f.error" class="pp-ferr" :title="f.error">渲染失败</span>
        </div>
      </div>
      <a-button
        block size="small" style="margin-top: 8px"
        :disabled="!files.length || !!errorCount"
        :loading="building"
        @click="runBuild"
      ><template #icon><CaretRightOutlined /></template>构建验证
      </a-button>
    </div>

    <!-- 构建输出 -->
    <div v-if="buildResult" class="pp-section">
      <div class="pp-lhead">
        构建结果
        <a-tag :color="buildResult.ok ? 'success' : 'error'" size="small" style="margin-left: 6px">
          {{ buildResult.ok ? '通过' : '失败' }}
        </a-tag>
        <span v-if="buildResult.durationMs" class="pp-dir">{{ buildResult.durationMs }}ms</span>
      </div>
      <pre class="pp-log">{{ buildResult.output || '(无输出)' }}</pre>
    </div>
  </div>
</template>

<script setup>
// 编辑器项目区(#718/#145):渲染产物实例化验证。
// 数据流:服务端 preview-tree(测试数据或默认值) → invoke render_files 语义复用:
// 直接把服务端产物逐文件 write(经 convert_agent_write 不带哈希校验不适用,这里用
// workspace/projects 落盘 + convert_build_check 冒烟,全部复用转换台已验证命令)。
import { ref, computed, onMounted } from 'vue';
import { message } from 'ant-design-vue';
import { RedoOutlined, RocketOutlined, CaretRightOutlined } from '@ant-design/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { previewFileTree } from '@/api/editor/templateFiles';
import { getTemplateTestData } from '@/api/editor/templateExpose';

const props = defineProps({
  templateId: { type: [String, Number], required: true },
  variables: { type: Object, default: () => ({}) }, // 编辑器当前变量值(测试数据)
});

const PACKS = [
  { id: 'node', label: 'Node.js (npm build)' },
  { id: 'go', label: 'Go (go build)' },
  { id: 'java', label: 'Java (mvn compile)' },
  { id: 'python', label: 'Python (compileall)' },
  { id: 'rust', label: 'Rust (cargo check)' },
];

const packOptions = PACKS.map((p) => ({ label: p.label, value: p.id }));
const dataSource = ref('saved');
const packId = ref('node');
const rendering = ref(false);
const building = ref(false);
const files = ref([]);
const lastDir = ref('');
const buildResult = ref(null);

const errorCount = computed(() => files.value.filter((f) => f.error).length);
const shortDir = computed(() => {
  const d = lastDir.value;
  return d.length > 34 ? '…' + d.slice(-33) : d;
});

// 渲染全量树(服务端) + 落盘本地(转换台 Rust 命令)
const renderAll = async () => {
  rendering.value = true;
  buildResult.value = null;
  try {
    // 1) 变量来源
    let vars = {};
    if (dataSource.value === 'saved') {
      try {
        vars = (await getTemplateTestData({ templateId: props.templateId })) || {};
      } catch {
        vars = {};
      }
    } else {
      vars = props.variables || {};
    }

    // 2) 服务端渲染整树(未发布状态即可验证,读取工作区)
    const res = await previewFileTree({ templateId: Number(props.templateId), variables: vars });
    const tree = res?.data?.data?.tree || [];
    const flat = [];
    const walk = (nodes) => {
      for (const n of nodes || []) {
        if (n.children?.length) walk(n.children);
        else if (n.filePath) flat.push({ path: n.filePath, content: n.fileContent || '', error: n.renderError?.message || null });
      }
    };
    walk(tree);

    if (!flat.length) {
      message.warning('模板没有可渲染的文件');
      return;
    }
    files.value = flat;

    // 3) 本地落盘(仅成功文件;outputs 形状与 convert_build_check 约定一致)
    const outputs = flat
      .filter((f) => !f.error)
      .map((f) => ({ path: f.path, content: f.content }));
    const r = await invoke('convert_project_materialize', {
      templateId: String(props.templateId),
      outputs,
    });
    const parsed = typeof r === 'string' ? JSON.parse(r) : r;
    if (!parsed.ok) {
      message.error('落盘失败: ' + (parsed.error || ''));
      return;
    }
    lastDir.value = parsed.dir;
    message.success(`已渲染 ${outputs.length} 文件到本地`);
  } catch (error) {
    console.error('渲染失败:', error);
    message.error('渲染失败: ' + (error?.message || error));
  } finally {
    rendering.value = false;
  }
};

const runBuild = async () => {
  building.value = true;
  buildResult.value = null;
  try {
    const vars = dataSource.value === 'saved'
      ? (await getTemplateTestData({ templateId: props.templateId }).catch(() => ({}))) || {}
      : props.variables || {};
    const r = await invoke('convert_build_check', {
      outputs: files.value.filter((f) => !f.error).map((f) => ({ path: f.path, content: f.content })),
      variables: vars,
      packId: packId.value,
    });
    buildResult.value = typeof r === 'string' ? JSON.parse(r) : r;
    if (buildResult.value.ok) {
      message.success(`构建通过(${buildResult.value.durationMs}ms)`);
    } else {
      message.error('构建失败,详见输出');
    }
  } catch (error) {
    message.error('构建失败: ' + (error?.message || error));
  } finally {
    building.value = false;
  }
};
</script>

<style scoped>
.proj-panel {
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  background: var(--editor-panel-bg, #fff);
}

.pp-head {
  height: 34px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 6px 0 14px;
  border-bottom: 1px solid var(--editor-border, #f0f0ee);
}
.pp-title { font-size: 11px; font-weight: 700; letter-spacing: 0.5px; color: var(--color-text, #1b1c1f); }
.pp-head-actions { display: flex; opacity: 0; transition: opacity 0.12s; }
.pp-head:hover .pp-head-actions { opacity: 1; }
.pp-act {
  width: 24px; height: 24px; border: none; background: transparent; border-radius: 5px;
  cursor: pointer; color: var(--color-text-secondary, #64748b); font-size: 13px;
  display: inline-flex; align-items: center; justify-content: center;
}
.pp-act:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }

.pp-tip {
  flex-shrink: 0;
  padding: 6px 14px;
  font-size: 11px;
  color: var(--editor-muted, #94a3b8);
  border-bottom: 1px solid var(--editor-border, #f0f0ee);
}

.pp-form { padding: 10px 14px; border-bottom: 1px solid var(--editor-border, #f0f0ee); }
.pp-field { margin-bottom: 8px; }
.pp-label { font-size: 11px; color: var(--editor-muted, #94a3b8); margin-bottom: 3px; }

.pp-section { padding: 8px 14px 12px; border-bottom: 1px solid var(--editor-border, #f0f0ee); }
.pp-lhead {
  font-size: 11px; font-weight: 700; color: var(--editor-muted, #94a3b8);
  margin-bottom: 6px; display: flex; align-items: center; gap: 4px;
}
.pp-dir { font-weight: 400; font-size: 10px; margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 110px; }

.pp-files { max-height: 200px; overflow-y: auto; border: 1px solid var(--editor-border, #e2e8f0); border-radius: 6px; }
.pp-file { display: flex; align-items: center; gap: 6px; padding: 3px 8px; font-size: 12px; }
.pp-file + .pp-file { border-top: 1px solid var(--editor-border, #f1f5f9); }
.pp-fname { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--editor-primary, #333); }
.pp-file.err .pp-fname { color: var(--editor-muted, #94a3b8); }
.pp-ferr { flex-shrink: 0; font-size: 10.5px; color: #dc2626; }

.pp-log {
  margin: 0; padding: 8px 10px; border-radius: 6px;
  background: var(--editor-inset-bg, #17181c); color: #d4d4d4;
  font-size: 11px; line-height: 1.5; font-family: Consolas, monospace;
  max-height: 260px; overflow: auto; white-space: pre-wrap; word-break: break-all;
}
</style>
