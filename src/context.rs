use crate::logging::LogFormat;

#[derive(clap::Parser, Debug)]
pub struct Context {
    #[arg(long, default_value_t = log::Level::Info, env = "BILI_LV6_HARDCORE_LOG_LEVEL")]
    pub log_level: log::Level,

    #[arg(long, default_value = "simple", env = "BILI_LV6_HARDCORE_LOG_FORMAT")]
    pub log_format: LogFormat,

    #[arg(long)]
    pub devices: Vec<String>,

    #[arg(long, default_value = "adb", env = "BILI_LV6_HARDCORE_ADB")]
    pub adb: String,

    #[arg(long, env = "BILI_LV6_HARDCORE_API_URL")]
    pub api_url: String,
    #[arg(long, env = "BILI_LV6_HARDCORE_API_MODEL")]
    pub api_model: String,
    #[arg(long, env = "BILI_LV6_HARDCORE_API_KEY")]
    pub api_key: String,

    #[arg(
        long,
        default_value = "1.5",
        env = "BILI_LV6_HARDCORE_API_COST_INPUT_PER_MILLION_TOKENS"
    )]
    pub api_cost_input: f64,
    #[arg(
        long,
        default_value = "4.5",
        env = "BILI_LV6_HARDCORE_API_COST_OUTPUT_PER_MILLION_TOKENS"
    )]
    pub api_cost_output: f64,
}
