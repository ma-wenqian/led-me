import cv2
import numpy as np
import argparse

# 雅典娜点阵屏的物理尺寸
MATRIX_W = 27
MATRIX_H = 5
# 🌟 新版核心：硬件原生列映射模式，每帧恰好 27 个字节
BYTES_PER_FRAME = 27 

def preview_video(bin_path, fps=15, scale=30):
    try:
        with open(bin_path, 'rb') as f:
            data = f.read()
    except FileNotFoundError:
        print(f"❌ 找不到文件: {bin_path}")
        return

    total_frames = len(data) // BYTES_PER_FRAME
    if total_frames == 0:
        print("❌ 文件为空或格式不正确！(是不是拿旧版脚本生成的？)")
        return
        
    print(f"📺 开始预览赛博点阵屏 (原生列映射版)...")
    print(f"📁 文件: {bin_path} | 🎞️ 总帧数: {total_frames} | ⏱️ 帧率: {fps}fps")
    print(f"💡 提示: 选中弹出的画面窗口，按键盘 'q' 键可随时退出。")

    # 计算每帧停留的毫秒数
    delay_ms = max(1, int(1000 / fps))

    for i in range(total_frames):
        # 1. 每次精确切出 27 个字节 (对应 27 列)
        chunk = data[i * BYTES_PER_FRAME : (i + 1) * BYTES_PER_FRAME]
        if len(chunk) < BYTES_PER_FRAME:
            break
            
        # 2. 准备纯黑画布
        canvas = np.zeros((MATRIX_H, MATRIX_W), dtype=np.uint8)
        
        # 3. 🌟 完美逆向硬件解包 (Bit-Unpacking)
        for x in range(MATRIX_W):
            byte_val = chunk[x]
            for y in range(MATRIX_H):
                # 检查这个 Byte 的第 y 位是否为 1 (Bit 0 对应 y=0 即顶部)
                if (byte_val >> y) & 1:
                    canvas[y, x] = 255  # 255 代表纯白像素
        
        # 4. 放大画面保持“硬核马赛克”颗粒感！
        # 使用 INTER_NEAREST (最近邻插值) 确保边缘锐利不模糊
        preview_img = cv2.resize(canvas, 
                                 (MATRIX_W * scale, MATRIX_H * scale), 
                                 interpolation=cv2.INTER_NEAREST)
        
        # 5. 推送显示
        cv2.imshow("Athena LED Simulator", preview_img)
        
        # 等待期间如果按了 'q' 键就退出
        if cv2.waitKey(delay_ms) & 0xFF == ord('q'):
            print("⏹️ 预览已手动终止。")
            break

    cv2.destroyAllWindows()
    print("✅ 播放完毕。")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="预览雅典娜点阵屏 .bin 动画文件 (原生列映射版)")
    parser.add_argument("input", help="输入的 .bin 文件路径")
    parser.add_argument("--fps", type=int, default=15, help="播放帧率 (默认: 15，需与生成时一致)")
    parser.add_argument("--scale", type=int, default=30, help="画面放大倍数 (默认: 30倍)")
    args = parser.parse_args()
    
    preview_video(args.input, args.fps, args.scale)