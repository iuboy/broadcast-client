# 自动更新功能实现说明

## 已完成的功能

### 1. 后端实现 (Rust/Tauri)

**文件**: `src-tauri/src/lib.rs`

实现了以下 Tauri 命令：
- `get_update_info`: 获取可用更新信息
- `install_update`: 下载并安装更新
- `check_and_install_update`: 检查并安装更新

使用 `tauri-plugin-updater` 插件处理更新逻辑。

### 2. 前端实现 (Vue/TypeScript)

**文件**: `src/utils/useAppUpdate.ts`

创建了 `useAppUpdate` composable，提供：
- 自动检查更新（启动时）
- 定期检查更新（可配置间隔）
- 手动检查更新
- 下载并安装更新
- 更新提醒通知
- 关闭应用时自动安装更新

### 3. 用户界面

**文件**: `src/App.vue`

添加了以下 UI 组件：
- 底部更新按钮区域
- 更新对话框（显示版本信息、更新内容）
- 更新状态指示（下载中、已下载）
- 通知提醒

### 4. 配置文件

**文件**: `public/update-config.json`

可配置的更新设置：
```json
{
  "update": {
    "enabled": true,              // 是否启用自动更新
    "checkInterval": 86400,       // 检查间隔（秒），默认24小时
    "autoInstallOnExit": true,     // 关闭时自动安装
    "showNotification": true,      // 显示通知
    "endpoints": [...],           // 更新服务器地址
    "currentVersion": "0.1.0",    // 当前版本
    "releaseNotes": ""            // 发布说明
  }
}
```

### 5. Tauri 配置

**文件**: `src-tauri/tauri.conf.json`

添加了 updater 插件配置：
```json
{
  "plugins": {
    "updater": {
      "pubkey": "公钥",
      "endpoints": ["更新服务器URL"]
    }
  }
}
```

**文件**: `src-tauri/Cargo.toml`

添加了依赖：
```toml
tauri-plugin-updater = "2"
```

## 功能特性

### ✅ 已实现

1. **自动检查更新**
   - 应用启动时自动检查
   - 可配置的检查间隔（默认24小时）
   - 后台静默检查

2. **更新提醒**
   - 发现新版本时显示通知
   - 通知可点击查看详情
   - 可配置是否显示通知

3. **手动检查**
   - 用户可随时点击"检查更新"按钮
   - 即时反馈检查结果

4. **更新对话框**
   - 显示当前版本和新版本号
   - 显示发布日期
   - 显示更新内容说明
   - 支持 Markdown 格式的更新说明

5. **下载和安装**
   - 点击"立即更新"开始下载
   - 显示下载进度
   - 下载完成后提示关闭应用

6. **自动安装**
   - 默认启用关闭时自动安装
   - 可配置是否启用
   - 更新包后台下载
   - 关闭应用时自动应用

7. **配置化**
   - 所有设置可在配置文件中修改
   - 无需重新编译代码
   - 支持动态加载配置

## 使用说明

### 启用自动更新

在 `public/update-config.json` 中设置：
```json
{
  "update": {
    "enabled": true,
    "autoInstallOnExit": true
  }
}
```

### 禁用自动更新

```json
{
  "update": {
    "enabled": false
  }
}
```

### 更改检查频率

```json
{
  "update": {
    "checkInterval": 43200  // 12小时
  }
}
```

### 配置更新服务器

1. 在 `public/update-config.json` 中配置 endpoints
2. 在 `src-tauri/tauri.conf.json` 中配置 endpoints（保持一致）
3. 更新服务器需返回指定格式的 JSON 响应

## 发布更新的步骤

1. **更新版本号**
   - `src-tauri/tauri.conf.json` 的 version
   - `src-tauri/Cargo.toml` 的 version
   - `public/update-config.json` 的 currentVersion

2. **构建应用**
   ```bash
   pnpm tauri build
   ```

3. **上传更新包**
   - 将构建好的安装包上传到服务器
   - 确保服务器返回正确的 JSON 响应

4. **测试更新**
   - 在测试环境验证更新流程
   - 确认签名验证正确

## 注意事项

1. **更新服务器必须使用 HTTPS**
2. **需要配置正确的公钥用于签名验证**
3. **更新包必须用对应的私钥签名**
4. **需要为不同平台准备对应的更新包**
5. **建议在测试环境先验证更新流程**

## 文档

详细的使用和配置说明请参考：
- `UPDATE_GUIDE.md` - 完整的更新功能指南

## 技术栈

- **后端**: Rust + Tauri 2 + tauri-plugin-updater
- **前端**: Vue 3 + TypeScript + Element Plus
- **构建**: Tauri CLI

## 后续改进建议

1. 支持增量更新（减少下载量）
2. 添加更新进度条
3. 支持回滚到旧版本
4. 添加更新历史记录
5. 支持强制更新
6. 添加更新统计和日志
