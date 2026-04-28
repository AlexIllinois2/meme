# 快速启动指南

## 首次设置

### 1. 安装依赖

```bash
# 安装前端依赖
npm install

# 确保 Rust 工具链已安装
rustc --version
cargo --version
```

如果未安装 Rust，请访问 [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 2. 安装系统依赖（Linux）

```bash
# Ubuntu/Debian
sudo apt-get install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libXdo-devel \
    openssl-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

### 3. 运行开发环境

```bash
npm run tauri dev
```

这将：
- 启动 Vite 开发服务器（前端）
- 编译 Rust 代码（后端）
- 打开应用窗口

首次编译可能需要几分钟，请耐心等待。

## 常见问题

### Q: 编译时出现 "webkit2gtk not found" 错误
A: 请安装上述系统依赖。

### Q: Rust 编译失败
A: 尝试清理缓存后重新编译：
```bash
cd src-tauri
cargo clean
cd ..
npm run tauri dev
```

### Q: 前端热更新不工作
A: 确保 Vite 服务器正常运行，检查控制台是否有错误。

### Q: 数据库文件在哪里？
A: 
- Linux: `~/.local/share/meme/meme.db`
- Android: `<应用数据目录>/meme.db`

## 构建生产版本

```bash
npm run tauri build
```

构建完成后，可执行文件位于：
- Linux: `src-tauri/target/release/bundle/deb/` 或 `src-tauri/target/release/bundle/appimage/`
- Android: `src-tauri/target/aarch64-linux-android/release/bundle/apk/`

## 调试技巧

### 查看 Rust 日志

在开发模式下，Rust 的 `println!` 和 `eprintln!` 输出会显示在终端中。

### 查看前端日志

在 Tauri 窗口中右键 -> "检查元素" 打开开发者工具。

### 数据库查看

可以使用 SQLite 浏览器查看数据库：
```bash
sqlite3 ~/.local/share/meme/meme.db
.tables
.schema
```

## 下一步

1. 熟悉应用界面和功能
2. 创建第一个模式和分组
3. 导入一些表情包测试
4. 尝试搜索功能
5. 自定义设置

祝你使用愉快！🎉