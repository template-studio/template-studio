<template>
  <div class="fg">
    <div v-for="(v, key) in schema" :key="key" class="fg-field">
      <div class="fg-head">
        <span class="fg-name">{{ key }}</span>
        <span class="fg-type">{{ typeLabel(v) }}</span>
      </div>

      <!-- 标量控件 -->
      <div v-if="kindOf(v) === 'string'" class="fg-ctl">
        <a-input v-model:value="data[key]" size="small" :placeholder="v.default ? String(v.default) : ''" />
      </div>
      <div v-else-if="kindOf(v) === 'secret'" class="fg-ctl">
        <a-input-password v-model:value="data[key]" size="small" />
      </div>
      <div v-else-if="kindOf(v) === 'number'" class="fg-ctl">
        <a-input-number v-model:value="data[key]" size="small" style="width: 100%" />
      </div>
      <div v-else-if="kindOf(v) === 'boolean'" class="fg-ctl">
        <a-switch v-model:checked="data[key]" size="small" />
      </div>
      <div v-else-if="kindOf(v) === 'enum'" class="fg-ctl">
        <a-select
          :value="data[key]"
          :options="enumOptions(v)"
          size="small"
          style="width: 100%"
          @change="(val) => (data[key] = val)"
        />
      </div>

      <!-- 字符串数组:tags 输入 -->
      <div v-else-if="v.type === 'array' && isStrArr(data[key])" class="fg-ctl">
        <a-select
          mode="tags"
          :value="data[key] || []"
          size="small"
          style="width: 100%"
          placeholder="输入后回车添加"
          @change="(val) => (data[key] = val)"
        />
      </div>

      <!-- 通用数组:JSON 文本域(失焦解析) -->
      <div v-else-if="v.type === 'array'" class="fg-ctl">
        <a-textarea
          :value="arrayJson(data[key])"
          size="small"
          :auto-size="{ minRows: 1, maxRows: 6 }"
          @change="(e) => setArrayJson(data, key, e.target.value)"
        />
      </div>

      <!-- 嵌套对象:递归 -->
      <div v-else-if="kindOf(v) === 'object'" class="fg-nested">
        <FieldGroup :schema="v.properties || {}" :data="data[key] || {}" />
      </div>

      <!-- 对象数组:每项一组 + 增删 -->
      <div v-else-if="kindOf(v) === 'obj-arr'" class="fg-nested">
        <div v-for="(_, i) in data[key] || []" :key="i" class="fg-arr-item">
          <div class="fg-arr-bar">
            <span class="fg-arr-idx">#{{ i + 1 }}</span>
            <button class="fg-arr-del" title="删除此项" @click="data[key].splice(i, 1)">×</button>
          </div>
          <FieldGroup :schema="v.items?.properties || {}" :data="data[key][i]" />
        </div>
        <a-button size="small" block type="dashed" @click="addArrayItem(data, key, v)">+ 添加一项</a-button>
      </div>

      <div v-else class="fg-ctl">
        <a-input v-model:value="data[key]" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup>
// 测试数据表单递归渲染(#200 补10):按 schema 生成控件,直接改写传入的 data 对象,
// 父组件对 testData 的深度监听负责同步右侧 JSON 编辑器。
const props = defineProps({
  schema: { type: Object, default: () => ({}) },
  data: { type: Object, required: true },
});

const TYPE_LABELS = {
  string: '文本',
  secret: '密钥',
  integer: '整数',
  number: '数值',
  boolean: '布尔',
  enum: '枚举',
  array: '数组',
  object: '对象',
  object_arr: '对象数组',
};

const typeLabel = (v) => TYPE_LABELS[v?.type] || '文本';

const kindOf = (v) => {
  const t = v?.type;
  if (t === 'integer' || t === 'number') return 'number';
  if (t === 'boolean') return 'boolean';
  if (t === 'enum') return 'enum';
  if (t === 'secret') return 'secret';
  if (t === 'object') return 'object';
  if (t === 'object_arr') return 'obj-arr';
  // array 类型在模板层按 data 值细分(字符串数组 tags / 通用 JSON 文本域)
  return 'string';
};

// 字符串数组判定(模板里用 data[key] 判断)
const isStrArr = (val) => Array.isArray(val) && val.every((x) => typeof x === 'string');

const enumOptions = (v) =>
  (v?.enum || []).map((e) => ({ label: String(e), value: e }));

const arrayJson = (val) => {
  if (val === undefined || val === null) return '';
  return typeof val === 'string' ? val : JSON.stringify(val, null, 2);
};

const setArrayJson = (data, key, text) => {
  try {
    const parsed = JSON.parse(text);
    if (Array.isArray(parsed)) data[key] = parsed;
  } catch {
    // 非法 JSON 不写回,失焦后恢复为上次合法值
  }
};

// 按对象数组的 items 定义生成一项默认数据
const defaultFromSchema = (schema) => {
  const out = {};
  Object.entries(schema || {}).forEach(([k, v]) => {
    switch (v?.type) {
      case 'integer': case 'number': out[k] = 0; break;
      case 'boolean': out[k] = false; break;
      case 'enum': out[k] = v.enum?.[0] ?? ''; break;
      case 'object': out[k] = defaultFromSchema(v.properties); break;
      case 'object_arr': out[k] = []; break;
      case 'array': out[k] = []; break;
      default: out[k] = '';
    }
  });
  return out;
};

const addArrayItem = (data, key, v) => {
  if (!Array.isArray(data[key])) data[key] = [];
  data[key].push(defaultFromSchema(v.items?.properties));
};
</script>

<style scoped>
.fg-field {
  padding: 7px 0;
  border-bottom: 1px dashed var(--editor-border, #eee);
}
.fg-field:last-child { border-bottom: none; }
.fg-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.fg-name { font-size: 12.5px; font-weight: 500; color: var(--editor-primary, #1e293b); }
.fg-type { font-size: 10.5px; color: var(--editor-muted, #94a3b8); background: var(--editor-inset-bg, #f4f4f2); border-radius: 3px; padding: 0 5px; line-height: 17px; }
.fg-ctl { min-width: 0; }
.fg-nested {
  border-left: 2px solid var(--editor-border, #e2e8f0);
  padding-left: 10px;
  margin-top: 2px;
}
.fg-arr-item { margin-bottom: 8px; }
.fg-arr-bar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 2px; }
.fg-arr-idx { font-size: 10.5px; color: var(--editor-muted, #94a3b8); font-weight: 600; }
.fg-arr-del {
  width: 18px; height: 18px; border: none; background: transparent; border-radius: 4px;
  color: var(--editor-muted, #94a3b8); cursor: pointer; font-size: 13px; line-height: 1;
}
.fg-arr-del:hover { background: rgba(220, 38, 38, 0.1); color: #dc2626; }
</style>
