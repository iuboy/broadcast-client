#!/bin/bash
# Sidecar 构建脚本
# 用于构建 Rust 广播服务作为 Tauri sidecar

set -e

# 项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SERVER_DIR="$PROJECT_ROOT/rust-broadcast-server"
BINARIES_DIR="$SCRIPT_DIR/binaries"

# 创建 binaries 目录
mkdir -p "$BINARIES_DIR"

echo "正在构建广播服务 sidecar..."

# 检测当前平台
OS=$(uname -s)
ARCH=$(uname -m)

# 确定目标三元组
case "$OS" in
    Darwin)
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
        ;;
    Linux)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        TARGET="x86_64-pc-windows-msvc"
        ;;
    *)
        echo "不支持的操作系统: $OS"
        exit 1
        ;;
esac

echo "检测到平台: $TARGET"

# 构建 sidecar
cd "$SERVER_DIR"

# 添加目标（如果需要）
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "正在添加 Rust 目标: $TARGET"
    rustup target add "$TARGET"
fi

# 构建 release 版本
echo "正在构建 release 版本..."
cargo build --release --target "$TARGET"

# 确定输出文件名
case "$OS" in
    MINGW*|MSYS*|CYGWIN*)
        BINARY_NAME="broadcast-server-$TARGET.exe"
        SOURCE_BINARY="target/$TARGET/release/broadcast-server.exe"
        ;;
    *)
        BINARY_NAME="broadcast-server-$TARGET"
        SOURCE_BINARY="target/$TARGET/release/broadcast-server"
        ;;
esac

# 复制二进制文件
echo "正在复制二进制文件到 $BINARIES_DIR/$BINARY_NAME"
cp "$SOURCE_BINARY" "$BINARIES_DIR/$BINARY_NAME"

# 设置可执行权限
chmod +x "$BINARIES_DIR/$BINARY_NAME"

echo "✅ Sidecar 构建完成: $BINARIES_DIR/$BINARY_NAME"
