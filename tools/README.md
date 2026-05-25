# 🛠️ 雅典娜 LED 动画转换工具 (PC端)

这两个脚本用于在电脑上将普通视频转换为路由器点阵屏专用的极致压缩 `.bin` 格式二进制裸流。

## 环境准备
请确保电脑已安装 Python，并安装所需依赖：
```bash
pip install opencv-python numpy


### 🎞️ 2. 转换视频 (convert_vid.py)

该脚本会将普通视频（如 `bad_apple.mp4`）按硬件底层走线规则（列映射模式）极限压缩为专属的 `.bin` 裸流文件。一分钟的 15FPS 动画仅占用不到 25 KB！

在 `tools/` 目录下执行：

```bash
python convert_vid.py bad_apple.mp4 bad_apple.bin --align center --fps 15

```

* **`--align`**: 画面对齐方式。可选 `center` (居中，默认), `left` (靠左), `right` (靠右)。由于屏幕长宽比极大 (27:5)，强烈建议多尝试不同的对齐方式。
* **`--fps`**: 目标帧率。推荐 `15` 或 `10`。

### 📺 3. 电脑端模拟预览 (preview_bin.py)

在传到路由器之前，你可以直接在电脑上以“马赛克像素风”预览转换效果：

```bash
python preview_bin.py bad_apple.bin --fps 15 --scale 30

```

* **`--scale 30`**: 将画面放大 30 倍以便在电脑高分屏上观看。按 `q` 键可随时退出预览。

### 🚀 4. 上机播放

1. 使用 WinSCP、Termius 或其它 SFTP 工具，将生成好的 `bad_apple.bin` 文件上传到路由器的 `/etc/athena_led/anim/` 目录下（如目录不存在请手动创建）。
2. 打开路由器的 LuCI 控制面板，刷新页面。
3. 在模块配置表中选择 `🎬 动画播放 (.bin)`，此时右侧的参数下拉框会自动扫描并出现 `🎬 [Anim] bad_apple.bin` 选项。
4. 设定持续时间，保存并应用，享受你的赛博朋克追番体验吧！

