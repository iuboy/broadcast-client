# Broadcast Client 自动更新功能指南

## 功能概述

broadcast-client 现已支持自动更新功能，包括：
- 自动检查更新
- 更新提醒通知
- 关闭应用时自动安装更新
- 可配置的更新设置
- 友好的更新界面

## 配置说明

### 更新配置文件

更新配置位于 `public/update-config.json` 文件中：

```json
{
  "update": {
    "enabled": true,                    // 是否启用自动更新
    "checkInterval": 86400,             // 检查更新间隔（秒），默认24小时
    "autoInstallOnExit": true,          // 关闭应用时自动安装更新
    "showNotification": true,          // 是否显示更新通知
    "endpoints": [                      // 更新服务器地址
      "https://your-update-server.com/{{target}}/{{current_version}}"
    ],
    "currentVersion": "0.1.0",          // 当前版本号
    "releaseNotes": ""                  // 发布说明（可选）
  }
}
```

### 配置参数说明

- **enabled**: 控制是否启用自动更新功能
- **checkInterval**: 自动检查更新的时间间隔（秒）
  - 86400 = 24小时
  - 43200 = 12小时
  - 0 = 不自动检查，仅手动检查
- **autoInstallOnExit**: 关闭应用时是否自动下载并安装更新
- **showNotification**: 是否显示更新提醒通知
- **endpoints**: 更新服务器的URL列表
  - `{{target}}` 会被替换为目标平台（如 darwin-aarch64, windows-x86_64）
  - `{{current_version}}` 会被替换为当前版本号
- **currentVersion**: 当前应用版本号
- **releaseNotes**: 当前版本的发布说明（可选）

## 使用方法

### 1. 手动检查更新

在应用底部点击"检查更新"按钮，系统会立即检查是否有新版本可用。

### 2. 更新通知

当检测到新版本时，系统会显示通知：
- 通知标题：发现新版本
- 通知内容：版本号和简要信息
- 点击通知可查看详细更新内容

### 3. 更新对话框

点击更新按钮后，会弹出更新对话框，显示：
- 当前版本号
- 新版本号
- 发布日期
- 更新内容说明
- 操作按钮：取消、立即更新

### 4. 下载和安装更新

点击"立即更新"按钮后：
1. 系统开始下载更新包
2. 下载完成后显示成功提示
3. 关闭应用时自动应用更新

### 5. 自动安装更新

如果配置了 `autoInstallOnExit: true`：
- 检测到更新时会在后台自动下载
- 关闭应用时显示安装提示
- 下次启动应用时即为新版本

## 更新服务器设置

### Tauri 更新服务器配置

在 `src-tauri/tauri.conf.json` 中配置：

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEIxRDI0Njk2NzY4OTJDQUMKUldTZkR2WjU4bUx1S0VqSWx0b1J5VWJMVVJ2M1V5UThpQnN6R3pMRXJpOE0K",
      "endpoints": [
        "https://your-update-server.com/{{target}}/{{current_version}}"
      ]
    }
  }
}
```

### 更新服务器响应格式

更新服务器应返回以下 JSON 格式：

```json
{
  "version": "0.2.0",
  "body": "更新说明：\n- 新增功能A\n- 修复问题B\n- 优化性能C",
  "date": "2024-01-15",
  "platforms": {
    "darwin-aarch64": {
      "signature": "签名信息",
      "url": "https://example.com/updates/broadcast-client_0.2.0_aarch64.dmg"
    },
    "windows-x86_64": {
      "signature": "签名信息",
      "url": "https://example.com/updates/broadcast-client_0.2.0_x64-setup.exe"
    }
  }
}
```

## 构建和发布更新

### 1. 准备更新服务器

设置一个静态文件服务器或专门的更新服务器来托管更新包。

### 2. 构建新版本

```bash
cd broadcast-client
pnpm tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

### 3. 上传更新包

将构建好的安装包上传到更新服务器的指定路径。

### 4. 更新服务器配置

根据新版本信息更新服务器的 JSON 响应文件。

### 5. 更新版本号

在以下文件中更新版本号：
- `src-tauri/tauri.conf.json` 中的 `version` 字段
- `src-tauri/Cargo.toml` 中的 `version` 字段
- `public/update-config.json` 中的 `currentVersion` 字段

## 技术实现

### 后端 (Rust/Tauri)

在 `src-tauri/src/lib.rs` 中实现了以下命令：

- `get_update_info`: 获取更新信息
- `install_update`: 下载并安装更新
- `check_and_install_update`: 检查并安装更新

使用 `tauri-plugin-updater` 插件处理更新逻辑。

### 前端 (Vue/TypeScript)

在 `src/utils/useAppUpdate.ts` 中创建了 `useAppUpdate` composable：

- `checkForUpdates`: 检查更新
- `downloadAndUpdate`: 下载并安装更新
- `showUpdateDialog`: 显示更新对话框
- 自动检查更新（启动时）
- 定期检查更新
- 关闭应用时自动安装

### 用户界面

在 `src/App.vue` 中集成了更新功能：

- 底部更新按钮区域
- 更新对话框
- 更新状态显示
- 下载进度提示

## 注意事项

1. **公钥配置**: 确保 `tauri.conf.json` 中的 `pubkey` 正确配置，用于验证更新包的签名。

2. **HTTPS**: 更新服务器必须使用 HTTPS 协议以确保安全性。

3. **版本号格式**: 建议使用语义化版本号（如 0.1.0, 1.0.0）。

4. **跨平台**: 需要为不同平台（Windows、macOS、Linux）准备对应的更新包。

5. **签名验证**: 发布的更新包必须使用对应的私钥签名，否则客户端会拒绝安装。

6. **网络环境**: 确保客户端能够访问更新服务器。

7. **用户通知**: 建议在更新前通知用户，避免意外重启。

## 故障排查

### 检查更新失败

- 检查网络连接
- 确认更新服务器地址正确
- 查看浏览器控制台错误日志

### 下载更新失败

- 检查更新包 URL 是否可访问
- 确认文件大小合理
- 验证服务器响应格式正确

### 安装更新失败

- 确认公钥配置正确
- 检查更新包签名
- 确保有足够的磁盘空间

## 最佳实践

1. **版本控制**: 使用 Git 管理版本号变更
2. **测试更新**: 在测试环境中先验证更新流程
3. **发布说明**: 提供清晰的更新说明
4. **回滚方案**: 准备旧版本安装包以备回滚
5. **监控更新**: 收集更新成功率和错误日志

## 常见问题

**Q: 如何禁用自动更新？**
A: 在 `public/update-config.json` 中设置 `"enabled": false`

**Q: 如何更改检查更新的频率？**
A: 修改 `checkInterval` 参数（单位为秒）

**Q: 更新会丢失用户数据吗？**
A: 不会，更新过程只替换应用文件，不会影响用户数据

**Q: 可以手动安装更新吗？**
A: 可以，下载安装包后手动安装，但建议使用自动更新功能

**Q: 支持增量更新吗？**
A: 目前不支持，每次更新都是完整下载新版本

## 更新日志

### v1.0.0 (当前版本)
- 实现自动更新功能
- 添加更新检查和提醒
- 支持关闭时自动安装
- 提供配置化的更新设置

## 技术支持

如有问题，请查看：
- Tauri 官方文档: https://tauri.app/v2/guides/distribution/update/
- 项目 Issues: [GitHub Issues 链接]
