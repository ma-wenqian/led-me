use anyhow::Result;
use sysfs_gpio::{Direction, Pin};
use crate::char_dict::CHAR_DICT;
use std::fs;
use std::time::{Instant, Duration};

const LOW: u8 = 0x00;
const HIGH: u8 = 0x01;

// Display mode commands
const COMMAND1: u8 = 0b00000011; // Display mode
const COMMAND2: u8 = 0b01000000; // Data mode
const COMMAND3: u8 = 0b11000000; // Display address

pub struct LedScreen {
    left_screen: LedScreenUnit,
    right_screen: LedScreenUnit,
}

pub struct LedScreenUnit {
    stb: Pin,
    clk: Pin,
    dio: Pin,
}

impl LedScreen {
    pub fn new(stb_left: u64, stb_right: u64, clk: u64, dio: u64) -> Result<Self> {
        let left_screen = LedScreenUnit::new(stb_left, clk, dio)?;
        let right_screen = LedScreenUnit::new(stb_right, clk, dio)?;
        
        let mut screen = Self {
            left_screen,
            right_screen,
        };
        
        screen.set_show_model()?;
        screen.set_data_model()?;
        
        Ok(screen)
    }

    pub fn set_show_model(&mut self) -> Result<()> {
        self.left_screen.set_show_model()?;
        self.right_screen.set_show_model()?;
        Ok(())
    }

    pub fn set_data_model(&mut self) -> Result<()> {
        self.left_screen.set_data_model()?;
        self.right_screen.set_data_model()?;
        Ok(())
    }

    pub fn power(&mut self, run: bool, light_level: u8) -> Result<()> {
        self.left_screen.power(run, light_level)?;
        self.right_screen.power(run, light_level)?;
        Ok(())
    }

    // 2. 这里也要加上 async 关键字
    pub async fn write_data(&mut self, text: &[u8], status: u8) -> Result<()> {
        let mut display_data = Vec::new();
        
        let content = std::str::from_utf8(text).unwrap_or("");
        
        for ch in content.chars() {
            let key = ch.to_ascii_uppercase(); 
            
            if let Some(bytes) = CHAR_DICT.get(&key) {
                display_data.extend_from_slice(bytes);
                display_data.push(0x00); // 加空格
            }
        }

        // 修复：砍掉最后一个多余的尾部空格！
        // 这样 28 列的 "10:10:10" 就会瞬间变成 27 列！
        if !display_data.is_empty() {
            display_data.pop(); 
        }

        // 判断逻辑完全不需要改，保持 27 即可
        if display_data.len() > 27 {
            self.flow(&display_data, status).await?;
        } else {
            self.static_display(&display_data, status)?;
        }
        Ok(())
    }

    // ==========================================
    // 🎬 动画播放引擎 (0 CPU 消耗，直接内存推流)
    // ==========================================
    pub async fn play_animation(&mut self, file_name: &str, duration_secs: u64, status: u8) -> Result<()> {
        let file_path = format!("/etc/athena_led/anim/{}", file_name);
        
        if let Ok(metadata) = fs::metadata(&file_path) {
        if metadata.len() > 50 * 1024 * 1024 { // 50 MB 限制
            eprintln!("❌ 动画文件过大 (超过 5MB)，拒绝加载: {}", file_path);
            return self.static_display(b"TOO LARGE", status);
        }
    }
        // 1. 一次性把整个动画文件读进内存
        let anim_data = match fs::read(&file_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❌ 无法读取动画文件 {}: {}", file_path, e);
                // 读不到文件时防呆：显示一个错误提示并退出
                return self.static_display(b"FILE ERR", status);
            }
        };

        let frames_count = anim_data.len() / 27;
        if frames_count == 0 {
            eprintln!("❌ 动画文件为空或已损坏: {}", file_path);
            return Ok(());
        }

        let start_time = Instant::now();
        let total_duration = Duration::from_secs(duration_secs);
        
        // 2. 设定帧间隔 (15 FPS = 约 66 毫秒)
        let frame_interval = Duration::from_millis(66); 

        // 3. 切片读取：每次精准切出 27 个字节！
        // .cycle() 魔法：播到底部自动从头循环，直到总时长结束！
        let mut frame_iter = anim_data.chunks_exact(27).cycle();

        // 4. 开始无情推流
        while start_time.elapsed() < total_duration {
            if let Some(frame_chunk) = frame_iter.next() {
                // 震惊！由于 .bin 已经做好了列映射，我们直接把这 27 字节塞给底层！
                self.do_write_data(frame_chunk, status)?;
            }
            
            // 异步休眠，挂起当前任务，立刻将 CPU 交还给按键监听线程！
            tokio::time::sleep(frame_interval).await;
        }

        Ok(())
    }



    // 🌟 专为动态模块（天气、时间）设计的“强制静态、完美居中、零浪费”特化方法

    pub async fn write_data_static(&mut self, text: &[u8], status: u8) -> Result<()> {
        let mut display_data = Vec::new();
        let content = std::str::from_utf8(text).unwrap_or("");
        
        for ch in content.chars() {
            let key = ch.to_ascii_uppercase(); 
            if let Some(bytes) = CHAR_DICT.get(&key) {
                display_data.extend_from_slice(bytes);
                display_data.push(0x00); 
            }
        }

        if !display_data.is_empty() {
            display_data.pop(); 
        }

        self.static_display(&display_data, status)?;
        Ok(())
    }

    // 1. 加上 async 关键字
    async fn flow(&mut self, data: &[u8], status: u8) -> Result<()> {
        let mut start = 0;
        for i in 1..=data.len() {
            let mut off = [0u8; 27];
            if i > 27 {
                start += 1;
            }
            off[..i.min(27)].copy_from_slice(&data[start..start + i.min(27)]);
            self.do_write_data(&off, status)?;
            
            // 🚨 核心修复：把原先的 std::thread::sleep 换成 tokio 的异步 sleep！
            // 这样休眠时，程序立刻把控制权交还给主线程去检查按键！
            tokio::time::sleep(std::time::Duration::from_millis(128)).await;
        }
        Ok(())
    }

    fn static_display(&mut self, data: &[u8], status: u8) -> Result<()> {
        let mut display_data = [0u8; 27];
        if data.len() < 27 {
            let offset = (27 - data.len()) / 2;
            display_data[offset..offset + data.len()].copy_from_slice(data);
        } else {
            display_data[..27].copy_from_slice(&data[..27]);
        }
        self.do_write_data(&display_data, status)?;
        Ok(())
    }

    fn do_write_data(&mut self, values: &[u8], status: u8) -> Result<()> {
        self.left_screen.printf(&values[..14])?;
        let mut right_data = values[14..27].to_vec();
        right_data.push(status);
        self.right_screen.printf(&right_data)?;
        Ok(())
    }
}

impl LedScreenUnit {
    fn new(stb: u64, clk: u64, dio: u64) -> Result<Self> {
        let stb_pin = Pin::new(stb);
        let clk_pin = Pin::new(clk);
        let dio_pin = Pin::new(dio);

        stb_pin.export()?;
        clk_pin.export()?;
        dio_pin.export()?;

        stb_pin.set_direction(Direction::Out)?;
        clk_pin.set_direction(Direction::Out)?;
        dio_pin.set_direction(Direction::Out)?;

        Ok(Self {
            stb: stb_pin,
            clk: clk_pin,
            dio: dio_pin,
        })
    }

    fn set_show_model(&mut self) -> Result<()> {
        self.do_write_data(COMMAND1, &[])?;
        Ok(())
    }

    fn set_data_model(&mut self) -> Result<()> {
        self.do_write_data(COMMAND2, &[])?;
        Ok(())
    }

    fn power(&mut self, run: bool, light_level: u8) -> Result<()> {
        let command = if run {
            (light_level << 5 >> 5 | 0b11111000) & 0b10001111
        } else {
            0b10000000
        };
        self.do_write_data(command, &[])?;
        Ok(())
    }

    fn printf(&mut self, values: &[u8]) -> Result<()> {
        self.do_write_data(COMMAND3, values)?;
        Ok(())
    }

    fn do_write_data(&mut self, command: u8, values: &[u8]) -> Result<()> {
        self.stb.set_value(LOW)?;
        self.write_command_byte(command)?;
        
        for (i, &value) in values.iter().enumerate() {
            self.write_data_byte(value, i % 2 != 0)?;
        }
        
        self.stb.set_value(HIGH)?;
        Ok(())
    }

    fn write_command_byte(&mut self, value: u8) -> Result<()> {
        for i in 0..8 {
            let bit = (value >> i) & 0x01;
            self.write_bit(bit)?;
        }
        Ok(())
    }

    fn write_data_byte(&mut self, value: u8, fill_data: bool) -> Result<()> {
        for i in 0..5 {
            let bit = (value >> i) & 0x01;
            self.write_bit(bit)?;
        }
        
        if fill_data {
            for _ in 0..6 {
                self.write_bit(LOW)?;
            }
        }
        Ok(())
    }

    fn write_bit(&mut self, bit: u8) -> Result<()> {
        self.clk.set_value(LOW)?;
        self.dio.set_value(bit)?;
        self.clk.set_value(HIGH)?;
        Ok(())
    }
}

impl Drop for LedScreenUnit {
    fn drop(&mut self) {
        let _ = self.stb.unexport();
        let _ = self.clk.unexport();
        let _ = self.dio.unexport();
    }
}
