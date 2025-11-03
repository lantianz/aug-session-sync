<script setup>
import { ref, computed, onMounted, watch, h } from 'vue'
import {
  NCard,
  NSpace,
  NInput,
  NButton,
  NDataTable,
  NPagination,
  NModal,
  NIcon,
  NTooltip,
  NTag,
  NForm,
  NFormItem,
  NDescriptions,
  NDescriptionsItem,
  useMessage
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()

// 数据状态
const tokens = ref([])
const loading = ref(false)
const searchKeyword = ref('')
const sortOption = ref('created_at_desc')
const currentPage = ref(1)
const pageSize = ref(15)

// 远端加载对话框
const showRemoteDialog = ref(false)
const remoteApiUrl = ref('')
const remoteLoading = ref(false)

// 详情对话框
const showDetailDialog = ref(false)
const currentDetailToken = ref(null)

// 编辑对话框
const showEditDialog = ref(false)
const currentEditToken = ref(null)
const editFormData = ref({})
const editLoading = ref(false)

// 解析加载状态（记录正在解析的 Token ID）
const parsingTokenId = ref(null)

// 排序选项
const sortOptions = [
  { label: '创建时间（降序）', value: 'created_at_desc' },
  { label: '创建时间（升序）', value: 'created_at_asc' },
  { label: '点数（降序）', value: 'credits_desc' },
  { label: '点数（升序）', value: 'credits_asc' }
]

// 加载 tokens
async function loadTokens() {
  loading.value = true
  try {
    const data = await invoke('read_tokens')
    tokens.value = data
  } catch (error) {
    message?.error(`加载失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 搜索过滤
const filteredTokens = computed(() => {
  let result = tokens.value

  // 搜索过滤
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(token => {
      return (
        token.access_token?.toLowerCase().includes(keyword) ||
        token.email_note?.toLowerCase().includes(keyword) ||
        token.auth_session?.toLowerCase().includes(keyword)
      )
    })
  }

  // 排序
  result = [...result]
  switch (sortOption.value) {
    case 'created_at_desc':
      result.sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
      break
    case 'created_at_asc':
      result.sort((a, b) => new Date(a.created_at) - new Date(b.created_at))
      break
    case 'credits_desc':
      result.sort((a, b) => {
        const aCredits = a.portal_info?.credits_balance || 0
        const bCredits = b.portal_info?.credits_balance || 0
        return bCredits - aCredits
      })
      break
    case 'credits_asc':
      result.sort((a, b) => {
        const aCredits = a.portal_info?.credits_balance || 0
        const bCredits = b.portal_info?.credits_balance || 0
        return aCredits - bCredits
      })
      break
  }

  return result
})

// 分页数据
const paginatedTokens = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredTokens.value.slice(start, end)
})

// 总页数
const totalPages = computed(() => {
  return Math.ceil(filteredTokens.value.length / pageSize.value)
})

// 执行搜索
function handleSearch() {
  currentPage.value = 1
}

// 复制到剪贴板
async function copyToClipboard(text) {
  try {
    await navigator.clipboard.writeText(text)
    message?.success('复制成功')
  } catch (error) {
    message?.error(`复制失败: ${error}`)
  }
}

// 截断文本
function truncateText(text, maxLength = 30) {
  if (!text) return '-'
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

// 格式化时间
function formatDate(dateString) {
  if (!dateString) return '-'
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 表格列定义
const columns = [
  {
    title: 'Session',
    key: 'auth_session',
    width: 200,
    ellipsis: {
      tooltip: true
    },
    render: (row) => {
      return h(
        'span',
        {
          style: 'cursor: pointer; color: #18a058;',
          onClick: () => copyToClipboard(row.auth_session)
        },
        truncateText(row.auth_session, 30)
      )
    }
  },
  {
    title: '邮箱',
    key: 'email_note',
    width: 200,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: '账号状态',
    key: 'ban_status',
    width: 120,
    render: (row) => {
      const status = row.ban_status || ''
      let type = 'default'
      let text = '未知'

      switch (status) {
        case 'ACTIVE':
          type = 'success'
          text = '正常'
          break
        case 'BANNED':
          type = 'error'
          text = '已封禁'
          break
        case 'UNKNOWN':
          type = 'default'
          text = '未知'
          break
        default:
          type = 'default'
          text = '未知'
      }

      return h(NTag, { type, size: 'small' }, { default: () => text })
    }
  },
  {
    title: '租户 URL',
    key: 'tenant_url',
    width: 250,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: '访问令牌',
    key: 'access_token',
    width: 200,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: '点数',
    key: 'credits',
    width: 100,
    render: (row) => row.portal_info?.credits_balance || '-'
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 150,
    render: (row) => formatDate(row.created_at)
  },
  {
    title: '更新时间',
    key: 'updated_at',
    width: 150,
    render: (row) => formatDate(row.updated_at)
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    fixed: 'right',
    render: (row) => {
      return h(
        NSpace,
        { size: 8 },
        {
          default: () => [
            h(
              NTooltip,
              {},
              {
                trigger: () => h(
                  NButton,
                  {
                    text: true,
                    size: 'small',
                    onClick: () => showDetail(row)
                  },
                  { default: () => '详情' }
                ),
                default: () => '查看完整信息'
              }
            ),
            h(
              NTooltip,
              {},
              {
                trigger: () => h(
                  NButton,
                  {
                    text: true,
                    type: 'primary',
                    size: 'small',
                    onClick: () => showEdit(row)
                  },
                  { default: () => '编辑' }
                ),
                default: () => '编辑 Token 信息'
              }
            ),
            h(
              NTooltip,
              {},
              {
                trigger: () => h(
                  NButton,
                  {
                    text: true,
                    type: 'info',
                    size: 'small',
                    loading: parsingTokenId.value === row.id,
                    onClick: () => handleParse(row)
                  },
                  { default: () => '解析' }
                ),
                default: () => '重新解析 Session'
              }
            )
          ]
        }
      )
    }
  }
]

// 显示详情
function showDetail(token) {
  currentDetailToken.value = token
  showDetailDialog.value = true
}

// 显示编辑对话框
function showEdit(token) {
  currentEditToken.value = token
  // 复制可编辑字段到表单数据（只包括可编辑的字段）
  editFormData.value = {
    tenant_url: token.tenant_url || '',
    access_token: token.access_token || '',
    email_note: token.email_note || '',
    portal_url: token.portal_url || ''
  }
  showEditDialog.value = true
}

// 保存编辑
async function saveEdit() {
  if (!currentEditToken.value) return

  editLoading.value = true
  try {
    // 构建更新后的 token 对象
    const updatedToken = {
      ...currentEditToken.value,
      ...editFormData.value,
      updated_at: new Date().toISOString()
    }

    // 调用后端更新命令
    await invoke('update_token', { token: updatedToken })

    message?.success('更新成功')
    showEditDialog.value = false

    // 重新加载数据
    await loadTokens()
  } catch (error) {
    message?.error(`更新失败: ${error}`)
  } finally {
    editLoading.value = false
  }
}

// 解析 Session
async function handleParse(token) {
  if (!token.auth_session) {
    message?.warning('该 Token 没有 auth_session 字段')
    return
  }

  parsingTokenId.value = token.id
  try {
    console.log('=== 开始重新解析 Session ===')
    console.log('Token ID:', token.id)
    console.log('Session:', token.auth_session.substring(0, 50) + '...')

    // 调用后端解析命令
    const result = await invoke('parse_session', { session: token.auth_session })
    console.log('解析结果:', result)

    // 构建更新后的 token 对象
    const updatedToken = {
      ...token,
      tenant_url: result.tenant_url || token.tenant_url,
      access_token: result.access_token || token.access_token,
      email_note: result.email || token.email_note,
      portal_info: {
        credits_balance: result.credits_balance || token.portal_info?.credits_balance || 0,
        expiry_date: result.expiry_date || token.portal_info?.expiry_date || ''
      },
      updated_at: new Date().toISOString()
    }

    // 调用后端更新命令
    await invoke('update_token', { token: updatedToken })

    console.log('=== 解析并更新成功 ===')
    message?.success('Session 解析成功，Token 信息已更新')

    // 重新加载数据
    await loadTokens()
  } catch (error) {
    console.error('解析失败:', error)
    message?.error(`解析失败: ${error}`)
  } finally {
    parsingTokenId.value = null
  }
}

// 从远端 API 导入
async function handleRemoteImport() {
  if (!remoteApiUrl.value.trim()) {
    message?.warning('请输入远端 API 地址')
    return
  }

  console.log('=== 开始远端导入 ===')
  console.log('API 地址:', remoteApiUrl.value)

  remoteLoading.value = true
  try {
    console.log('步骤1: 调用后端 import_from_remote 命令...')
    const result = await invoke('import_from_remote', { apiUrl: remoteApiUrl.value })

    console.log('步骤2: 后端返回结果:')
    console.log('  - 导入记录数:', result.imported)
    console.log('  - 跳过记录数:', result.skipped)
    console.log('  - 完整结果:', result)

    console.log('=== 远端导入成功 ===')
    message?.success(`成功导入 ${result.imported} 条记录，跳过 ${result.skipped} 条重复记录`)
    showRemoteDialog.value = false
    // 不清空 API URL，保持用户输入

    console.log('步骤3: 重新加载本地 tokens...')
    await loadTokens()
    console.log('本地 tokens 已更新')
  } catch (error) {
    console.log('=== 远端导入失败 ===')
    console.error('错误信息:', error)
    console.error('错误详情:', error.toString())
    message?.error(`导入失败: ${error}`)
  } finally {
    remoteLoading.value = false
  }
}

// 组件挂载时加载数据
onMounted(() => {
  loadTokens()

  // 从 localStorage 恢复 API URL
  const savedApiUrl = localStorage.getItem('remote_api_url')
  if (savedApiUrl) {
    remoteApiUrl.value = savedApiUrl
  }
})

// 监听 API URL 变化，保存到 localStorage
watch(remoteApiUrl, (newUrl) => {
  if (newUrl) {
    localStorage.setItem('remote_api_url', newUrl)
  }
})
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column; padding: 20px;">
    <!-- 顶部工具栏 -->
    <NCard size="small" style="flex-shrink: 0;">
      <NSpace :size="12">
        <NInput
          v-model:value="searchKeyword"
          placeholder="搜索 Token / 邮箱 / Session"
          style="width: 300px;"
          clearable
          @keyup.enter="handleSearch"
        />
        <NButton type="primary" @click="handleSearch">
          检索
        </NButton>
        <NSelect
          v-model:value="sortOption"
          :options="sortOptions"
          style="width: 180px;"
        />
        <NButton @click="showRemoteDialog = true">
          远端加载
        </NButton>
        <NButton disabled>
          同步 ATM
        </NButton>
      </NSpace>
    </NCard>

    <!-- 数据表格 -->
    <NCard size="small" style="flex: 1; margin-top: 16px; display: flex; flex-direction: column; overflow: hidden;">
      <div style="flex: 1; overflow: hidden;">
        <NDataTable
          :columns="columns"
          :data="paginatedTokens"
          :loading="loading"
          :bordered="false"
          :single-line="false"
          :scroll-x="1300"
          :max-height="'100%'"
          style="height: 100%;"
        />
      </div>
    </NCard>

    <!-- 分页 -->
    <NSpace justify="center" style="flex-shrink: 0; margin-top: 16px;">
      <NPagination
        v-model:page="currentPage"
        :page-count="totalPages"
        :page-size="pageSize"
        show-size-picker
        :page-sizes="[10, 15, 20, 50]"
        @update:page-size="(size) => { pageSize = size; currentPage = 1 }"
      >
        <template #prefix>
          共 {{ filteredTokens.length }} 条记录
        </template>
      </NPagination>
    </NSpace>

    <!-- 远端加载对话框 -->
    <NModal
      v-model:show="showRemoteDialog"
      preset="dialog"
      title="从远端 API 导入"
      positive-text="确定"
      negative-text="取消"
      :loading="remoteLoading"
      @positive-click="handleRemoteImport"
    >
      <NSpace vertical :size="16" style="margin-top: 16px;">
        <NInput
          v-model:value="remoteApiUrl"
          placeholder="请输入远端 API 地址"
          clearable
        />
        <NCard size="small" style="background-color: #1a1a1e; border: 1px solid #2a2a2e;">
          <NSpace :size="8" align="center">
            <NIcon :size="20" color="#63e2b7">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path fill="currentColor" d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z"/>
              </svg>
            </NIcon>
            <div style="font-size: 13px; line-height: 1.6; color: #e0e0e0;">
              <div style="font-weight: 600; margin-bottom: 4px; color: #ffffff;">API 规范：</div>
              <div>• 请求方式：GET</div>
              <div>• 返回格式：JSON</div>
              <div>• 返回结构：{ "status": 1, "data": [...] }</div>
              <div style="color: #a0a0a0;">（status: 1=成功，0=失败）</div>
            </div>
          </NSpace>
        </NCard>
      </NSpace>
    </NModal>

    <!-- 详情对话框 -->
    <NModal
      v-model:show="showDetailDialog"
      preset="card"
      title="Token 详细信息"
      style="width: 700px;"
      :bordered="false"
      :segmented="{ content: true }"
      transform-origin="center"
    >
      <div style="max-height: 60vh; overflow-y: auto; font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;">
        <NDescriptions
          v-if="currentDetailToken"
          :column="1"
          label-placement="left"
          bordered
          size="small"
          label-style="width: 140px; font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;"
        >
          <NDescriptionsItem label="ID">
            <span style="font-family: Consolas, monospace;">{{ currentDetailToken.id }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="邮箱">
            <span style="font-family: Consolas, monospace;">{{ currentDetailToken.email_note || '-' }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="Session">
            <div style="word-break: break-all; font-family: Consolas, monospace; font-size: 12px;">
              {{ currentDetailToken.auth_session }}
            </div>
          </NDescriptionsItem>
          <NDescriptionsItem label="租户 URL">
            <div style="word-break: break-all; font-family: Consolas, monospace;">
              {{ currentDetailToken.tenant_url || '-' }}
            </div>
          </NDescriptionsItem>
          <NDescriptionsItem label="访问令牌">
            <div style="word-break: break-all; font-family: Consolas, monospace; font-size: 12px;">
              {{ currentDetailToken.access_token || '-' }}
            </div>
          </NDescriptionsItem>
          <NDescriptionsItem label="Portal URL">
            <div style="word-break: break-all; font-family: Consolas, monospace;">
              {{ currentDetailToken.portal_url || '-' }}
            </div>
          </NDescriptionsItem>
          <NDescriptionsItem label="账号状态">
            <NTag
              :type="currentDetailToken.ban_status === 'ACTIVE' ? 'success' : currentDetailToken.ban_status === 'BANNED' ? 'error' : 'default'"
              size="small"
            >
              {{ currentDetailToken.ban_status === 'ACTIVE' ? '正常' : currentDetailToken.ban_status === 'BANNED' ? '已封禁' : '未知' }}
            </NTag>
          </NDescriptionsItem>
          <NDescriptionsItem label="点数余额">
            <span style="font-family: Consolas, monospace;">{{ currentDetailToken.portal_info?.credits_balance || '-' }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="过期时间">
            <span style="font-family: Consolas, monospace;">{{ currentDetailToken.portal_info?.expiry_date || '-' }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="标签名称">
            <span style="font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;">{{ currentDetailToken.tag_name || '-' }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="标签颜色">
            <div v-if="currentDetailToken.tag_color" style="display: flex; align-items: center; gap: 8px;">
              <div :style="{ width: '20px', height: '20px', backgroundColor: currentDetailToken.tag_color, border: '1px solid #ccc' }"></div>
              <span style="font-family: Consolas, monospace;">{{ currentDetailToken.tag_color }}</span>
            </div>
            <span v-else>-</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="跳过检查">
            {{ currentDetailToken.skip_check ? '是' : '否' }}
          </NDescriptionsItem>
          <NDescriptionsItem label="创建时间">
            <span style="font-family: Consolas, monospace;">{{ formatDate(currentDetailToken.created_at) }}</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="更新时间">
            <span style="font-family: Consolas, monospace;">{{ formatDate(currentDetailToken.updated_at) }}</span>
          </NDescriptionsItem>
        </NDescriptions>
      </div>
    </NModal>

    <!-- 编辑对话框 -->
    <NModal
      v-model:show="showEditDialog"
      preset="card"
      title="编辑 Token 信息"
      style="width: 700px;"
      :bordered="false"
      :segmented="{ content: true, footer: 'soft' }"
      transform-origin="center"
    >
      <div style="max-height: 60vh; overflow-y: auto; padding: 0 24px; font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;">
        <NForm
          v-if="currentEditToken"
          :model="editFormData"
          label-placement="left"
          label-width="90px"
        >
          <NFormItem label="ID">
            <NInput :value="currentEditToken.id" disabled style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="Session">
            <NInput :value="currentEditToken.auth_session" disabled type="textarea" :rows="4" style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="创建时间">
            <NInput :value="formatDate(currentEditToken.created_at)" disabled style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="邮箱">
            <NInput v-model:value="editFormData.email_note" placeholder="请输入邮箱" style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="租户 URL">
            <NInput v-model:value="editFormData.tenant_url" placeholder="请输入租户 URL" style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="访问令牌">
            <NInput v-model:value="editFormData.access_token" placeholder="请输入访问令牌" style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="Portal URL">
            <NInput v-model:value="editFormData.portal_url" placeholder="请输入 Portal URL" style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="账号状态">
            <NInput :value="currentEditToken.ban_status === 'ACTIVE' ? '正常' : currentEditToken.ban_status === 'BANNED' ? '已封禁' : '未知'" disabled style="font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;" />
          </NFormItem>
          <NFormItem label="点数余额">
            <NInput :value="String(currentEditToken.portal_info?.credits_balance || 0)" disabled style="font-family: Consolas, monospace;" />
          </NFormItem>
          <NFormItem label="过期时间">
            <NInput :value="currentEditToken.portal_info?.expiry_date || '-'" disabled style="font-family: Consolas, monospace;" />
          </NFormItem>
        </NForm>
      </div>
      <template #footer>
        <NSpace justify="end">
          <NButton @click="showEditDialog = false">取消</NButton>
          <NButton type="primary" :loading="editLoading" @click="saveEdit">保存</NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

