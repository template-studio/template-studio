<template>
  <div class="tables-view">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <h2 class="page-title">{{ project?.name }}</h2>
        <a-tag v-if="project?.datasource" :color="getDatabaseColor(project.datasource.type_)">
          {{ getDatabaseLabel(project.datasource.type_) }}
        </a-tag>
        <span class="database-name">{{ project?.database_name }}</span>
      </div>
      <div class="header-right">
        <a-button type="primary" @click="showAICreateTableDialog">
          <template #icon>
            <RobotOutlined />
          </template>
          AI 建表
        </a-button>
        <a-button type="primary" @click="showAddTableDialog">
          <template #icon>
            <PlusOutlined />
          </template>
          新增表
        </a-button>
        <a-button @click="showSqlImportDialog">
          <template #icon>
            <FileTextOutlined />
          </template>
          从SQL导入
        </a-button>
        <a-button @click="importTables" :loading="importing">
          <template #icon>
            <ImportOutlined />
          </template>
          导入表结构
        </a-button>
        <a-button @click="loadTables">
          <template #icon>
            <ReloadOutlined />
          </template>
          刷新
        </a-button>
        <a-button v-if="selectedRowKeys.length > 0" danger @click="batchDeleteTables">
          批量删除 ({{ selectedRowKeys.length }})
        </a-button>
      </div>
    </div>

    <!-- 表列表 -->
    <a-card :bordered="false" class="table-card">
      <a-table
        :columns="columns"
        :data-source="tables"
        :row-key="record => record.id"
        :row-selection="rowSelection"
        :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total) => `共 ${total} 张表` }"
        :loading="loading"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <a @click="viewColumns(record)" style="font-weight: 500">
              <TableOutlined style="margin-right: 4px" />
              {{ record.name }}
            </a>
            <a-tag v-if="record.table_type === 'view'" color="purple" style="margin-left: 8px">
              视图
            </a-tag>
          </template>

          <template v-else-if="column.key === 'comment'">
            <span style="color: var(--color-text-secondary)">{{ record.comment || '-' }}</span>
          </template>

          <template v-else-if="column.key === 'engine'">
            <a-tag v-if="record.engine" color="cyan">{{ record.engine }}</a-tag>
            <span v-else style="color: var(--color-text-secondary)">-</span>
          </template>

          <template v-else-if="column.key === 'column_count'">
            <a-tag color="blue">{{ record.column_count }}</a-tag>
          </template>

          <template v-else-if="column.key === 'updated_at'">
            <span style="color: var(--color-text-secondary)">{{ formatDate(record.updated_at) }}</span>
          </template>

          <template v-else-if="column.key === 'action'">
            <a-space>
              <a-button type="link" size="small" @click="openTableConfig(record)">
                <SettingOutlined /> 配置
              </a-button>
              <a-button type="link" size="small" @click="viewColumns(record)">
                查看字段
              </a-button>
              <a-button type="link" size="small" @click="editTable(record)">
                编辑
              </a-button>
              <a-popconfirm
                title="确定要删除这张表吗？"
                ok-text="确定"
                cancel-text="取消"
                @confirm="deleteTable(record)"
              >
                <a-button type="link" size="small" danger>
                  删除
                </a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>

    <!-- 导入进度对话框 -->
    <a-modal
      v-model:open="importDialogVisible"
      title="导入表结构"
      :closable="!importing"
      width="1200px"
    >
      <template #footer>
        <!-- 步骤1（选择表）：不显示footer -->
        <template v-if="importStep.current === 1"></template>

        <!-- 步骤4（完成）：显示取消和确认按钮 -->
        <template v-else-if="importStep.current === 3">
          <a-button @click="closeImportDialog">取消</a-button>
          <a-button type="primary" @click="closeImportDialog">确认</a-button>
        </template>

        <!-- 其他步骤：使用默认footer -->
        <template v-else>
          <a-button @click="importDialogVisible = false">取消</a-button>
        </template>
      </template>

      <a-steps :current="importStep.current" size="small">
        <a-step title="连接数据库" />
        <a-step title="选择表" />
        <a-step title="导入数据" />
        <a-step title="完成" />
      </a-steps>

      <!-- 步骤 0：连接中 -->
      <div v-if="importStep.current === 0" style="margin-top: 24px; text-align: center; padding: 60px 0">
        <LoadingOutlined style="font-size: 48px; color: #1890ff; margin-bottom: 16px" />
        <div style="font-size: 16px">{{ importProgress.message }}</div>
      </div>

      <!-- 步骤 1：选择表 -->
      <div v-if="importStep.current === 1" style="margin-top: 24px">
        <!-- 无表的情况 -->
        <div v-if="availableTables.length === 0" style="text-align: center; padding: 40px 0">
          <a-empty :description="importProgress.message" />
        </div>

        <!-- 表列表选择 -->
        <div v-else>
          <!-- 提示信息 -->
          <div style="margin-bottom: 12px">
            <span style="font-weight: 500; font-size: 14px">{{ importProgress.message }}</span>
          </div>

          <!-- 工具栏：搜索和批量操作 -->
          <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center; gap: 12px">
            <a-input
              v-model:value="searchKeyword"
              placeholder="搜索表名..."
              style="width: 200px"
              allowClear
              size="small"
            >
              <template #prefix>
                <SearchOutlined />
              </template>
            </a-input>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span v-if="selectedTables.length > 0" style="color: var(--color-primary); font-size: 12px;">
                已选 {{ selectedTables.length }} 张
              </span>
              <a-space>
                <a-button size="small" @click="selectAllTables">全选</a-button>
                <a-button size="small" @click="invertSelection">反选</a-button>
                <a-button size="small" @click="unselectAllTables">清空</a-button>
              </a-space>
            </div>
          </div>

          <a-table
            :columns="importTableColumns"
            :data-source="filteredTables"
            :row-selection="importRowSelection"
            :pagination="{ pageSize: 20, size: 'small', showSizeChanger: true, showTotal: (total) => `共 ${total} 条` }"
            :scroll="{ y: 400 }"
            size="small"
            row-key="name"
          >
            <template #bodyCell="{ column, record }">
              <template v-if="column.key === 'name'">
                <a-tag v-if="record.table_type === 'view'" color="purple" size="small">视图</a-tag>
                <span style="margin-left: 8px">{{ record.name }}</span>
              </template>

              <template v-else-if="column.key === 'comment'">
                <span style="color: var(--color-text-secondary)">{{ record.comment || '-' }}</span>
              </template>

              <template v-else-if="column.key === 'table_type'">
                <a-tag :color="record.table_type === 'BASE TABLE' ? 'blue' : 'purple'" size="small">
                  {{ record.table_type === 'BASE TABLE' ? '表' : '视图' }}
                </a-tag>
              </template>

              <template v-else-if="column.key === 'engine'">
                <span style="color: var(--color-text-secondary)">{{ record.engine || '-' }}</span>
              </template>
            </template>
          </a-table>

          <div style="margin-top: 16px; text-align: right">
            <a-button @click="importDialogVisible = false">取消</a-button>
            <a-button
              type="primary"
              @click="startImport"
              :disabled="selectedTables.length === 0"
              style="margin-left: 8px"
            >
              导入选中的 {{ selectedTables.length }} 张表
            </a-button>
          </div>
        </div>
      </div>

      <!-- 步骤 3：导入进度 -->
      <div v-if="importStep.current === 2" style="margin-top: 24px">
        <a-progress
          :percent="importProgress.percent"
          :status="importProgress.status"
          :format="() => importProgress.message"
        />

        <div v-if="importProgress.details.length > 0" style="margin-top: 16px; max-height: 300px; overflow-y: auto">
          <a-list size="small" :data-source="importProgress.details">
            <template #renderItem="{ item }">
              <a-list-item>
                <CheckCircleOutlined v-if="item.status === 'success'" style="color: #52c41a" />
                <LoadingOutlined v-else-if="item.status === 'loading'" style="color: #1890ff" />
                <CloseCircleOutlined v-else style="color: #ff4d4f" />
                <span style="margin-left: 8px">{{ item.table }} - {{ item.message }}</span>
              </a-list-item>
            </template>
          </a-list>
        </div>
      </div>

      <!-- 步骤 4：完成 -->
      <div v-if="importStep.current === 3" style="margin-top: 24px; text-align: center">
        <CheckCircleOutlined style="font-size: 48px; color: #52c41a; margin-bottom: 16px" />
        <div style="font-size: 16px; margin-bottom: 8px">导入完成！</div>
        <div style="color: var(--color-text-secondary)">成功导入 {{ importProgress.successCount }} 张表</div>
        <div v-if="importProgress.failCount > 0" style="color: #ff4d4f; margin-top: 8px">
          失败 {{ importProgress.failCount }} 张表
        </div>
      </div>
    </a-modal>

    <!-- SQL导入对话框 -->
    <a-modal
      v-model:open="sqlImportDialogVisible"
      title="从SQL导入表结构"
      width="800px"
      ok-text="导入"
      cancel-text="取消"
      @ok="importFromSql"
      @cancel="closeSqlImportDialog"
    >
      <a-form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }">
        <a-form-item label="SQL类型" required>
          <a-select v-model:value="sqlImportForm.dialect" placeholder="请选择SQL类型">
            <a-select-option value="mysql">MySQL</a-select-option>
            <a-select-option value="postgresql">PostgreSQL</a-select-option>
            <a-select-option value="sqlite">SQLite</a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="上传文件">
          <a-upload
            :before-upload="handleSqlFileUpload"
            :show-upload-list="false"
            accept=".sql"
          >
            <a-button>
              <UploadOutlined /> 选择SQL文件
            </a-button>
          </a-upload>
          <div v-if="sqlImportForm.fileName" style="margin-top: 8px; color: var(--color-text-secondary)">
            已选择: {{ sqlImportForm.fileName }}
          </div>
        </a-form-item>

        <a-form-item label="或输入SQL">
          <a-textarea
            v-model:value="sqlImportForm.sqlContent"
            :rows="12"
            placeholder="请输入CREATE TABLE语句，支持CREATE TABLE语法..."
          />
        </a-form-item>

        <a-form-item label="示例">
          <div style="background: var(--color-surface); padding: 12px; border-radius: 4px; font-size: 12px; font-family: monospace;">
CREATE TABLE users (<br>
&nbsp;&nbsp;id INT PRIMARY KEY,<br>
&nbsp;&nbsp;username VARCHAR(50) NOT NULL,<br>
&nbsp;&nbsp;email VARCHAR(100) UNIQUE,<br>
&nbsp;&nbsp;created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP<br>
);
          </div>
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 新增表对话框 -->
    <a-modal
      v-model:open="addTableDialogVisible"
      title="新增表"
      width="600px"
      ok-text="确定"
      cancel-text="取消"
      @ok="saveAddTable"
      @cancel="closeAddTableDialog"
    >
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="表名" required>
          <a-input v-model:value="addTableForm.name" placeholder="请输入表名" />
        </a-form-item>
        <a-form-item label="表类型" required>
          <a-radio-group v-model:value="addTableForm.tableType">
            <a-radio value="table">表</a-radio>
            <a-radio value="view">视图</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="引擎">
          <a-input v-model:value="addTableForm.engine" placeholder="如：InnoDB" />
        </a-form-item>
        <a-form-item label="说明">
          <a-textarea v-model:value="addTableForm.comment" :rows="3" placeholder="请输入表说明" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 编辑表对话框 -->
    <a-modal
      v-model:open="editTableDialogVisible"
      title="编辑表"
      width="600px"
      ok-text="确定"
      cancel-text="取消"
      @ok="saveEditTable"
      @cancel="closeEditTableDialog"
    >
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="表名" required>
          <a-input v-model:value="editTableForm.name" placeholder="请输入表名" />
        </a-form-item>
        <a-form-item label="表类型" required>
          <a-radio-group v-model:value="editTableForm.tableType">
            <a-radio value="table">表</a-radio>
            <a-radio value="view">视图</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item label="引擎">
          <a-input v-model:value="editTableForm.engine" placeholder="如：InnoDB" />
        </a-form-item>
        <a-form-item label="说明">
          <a-textarea v-model:value="editTableForm.comment" :rows="3" placeholder="请输入表说明" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 列详情抽屉 -->
    <a-drawer
      v-model:open="columnsDrawerVisible"
      title="表结构"
      width="900"
      placement="right"
    >
      <template #title>
        <div style="display: flex; align-items: center; gap: 8px">
          <TableOutlined />
          <span style="font-weight: 600">{{ currentTable?.name }}</span>
          <a-tag v-if="currentTable" :color="currentTable.table_type === 'table' ? 'blue' : 'purple'">
            {{ currentTable.table_type === 'table' ? '表' : '视图' }}
          </a-tag>
        </div>
      </template>

      <div v-if="currentTable" style="margin-bottom: 16px">
        <a-descriptions size="small" :column="2">
          <a-descriptions-item label="表名">{{ currentTable.name }}</a-descriptions-item>
          <a-descriptions-item label="引擎">{{ currentTable.engine || '-' }}</a-descriptions-item>
          <a-descriptions-item label="说明">{{ currentTable.comment || '-' }}</a-descriptions-item>
          <a-descriptions-item label="列数">{{ currentTable.column_count }}</a-descriptions-item>
        </a-descriptions>
      </div>

      <!-- 新增字段按钮 -->
      <div style="margin-bottom: 16px">
        <a-button type="primary" @click="showAddColumnDialog">
          <PlusOutlined /> 新增字段
        </a-button>
      </div>

      <a-table
        :columns="columnColumns"
        :data-source="currentColumns"
        :row-key="record => record.id"
        :pagination="false"
        size="small"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <a-space v-if="record.is_primary_key">
              <KeyOutlined style="color: #faad14" />
              <span style="font-weight: 600">{{ record.name }}</span>
            </a-space>
            <span v-else>{{ record.name }}</span>
          </template>

          <template v-else-if="column.key === 'data_type'">
            <a-tag color="blue">{{ record.data_type }}</a-tag>
            <span v-if="record.length" style="color: var(--color-text-secondary); margin-left: 4px">
              ({{ record.length }})
            </span>
          </template>

          <template v-else-if="column.key === 'is_nullable'">
            <a-tag :color="record.is_nullable ? 'orange' : 'green'">
              {{ record.is_nullable ? '可空' : '必填' }}
            </a-tag>
          </template>

          <template v-else-if="column.key === 'is_primary_key'">
            <a-tag v-if="record.is_primary_key" color="gold">
              <KeyOutlined /> 主键
            </a-tag>
            <span v-else style="color: var(--color-text-secondary)">-</span>
          </template>

          <template v-else-if="column.key === 'default_value'">
            <span style="color: var(--color-text-secondary)">{{ record.default_value || '-' }}</span>
          </template>

          <template v-else-if="column.key === 'comment'">
            <span style="color: var(--color-text-secondary)">{{ record.comment || '-' }}</span>
          </template>

          <template v-else-if="column.key === 'column_action'">
            <a-space>
              <a-button type="link" size="small" @click="editColumn(record)">
                编辑
              </a-button>
              <a-popconfirm
                title="确定要删除这个字段吗？"
                ok-text="确定"
                cancel-text="取消"
                @confirm="deleteColumn(record)"
              >
                <a-button type="link" size="small" danger>
                  删除
                </a-button>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-drawer>

    <!-- 新增/编辑列对话框 -->
    <a-modal
      v-model:open="columnDialogVisible"
      :title="columnDialogMode === 'add' ? '新增字段' : '编辑字段'"
      width="700px"
      ok-text="确定"
      cancel-text="取消"
      @ok="saveColumn"
      @cancel="closeColumnDialog"
    >
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="字段名" required>
          <a-input v-model:value="columnForm.name" placeholder="请输入字段名" />
        </a-form-item>
        <a-form-item label="数据类型" required>
          <a-select v-model:value="columnForm.dataType" placeholder="请选择数据类型" show-search>
            <a-select-option value="varchar">VARCHAR</a-select-option>
            <a-select-option value="char">CHAR</a-select-option>
            <a-select-option value="text">TEXT</a-select-option>
            <a-select-option value="int">INT</a-select-option>
            <a-select-option value="bigint">BIGINT</a-select-option>
            <a-select-option value="float">FLOAT</a-select-option>
            <a-select-option value="double">DOUBLE</a-select-option>
            <a-select-option value="decimal">DECIMAL</a-select-option>
            <a-select-option value="datetime">DATETIME</a-select-option>
            <a-select-option value="date">DATE</a-select-option>
            <a-select-option value="timestamp">TIMESTAMP</a-select-option>
            <a-select-option value="boolean">BOOLEAN</a-select-option>
            <a-select-option value="json">JSON</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="长度">
          <a-input-number v-model:value="columnForm.length" :min="1" :max="65535" style="width: 100%" placeholder="长度" />
        </a-form-item>
        <a-form-item label="必填">
          <a-switch v-model:checked="columnForm.isNullable" checked-children="可空" un-checked-children="必填" />
        </a-form-item>
        <a-form-item label="主键">
          <a-switch v-model:checked="columnForm.isPrimaryKey" />
        </a-form-item>
        <a-form-item label="唯一">
          <a-switch v-model:checked="columnForm.isUnique" />
        </a-form-item>
        <a-form-item label="默认值">
          <a-input v-model:value="columnForm.defaultValue" placeholder="请输入默认值" />
        </a-form-item>
        <a-form-item label="说明">
          <a-textarea v-model:value="columnForm.comment" :rows="2" placeholder="请输入字段说明" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- AI 建表抽屉 -->
    <a-drawer
      v-model:open="aiCreateTableVisible"
      title="AI 智能建表"
      placement="right"
      :width="aiDrawerWidth"
      :closable="true"
      :maskClosable="false"
    >
      <!-- 步骤指示器 -->
      <a-steps :current="aiCreateStep - 1" size="small" style="margin-bottom: 24px">
        <a-step title="输入描述" />
        <a-step title="生成 SQL" />
        <a-step title="预览字段" />
        <a-step title="完成" />
      </a-steps>

      <!-- 步骤1：选择 SQL 类型和输入描述 -->
      <div v-if="aiCreateStep === 1">
        <a-form layout="vertical">
          <a-form-item label="SQL 类型" required>
            <a-select v-model:value="aiCreateForm.sqlType" placeholder="请选择数据库类型">
              <a-select-option value="mysql">MySQL</a-select-option>
              <a-select-option value="postgresql">PostgreSQL</a-select-option>
              <a-select-option value="sqlite">SQLite</a-select-option>
            </a-select>
          </a-form-item>

          <a-form-item label="表描述" required>
            <a-textarea
              v-model:value="aiCreateForm.description"
              :rows="8"
              placeholder="请描述需要创建的表，例如：&#10;&#10;用户表：&#10;- id：主键，自增&#10;- username：用户名，唯一，不为空&#10;- email：邮箱，唯一&#10;- password：密码，加密存储&#10;- created_at：创建时间，默认当前时间&#10;- updated_at：更新时间，自动更新&#10;&#10;可以一次创建多个表，每表一行描述。"
              show-count
              :maxlength="2000"
            />
          </a-form-item>

          <a-form-item>
            <a-checkbox v-model:checked="aiCreateForm.followPreferences">
              遵循项目规范
            </a-checkbox>
            <div class="preference-hint">
              勾选后，AI 将按照项目配置的表规范生成 SQL（主键规范、审计字段、软删除等）
            </div>
          </a-form-item>

          <a-alert
            message="提示"
            description="AI 将根据您的描述自动生成建表 SQL 语句。支持一次创建多个表。"
            type="info"
            show-icon
            style="margin-bottom: 16px;"
          />

          <div style="display: flex; justify-content: flex-end; gap: 8px;">
            <a-button @click="aiCreateTableVisible = false">取消</a-button>
            <a-button type="primary" @click="generateAISQL" :loading="aiGenerating">
              生成 SQL
            </a-button>
          </div>
        </a-form>
      </div>

      <!-- 步骤2：SQL 预览和编辑 -->
      <div v-if="aiCreateStep === 2">
        <div style="margin-bottom: 12px">
          <a-alert
            :message="aiGeneratedMessage"
            :type="aiGeneratedMessage.includes('成功') ? 'success' : 'info'"
            show-icon
            closable
          />
        </div>

        <!-- Diff 视图 -->
        <div v-if="aiShowDiff && aiOriginalSQL" :style="{ height: 'calc(100vh - 280px)', display: 'flex', flexDirection: 'column' }">
          <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
            <div style="font-weight: 500; color: var(--color-text);">
              📊 SQL 差异对比
            </div>
            <a-button
              @click="aiShowDiff = false"
              size="small"
            >
              返回编辑
            </a-button>
          </div>
          <div style="flex: 1;">
            <SQLDiffEditor
              :original="aiOriginalSQL"
              :modified="aiCreateForm.generatedSQL"
              theme="light"
            />
          </div>
        </div>

        <!-- 编辑和对话界面 -->
        <div v-else :style="{ display: 'flex', gap: '16px', height: aiShowConversation ? 'calc(100vh - 280px)' : 'auto' }">
          <!-- 左侧：SQL 编辑框 -->
          <div :style="{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }">
            <div style="margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <div style="font-weight: 500; color: var(--color-text);">
                📝 SQL 编辑器
              </div>
              <a-button
                v-if="aiOriginalSQL"
                @click="aiShowDiff = true"
                size="small"
              >
                <template #icon><FileTextOutlined /></template>
                查看差异
              </a-button>
            </div>
            <div :style="{ height: aiShowConversation ? 'calc(100vh - 340px)' : '400px' }">
              <SQLEditor
                v-model="aiCreateForm.generatedSQL"
                theme="light"
              />
            </div>
          </div>

          <!-- 右侧：AI 对话界面（点击"继续优化"后显示） -->
          <div v-if="aiShowConversation" :style="{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }">
            <div style="margin-bottom: 8px; font-weight: 500; color: var(--color-text);">
              💬 AI 对话助手
            </div>

            <!-- 对话历史显示 -->
            <div style="flex: 1; overflow-y: auto; background: var(--color-bg-secondary); padding: 12px; border-radius: 6px; margin-bottom: 12px; max-height: 450px;">
              <div v-if="aiConversationHistory.length === 0" style="text-align: center; color: var(--color-text-secondary); padding: 40px 0;">
                <div style="font-size: 48px; margin-bottom: 16px;">💬</div>
                <div>开始与 AI 对话优化 SQL</div>
              </div>
              <div v-else>
                <div v-for="(msg, index) in aiConversationHistory" :key="index"
                     style="margin-bottom: 12px; padding: 10px; background: var(--color-bg-container); border-radius: 6px;">
                  <div style="font-size: 12px; color: var(--color-text-secondary); margin-bottom: 6px; font-weight: 500;">
                    {{ msg.role === 'user' ? '👤 您' : '🤖 AI 助手' }}
                  </div>
                  <div style="font-size: 13px; color: var(--color-text); white-space: pre-wrap; line-height: 1.6;">
                    {{ msg.content }}
                  </div>
                </div>
              </div>
            </div>

            <!-- 输入框 -->
            <a-input-search
              v-model:value="aiUserMessage"
              placeholder="输入修改建议，例如：给 username 字段添加索引、把 email 改为可选..."
              enter-button="发送"
              size="large"
              @search="continueAIConversation"
              :disabled="aiGenerating"
            />

            <div style="color: var(--color-text-secondary); font-size: 12px; margin-top: 8px;">
              💡 提示：描述您想要的修改，AI 会基于当前 SQL 进行优化
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div style="display: flex; justify-content: space-between; margin-top: 16px;">
          <a-button @click="aiCreateStep = 1" :disabled="aiExecuting">上一步</a-button>
          <div style="display: flex; gap: 8px;">
            <a-button v-if="!aiShowConversation" @click="aiShowConversation = true">
              <template #icon><MessageOutlined /></template>
              继续优化
            </a-button>
            <a-button type="primary" @click="parseAISQL" :loading="aiParsing">
              下一步
            </a-button>
          </div>
        </div>
      </div>

      <!-- 步骤3：字段预览 -->
      <div v-if="aiCreateStep === 3">
        <a-spin :spinning="aiParsing">
          <div v-if="aiParsedTables.length > 0">
            <a-collapse v-model:activeKey="aiCollapseActiveKey" accordion>
              <a-collapse-panel v-for="table in aiParsedTables" :key="table.name">
                <template #header>
                  <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                    <span style="font-weight: 500">
                      <TableOutlined style="margin-right: 8px" />
                      {{ table.name }}
                      <a-tag v-if="table.comment" color="blue" style="margin-left: 8px">
                        {{ table.comment }}
                      </a-tag>
                    </span>
                    <a-tag color="green">{{ table.columns?.length || 0 }} 个字段</a-tag>
                  </div>
                </template>

                <a-table
                  :columns="aiColumnColumns"
                  :data-source="table.columns || []"
                  :pagination="false"
                  size="small"
                  :scroll="{ y: 240 }"
                >
                  <template #bodyCell="{ column, record }">
                    <template v-if="column.key === 'name'">
                      <div v-if="record.isPrimaryKey" style="display: flex; align-items: center; gap: 4px;">
                        <KeyOutlined style="color: #faad14; font-size: 12px;" />
                        <span style="font-weight: 500;">{{ record.name }}</span>
                      </div>
                      <span v-else>{{ record.name }}</span>
                    </template>

                    <template v-else-if="column.key === 'dataType'">
                      <a-tag color="blue" size="small">{{ record.dataType }}</a-tag>
                    </template>

                    <template v-else-if="column.key === 'isNullable'">
                      <a-tag :color="record.isNullable ? 'green' : 'red'" size="small">
                        {{ record.isNullable ? '可空' : '必填' }}
                      </a-tag>
                    </template>

                    <template v-else-if="column.key === 'isPrimaryKey'">
                      <a-tag v-if="record.isPrimaryKey" color="orange" size="small">主键</a-tag>
                      <span v-else>-</span>
                    </template>
                  </template>
                </a-table>
              </a-collapse-panel>
            </a-collapse>

            <div style="margin-top: 16px">
              <a-alert
                :message="`即将创建 ${aiParsedTables.length} 张表，${aiParsedTables.reduce((sum, t) => sum + (t.columns?.length || 0), 0)} 个字段`"
                type="info"
                show-icon
              />
            </div>
          </div>

          <a-empty v-else description="未解析到表结构" />
        </a-spin>

        <div style="display: flex; justify-content: space-between; margin-top: 24px;">
          <a-button @click="aiCreateStep = 2" :disabled="aiExecuting">上一步</a-button>
          <a-button type="primary" @click="executeAISQL" :loading="aiExecuting">
            完成
          </a-button>
        </div>
      </div>

      <!-- 步骤4：执行结果 -->
      <div v-if="aiCreateStep === 4">
        <a-result
          :status="aiExecuteError ? 'error' : 'success'"
          :title="aiExecuteError ? '执行失败' : '执行成功'"
          :sub-title="aiExecuteResult"
        >
          <template #extra>
            <div v-if="aiExecuteError" style="margin-top: 16px; text-align: left;">
              <a-alert
                message="错误信息"
                :description="aiExecuteError"
                type="error"
                show-icon
                style="margin-bottom: 16px;"
              />
            </div>
            <div style="display: flex; justify-content: center; gap: 8px;">
              <a-button @click="aiCreateTableVisible = false">关闭</a-button>
              <a-button v-if="aiExecuteError" type="primary" @click="fixAISQL" :loading="aiFixing">
                AI 修复
              </a-button>
            </div>
          </template>
        </a-result>
      </div>
    </a-drawer>

    <!-- 表配置抽屉 -->
    <TableConfigDrawer
      v-model:open="tableConfigVisible"
      :table="currentConfigTable"
      @saved="onTableConfigSaved"
    />
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import SQLEditor from '@/components/SQLEditor.vue'
import SQLDiffEditor from '@/components/SQLDiffEditor.vue'
import {
  ImportOutlined,
  ReloadOutlined,
  TableOutlined,
  KeyOutlined,
  CheckCircleOutlined,
  LoadingOutlined,
  CloseCircleOutlined,
  SearchOutlined,
  PlusOutlined,
  FileTextOutlined,
  UploadOutlined,
  RobotOutlined,
  MessageOutlined,
  SettingOutlined
} from '@ant-design/icons-vue'
import { TableConfigDrawer } from '@/components/tableConfig'
import { message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import * as projectsApi from '../../api/projects'

const route = useRoute()
const router = useRouter()

// 状态
const loading = ref(false)
const importing = ref(false)
const project = ref(null)
const tables = ref([])

// 表格行选择
const selectedRowKeys = ref([])

const rowSelection = {
  selectedRowKeys: selectedRowKeys,
  onChange: (selectedKeys) => {
    selectedRowKeys.value = selectedKeys
  },
  columnWidth: 30,
  columnTitle: ' '
}

// 新增表相关状态
const addTableDialogVisible = ref(false)
const addTableForm = reactive({
  name: '',
  comment: '',
  engine: '',
  tableType: 'table'
})

// 编辑表相关状态
const editTableDialogVisible = ref(false)
const editTableForm = reactive({
  id: null,
  name: '',
  comment: '',
  engine: '',
  tableType: 'table'
})

// 列管理相关状态
const columnDialogVisible = ref(false)

// 表配置相关状态
const tableConfigVisible = ref(false)
const currentConfigTable = ref(null)
const columnDialogMode = ref('add') // 'add' or 'edit'
const columnForm = reactive({
  id: null,
  name: '',
  dataType: 'varchar',
  length: null,
  isNullable: true,
  isPrimaryKey: false,
  isUnique: false,
  defaultValue: '',
  comment: ''
})

// SQL导入相关状态
const sqlImportDialogVisible = ref(false)
const sqlImportForm = reactive({
  dialect: 'mysql',
  sqlContent: '',
  fileName: ''
})

// AI 建表相关状态
const aiCreateTableVisible = ref(false)
const aiCreateStep = ref(1)
const aiGenerating = ref(false)
const aiParsing = ref(false)
const aiExecuting = ref(false)
const aiFixing = ref(false)
const aiGeneratedMessage = ref('')
const aiExecuteResult = ref('')
const aiExecuteError = ref('')
const aiCollapseActiveKey = ref([])
const aiShowConversation = ref(false) // 控制是否显示对话界面
const aiShowDiff = ref(false) // 控制是否显示 diff 视图
const aiOriginalSQL = ref('') // 保存修改前的 SQL

const aiCreateForm = reactive({
  sqlType: 'mysql',
  description: '',
  generatedSQL: '',
  followPreferences: true // 遵循项目规范
})

// AI 对话历史
const aiConversationHistory = ref([])
const aiUserMessage = ref('')

// 抽屉宽度动态计算
const aiDrawerWidth = computed(() => {
  return aiShowConversation.value ? 1400 : 1000
})

const aiParsedTables = ref([])

const aiColumnColumns = [
  { title: '字段名', dataIndex: 'name', key: 'name', width: 150 },
  { title: '类型', dataIndex: 'dataType', key: 'dataType', width: 120 },
  { title: '可空', dataIndex: 'isNullable', key: 'isNullable', width: 80 },
  { title: '主键', dataIndex: 'isPrimaryKey', key: 'isPrimaryKey', width: 80 },
  { title: '说明', dataIndex: 'comment', key: 'comment', ellipsis: true }
]

// 导入相关状态
const importDialogVisible = ref(false)
const importStep = reactive({ current: 0 })
const importProgress = reactive({
  percent: 0,
  status: 'active',
  message: '',
  details: [],
  successCount: 0,
  failCount: 0
})

// 可用的表列表（从数据库读取）
const availableTables = ref([])
// 用户选中的表
const selectedTables = ref([])
const searchKeyword = ref('')

// 导入对话框表格行选择配置
const importRowSelection = {
  selectedRowKeys: selectedTables,
  onChange: (selectedKeys) => {
    selectedTables.value = selectedKeys
  }
}

// 过滤后的表列表
const filteredTables = computed(() => {
  if (!searchKeyword.value) {
    return availableTables.value
  }
  const keyword = searchKeyword.value.toLowerCase()
  return availableTables.value.filter(table =>
    table.name.toLowerCase().includes(keyword) ||
    (table.comment && table.comment.toLowerCase().includes(keyword))
  )
})

// 导入对话框的表格列定义
const importTableColumns = [
  { title: '表名', dataIndex: 'name', key: 'name', width: 150, ellipsis: true },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 60, ellipsis: true },
  { title: '类型', dataIndex: 'table_type', key: 'table_type', width: 100 },
  { title: '引擎', dataIndex: 'engine', key: 'engine', width: 100 }
]

// 列详情抽屉
const columnsDrawerVisible = ref(false)
const currentTable = ref(null)
const currentColumns = ref([])

// 表格列定义
const columns = [
  { title: '表名', dataIndex: 'name', key: 'name', width: 150, ellipsis: true },
  { title: '引擎', dataIndex: 'engine', key: 'engine', width: 100 },
  { title: '类型', dataIndex: 'table_type', key: 'table_type', width: 80 },
  { title: '列数', dataIndex: 'column_count', key: 'column_count', width: 80 },
  { title: '更新时间', dataIndex: 'updated_at', key: 'updated_at', width: 150 },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 60, ellipsis: true },
  { title: '操作', key: 'action', width: 100, fixed: 'right' }
]

const columnColumns = [
  { title: '列名', dataIndex: 'name', key: 'name', width: 120, ellipsis: true },
  { title: '类型', dataIndex: 'data_type', key: 'data_type', width: 150 },
  { title: '允许空值', dataIndex: 'is_nullable', key: 'is_nullable', width: 100 },
  { title: '主键', dataIndex: 'is_primary_key', key: 'is_primary_key', width: 100 },
  { title: '默认值', dataIndex: 'default_value', key: 'default_value', width: 120 },
  { title: '说明', dataIndex: 'comment', key: 'comment', width: 60, ellipsis: true },
  { title: '位置', dataIndex: 'ordinal_position', key: 'ordinal_position', width: 60 },
  { title: '操作', key: 'column_action', width: 120, fixed: 'right' }
]

// 加载项目信息
const loadProject = async () => {
  try {
    const projectId = parseInt(route.params.id)
    const data = await invoke('db_get_project', { id: projectId })
    const projectData = JSON.parse(data)

    // 加载数据源信息
    const datasourceData = await invoke('db_get_datasource', { id: projectData.datasource_id })
    projectData.datasource = JSON.parse(datasourceData)

    project.value = projectData
  } catch (error) {
    message.error('加载项目失败: ' + error)
  }
}

// 加载表列表
const loadTables = async () => {
  try {
    loading.value = true
    const projectId = parseInt(route.params.id)
    const data = await projectsApi.getProjectTables(projectId)
    tables.value = data
  } catch (error) {
    message.error('加载表列表失败: ' + error)
  } finally {
    loading.value = false
  }
}

// 查看列详情
const viewColumns = async (table) => {
  try {
    currentTable.value = table
    columnsDrawerVisible.value = true

    const data = await projectsApi.getTableColumns(table.id)
    currentColumns.value = data
  } catch (error) {
    message.error('加载列信息失败: ' + error)
  }
}

// 删除表
const deleteTable = async (table) => {
  try {
    await projectsApi.deleteTable(table.id)
    message.success(`表 "${table.name}" 删除成功`)
    await loadTables() // 刷新表列表
  } catch (error) {
    message.error('删除表失败: ' + error)
  }
}

// 批量删除表
const batchDeleteTables = async () => {
  if (selectedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的表')
    return
  }

  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${selectedRowKeys.value.length} 张表吗？此操作不可恢复！`,
    okText: '确定',
    cancelText: '取消',
    okType: 'danger',
    onOk: async () => {
      try {
        // 并发删除所有选中的表
        await Promise.all(
          selectedRowKeys.value.map(id => projectsApi.deleteTable(id))
        )
        message.success(`成功删除 ${selectedRowKeys.value.length} 张表`)
        selectedRowKeys.value = [] // 清空选择
        await loadTables() // 刷新表列表
      } catch (error) {
        message.error('批量删除失败: ' + error)
      }
    }
  })
}

// 新增表
const showAddTableDialog = () => {
  addTableForm.name = ''
  addTableForm.comment = ''
  addTableForm.engine = ''
  addTableForm.tableType = 'table'
  addTableDialogVisible.value = true
}

// 保存新增表
const saveAddTable = async () => {
  try {
    if (!addTableForm.name) {
      message.warning('请输入表名')
      return
    }

    const projectId = parseInt(route.params.id)
    await invoke('db_create_table', {
      projectId,
      name: addTableForm.name,
      comment: addTableForm.comment || null,
      engine: addTableForm.engine || null,
      tableType: addTableForm.tableType
    })

    message.success('表添加成功')
    closeAddTableDialog()
    await loadTables() // 刷新表列表
  } catch (error) {
    message.error('添加表失败: ' + error)
  }
}

// 关闭新增表对话框
const closeAddTableDialog = () => {
  addTableDialogVisible.value = false
  // 重置表单
  addTableForm.name = ''
  addTableForm.comment = ''
  addTableForm.engine = ''
  addTableForm.tableType = 'table'
}

// SQL导入相关方法
const showSqlImportDialog = () => {
  sqlImportForm.dialect = project.value?.datasource?.type_ || 'mysql'
  sqlImportForm.sqlContent = ''
  sqlImportForm.fileName = ''
  sqlImportDialogVisible.value = true
}

const handleSqlFileUpload = (file) => {
  const reader = new FileReader()
  reader.onload = (e) => {
    sqlImportForm.sqlContent = e.target.result
    sqlImportForm.fileName = file.name
  }
  reader.readAsText(file)
  return false // 阻止自动上传
}

const importFromSql = async () => {
  if (!sqlImportForm.sqlContent.trim()) {
    message.warning('请输入或上传SQL内容')
    return
  }

  try {
    const projectId = parseInt(route.params.id)
    const result = await projectsApi.parseSqlAndCreate(
      projectId,
      sqlImportForm.sqlContent,
      sqlImportForm.dialect
    )

    message.success(result)
    closeSqlImportDialog()
    await loadTables() // 刷新表列表
  } catch (error) {
    message.error('SQL导入失败: ' + error)
  }
}

const closeSqlImportDialog = () => {
  sqlImportDialogVisible.value = false
  // 重置表单
  sqlImportForm.dialect = 'mysql'
  sqlImportForm.sqlContent = ''
  sqlImportForm.fileName = ''
}

// 构建表规范 prompt
const buildPreferencesPrompt = (prefs) => {
  const rules = []

  // 主键规范
  if (prefs.pkEnabled) {
    rules.push(`【主键规范】必须包含主键字段：${prefs.pkFieldName || 'id'}，类型 ${prefs.pkFieldType || 'BIGINT'}${prefs.pkAutoIncrement ? '，自增' : ''}，注释"${prefs.pkComment || '主键ID'}"`)
  }

  // 审计字段
  if (prefs.auditEnabled && prefs.auditFields) {
    const auditFields = []
    if (prefs.auditFields.createdAt?.enabled) {
      auditFields.push(`${prefs.auditFields.createdAt.fieldName || 'created_at'}(${prefs.auditFields.createdAt.fieldType || 'TIMESTAMP'})`)
    }
    if (prefs.auditFields.updatedAt?.enabled) {
      auditFields.push(`${prefs.auditFields.updatedAt.fieldName || 'updated_at'}(${prefs.auditFields.updatedAt.fieldType || 'TIMESTAMP'})`)
    }
    if (prefs.auditFields.createdBy?.enabled) {
      auditFields.push(`${prefs.auditFields.createdBy.fieldName || 'created_by'}(${prefs.auditFields.createdBy.fieldType || 'BIGINT'})`)
    }
    if (prefs.auditFields.updatedBy?.enabled) {
      auditFields.push(`${prefs.auditFields.updatedBy.fieldName || 'updated_by'}(${prefs.auditFields.updatedBy.fieldType || 'BIGINT'})`)
    }
    if (auditFields.length > 0) {
      rules.push(`【审计字段】每个表必须包含：${auditFields.join('、')}`)
    }
  }

  // 软删除字段
  if (prefs.softDeleteEnabled) {
    rules.push(`【软删除】每个表必须包含软删除字段：${prefs.softDeleteField || 'deleted_at'}，类型 ${prefs.softDeleteFieldType || 'TIMESTAMP'}，允许 NULL，默认 NULL`)
  }

  // 命名规范
  if (prefs.booleanPrefix) {
    rules.push(`【命名规范】布尔字段使用前缀 "${prefs.booleanPrefix}"，如 ${prefs.booleanPrefix}active`)
  }
  if (prefs.datetimeSuffix) {
    rules.push(`【命名规范】时间字段使用后缀 "${prefs.datetimeSuffix}"，如 created${prefs.datetimeSuffix}`)
  }

  // 存储配置
  if (prefs.engineType) {
    rules.push(`【存储引擎】使用 ${prefs.engineType}`)
  }
  if (prefs.charset) {
    rules.push(`【字符集】使用 ${prefs.charset}`)
  }
  if (prefs.collation) {
    rules.push(`【排序规则】使用 ${prefs.collation}`)
  }

  if (rules.length === 0) {
    return ''
  }

  return `\n\n项目表规范（必须严格遵守）：
${rules.map((r, i) => `${i + 9}. ${r}`).join('\n')}`
}

// AI 建表相关方法
const showAICreateTableDialog = () => {
  aiCreateStep.value = 1
  aiCreateForm.sqlType = project.value?.datasource?.type_ || 'mysql'
  aiCreateForm.description = ''
  aiCreateForm.generatedSQL = ''
  aiCreateForm.followPreferences = true
  aiGeneratedMessage.value = ''
  aiParsedTables.value = []
  aiExecuteResult.value = ''
  aiExecuteError.value = ''
  aiCollapseActiveKey.value = []
  aiShowConversation.value = false // 重置对话显示状态
  aiShowDiff.value = false // 重置 diff 显示状态
  aiOriginalSQL.value = '' // 重置原始 SQL
  aiConversationHistory.value = []
  aiUserMessage.value = ''
  aiCreateTableVisible.value = true
}

// 生成 AI SQL
const generateAISQL = async () => {
  if (!aiCreateForm.description.trim()) {
    message.warning('请输入表描述')
    return
  }

  // 重置对话显示状态
  aiShowConversation.value = false
  aiShowDiff.value = false
  aiOriginalSQL.value = ''
  aiConversationHistory.value = []
  aiUserMessage.value = ''

  aiGenerating.value = true
  try {
    // 从 localStorage 读取默认服务配置
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')

    if (!defaultService.provider || !defaultService.model) {
      message.warning('请先在"设置 → AI 服务 → 默认服务"中配置默认提供商和模型')
      return
    }

    // 获取表规范配置（如果勾选了遵循规范）
    let preferencesText = ''
    if (aiCreateForm.followPreferences) {
      try {
        const projectId = parseInt(route.params.id)
        const prefsResult = await invoke('db_get_table_preferences', { projectId })
        if (prefsResult) {
          const prefs = typeof prefsResult === 'string' ? JSON.parse(prefsResult) : prefsResult
          preferencesText = buildPreferencesPrompt(prefs)
        }
      } catch (error) {
        console.warn('获取表规范配置失败，将使用默认规范:', error)
      }
    }

    // 构建初始消息
    let userPrompt = `请根据以下描述生成 ${aiCreateForm.sqlType.toUpperCase()} 建表 SQL 语句：

${aiCreateForm.description}

要求：
1. 生成标准的 ${aiCreateForm.sqlType.toUpperCase()} CREATE TABLE 语句
2. 包含所有字段的类型、约束、默认值、注释
3. 主键使用 AUTO_INCREMENT (MySQL) 或 SERIAL (PostgreSQL) 或 INTEGER PRIMARY KEY AUTOINCREMENT (SQLite)
4. 时间字段使用 TIMESTAMP 或 DATETIME
5. 创建时间字段默认值使用 CURRENT_TIMESTAMP
6. 每个表添加 ENGINE=InnoDB (仅 MySQL)
7. 支持一次创建多个表，每条 CREATE 语句用分号分隔
8. 只返回 SQL 语句，不要其他解释文字
${preferencesText}
请直接输出 SQL 语句：`

    // 初始化对话历史
    aiConversationHistory.value = [
      { role: 'user', content: userPrompt }
    ]

    // 调用后端 AI 生成命令（支持多轮对话）
    const result = await invoke('ai_generate_sql', {
      provider: defaultService.provider,
      model: defaultService.model,
      messages: aiConversationHistory.value
    })

    // 清理 SQL（移除 markdown 代码块标记）
    const cleanedSQL = result
      .replace(/```sql\n?/g, '')
      .replace(/```\n?/g, '')
      .trim()

    aiCreateForm.generatedSQL = cleanedSQL
    aiGeneratedMessage.value = `SQL 生成成功，共 ${cleanedSQL.split(';').filter(s => s.trim().toUpperCase().includes('CREATE')).length} 条 CREATE 语句`

    // 添加 AI 响应到对话历史（不包含 SQL，节省 token）
    aiConversationHistory.value.push({
      role: 'assistant',
      content: `已生成 SQL，包含 ${cleanedSQL.split(';').filter(s => s.trim().toUpperCase().includes('CREATE')).length} 个表。`
    })

    aiCreateStep.value = 2
  } catch (error) {
    message.error('生成 SQL 失败: ' + error)
    console.error('AI 生成错误:', error)
  } finally {
    aiGenerating.value = false
  }
}

// 继续和 AI 对话优化 SQL
const continueAIConversation = async () => {
  if (!aiUserMessage.value.trim()) {
    message.warning('请输入修改建议')
    return
  }

  // 立即保存并清空输入框
  const userInput = aiUserMessage.value.trim()
  aiUserMessage.value = ''

  aiGenerating.value = true
  try {
    // 从 localStorage 读取默认服务配置
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')

    if (!defaultService.provider || !defaultService.model) {
      message.warning('请先配置默认服务')
      return
    }

    // 添加用户消息到对话历史
    aiConversationHistory.value.push({
      role: 'user',
      content: userInput + `\n\n当前 SQL：\n${aiCreateForm.generatedSQL}`
    })

    // 调用后端 AI 生成命令
    const result = await invoke('ai_generate_sql', {
      provider: defaultService.provider,
      model: defaultService.model,
      messages: aiConversationHistory.value
    })

    // 清理 SQL
    const cleanedSQL = result
      .replace(/```sql\n?/g, '')
      .replace(/```\n?/g, '')
      .trim()

    // 检查是否有新的 SQL
    if (cleanedSQL && cleanedSQL.toUpperCase().includes('CREATE')) {
      // 保存当前 SQL 作为原始版本（用于 diff 对比）
      if (!aiOriginalSQL.value) {
        aiOriginalSQL.value = aiCreateForm.generatedSQL
      }

      aiCreateForm.generatedSQL = cleanedSQL
      aiGeneratedMessage.value = `SQL 已更新，共 ${cleanedSQL.split(';').filter(s => s.trim().toUpperCase().includes('CREATE')).length} 条 CREATE 语句`

      // 添加 AI 响应到对话历史
      aiConversationHistory.value.push({
        role: 'assistant',
        content: `已根据您的建议"${userInput}"更新 SQL。`
      })

      message.success('SQL 已更新，可点击"查看差异"查看修改前后对比')
    } else {
      // 如果没有返回 SQL，说明 AI 只是解释或建议
      aiGeneratedMessage.value = result

      // 添加完整响应到对话历史
      aiConversationHistory.value.push({
        role: 'assistant',
        content: result
      })

      message.info('AI 已回复，请查看上方对话历史')
    }
  } catch (error) {
    message.error('AI 生成错误: ' + error)
    console.error('AI 生成错误:', error)
  } finally {
    aiGenerating.value = false
  }
}

// 解析 AI 生成的 SQL
const parseAISQL = async () => {
  if (!aiCreateForm.generatedSQL.trim()) {
    message.warning('没有可解析的 SQL')
    return
  }

  aiParsing.value = true
  try {
    const projectId = parseInt(route.params.id)
    const result = await invoke('parse_ai_sql', {
      projectId,
      sql: aiCreateForm.generatedSQL,
      dialect: aiCreateForm.sqlType
    })

    const parsed = JSON.parse(result)
    aiParsedTables.value = parsed.tables || []

    if (aiParsedTables.value.length === 0) {
      message.warning('未解析到任何表，请检查 SQL 格式')
      return
    }

    aiCreateStep.value = 3
  } catch (error) {
    message.error('解析 SQL 失败: ' + error)
    console.error('解析错误:', error)
  } finally {
    aiParsing.value = false
  }
}

// 执行 AI 生成的 SQL
const executeAISQL = async () => {
  aiExecuting.value = true
  aiExecuteError.value = ''
  aiExecuteResult.value = ''

  try {
    const projectId = parseInt(route.params.id)
    const result = await invoke('execute_ai_sql', {
      projectId,
      sql: aiCreateForm.generatedSQL,
      dialect: aiCreateForm.sqlType
    })

    aiExecuteResult.value = result || '所有表创建成功'
    aiCreateStep.value = 4

    // 刷新表列表
    await loadTables()
  } catch (error) {
    aiExecuteError.value = error
    aiExecuteResult.value = '执行失败，请查看错误信息'
    aiCreateStep.value = 4
  } finally {
    aiExecuting.value = false
  }
}

// AI 修复 SQL
const fixAISQL = async () => {
  if (!aiExecuteError.value) {
    return
  }

  aiFixing.value = true
  try {
    // 从 localStorage 读取默认服务配置
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')

    const prompt = `以下 SQL 执行时出现错误：

${aiCreateForm.generatedSQL}

错误信息：
${aiExecuteError.value}

请分析错误原因并修复 SQL 语句。要求：
1. 保持原有的表结构和字段定义
2. 只修复导致错误的部分
3. 确保语法符合 ${aiCreateForm.sqlType.toUpperCase()} 标准
4. 只返回修复后的完整 SQL，不要其他解释

请直接输出修复后的 SQL：`

    // 调用后端 AI 修复命令
    const result = await invoke('ai_fix_sql', {
      provider: defaultService.provider,
      model: defaultService.model,
      sql: aiCreateForm.generatedSQL,
      error: aiExecuteError.value,
      dialect: aiCreateForm.sqlType
    })

    // 清理 SQL
    const cleanedSQL = result
      .replace(/```sql\n?/g, '')
      .replace(/```\n?/g, '')
      .trim()

    aiCreateForm.generatedSQL = cleanedSQL
    aiGeneratedMessage.value = 'SQL 已修复，请检查后继续执行'
    message.success('SQL 已修复')

    // 重新解析
    await parseAISQL()
    aiCreateStep.value = 3
  } catch (error) {
    message.error('修复 SQL 失败: ' + error)
    console.error('AI 修复错误:', error)
  } finally {
    aiFixing.value = false
  }
}

// 打开表配置
const openTableConfig = (table) => {
  currentConfigTable.value = table
  tableConfigVisible.value = true
}

// 表配置保存后刷新
const onTableConfigSaved = () => {
  // 可选：刷新表列表
}

// 编辑表
const editTable = (table) => {
  editTableForm.id = table.id
  editTableForm.name = table.name
  editTableForm.comment = table.comment || ''
  editTableForm.engine = table.engine || ''
  editTableForm.tableType = table.table_type
  editTableDialogVisible.value = true
}

// 保存编辑的表
const saveEditTable = async () => {
  try {
    await projectsApi.updateTable(editTableForm.id, {
      name: editTableForm.name,
      comment: editTableForm.comment,
      engine: editTableForm.engine,
      tableType: editTableForm.tableType
    })
    message.success('表信息更新成功')
    closeEditTableDialog()
    await loadTables() // 刷新表列表
  } catch (error) {
    message.error('更新表失败: ' + error)
  }
}

// 关闭编辑表对话框
const closeEditTableDialog = () => {
  editTableDialogVisible.value = false
  // 重置表单
  editTableForm.id = null
  editTableForm.name = ''
  editTableForm.comment = ''
  editTableForm.engine = ''
  editTableForm.tableType = 'table'
}

// 新增列
const showAddColumnDialog = () => {
  columnDialogMode.value = 'add'
  columnForm.id = null
  columnForm.name = ''
  columnForm.dataType = 'varchar'
  columnForm.length = null
  columnForm.isNullable = true
  columnForm.isPrimaryKey = false
  columnForm.isUnique = false
  columnForm.defaultValue = ''
  columnForm.comment = ''
  columnDialogVisible.value = true
}

// 编辑列
const editColumn = (column) => {
  columnDialogMode.value = 'edit'
  columnForm.id = column.id
  columnForm.name = column.name
  columnForm.dataType = column.data_type
  columnForm.length = column.length
  columnForm.isNullable = column.is_nullable
  columnForm.isPrimaryKey = column.is_primary_key
  columnForm.isUnique = column.is_unique
  columnForm.defaultValue = column.default_value || ''
  columnForm.comment = column.comment || ''
  columnDialogVisible.value = true
}

// 保存列
const saveColumn = async () => {
  try {
    if (columnDialogMode.value === 'add') {
      // 计算新位置
      const maxPosition = currentColumns.value.reduce((max, col) => Math.max(max, col.ordinal_position || 0), 0)
      const newPosition = maxPosition + 1

      await projectsApi.createColumn({
        tableId: currentTable.value.id,
        name: columnForm.name,
        dataType: columnForm.dataType,
        length: columnForm.length,
        isNullable: columnForm.isNullable,
        isPrimaryKey: columnForm.isPrimaryKey,
        isUnique: columnForm.isUnique,
        defaultValue: columnForm.defaultValue,
        comment: columnForm.comment,
        ordinalPosition: newPosition
      })
      message.success('字段添加成功')
    } else {
      await projectsApi.updateColumn(columnForm.id, {
        name: columnForm.name,
        dataType: columnForm.dataType,
        length: columnForm.length,
        isNullable: columnForm.isNullable,
        isPrimaryKey: columnForm.isPrimaryKey,
        isUnique: columnForm.isUnique,
        defaultValue: columnForm.defaultValue,
        comment: columnForm.comment
      })
      message.success('字段更新成功')
    }
    closeColumnDialog()
    await viewColumns(currentTable.value) // 刷新列列表
    await loadTables() // 刷新表列表（更新列数）
  } catch (error) {
    message.error('保存字段失败: ' + error)
  }
}

// 关闭列对话框
const closeColumnDialog = () => {
  columnDialogVisible.value = false
  // 重置表单
  columnForm.id = null
  columnForm.name = ''
  columnForm.dataType = 'varchar'
  columnForm.length = null
  columnForm.isNullable = true
  columnForm.isPrimaryKey = false
  columnForm.isUnique = false
  columnForm.defaultValue = ''
  columnForm.comment = ''
}

// 删除列
const deleteColumn = async (column) => {
  try {
    await projectsApi.deleteColumn(column.id)
    message.success(`字段 "${column.name}" 删除成功`)
    await viewColumns(currentTable.value) // 刷新列列表
    await loadTables() // 刷新表列表（更新列数）
  } catch (error) {
    message.error('删除字段失败: ' + error)
  }
}

// 格式化数字
const formatNumber = (num) => {
  if (!num) return '-'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

// 格式化日期
const formatDate = (dateStr) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

// 全选表
const selectAllTables = () => {
  selectedTables.value = availableTables.value.map(t => t.name)
}

// 反选
const invertSelection = () => {
  const allNames = availableTables.value.map(t => t.name)
  const currentSelected = selectedTables.value
  selectedTables.value = allNames.filter(name => !currentSelected.includes(name))
}

// 清空选择
const unselectAllTables = () => {
  selectedTables.value = []
}

// 从数据库读取可用的表列表
const fetchAvailableTables = async () => {
  try {
    const datasource = project.value.datasource
    const dbName = project.value.database_name

    // 调用后端 API 读取表列表
    let tablesData = []

    if (datasource.type_ === 'mysql') {
      const result = await invoke('cmd_fetch_mysql_tables', {
        datasourceId: datasource.id,
        databaseName: dbName
      })
      tablesData = JSON.parse(result)
    } else if (datasource.type_ === 'postgresql') {
      const result = await invoke('cmd_fetch_postgresql_tables', {
        datasourceId: datasource.id,
        databaseName: dbName
      })
      tablesData = JSON.parse(result)
    } else if (datasource.type_ === 'sqlite') {
      const result = await invoke('cmd_fetch_sqlite_tables', {
        datasourceId: datasource.id
      })
      tablesData = JSON.parse(result)
    } else {
      throw new Error(`不支持的数据源类型: ${datasource.type_}`)
    }

    availableTables.value = tablesData

    // 过滤掉已经导入的表
    const existingTableNames = new Set(tables.value.map(t => t.name))
    availableTables.value = availableTables.value.filter(t => !existingTableNames.has(t.name))
  } catch (error) {
    throw error
  }
}

// 开始导入
const startImport = async () => {
  if (selectedTables.value.length === 0) {
    importProgress.message = '请至少选择一张表'
    importProgress.status = 'exception'
    return
  }

  importing.value = true
  importStep.current = 2
  importProgress.percent = 0
  importProgress.status = 'active'
  importProgress.message = '正在导入表结构...'
  importProgress.details = []
  importProgress.successCount = 0
  importProgress.failCount = 0

  try {
    const projectId = parseInt(route.params.id)
    const datasource = project.value.datasource
    const dbName = project.value.database_name
    const tablesToImport = availableTables.value.filter(t =>
      selectedTables.value.includes(t.name)
    )

    // 调用后端 API 导入每个表
    for (let i = 0; i < tablesToImport.length; i++) {
      const table = tablesToImport[i]
      const percent = Math.round(((i + 1) / tablesToImport.length) * 100)

      importProgress.percent = percent
      importProgress.message = `正在导入 (${i + 1}/${tablesToImport.length}): ${table.name}`

      importProgress.details.push({
        table: table.name,
        status: 'loading',
        message: '正在导入...'
      })

      try {
        await invoke('cmd_import_single_table', {
          projectId,
          datasourceId: datasource.id,
          databaseName: dbName,
          tableName: table.name,
          tableComment: table.comment || null,
          tableType: table.table_type,
          engine: table.engine || null,
          rowCount: table.row_count || 0
        })

        importProgress.details[importProgress.details.length - 1].status = 'success'
        importProgress.details[importProgress.details.length - 1].message = '导入成功'
        importProgress.successCount++
      } catch (error) {
        importProgress.details[importProgress.details.length - 1].status = 'error'
        importProgress.details[importProgress.details.length - 1].message = '导入失败: ' + error
        importProgress.failCount++
      }
    }

    // 完成
    importStep.current = 3
    importProgress.percent = 100

    if (importProgress.failCount === 0) {
      importProgress.status = 'success'
      importProgress.message = '导入完成'
    } else {
      importProgress.status = 'exception'
      importProgress.message = `部分表导入失败（成功 ${importProgress.successCount}，失败 ${importProgress.failCount}）`
    }

    // 刷新表列表
    await loadTables()
  } catch (error) {
    importProgress.status = 'exception'
    importProgress.message = '导入失败: ' + error
  } finally {
    importing.value = false
  }
}

// 关闭导入对话框
const closeImportDialog = () => {
  importDialogVisible.value = false
  // 重置状态
  importStep.current = 0
  availableTables.value = []
  selectedTables.value = []
}

// 导入表结构
const importTables = async () => {
  if (!project.value?.datasource) {
    message.error('项目未关联数据源')
    return
  }

  importDialogVisible.value = true

  // 重置状态
  importStep.current = 0
  importProgress.percent = 0
  importProgress.status = 'active'
  importProgress.message = '正在连接数据库...'
  importProgress.details = []
  importProgress.successCount = 0
  importProgress.failCount = 0
  availableTables.value = []
  selectedTables.value = []

  try {
    // 步骤 1: 连接数据库
    await new Promise(resolve => setTimeout(resolve, 500))

    // 步骤 2: 读取表列表
    importStep.current = 1
    importProgress.percent = 50
    importProgress.message = '正在读取表列表...'

    await fetchAvailableTables()

    if (availableTables.value.length === 0) {
      importProgress.message = '未发现新的表（所有表已导入）'
      importProgress.status = 'exception'
    } else {
      importProgress.message = `发现 ${availableTables.value.length} 张表，请选择要导入的表`
      importProgress.status = 'active'
    }
  } catch (error) {
    importProgress.status = 'exception'
    importProgress.message = '连接失败: ' + error
  }
}

// 获取数据库标签颜色
const getDatabaseColor = (type) => {
  const colors = {
    mysql: 'blue',
    postgresql: 'cyan',
    sqlite: 'green'
  }
  return colors[type] || 'default'
}

// 获取数据库标签文本
const getDatabaseLabel = (type) => {
  const labels = {
    mysql: 'MySQL',
    postgresql: 'PostgreSQL',
    sqlite: 'SQLite'
  }
  return labels[type] || type
}

// 组件挂载时加载数据
onMounted(async () => {
  await loadProject()
  await loadTables()
})
</script>

<style scoped>
.tables-view {
  padding: var(--spacing-lg);
  min-height: calc(100vh - var(--navbar-height));
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-lg);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
}

.database-name {
  color: var(--color-text-secondary);
  font-size: 14px;
  font-family: 'Courier New', 'Consolas', monospace;
}

.header-right {
  display: flex;
  gap: var(--spacing-sm);
}

.table-card {
  margin-top: var(--spacing-md);
}

:deep(.ant-table) {
  font-size: 14px;
}

:deep(.ant-table-thead > tr > th) {
  font-weight: 600;
  background: var(--color-surface);
}

:deep(.ant-table-tbody > tr:hover > td) {
  background: var(--color-surface);
}

/* 隐藏表头的全选复选框，但保留单元格 */
:deep(.ant-table-thead > tr > th.ant-table-selection-column .ant-checkbox-wrapper) {
  display: none;
}

/* 遵循规范提示样式 */
.preference-hint {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}
</style>
