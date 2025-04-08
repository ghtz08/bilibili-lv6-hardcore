use crate::logging::LogFormat;

#[derive(clap::Parser, Debug)]
pub struct Context {
    #[arg(long, default_value_t = log::Level::Info, env = "BILI_LV6_HARDCORE_LOG_LEVEL")]
    pub log_level: log::Level,

    #[arg(long, default_value = "simple", env = "BILI_LV6_HARDCORE_LOG_FORMAT")]
    pub log_format: LogFormat,

    #[arg(long)]
    pub devices: Vec<String>,
}
