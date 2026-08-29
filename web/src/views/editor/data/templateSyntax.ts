/**
 * 模板语法参考数据（编辑器「模板语法」侧栏面板）
 *
 * 语法基于 MiniJinja 模板引擎（crates/template_core）。
 * 注意：此文件曾被根 .gitignore 的 `data/` 规则误伤而未入库，现为依据
 * VariableSidebar / editor index.vue 的使用约定重建。
 */

export interface TemplateSyntaxItem {
  name: string;
  display_name?: string;
  insertText?: string;
  description?: string;
  usage?: string;
  type?: string;
}

export interface TemplateSyntaxCategory {
  name: string;
  syntaxes: TemplateSyntaxItem[];
}

export const templateSyntaxCategories: TemplateSyntaxCategory[] = [
  {
    name: '变量输出',
    syntaxes: [
      {
        name: 'variable',
        display_name: '变量插值 {{ }}',
        insertText: '{{ variable_name }}',
        description: '输出变量的值，支持点号与下标访问属性。',
        usage: '{{ user.name }}、{{ items[0] }}',
        type: 'output',
      },
      {
        name: 'attribute',
        display_name: '属性访问',
        insertText: '{{ .property }}',
        description: '在编辑器中插入变量属性引用（点号前缀形式）。',
        usage: '先输入 {{ 再选择变量，或直接插入 {{ .field }}',
        type: 'output',
      },
      {
        name: 'expression',
        display_name: '表达式输出',
        insertText: '{{ a + b }}',
        description: '在插值中可以使用算术、比较与逻辑表达式。',
        usage: '{{ price * count }}、{{ ok and ready }}',
        type: 'output',
      },
      {
        name: 'loop.index',
        display_name: '循环变量 loop',
        insertText: '{{ loop.index }}',
        description: 'for 循环内的内置变量：index（从 1 开始）、index0、first、last、length。',
        usage: '{% for item in items %}{{ loop.index }}: {{ item }}{% endfor %}',
        type: 'output',
      },
    ],
  },
  {
    name: '条件控制',
    syntaxes: [
      {
        name: 'if',
        display_name: '条件判断 if/else',
        insertText: '{% if condition %}\n\n{% endif %}',
        description: '条件渲染块，支持 elif 与 else 分支。',
        usage: '{% if score > 80 %}优秀{% elif score > 60 %}及格{% else %}不及格{% endif %}',
        type: 'block',
      },
      {
        name: 'is-defined',
        display_name: '判断变量已定义',
        insertText: "{% if variable_name is defined %}",
        description: '用 is 测试判断变量是否存在，常与 default 过滤器配合。',
        usage: '{% if version is defined %}v{{ version }}{% endif %}',
        type: 'test',
      },
      {
        name: 'is-undefined',
        display_name: '判断变量未定义',
        insertText: '{% if variable_name is undefined %}',
        description: '变量缺失时的兜底分支。',
        usage: '{% if logo is undefined %}<img src="default.png" />{% endif %}',
        type: 'test',
      },
    ],
  },
  {
    name: '循环迭代',
    syntaxes: [
      {
        name: 'for',
        display_name: 'for 循环',
        insertText: '{% for item in items %}\n\n{% endfor %}',
        description: '遍历列表或对象，循环内可用 loop 内置变量。',
        usage: '{% for item in items %}{{ item.name }}{% endfor %}',
        type: 'block',
      },
      {
        name: 'for-else',
        display_name: '空列表兜底 else',
        insertText: '{% for item in items %}\n\n{% else %}\n\n{% endfor %}',
        description: '列表为空时渲染 else 分支。',
        usage: '{% for item in items %}{{ item }}{% else %}暂无数据{% endfor %}',
        type: 'block',
      },
    ],
  },
  {
    name: '模板指令',
    syntaxes: [
      {
        name: 'set',
        display_name: '变量赋值 set',
        insertText: "{% set name = value %}",
        description: '在模板内定义局部变量，供后续使用。',
        usage: "{% set fullName = user.first ~ ' ' ~ user.last %}",
        type: 'block',
      },
      {
        name: 'filter',
        display_name: '过滤块 filter',
        insertText: '{% filter upper %}\n\n{% endfilter %}',
        description: '对整块内容应用过滤器。',
        usage: '{% filter trim %}  文本  {% endfilter %}',
        type: 'block',
      },
      {
        name: 'raw',
        display_name: '原样输出 raw',
        insertText: '{% raw %}\n\n{% endraw %}',
        description: '块内内容不做模板解析，用于输出字面量语法。',
        usage: '{% raw %}{{ not_rendered }}{% endraw %}',
        type: 'block',
      },
    ],
  },
  {
    name: '常用过滤器',
    syntaxes: [
      {
        name: 'default',
        display_name: '缺省值 default',
        insertText: "{{ variable_name | default('默认值') }}",
        description: '变量未定义或为空时使用默认值。',
        usage: "{{ app_name | default('my-app') }}",
        type: 'filter',
      },
      {
        name: 'upper',
        display_name: '转大写 upper',
        insertText: '{{ variable_name | upper }}',
        description: '将字符串转为大写。',
        usage: "{{ project | upper }} → MY-PROJECT",
        type: 'filter',
      },
      {
        name: 'lower',
        display_name: '转小写 lower',
        insertText: '{{ variable_name | lower }}',
        description: '将字符串转为小写。',
        usage: '{{ DB_HOST | lower }}',
        type: 'filter',
      },
      {
        name: 'capitalize',
        display_name: '首字母大写 capitalize',
        insertText: '{{ variable_name | capitalize }}',
        description: '首字母大写、其余小写。',
        usage: "{{ name | capitalize }} → Tom",
        type: 'filter',
      },
      {
        name: 'trim',
        display_name: '去除首尾空白 trim',
        insertText: '{{ variable_name | trim }}',
        description: '去掉字符串两端的空白字符。',
        usage: '{{ input | trim }}',
        type: 'filter',
      },
      {
        name: 'replace',
        display_name: '替换 replace',
        insertText: "{{ variable_name | replace('from', 'to') }}",
        description: '替换子字符串。',
        usage: "{{ name | replace('-', '_') }}",
        type: 'filter',
      },
      {
        name: 'join',
        display_name: '连接 join',
        insertText: "{{ items | join(', ') }}",
        description: '将列表拼接为字符串。',
        usage: "{{ tags | join(', ') }}",
        type: 'filter',
      },
      {
        name: 'length',
        display_name: '长度 length',
        insertText: '{{ variable_name | length }}',
        description: '返回字符串长度或列表元素个数。',
        usage: '{% if items | length > 0 %}…{% endif %}',
        type: 'filter',
      },
      {
        name: 'first',
        display_name: '首元素 first',
        insertText: '{{ items | first }}',
        description: '取列表第一个元素。',
        usage: '{{ versions | first }}',
        type: 'filter',
      },
      {
        name: 'sort',
        display_name: '排序 sort',
        insertText: '{{ items | sort }}',
        description: '排序列表（可配合 reverse）。',
        usage: '{{ names | sort | join(", ") }}',
        type: 'filter',
      },
      {
        name: 'tojson',
        display_name: '转 JSON tojson',
        insertText: '{{ variable_name | tojson }}',
        description: '序列化为 JSON 字符串，常用于生成配置文件。',
        usage: '{{ config | tojson }}',
        type: 'filter',
      },
    ],
  },
  {
    name: '注释与空白',
    syntaxes: [
      {
        name: 'comment',
        display_name: '注释 {# #}',
        insertText: '{# 注释内容 #}',
        description: '注释不会出现在渲染结果中。',
        usage: '{# 这里是说明 #}',
        type: 'comment',
      },
      {
        name: 'whitespace-control',
        display_name: '空白控制 {{- -}}',
        insertText: '{{- variable_name -}}',
        description: '减号去除标签两侧的空白/换行，避免多余空行。',
        usage: '{%- if ok -%}…{%- endif -%}',
        type: 'output',
      },
    ],
  },
];
