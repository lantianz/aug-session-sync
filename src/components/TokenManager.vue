<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch, h } from 'vue'
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
  NSelect,
  NNumberAnimation,
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
const pageSize = ref(10)

// 表格容器引用和高度
const tableContainerRef = ref(null)
const tableHeight = ref(600)

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

// 解析加载状态（使用 Map 记录每个 Token 的解析状态）
const parsingTokensMap = ref(new Map())

// 全部解析状态 - 使用 localStorage 持久化
const batchParsingLoading = ref(false)
const batchParsingProgress = ref({ current: 0, total: 0 })

// 批量解析时收集的更新数据
const batchUpdatedTokens = ref(new Map())

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
async function handleSearch() {
  // 重新加载本地文件以获取最新数据
  await loadTokens()
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

// 格式化时间为统一格式：YYYY-MM-DD HH:mm:ss
function formatDate(dateString) {
  if (!dateString) return '-'
  const date = new Date(dateString)

  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}

// 根据点数获取标签类型
function getCreditsTagType(credits) {
  if (credits <= 500) {
    return 'error'
  } else if (credits <= 5000) {
    return 'warning'
  }
  return 'success'
}

// 渲染点数标签
function renderCreditsTag(credits) {
  const creditsValue = credits ?? 0
  const type = getCreditsTagType(creditsValue)

  return h(
    NTag,
    { type, size: 'small' },
    {
      // 暂时注释掉动画效果,直接显示数字
      default: () => creditsValue
      // default: () => h(NNumberAnimation, {
      //   from: 0,
      //   to: creditsValue,
      //   duration: 1000,
      //   precision: 0
      // })
    }
  )
}

// 表格列定义
const columns = [
  {
    title: 'Session',
    key: 'auth_session',
    width: 120,
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
    width: 180,
    minWidth: 80,
    maxWidth: 400,
    resizable: true,
    ellipsis: {
      tooltip: true
    }
  },
  {
    title: '账号状态',
    key: 'ban_status',
    width: 100,
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
    title: '点数',
    key: 'credits',
    width: 100,
    render: (row) => renderCreditsTag(row.portal_info?.credits_balance)
  },
  {
    title: '租户 URL',
    key: 'tenant_url',
    width: 200,
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
    title: '创建时间',
    key: 'created_at',
    minWidth: 180,
    render: (row) => formatDate(row.created_at)
  },
  {
    title: '更新时间',
    key: 'updated_at',
    minWidth: 180,
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
        { size: 0 },
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
                    loading: parsingTokensMap.value.has(row.id),
                    onClick: () => handleParse(row)
                  },
                  { default: () => '解析' }
                ),
                default: () => '解析 Session'
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
async function handleParse(token, options = {}) {
  const { silent = false, skipReload = false } = options

  if (!token.auth_session) {
    if (!silent) {
      message?.warning('该 Token 没有 auth_session 字段')
    }
    return { success: false, error: '缺少 auth_session' }
  }

  // 设置该 Token 的解析状态
  parsingTokensMap.value.set(token.id, true)
  if (!silent) {
    console.log(`[单个解析] 开始解析 Token ID: ${token.id}`)
  }

  try {
    if (!silent) {
      console.log('=== 开始解析 Session ===')
      console.log('Token ID:', token.id)
      console.log('Session:', token.auth_session.substring(0, 50) + '...')
    }

    // 调用后端解析命令
    const result = await invoke('parse_session', { session: token.auth_session })
    if (!silent) {
      console.log('解析结果:', result)
    }

    // 构建更新后的 token 对象
    const updatedToken = {
      ...token,
      tenant_url: result.tenant_url || token.tenant_url,
      access_token: result.access_token || token.access_token,
      email_note: result.email || token.email_note,
      portal_info: {
        credits_balance: result.credits_balance ?? token.portal_info?.credits_balance ?? 0,
        expiry_date: result.expiry_date || token.portal_info?.expiry_date || ''
      },
      updated_at: new Date().toISOString()
    }

    // 调用后端更新命令
    await invoke('update_token', { token: updatedToken })

    if (!silent) {
      console.log('=== 解析并更新成功 ===')
      message?.success('Session 解析成功，Token 信息已更新')
    }

    // 如果不是批量解析模式,立即重新加载数据
    if (!skipReload) {
      await loadTokens()
    }

    return { success: true, updatedToken }
  } catch (error) {
    console.error(`[解析失败] Token ID: ${token.id}, 错误:`, error)
    if (!silent) {
      message?.error(`解析失败: ${error}`)
    }
    return { success: false, error: error.toString() }
  } finally {
    // 清除该 Token 的解析状态
    parsingTokensMap.value.delete(token.id)
    if (!silent) {
      console.log(`[单个解析] Token ID ${token.id} 解析状态已清除`)
    }
  }
}

// 全部解析功能
async function handleBatchParse() {
  const tokensToProcess = filteredTokens.value

  if (tokensToProcess.length === 0) {
    message?.warning('没有可解析的 Token')
    return
  }

  console.log('[批量解析] 开始批量解析,总数:', tokensToProcess.length)

  // 设置初始状态
  batchParsingLoading.value = true
  batchParsingProgress.value = { current: 0, total: tokensToProcess.length }
  batchUpdatedTokens.value.clear()

  // 保存状态到 localStorage
  saveBatchParsingState()

  let successCount = 0
  let failedCount = 0
  const concurrentLimit = 3 // 并发限制
  const queue = [...tokensToProcess]

  try {
    // 并发处理函数
    const processToken = async () => {
      while (queue.length > 0) {
        const token = queue.shift()
        if (!token) break

        // 更新进度
        const currentProgress = tokensToProcess.length - queue.length
        batchParsingProgress.value.current = currentProgress
        console.log(`[批量解析] 进度: ${currentProgress}/${tokensToProcess.length}`)

        // 解析 Token,跳过立即刷新
        const result = await handleParse(token, { silent: true, skipReload: true })

        if (result.success) {
          successCount++
          // 收集更新的 Token 数据
          if (result.updatedToken) {
            batchUpdatedTokens.value.set(token.id, result.updatedToken)
          }
        } else {
          failedCount++
        }
      }
    }

    // 创建并发任务
    const workers = Array(Math.min(concurrentLimit, tokensToProcess.length))
      .fill(null)
      .map(() => processToken())

    // 等待所有任务完成
    await Promise.all(workers)

    console.log('[批量解析] 所有任务完成,成功:', successCount, '失败:', failedCount)

    // 批量解析完成后,统一刷新一次数据
    console.log('[批量解析] 开始刷新数据...')
    await loadTokens()
    console.log('[批量解析] 数据刷新完成')

    // 显示结果
    message?.success(`解析完成: 成功 ${successCount} 个，失败 ${failedCount} 个`)
  } catch (error) {
    console.error('[批量解析] 批量解析出错:', error)
    message?.error(`批量解析出错: ${error}`)
  } finally {
    console.log('[批量解析] 清除状态...')

    // 先清除 localStorage,避免 watch 触发时重新保存
    clearBatchParsingState()

    // 然后清除内存状态
    batchParsingLoading.value = false
    batchParsingProgress.value = { current: 0, total: 0 }
    batchUpdatedTokens.value.clear()

    console.log('[批量解析] 状态已清除')
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

// 计算表格高度
function calculateTableHeight() {
  if (tableContainerRef.value) {
    const containerHeight = tableContainerRef.value.offsetHeight
    tableHeight.value = containerHeight - 72
  }
}

// 监听窗口大小变化
function handleResize() {
  calculateTableHeight()
}

// 从 localStorage 恢复批量解析状态
function restoreBatchParsingState() {
  try {
    const savedState = localStorage.getItem('batch_parsing_state')
    if (savedState) {
      const state = JSON.parse(savedState)
      // 只有在 loading 为 true 且有有效进度时才恢复状态
      if (state.loading && state.progress && state.progress.total > 0) {
        console.log('[状态恢复] 检测到未完成的批量解析:', state.progress)
        batchParsingLoading.value = state.loading
        batchParsingProgress.value = state.progress
      } else {
        console.log('[状态恢复] 检测到无效或已完成的状态,清除')
        clearBatchParsingState()
      }
    }
  } catch (error) {
    console.error('[状态恢复] 恢复批量解析状态失败:', error)
    clearBatchParsingState()
  }
}

// 保存批量解析状态到 localStorage
function saveBatchParsingState() {
  try {
    const state = {
      loading: batchParsingLoading.value,
      progress: batchParsingProgress.value,
      timestamp: Date.now() // 添加时间戳用于调试
    }
    console.log('[状态保存] 保存批量解析状态:', state)
    localStorage.setItem('batch_parsing_state', JSON.stringify(state))
  } catch (error) {
    console.error('[状态保存] 保存批量解析状态失败:', error)
  }
}

// 清除批量解析状态
function clearBatchParsingState() {
  try {
    console.log('[状态清除] 清除批量解析状态')
    localStorage.removeItem('batch_parsing_state')
  } catch (error) {
    console.error('[状态清除] 清除批量解析状态失败:', error)
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

  // 恢复批量解析状态
  restoreBatchParsingState()

  // 计算初始高度
  nextTick(() => {
    calculateTableHeight()
  })

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
})

// 组件卸载时移除监听
onUnmounted(() => {
  window.removeEventListener('resize', handleResize)

  // 如果正在批量解析,保存状态
  if (batchParsingLoading.value) {
    saveBatchParsingState()
  }
})

// 监听 API URL 变化，保存到 localStorage
watch(remoteApiUrl, (newUrl) => {
  if (newUrl) {
    localStorage.setItem('remote_api_url', newUrl)
  }
})

// 监听批量解析进度变化,实时保存 (仅在解析中时保存)
watch(batchParsingProgress, (newProgress) => {
  // 只有在 loading 状态且进度有效时才保存
  if (batchParsingLoading.value && newProgress.total > 0) {
    saveBatchParsingState()
  }
}, { deep: true })
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column; padding: 20px;">
    <!-- 顶部工具栏 -->
    <NCard size="small" style="flex-shrink: 0;">
      <NSpace :size="12" align="center">
        <NInput
          v-model:value="searchKeyword"
          placeholder="搜索 Token / 邮箱 / Session"
          style="width: 300px;"
          clearable
          @keyup.enter="handleSearch"
        />
        <NSelect
          v-model:value="sortOption"
          :options="sortOptions"
          style="width: 180px;"
        />
        <NButton type="primary" @click="handleSearch">
          检索
        </NButton>
      </NSpace>
      <NSpace :size="12" align="center" style="margin-top: 8px;">
        <NButton @click="showRemoteDialog = true">
          远端加载
        </NButton>
        <NButton
          type="info"
          :loading="batchParsingLoading"
          :disabled="filteredTokens.length === 0"
          @click="handleBatchParse"
        >
          {{ batchParsingLoading ? `正在解析 ${batchParsingProgress.current}/${batchParsingProgress.total}` : '全部解析' }}
        </NButton>
        <NButton disabled>
          同步 ATM
        </NButton>
      </NSpace>
    </NCard>

    <!-- 数据表格 -->
    <div
      ref="tableContainerRef"
      style="flex: 1; margin-top: 16px; min-height: 0; display: flex; flex-direction: column;"
    >
      <NCard
        size="small"
        :content-style="{ padding: '16px' }"
        style="height: 100%;"
      >
        <NDataTable
          :columns="columns"
          :data="paginatedTokens"
          :loading="loading"
          :bordered="false"
          :single-line="false"
          :scroll-x="1580"
          :max-height="tableHeight"
        />
      </NCard>
    </div>

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
            <component :is="() => renderCreditsTag(currentDetailToken.portal_info?.credits_balance)" />
          </NDescriptionsItem>
          <NDescriptionsItem label="过期时间">
            <span style="font-family: Consolas, monospace;">{{ formatDate(currentDetailToken.portal_info?.expiry_date) }}</span>
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
            <div style="display: flex; align-items: center;">
              <component :is="() => renderCreditsTag(currentEditToken.portal_info?.credits_balance)" />
            </div>
          </NFormItem>
          <NFormItem label="过期时间">
            <NInput :value="formatDate(currentEditToken.portal_info?.expiry_date)" disabled style="font-family: Consolas, monospace;" />
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

