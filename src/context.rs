use std::path::PathBuf;

use crate::logging::LogFormat;

#[derive(clap::Parser, Debug)]
pub struct Context {
    #[arg(long, default_value_t = log::Level::Info, env = "BILI_LV6_HARDCORE_LOG_LEVEL")]
    pub log_level: log::Level,

    #[arg(long, default_value = "simple", env = "BILI_LV6_HARDCORE_LOG_FORMAT")]
    pub log_format: LogFormat,

    /// 连接指定的设备，adb 的 -s 参数的值
    #[arg(long)]
    pub device: Option<String>,

    #[arg(long, default_value = "adb", env = "BILI_LV6_HARDCORE_ADB")]
    pub adb: String,

    // 遇到无法识别的答案，容许随机选择的比例
    #[arg(
        long,
        default_value_t = 0.03,
        env = "BILI_LV6_HARDCORE_ANSWER_FALLBACK_RATIO"
    )]
    pub answer_fallback_ratio: f32,

    // 是否开启模型的思考功能
    #[arg(
        long,
        default_value_t = false,
        env = "BILI_LV6_HARDCORE_ANSWER_THINKING"
    )]
    pub answer_thinking: bool,

    #[arg(long, env = "BILI_LV6_HARDCORE_API_URL")]
    pub api_url: String,
    #[arg(long, env = "BILI_LV6_HARDCORE_API_MODEL")]
    pub api_model: String,
    #[arg(long, env = "BILI_LV6_HARDCORE_API_KEY")]
    pub api_key: String,

    /// 每百万输入token的费用
    #[arg(
        long,
        default_value = "1.5",
        env = "BILI_LV6_HARDCORE_API_COST_INPUT_PER_MILLION_TOKENS"
    )]
    pub api_cost_input: f64,
    /// 每百万输出token的费用
    #[arg(
        long,
        default_value = "4.5",
        env = "BILI_LV6_HARDCORE_API_COST_OUTPUT_PER_MILLION_TOKENS"
    )]
    pub api_cost_output: f64,

    /// 调试用，保存未识别的截图
    #[arg(long, env = "BILI_LV6_HARDCORE_DEBUG_SAVE_PATH")]
    pub debug_save_path: Option<PathBuf>,
}

impl Context {
    pub fn check(&mut self) {
        assert!(self.answer_fallback_ratio >= 0.0 && self.answer_fallback_ratio <= 1.0);
    }
}
