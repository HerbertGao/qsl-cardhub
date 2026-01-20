// 字体加载器
//
// 跨平台字体加载,支持系统字体和内嵌字体

use anyhow::Result;
use rusttype::Font;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 内嵌字体: Liberation Sans Bold (用于英文,开源字体)
const LIBERATION_SANS_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

/// 内嵌字体: Source Han Sans SC Bold (思源黑体，用于中文,开源字体)
const SOURCE_HAN_SANS_BOLD: &[u8] = include_bytes!("../../assets/fonts/SourceHanSansSC-Bold.otf");

/// 字体加载器
pub struct FontLoader {
    /// 字体缓存
    fonts: HashMap<String, Font<'static>>,
}

impl FontLoader {
    /// 创建新的字体加载器
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// 加载字体
    ///
    /// 优先尝试加载系统字体,失败则使用内嵌字体
    ///
    /// # 参数
    /// - `font_name`: 字体名称(用于缓存，"english" 或 "chinese")
    pub fn load_font(&mut self, font_name: &str) -> Result<&Font<'static>> {
        // 如果已经加载,直接返回
        if self.fonts.contains_key(font_name) {
            return Ok(self.fonts.get(font_name).unwrap());
        }

        log::info!("正在加载字体: {}", font_name);

        // 根据字体名称选择加载策略
        let font = match font_name {
            "chinese" => self.load_chinese_font()?,
            _ => self.load_english_font()?,
        };

        self.fonts.insert(font_name.to_string(), font);
        Ok(self.fonts.get(font_name).unwrap())
    }

    /// 加载英文字体（直接使用内嵌的 Bold 字体）
    fn load_english_font(&self) -> Result<Font<'static>> {
        // 直接使用内嵌的 Arial Bold 字体，确保粗体效果
        log::info!("使用内嵌 Arial Bold 字体");
        self.load_embedded_font()
    }

    /// 加载中文字体（直接使用内嵌的 Bold 字体）
    fn load_chinese_font(&self) -> Result<Font<'static>> {
        // 直接使用内嵌的思源黑体 Bold 字体，确保粗体效果
        log::info!("使用内嵌思源黑体 Bold 字体");
        self.load_embedded_chinese_font()
    }

    /// 尝试加载系统字体
    fn try_load_system_font(&self) -> Result<Font<'static>> {
        // 不同平台的字体路径
        let font_paths = self.get_system_font_paths();

        for path in font_paths {
            if let Ok(font_data) = fs::read(&path) {
                if let Some(font) = Font::try_from_vec(font_data) {
                    log::debug!("使用系统字体: {}", path.display());
                    return Ok(font);
                }
            }
        }

        anyhow::bail!("未找到可用的系统字体")
    }

    /// 获取系统字体路径列表
    fn get_system_font_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // macOS 系统字体
            paths.push(PathBuf::from("/System/Library/Fonts/Helvetica.ttc"));
            paths.push(PathBuf::from("/System/Library/Fonts/Arial.ttf"));
            paths.push(PathBuf::from("/Library/Fonts/Arial.ttf"));
            paths.push(PathBuf::from("/System/Library/Fonts/HelveticaNeue.ttc"));
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 系统字体
            let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            paths.push(PathBuf::from(format!("{}\\Fonts\\arial.ttf", windows_dir)));
            paths.push(PathBuf::from(format!("{}\\Fonts\\calibri.ttf", windows_dir)));
            paths.push(PathBuf::from(format!("{}\\Fonts\\segoeui.ttf", windows_dir)));
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 系统字体
            paths.push(PathBuf::from("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"));
            paths.push(PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"));
            paths.push(PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"));
            paths.push(PathBuf::from("/usr/share/fonts/truetype/freefont/FreeSans.ttf"));
        }

        paths
    }

    /// 尝试加载系统中文字体
    fn try_load_system_chinese_font(&self) -> Result<Font<'static>> {
        let font_paths = self.get_system_chinese_font_paths();

        for path in font_paths {
            if let Ok(font_data) = fs::read(&path) {
                if let Some(font) = Font::try_from_vec(font_data) {
                    log::debug!("使用系统中文字体: {}", path.display());
                    return Ok(font);
                }
            }
        }

        anyhow::bail!("未找到可用的系统中文字体")
    }

    /// 获取系统中文字体路径列表
    fn get_system_chinese_font_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "macos")]
        {
            // macOS 系统中文字体
            paths.push(PathBuf::from("/System/Library/Fonts/PingFang.ttc"));
            paths.push(PathBuf::from("/System/Library/Fonts/STHeiti Light.ttc"));
            paths.push(PathBuf::from("/System/Library/Fonts/Hiragino Sans GB.ttc"));
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 系统中文字体
            let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            paths.push(PathBuf::from(format!("{}\\Fonts\\msyh.ttc", windows_dir))); // 微软雅黑
            paths.push(PathBuf::from(format!("{}\\Fonts\\msyhbd.ttc", windows_dir))); // 微软雅黑粗体
            paths.push(PathBuf::from(format!("{}\\Fonts\\simsun.ttc", windows_dir))); // 宋体
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 系统中文字体
            paths.push(PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"));
            paths.push(PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttf"));
            paths.push(PathBuf::from("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc")); // 文泉驿
        }

        paths
    }

    /// 加载内嵌英文字体
    ///
    /// 使用编译时嵌入的 Liberation Sans Bold 字体(开源)
    fn load_embedded_font(&self) -> Result<Font<'static>> {
        log::debug!("尝试加载内嵌英文字体 Liberation Sans Bold");

        if let Some(font) = Font::try_from_bytes(LIBERATION_SANS_BOLD) {
            log::info!("成功加载内嵌字体: Liberation Sans Bold");
            return Ok(font);
        }

        anyhow::bail!("无法加载内嵌英文字体")
    }

    /// 加载内嵌中文字体
    ///
    /// 使用编译时嵌入的思源黑体
    fn load_embedded_chinese_font(&self) -> Result<Font<'static>> {
        log::debug!("尝试加载内嵌中文字体 Source Han Sans SC Bold");

        if let Some(font) = Font::try_from_bytes(SOURCE_HAN_SANS_BOLD) {
            log::info!("成功加载内嵌中文字体: Source Han Sans SC Bold");
            return Ok(font);
        }

        anyhow::bail!("无法加载内嵌中文字体")
    }

    /// 获取已加载的字体
    pub fn get_font(&self, font_name: &str) -> Option<&Font<'static>> {
        self.fonts.get(font_name)
    }

    /// 清空字体缓存
    pub fn clear_cache(&mut self) {
        self.fonts.clear();
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_english_font() {
        let mut loader = FontLoader::new();
        let font = loader.load_font("english");
        assert!(font.is_ok(), "应该成功加载英文字体");
    }

    #[test]
    fn test_load_chinese_font() {
        let mut loader = FontLoader::new();
        let font = loader.load_font("chinese");
        assert!(font.is_ok(), "应该成功加载中文字体");
    }

    #[test]
    fn test_font_cache() {
        let mut loader = FontLoader::new();

        // 第一次加载
        loader.load_font("english").unwrap();

        // 第二次应该从缓存获取
        let cached_font = loader.get_font("english");
        assert!(cached_font.is_some(), "应该从缓存中获取字体");
    }
}
