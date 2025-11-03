<script setup>
import { ref, h, defineComponent, onMounted, watch } from 'vue'
import { NCard, NInput, NButton, NSpace, NGrid, NGridItem, NIcon, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()
const url = ref('')
const textContent = ref('')
const loading = ref(false)
const isParsed = ref(false)
const parsedData = ref(null)

// 组件挂载时从 localStorage 恢复解析文本
onMounted(() => {
  const savedText = localStorage.getItem('parser_session_text')
  if (savedText) {
    textContent.value = savedText
  }
})

// 监听文本内容变化，保存到 localStorage
watch(textContent, (newText) => {
  localStorage.setItem('parser_session_text', newText)
})

// 获取文本内容
async function fetchText() {
  if (!url.value.trim()) {
    message?.warning('请输入URL')
    return
  }

  loading.value = true
  try {
    const text = await invoke('fetch_text_from_url', { url: url.value })
    textContent.value = text
    // 重置解析状态
    isParsed.value = false
    parsedData.value = null
    message?.success('文本获取成功')
  } catch (error) {
    message?.error(`获取失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 复制到剪贴板
async function copyToClipboard() {
  if (!textContent.value.trim()) {
    message?.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(textContent.value)
    message?.success('复制成功')
  } catch (error) {
    message?.error(`复制失败: ${error}`)
  }
}

// 解析 Session 并提取 Token
async function parseContent() {
  console.log('=== 开始解析 Session ===')
  console.log('输入数据 (Session):', textContent.value)

  if (!textContent.value.trim()) {
    console.log('解析失败: Session 为空')
    message?.warning('请先获取 Session 内容')
    isParsed.value = false
    return
  }

  loading.value = true
  try {
    console.log('步骤2: 调用后端 parse_session 命令...')
    const session = textContent.value.trim()
    
    const result = await invoke('parse_session', { session })
    console.log('步骤3: 后端返回结果:', result)

    // 步骤4: 构建 Token 数据对象（使用下划线命名）
    console.log('步骤4: 构建 Token 数据对象')
    const tokenData = {
      tenant_url: result.tenant_url,
      access_token: result.access_token,
      portal_url: null,
      email_note: result.email || null,
      auth_session: session,
      suspensions: null,
      credits_balance: result.credits_balance || null,
      expiry_date: result.expiry_date || null,
      ban_status: 'ACTIVE'
    }
    console.log('Token 数据:', tokenData)

    // 步骤5: 将 Token 数据转换为 JSON 字符串，显示在文本框中
    console.log('步骤5: 将 Token 数据转换为 JSON')
    const jsonString = JSON.stringify(tokenData, null, 2)
    textContent.value = jsonString
    console.log('JSON 字符串长度:', jsonString.length)

    // 步骤6: 保存解析结果
    console.log('步骤6: 保存解析结果')
    parsedData.value = tokenData
    isParsed.value = true
    
    console.log('=== 解析成功 ===')
    console.log('解析后的数据:', parsedData.value)
    console.log('isParsed 状态:', isParsed.value)
    
    message?.success('Session 解析成功！已提取 Token 信息')
  } catch (error) {
    console.log('=== 解析失败 ===')
    console.error('错误信息:', error)
    
    isParsed.value = false
    parsedData.value = null
    
    let errorMessage = error.toString()
    if (errorMessage.includes('SESSION_ERROR_OR_ACCOUNT_BANNED')) {
      errorMessage = 'Session 无效或账号已被封禁'
    }
    
    message?.error(`解析失败: ${errorMessage}`)
  } finally {
    loading.value = false
  }
}

// 保存到 Token 管理
async function saveToTokenManager() {
  if (!parsedData.value) {
    message?.warning('请先解析 Session')
    return
  }

  try {
    // 生成唯一 ID
    const id = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15)
    const now = new Date().toISOString()
    
    const tokenRecord = {
      id,
      tenant_url: parsedData.value.tenant_url,
      access_token: parsedData.value.access_token,
      created_at: now,
      updated_at: now,
      portal_url: parsedData.value.portal_url,
      ban_status: parsedData.value.ban_status,
      portal_info: parsedData.value.credits_balance || parsedData.value.expiry_date ? {
        credits_balance: parsedData.value.credits_balance,
        expiry_date: parsedData.value.expiry_date
      } : null,
      email_note: parsedData.value.email_note,
      tag_name: null,
      tag_color: null,
      auth_session: parsedData.value.auth_session,
      suspensions: parsedData.value.suspensions,
      skip_check: false,
      balance_color_mode: null
    }

    await invoke('add_token', { token: tokenRecord })
    message?.success('已保存到 Token 管理')
  } catch (error) {
    message?.error(`保存失败: ${error}`)
  }
}
</script>

<template>
  <div style="padding: 20px;">
    <NSpace vertical :size="16">
      <!-- URL 输入和获取 -->
      <NCard title="获取 Session" size="small">
        <NSpace vertical :size="12">
          <NInput
            v-model:value="url"
            placeholder="请输入包含 Session 的 URL"
            :disabled="loading"
            @keyup.enter="fetchText"
          />
          <NButton
            type="primary"
            :loading="loading"
            @click="fetchText"
            block
          >
            获取文本
          </NButton>
        </NSpace>
      </NCard>

      <!-- 文本内容显示 -->
      <NCard title="文本内容" size="small">
        <template #header-extra>
          <NButton
            text
            @click="copyToClipboard"
            size="small"
            title="复制内容"
          >
            <NIcon :size="18">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d="M19,21H8V7H19M19,5H8A2,2 0 0,0 6,7V21A2,2 0 0,0 8,23H19A2,2 0 0,0 21,21V7A2,2 0 0,0 19,5M16,1H4A2,2 0 0,0 2,3V17H4V3H16V1Z"
                />
              </svg>
            </NIcon>
          </NButton>
        </template>
        <NInput
          v-model:value="textContent"
          type="textarea"
          placeholder="获取的文本内容将显示在这里"
          :rows="10"
          :disabled="loading"
        />
      </NCard>

      <!-- 操作按钮 -->
      <NSpace justify="center" :size="12">
        <NButton
          type="primary"
          @click="parseContent"
          :loading="loading"
          size="medium"
        >
          解析
        </NButton>
        <NButton
          type="success"
          @click="saveToTokenManager"
          :disabled="!isParsed"
          size="medium"
        >
          保存到 Token 管理
        </NButton>
      </NSpace>
    </NSpace>
  </div>
</template>

