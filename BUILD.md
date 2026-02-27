# Broadcast Client 跨平台构建指南

## 📋 项目概述

Broadcast Client 是一个基于 Tauri 的跨平台桌面应用，使用：
- **前端**：Vue 3 + TypeScript + Element Plus
- **后端**：Rust (Tauri)
- **构建工具**：Vite + Tauri CLI

## 🔧 前置要求

### 通用要求
- Node.js >= 18.0.0
- pnpm >= 8.0.0
- Rust >= 1.70.0

### Windows 特定要求
1. 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
   - 在安装时选择 "C++ 桌面开发"
   - 包含 Windows 10 SDK
2. 安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
3. 可选：安装 [WiX Toolset](https://wixtoolset.org/) 用于 MSI 安装包

### macOS 特定要求
1. 安装 Xcode 命令行工具：
   ```bash
   xcode-select --install
   ```
2. 安装 Xcode（完整版）用于代码签名：
   ```bash
   xcode-select --switch /Applications/Xcode.app
   ```
3. 对于 Intel Mac，需要额外安装：
   ```bash
   rustup target add x86_64-apple-darwin
   ```

### Linux 特定要求

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Fedora/RHEL
```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

#### Arch Linux
```bash
sudo pacman -Syu --needed \
    webkit2gtk-4.1 \
    base-devel \
    curl \
    wget \
    file \
    libayatana-appindicator \
    librsvg
```

## 📦 安装依赖

```bash
cd broadcast-client
pnpm install
```

## 🚀 构建命令

### 1. 开发模式

所有平台通用：
```bash
pnpm tauri dev
```

功能：
- 启动前端开发服务器（Vite）
- 启动 Tauri 开发窗口
- 支持热重载
- 自动编译 Rust 代码

### 2. 生产构建

#### 构建当前平台

```bash
pnpm tauri build
```

这将构建适合当前操作系统的安装包。

#### 构建特定目标

**Windows：**
```bash
# MSI 安装包（推荐）
pnpm tauri build -- --target msi

# NSIS 安装包（更常见）
pnpm tauri build -- --target nsis

# 构建所有 Windows 格式
pnpm tauri build -- --target msi --target nsis
```

**macOS：**
```bash
# DMG 磁盘映像（推荐）
pnpm tauri build -- --target dmg

# APP 应用程序包
pnpm tauri build -- --target app

# 构建所有 macOS 格式
pnpm tauri build -- --target dmg --target app
```

**Linux：**
```bash
# AppImage（通用，推荐）
pnpm tauri build -- --target appimage

# DEB 包（Debian/Ubuntu）
pnpm tauri build -- --target deb

# RPM 包（Fedora/RHEL）
pnpm tauri build -- --target rpm

# 构建所有 Linux 格式
pnpm tauri build -- --target appimage --target deb --target rpm
```

### 3. 仅构建前端

```bash
pnpm build
```

这会在 `dist/` 目录生成静态文件，用于：
- 部署到 Web 服务器
- 测试前端代码
- 手动打包到其他平台

## 📂 构建产物位置

构建完成后，安装包位于：

### Windows
```
src-tauri/target/release/bundle/msi/
  └── broadcast-client_0.1.0_x64_en-US.msi

src-tauri/target/release/bundle/nsis/
  └── broadcast-client_0.1.0_x64-setup.exe
```

### macOS
```
src-tauri/target/release/bundle/dmg/
  └── broadcast-client_0.1.0_x64.dmg

src-tauri/target/release/bundle/macos/
  └── broadcast-client.app
```

### Linux
```
src-tauri/target/release/bundle/appimage/
  └── broadcast-client_0.1.0_amd64.AppImage

src-tauri/target/release/bundle/deb/
  └── broadcast-client_0.1.0_amd64.deb

src-tauri/target/release/bundle/rpm/
  └── broadcast-client-0.1.0-1.x86_64.rpm
```

## 🎯 平台特定构建

### Windows 上的构建

```bash
# 1. 打开 PowerShell 或 CMD
cd broadcast-client

# 2. 安装依赖
pnpm install

# 3. 构建
pnpm tauri build -- --target nsis

# 4. 安装
cd src-tauri/target/release/bundle/nsis
./broadcast-client_0.1.0_x64-setup.exe
```

### macOS 上的构建

#### Intel Mac (x86_64)
```bash
cd broadcast-client
pnpm install
pnpm tauri build -- --target dmg
```

#### Apple Silicon Mac (ARM64)
```bash
cd broadcast-client
pnpm install
pnpm tauri build -- --target aarch64-apple-darwin
```

#### Universal Binary (支持两种架构)
```bash
# 先构建两个架构
pnpm tauri build -- --target x86_64-apple-darwin
pnpm tauri build -- --target aarch64-apple-darwin

# 使用 lipo 合并
lipo -create -output broadcast-client.universal \
  src-tauri/target/release/bundle/macos/broadcast-client.app/Contents/MacOS/broadcast-client
```

### Linux 上的构建

```bash
cd broadcast-client
pnpm install
pnpm tauri build -- --target appimage

# 运行 AppImage
chmod +x src-tauri/target/release/bundle/appimage/broadcast-client_0.1.0_amd64.AppImage
./src-tauri/target/release/bundle/appimage/broadcast-client_0.1.0_amd64.AppImage
```

## 🔧 高级配置

### 自定义构建配置

编辑 `src-tauri/tauri.conf.json`：

```json
{
  "bundle": {
    "targets": "all",
    "outDir": "../builds",
    "publisher": "your-publisher-name"
  },
  "app": {
    "windows": [
      {
        "title": "Broadcast Client",
        "width": 1024,
        "height": 768,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

### 环境变量

```bash
# 设置 Rust 工具链
export RUSTUP_HOME=$HOME/.rustup
export PATH=$PATH:$RUSTUP_HOME/bin

# 设置构建目标
export TAURI_BUNDLE_TARGET="msi"  # Windows
export TAURI_BUNDLE_TARGET="dmg"   # macOS
export TAURI_BUNDLE_TARGET="appimage"  # Linux
```

### 代码签名

#### Windows 代码签名
```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      },
      "certificateThumbprint": "YOUR_CERTIFICATE_THUMBPRINT",
      "digestAlgorithm": "sha256"
    }
  }
}
```

#### macOS 代码签名
```bash
# 导入证书
security import certificate.p12 -k ~/Library/Keychains/login.keychain-db -P password -T /usr/bin/codesign

# 构建时自动签名
pnpm tauri build -- --codesign
```

## 🐛 常见问题

### 问题 1：Rust 编译失败

**症状：**
```
error: linker `link.exe` not found
```

**解决方案（Windows）：**
1. 安装 Visual Studio Build Tools
2. 确保安装了 C++ 桌面开发组件

**解决方案（Linux）：**
```bash
sudo apt install build-essential
```

### 问题 2：依赖安装失败

**症状：**
```
Error: Cannot find module 'xxx'
```

**解决方案：**
```bash
rm -rf node_modules
rm pnpm-lock.yaml
pnpm install
```

### 问题 3：WebView2 问题（Windows）

**症状：**
```
Error: Edge webview2 runtime not found
```

**解决方案：**
下载并安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### 问题 4：macOS 代码签名错误

**症状：**
```
error: code object is not signed at all
```

**解决方案：**
```bash
# 清理构建缓存
rm -rf src-tauri/target

# 重新构建
pnpm tauri build
```

### 问题 5：Linux 图标问题

**症状：**
```
Error: Icon not found
```

**解决方案：**
确保图标文件存在于 `src-tauri/icons/` 目录：
```bash
ls src-tauri/icons/
# 应该看到：32x32.png, 128x128.png, icon.png 等
```

## 🔄 CI/CD 自动构建

### GitHub Actions 示例

创建 `.github/workflows/build.yml`：

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: 'macos-latest'
            target: 'universal-apple-darwin'
            bundle_target: 'dmg'
          - platform: 'ubuntu-22.04'
            target: 'x86_64-unknown-linux-gnu'
            bundle_target: 'appimage'
          - platform: 'windows-latest'
            target: 'x86_64-pc-windows-msvc'
            bundle_target: 'nsis'
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
      
      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8
      
      - name: Install dependencies
        run: pnpm install
        
      - name: Build Tauri App
        run: pnpm tauri build -- --target ${{ matrix.target }}
        
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}-${{ matrix.bundle_target }}
          path: src-tauri/target/release/bundle/${{ matrix.bundle_target }}/*
```

## 📝 快速参考

### 常用命令速查

| 操作 | 命令 |
|------|------|
| 开发模式 | `pnpm tauri dev` |
| 构建当前平台 | `pnpm tauri build` |
| 构建 Windows (NSIS) | `pnpm tauri build -- --target nsis` |
| 构建 macOS (DMG) | `pnpm tauri build -- --target dmg` |
| 构建 Linux (AppImage) | `pnpm tauri build -- --target appimage` |
| 清理构建缓存 | `pnpm tauri clean` |
| 仅构建前端 | `pnpm build` |

### 平台特定输出

| 平台 | 输出目录 | 主要格式 |
|------|----------|----------|
| Windows | `src-tauri/target/release/bundle/nsis/` | `.exe`, `.msi` |
| macOS | `src-tauri/target/release/bundle/dmg/` | `.dmg`, `.app` |
| Linux | `src-tauri/target/release/bundle/appimage/` | `.AppImage`, `.deb`, `.rpm` |

## 📚 相关资源

- [Tauri 官方文档](https://tauri.app/v1/guides/)
- [Tauri CLI 命令参考](https://tauri.app/v1/api/cli/)
- [Rust 安装指南](https://www.rust-lang.org/tools/install)
- [Node.js 下载](https://nodejs.org/)
- [Vite 构建选项](https://vitejs.dev/config/build-options.html)

## 🤝 贡献

如果遇到构建问题，请：
1. 检查本文档的常见问题部分
2. 查看项目 Issues
3. 提交新的 Issue 并附上：
   - 操作系统版本
   - Node.js 和 Rust 版本
   - 完整的错误日志
   - 构建命令

## 📄 许可证

本项目遵循项目根目录的 LICENSE 文件。
