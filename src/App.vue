<script setup>
import { ref, onMounted, onUnmounted, defineComponent, h, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  NSpace,
  NCard,
  NInput,
  NButton,
  NModal,
  NGrid,
  NGi,
  NCode,
  useMessage
} from 'naive-ui'
import AppLayout from './components/layout/AppLayout.vue'

// 状态管理
const url = ref('')
const textContent = ref('')
const filePath = ref('')
const loading = ref(false)
const previewVisible = ref(false)
const beforeJson = ref('')
const afterJson = ref('')

let message = null

// 设置 message 实例
function setMessage(msg) {
  message = msg
}

// 保存配置到本地
async function saveConfig() {
  try {
    await invoke('save_config', {
      config: {
        url: url.value,
        file_path: filePath.value
      }
    })
  } catch (error) {
    console.error('保存配置失败:', error)
  }
}

// 加载配置
async function loadConfig() {
  try {
    const config = await invoke('load_config')
    if (config.url) {
      url.value = config.url
    }
    if (config.file_path) {
      filePath.value = config.file_path
    }
  } catch (error) {
    console.error('加载配置失败:', error)
  }
}

// 监听配置变化，自动保存
watch([url, filePath], () => {
  saveConfig()
}, { deep: true })

// 获取文本内容
async function fetchText() {
  if (!url.value.trim()) {
    message?.warning('请输入URL')
    return
  }

  loading.value = true
  try {
    const text = await invoke('fetch_text_from_url', { url: url.value.trim() })
    textContent.value = text
    message?.success('文本获取成功')
  } catch (error) {
    message?.error(`获取失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 选择文件
async function selectFile() {
  try {
    const selected = await invoke('select_file')
    if (selected) {
      filePath.value = selected
    }
  } catch (error) {
    message?.error(`选择文件失败: ${error}`)
  }
}

// 预览对比
async function showPreview() {
  if (!textContent.value.trim()) {
    message?.warning('请先获取文本内容')
    return
  }

  if (!filePath.value.trim()) {
    message?.warning('请选择或输入文件路径')
    return
  }

  try {
    // 解析文本为JSON对象
    let newData
    try {
      newData = JSON.parse(textContent.value)
    } catch (e) {
      message?.error('文本内容不是有效的JSON格式')
      return
    }

    // 读取现有文件内容
    const existingContent = await invoke('read_json_file', { filePath: filePath.value })
    beforeJson.value = JSON.stringify(JSON.parse(existingContent), null, 2)

    // 生成写入后的内容
    const existingArray = JSON.parse(existingContent)
    existingArray.push(newData)
    afterJson.value = JSON.stringify(existingArray, null, 2)

    previewVisible.value = true
  } catch (error) {
    message?.error(`预览失败: ${error}`)
  }
}

// 写入文件
async function writeToFile() {
  if (!textContent.value.trim()) {
    message?.warning('请先获取文本内容')
    return
  }

  if (!filePath.value.trim()) {
    message?.warning('请选择或输入文件路径')
    return
  }

  try {
    // 解析文本为JSON对象
    let newData
    try {
      newData = JSON.parse(textContent.value)
    } catch (e) {
      message?.error('文本内容不是有效的JSON格式')
      return
    }

    // 写入文件
    await invoke('append_to_json_file', {
      filePath: filePath.value,
      newData: newData
    })

    message?.success('写入成功')

    // 清空文本内容
    textContent.value = ''
  } catch (error) {
    message?.error(`写入失败: ${error}`)
  }
}

// 初始化
onMounted(async () => {
  // 加载配置
  await loadConfig()

  // 如果配置中没有文件路径，则获取默认路径
  if (!filePath.value) {
    try {
      const defaultPath = await invoke('get_default_file_path')
      filePath.value = defaultPath
    } catch (error) {
      console.error('获取默认路径失败:', error)
    }
  }
})

// 组件卸载时保存配置
onUnmounted(() => {
  saveConfig()
})

// 内部组件 - 在 NMessageProvider 内部使用 useMessage
const AppContent = defineComponent({
  setup() {
    const msg = useMessage()
    setMessage(msg)

    return () => h(NSpace, { vertical: true, size: 12 }, () => [
      // URL输入和文件路径 - 合并到一行
      h(NGrid, { cols: 2, xGap: 12 }, () => [
        // URL输入区域
        h(NGi, {}, () => [
          h(NCard, { title: 'URL输入', size: 'small' }, () => [
            h(NSpace, { vertical: true, size: 8 }, () => [
              h(NInput, {
                value: url.value,
                'onUpdate:value': (v) => (url.value = v),
                placeholder: '输入URL',
                disabled: loading.value,
                size: 'small'
              }),
              h(NButton, {
                type: 'primary',
                onClick: fetchText,
                loading: loading.value,
                size: 'small',
                block: true
              }, () => '获取文本')
            ])
          ])
        ]),

        // 文件路径区域
        h(NGi, {}, () => [
          h(NCard, { title: '文件路径', size: 'small' }, () => [
            h(NSpace, { vertical: true, size: 8 }, () => [
              h(NInput, {
                value: filePath.value,
                'onUpdate:value': (v) => (filePath.value = v),
                placeholder: '文件路径',
                size: 'small'
              }),
              h(NButton, {
                onClick: selectFile,
                size: 'small',
                block: true
              }, () => '选择文件')
            ])
          ])
        ])
      ]),

      // 文本内容
      h(NCard, { title: '文本内容', size: 'small' }, () => [
        h(NInput, {
          value: textContent.value,
          'onUpdate:value': (v) => (textContent.value = v),
          type: 'textarea',
          rows: 7,
          placeholder: '文本内容将显示在这里,支持编辑...',
          size: 'small'
        })
      ]),

      // 操作按钮
      h(NSpace, { justify: 'center', size: 12 }, () => [
        h(NButton, {
          type: 'info',
          onClick: showPreview,
          size: 'small'
        }, () => '预览'),
        h(NButton, {
          type: 'success',
          onClick: writeToFile,
          size: 'small'
        }, () => '写入文件')
      ]),

      // 预览模态框
      h(NModal, {
        show: previewVisible.value,
        'onUpdate:show': (v) => (previewVisible.value = v),
        preset: 'card',
        title: '写入预览',
        style: 'width: 90%; max-width: 1400px',
        bordered: false
      }, () => [
        h(NGrid, { cols: 2, xGap: 12 }, () => [
          h(NGi, {}, () => [
            h(NCard, { title: '写入前', bordered: false }, () => [
              h(NCode, {
                code: beforeJson.value,
                language: 'json',
                style: 'max-height: 600px; overflow: auto'
              })
            ])
          ]),
          h(NGi, {}, () => [
            h(NCard, { title: '写入后', bordered: false }, () => [
              h(NCode, {
                code: afterJson.value,
                language: 'json',
                style: 'max-height: 600px; overflow: auto'
              })
            ])
          ])
        ])
      ])
    ])
  }
})
</script>

<template>
  <AppLayout>
    <AppContent />
  </AppLayout>
</template>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'PingFang SC', 'Microsoft YaHei', '微软雅黑', 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background-color: #101014;
  color: #e0e0e0;
}

#app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
  background-color: #101014;
}

/* 全局滚动条样式 */
*::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

*::-webkit-scrollbar-button {
  display: none !important;
}

*::-webkit-scrollbar-track {
  background: transparent;
}

*::-webkit-scrollbar-corner {
  background: rgba(0, 0, 0, 0);
}

*::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 4px;
  transition: background 0.3s;
}

*::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}

*::-webkit-scrollbar-thumb:active {
  background: #6a6a6a;
}

/* 暗色模式下的滚动条样式 */
.dark ::-webkit-scrollbar-track {
  background: transparent;
}

.dark ::-webkit-scrollbar-thumb {
  background: #4a4a4a;
}

.dark ::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}

/* 文字不可选中样式 - 提升用户体验 */
button,
.n-button,
.n-button__content {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

.n-card-header__main,
.n-card-header {
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

/* 优化小窗口下的显示 */
.n-card {
  margin-bottom: 0 !important;
}

.n-card-header {
  padding: 8px 12px !important;
  font-size: 13px !important;
}

.n-card__content {
  padding: 10px 12px !important;
}

.n-input {
  font-size: 13px !important;
}

.n-button {
  font-size: 13px !important;
  padding: 0 12px !important;
  height: 28px !important;
}

.n-button--small-type {
  height: 28px !important;
}
</style>
