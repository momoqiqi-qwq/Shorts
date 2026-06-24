# Shotrs

Shotrs 是一个从零搭建的跨平台桌面截图工具示例项目，使用 Rust 2021、egui/eframe、xcap、image、ravif、arboard、serde 和 toml。

## 已实现功能

- 小型设置窗口，点击"开始截图"后进入全屏半透明遮罩层。
- 截图形状：矩形、Shift 锁正方形、椭圆/圆形、Lasso 自由手绘、多边形双击闭合、全屏、窗口悬停识别。
- 选区确认后按形状裁剪像素；椭圆、Lasso、多边形的区域外像素透明。
- 导出格式：PNG、JPEG、WebP、BMP、GIF、AVIF。
- JPEG/WebP/AVIF 质量设置，WebP 有损/无损切换。
- 导出后自动复制到系统剪贴板。
- 文件名模板：`screenshot_{timestamp}.{ext}`。
- 配置持久化到系统配置目录下的 `shotrs/config.toml`；Linux 通常是 `~/.config/shotrs/config.toml`。
- CLI 覆盖主题：`shotrs --theme light|dark|system`。
- 遮罩 RGBA、选框颜色、线宽、工具栏位置可配置。
- 设置页支持彩虹边框、彩虹动画速度与宽度。

## 用户体验优化点

1. **小窗口入口**：主窗口只有设置和启动按钮，截图时才全屏，减少常驻干扰。
2. **冻结屏幕后选择**：先捕获主屏画面，再在画面上绘制选区，避免拖拽时内容变化。
3. **窗口识别模式**：鼠标悬停窗口时高亮，点击即截取。
4. **多边形双击闭合**：符合图形工具习惯；Enter 确认，Esc 取消。
5. **尺寸标签和坐标放大镜**：拖拽时即时显示选区尺寸和当前坐标。
6. **彩虹边框**：用于直播、教学或高对比场景，让选区更醒目。
7. **统一配置**：所有主题、导出、行为设置通过 TOML 持久化，适合后续做设置同步。

## 后续可继续添加的功能

- 全局快捷键：例如 `PrintScreen` 或 `Ctrl+Shift+S` 唤起截图。
- 多显示器拼接截图：当前代码优先主显示器，可扩展为虚拟桌面大画布。
- 标注工具：箭头、马赛克、文字、编号、涂鸦。
- OCR：选区文字识别。
- 上传分享：S3、MinIO、GitHub Issue、企业内部图床。
- 截图历史：缩略图、搜索、重新复制。
- 延时截图倒计时浮层。
- HiDPI 精准映射与每显示器 DPI 适配。
- macOS 权限引导、Wayland 门户提示、Windows UAC 场景处理。

## 构建

### Windows 本机构建 exe

```powershell
rustup update stable
cargo build --release
.\target\release\shotrs.exe --theme system
```

也可以运行：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1
```

生成位置：

```text
target/release/shotrs.exe
```

### Linux 构建依赖

xcap 在 Linux 编译时需要系统库，Debian/Ubuntu 可先安装：

```bash
sudo apt-get update
sudo apt-get install -y pkg-config libclang-dev libxcb1-dev libxrandr-dev libdbus-1-dev libpipewire-0.3-dev libwayland-dev libegl-dev
cargo build --release
```

### GitHub Actions 自动生成 Windows exe

把项目推到 GitHub 后，`Actions -> Build Windows exe -> Run workflow`，产物会出现在 Artifacts 中。

## 目录结构

```text
src/
  main.rs
  capture.rs
  shape.rs
  overlay.rs
  export.rs
  theme.rs
  config.rs
```

## 注意事项

- 本项目是可运行的工程骨架，不包含安装包、代码签名或商店分发配置。
- Windows/macOS/Linux 的屏幕捕获权限由系统决定；首次运行可能需要授予屏幕录制或辅助功能权限。
- Wayland 环境下截图能力取决于桌面门户和系统策略。
