# 文本内容获取与保存工具

一个基于 Tauri 2 + Vue 3 + Naive UI 开发的桌面应用,用于从 URL 获取文本内容并增量保存到 JSON 文件。

## 功能特性

### 核心功能
- ✅ **从 URL 获取文本内容** - 直接获取文本,无需下载文件
- ✅ **文本编辑** - 支持查看和编辑获取的文本内容
- ✅ **JSON 解析** - 自动将文本解析为 JSON 对象
- ✅ **增量写入** - 将数据追加到 JSON 数组,不覆盖现有数据
- ✅ **预览对比** - 并排显示写入前后的 JSON 内容
- ✅ **文件路径管理** - 支持自定义文件路径或使用默认路径

### 技术栈
- **前端**: Vue 3 + Naive UI
- **桌面框架**: Tauri 2
- **后端**: Rust (reqwest, serde_json, tokio)

## 安装和运行

### 前置要求
- Node.js (推荐 v18+)
- Rust (推荐最新稳定版)
- npm 或其他包管理器

### 开发模式

1. 安装依赖:
```bash
npm install
```

2. 运行开发服务器:
```bash
npm run tauri dev
```

### 构建生产版本

```bash
npm run tauri build
```

构建完成后,可执行文件位于 `src-tauri/target/release/` 目录。

## 使用说明

### 1. 获取文本内容
- 在 "URL 输入" 区域输入目标 URL
- 点击 "获取文本" 按钮
- 文本内容将显示在 "文本内容" 区域

### 2. 编辑文本
- 在 "文本内容" 区域可以直接编辑获取的文本
- 确保文本是有效的 JSON 格式

### 3. 配置文件路径
- 默认路径: `%APPDATA%\com.cubezhao.atm\tokens.json`
- 可以手动输入路径或点击 "选择文件" 按钮选择

### 4. 预览写入
- 点击 "预览" 按钮
- 查看写入前后的 JSON 对比
- 确认无误后关闭预览窗口

### 5. 写入文件
- 点击 "写入文件" 按钮
- 数据将追加到 JSON 数组末尾
- 写入成功后文本内容会自动清空

## 项目结构

```
aug-session-sync/
├── src/                          # Vue 前端代码
│   ├── App.vue                   # 主应用组件
│   └── main.js                   # 入口文件
├── src-tauri/                    # Tauri 后端代码
│   ├── src/
│   │   ├── lib.rs               # 库入口
│   │   ├── http_client.rs       # HTTP 客户端
│   │   └── file_ops.rs          # 文件操作
│   ├── Cargo.toml               # Rust 依赖
│   └── tauri.conf.json          # Tauri 配置
├── package.json                  # Node 依赖
└── README.md                     # 项目说明
```

## API 说明

### Tauri 命令

#### `fetch_text_from_url(url: String) -> Result<String, String>`
从指定 URL 获取文本内容。

#### `read_json_file(file_path: String) -> Result<String, String>`
读取 JSON 文件内容。

#### `append_to_json_file(file_path: String, new_data: Value) -> Result<(), String>`
增量写入 JSON 文件。

#### `get_default_file_path() -> Result<String, String>`
获取默认文件路径。

#### `select_file() -> Result<Option<String>, String>`
打开文件选择对话框。

## 注意事项

1. **JSON 格式**: 确保文本内容是有效的 JSON 格式,否则无法解析
2. **文件权限**: 确保应用有权限访问目标文件路径
3. **数据备份**: 建议定期备份 JSON 文件,避免数据丢失
4. **网络连接**: 获取 URL 内容需要网络连接

## 错误处理

应用会显示以下类型的错误提示:
- URL 为空
- 网络请求失败
- JSON 格式无效
- 文件读写失败
- 路径不存在

## 开发说明

### 添加新功能

1. **后端功能**: 在 `src-tauri/src/` 中添加新的 Rust 模块
2. **前端功能**: 在 `src/App.vue` 中添加新的 Vue 组件或方法
3. **注册命令**: 在 `src-tauri/src/lib.rs` 中注册新的 Tauri 命令

### 调试

- 前端调试: 打开浏览器开发者工具 (F12)
- 后端调试: 查看终端输出的 Rust 日志

## 许可证

MIT License

## 作者

cubezhao

## 更新日志

### v0.1.0 (2025-11-02)
- ✅ 初始版本发布
- ✅ 实现核心功能:URL 获取、文本编辑、JSON 写入
- ✅ 实现预览对比功能
- ✅ 实现文件路径管理
