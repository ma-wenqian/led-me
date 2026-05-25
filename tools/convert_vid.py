import cv2
import numpy as np
import argparse

MATRIX_W = 27
MATRIX_H = 5

def convert_video(input_path, output_path, align='center', target_fps=15):
    cap = cv2.VideoCapture(input_path)
    if not cap.isOpened():
        print(f"❌ 无法打开视频: {input_path}")
        return

    original_fps = cap.get(cv2.CAP_PROP_FPS)
    frame_skip = max(1, int(original_fps / target_fps))

    print(f"🎬 开始处理 | 帧率: {target_fps} fps | 硬件列映射模式 (27 bytes/frame)")
    
    with open(output_path, 'wb') as out_file:
        count = 0
        frames_written = 0
        
        while True:
            ret, frame = cap.read()
            if not ret: break
                
            if count % frame_skip != 0:
                count += 1
                continue
            count += 1
            
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            _, binary = cv2.threshold(gray, 128, 255, cv2.THRESH_BINARY)
            
            h, w = binary.shape
            new_h = MATRIX_H
            new_w = int((w / h) * new_h)
            resized = cv2.resize(binary, (new_w, new_h), interpolation=cv2.INTER_AREA)
            _, resized = cv2.threshold(resized, 128, 255, cv2.THRESH_BINARY)
            
            canvas = np.zeros((MATRIX_H, MATRIX_W), dtype=np.uint8)
            paste_w = min(new_w, MATRIX_W)
            
            x_offset = 0
            if align == 'center': x_offset = (MATRIX_W - paste_w) // 2
            elif align == 'right': x_offset = MATRIX_W - paste_w
                
            normalized_frame = (resized[0:MATRIX_H, 0:paste_w] > 0).astype(np.uint8)
            canvas[0:MATRIX_H, x_offset:x_offset+paste_w] = normalized_frame
            
            # ===============================================
            # 🌟 为 Rust 硬件驱动量身定制的【列映射压缩】
            # ===============================================
            frame_bytes = bytearray()
            for x in range(MATRIX_W):
                byte_val = 0
                for y in range(MATRIX_H):
                    if canvas[y, x] > 0:
                        # 雅典娜驱动是低位在前 (i=0 对应 y=0)
                        byte_val |= (1 << y) 
                frame_bytes.append(byte_val)
            
            # 每一帧刚好写入 27 个字节！
            out_file.write(frame_bytes)
            frames_written += 1

    cap.release()
    print(f"✅ 完成！共写入 {frames_written} 帧。最终大小: {frames_written * 27 / 1024:.2f} KB")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    parser.add_argument("--align", choices=['left', 'center', 'right'], default='center')
    parser.add_argument("--fps", type=int, default=15)
    args = parser.parse_args()
    convert_video(args.input, args.output, args.align, args.fps)