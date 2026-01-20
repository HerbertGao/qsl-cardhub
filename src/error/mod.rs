// 错误处理模块
//
// 定义应用的自定义错误类型

use thiserror::Error;

/// 应用错误类型
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    // ========== 配置相关错误 ==========
    /// 配置文件不存在
    #[error("配置文件不存在: {0}")]
    ConfigNotFound(String),

    /// 配置文件解析失败
    #[error("配置文件解析失败: {0}")]
    ConfigParseFailed(String),

    /// 配置文件保存失败
    #[error("配置文件保存失败: {0}")]
    ConfigSaveFailed(String),

    /// Profile 不存在
    #[error("Profile 不存在: {0}")]
    ProfileNotFound(String),

    /// Profile 无效
    #[error("Profile 无效: {0}")]
    ProfileInvalid(String),

    // ========== 打印相关错误 ==========
    /// 打印机不可用
    #[error("打印机不可用: {0}")]
    PrinterUnavailable(String),

    /// 打印机未找到
    #[error("打印机未找到: {0}")]
    PrinterNotFound(String),

    /// 打印任务失败
    #[error("打印任务失败: {0}")]
    PrintJobFailed(String),

    /// TSPL 指令生成失败
    #[error("TSPL 指令生成失败: {0}")]
    TsplGenerationFailed(String),

    /// 条形码生成失败
    #[error("条形码生成失败: {0}")]
    BarcodeGenerationFailed(String),

    /// 字体加载失败
    #[error("字体加载失败: {0}")]
    FontLoadFailed(String),

    /// 文本渲染失败
    #[error("文本渲染失败: {0}")]
    TextRenderFailed(String),

    // ========== 文件相关错误 ==========
    /// 文件未找到
    #[error("文件未找到: {0}")]
    FileNotFound(String),

    /// 文件读取失败
    #[error("文件读取失败: {0}")]
    FileReadFailed(String),

    /// 文件写入失败
    #[error("文件写入失败: {0}")]
    FileWriteFailed(String),

    /// 目录创建失败
    #[error("目录创建失败: {0}")]
    DirectoryCreationFailed(String),

    // ========== 数据相关错误 ==========
    /// 数据无效
    #[error("数据无效: {0}")]
    DataInvalid(String),

    /// 数据格式错误
    #[error("数据格式错误: {0}")]
    DataFormatError(String),

    /// 参数无效
    #[error("参数无效: {0}")]
    InvalidParameter(String),

    // ========== 系统错误 ==========
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serde(String),

    /// 权限不足
    #[error("权限不足: {0}")]
    PermissionDenied(String),

    /// 操作超时
    #[error("操作超时: {0}")]
    Timeout(String),

    // ========== 通用错误 ==========
    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl AppError {
    /// 获取用户友好的中文错误消息
    pub fn user_message(&self) -> String {
        match self {
            // 配置相关
            AppError::ConfigNotFound(path) => {
                format!(
                    "找不到配置文件：{}\n\n请检查文件是否存在，或尝试重新创建配置。",
                    path
                )
            }
            AppError::ConfigParseFailed(reason) => {
                format!(
                    "配置文件格式错误：{}\n\n请检查配置文件格式是否正确，或删除后重新创建。",
                    reason
                )
            }
            AppError::ConfigSaveFailed(reason) => {
                format!(
                    "保存配置失败：{}\n\n请检查是否有足够的磁盘空间和文件写入权限。",
                    reason
                )
            }
            AppError::ProfileNotFound(id) => {
                format!(
                    "找不到 Profile：{}\n\n该配置可能已被删除，请选择其他配置或创建新配置。",
                    id
                )
            }
            AppError::ProfileInvalid(reason) => {
                format!("Profile 配置无效：{}\n\n请检查配置参数是否正确。", reason)
            }

            // 打印相关
            AppError::PrinterUnavailable(name) => {
                format!(
                    "打印机不可用：{}\n\n请检查打印机是否已连接、开机，并确保驱动程序已正确安装。",
                    name
                )
            }
            AppError::PrinterNotFound(name) => {
                format!(
                    "找不到打印机：{}\n\n请确认打印机名称是否正确，或在系统中添加该打印机。",
                    name
                )
            }
            AppError::PrintJobFailed(reason) => {
                format!(
                    "打印任务失败：{}\n\n请检查打印机状态和纸张是否正常。",
                    reason
                )
            }
            AppError::TsplGenerationFailed(reason) => {
                format!(
                    "生成打印指令失败：{}\n\n这可能是程序内部错误，请联系开发者。",
                    reason
                )
            }
            AppError::BarcodeGenerationFailed(reason) => {
                format!("生成条形码失败：{}\n\n请检查呼号格式是否正确。", reason)
            }
            AppError::FontLoadFailed(reason) => {
                format!("加载字体失败：{}\n\n请检查字体文件是否存在。", reason)
            }
            AppError::TextRenderFailed(reason) => {
                format!("渲染文本失败：{}", reason)
            }

            // 文件相关
            AppError::FileNotFound(path) => {
                format!("文件不存在：{}\n\n请检查文件路径是否正确。", path)
            }
            AppError::FileReadFailed(reason) => {
                format!("读取文件失败：{}\n\n请检查文件权限和路径是否正确。", reason)
            }
            AppError::FileWriteFailed(reason) => {
                format!("写入文件失败：{}\n\n请检查磁盘空间和文件权限。", reason)
            }
            AppError::DirectoryCreationFailed(reason) => {
                format!("创建目录失败：{}\n\n请检查路径和权限。", reason)
            }

            // 数据相关
            AppError::DataInvalid(reason) => {
                format!("数据无效：{}\n\n请检查输入的数据格式。", reason)
            }
            AppError::DataFormatError(reason) => {
                format!("数据格式错误：{}", reason)
            }
            AppError::InvalidParameter(reason) => {
                format!("参数无效：{}\n\n请检查输入参数。", reason)
            }

            // 系统错误
            AppError::Io(err) => {
                format!("系统 IO 错误：{}\n\n这可能是系统权限或资源问题。", err)
            }
            AppError::Serde(reason) => {
                format!("数据序列化错误：{}", reason)
            }
            AppError::PermissionDenied(reason) => {
                format!(
                    "权限不足：{}\n\n请以管理员身份运行程序，或检查文件/目录权限。",
                    reason
                )
            }
            AppError::Timeout(reason) => {
                format!("操作超时：{}\n\n请检查网络连接或稍后重试。", reason)
            }

            // 通用错误
            AppError::Other(msg) => msg.clone(),
        }
    }

    /// 获取错误类别（用于日志记录）
    pub fn category(&self) -> &'static str {
        match self {
            AppError::ConfigNotFound(_)
            | AppError::ConfigParseFailed(_)
            | AppError::ConfigSaveFailed(_)
            | AppError::ProfileNotFound(_)
            | AppError::ProfileInvalid(_) => "配置",

            AppError::PrinterUnavailable(_)
            | AppError::PrinterNotFound(_)
            | AppError::PrintJobFailed(_)
            | AppError::TsplGenerationFailed(_)
            | AppError::BarcodeGenerationFailed(_)
            | AppError::FontLoadFailed(_)
            | AppError::TextRenderFailed(_) => "打印",

            AppError::FileNotFound(_)
            | AppError::FileReadFailed(_)
            | AppError::FileWriteFailed(_)
            | AppError::DirectoryCreationFailed(_) => "文件",

            AppError::DataInvalid(_)
            | AppError::DataFormatError(_)
            | AppError::InvalidParameter(_) => "数据",

            AppError::Io(_)
            | AppError::Serde(_)
            | AppError::PermissionDenied(_)
            | AppError::Timeout(_) => "系统",

            AppError::Other(_) => "其他",
        }
    }

    /// 判断错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 可恢复的错误
            AppError::PrinterUnavailable(_)
            | AppError::PrintJobFailed(_)
            | AppError::FileReadFailed(_)
            | AppError::FileWriteFailed(_)
            | AppError::Timeout(_) => true,

            // 不可恢复的错误
            AppError::ConfigParseFailed(_)
            | AppError::ProfileInvalid(_)
            | AppError::DataInvalid(_)
            | AppError::DataFormatError(_)
            | AppError::InvalidParameter(_) => false,

            // 其他情况根据具体情况判断
            _ => true,
        }
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Serde(err.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(err: toml::ser::Error) -> Self {
        AppError::Serde(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serde(err.to_string())
    }
}

/// 应用 Result 类型别名
#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;
