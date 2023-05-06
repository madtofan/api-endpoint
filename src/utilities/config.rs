use clap::Parser;

#[derive(Parser)]
pub struct AppConfig {
    #[arg(long, env)]
    pub rust_log: String,
    #[arg(long, env)]
    pub bearer_secret: String,
    #[arg(long, env)]
    pub refresh_secret: String,
    #[arg(long, env)]
    pub service_url: String,
    #[arg(long, env)]
    pub service_port: u32,
    #[arg(long, env)]
    pub cors_origin: String,
    #[arg(long, env)]
    pub user_host: String,
    #[arg(long, env)]
    pub user_port: u32,
    #[arg(long, env)]
    pub email_host: String,
    #[arg(long, env)]
    pub email_port: u32,
    #[arg(long, env)]
    pub templating_host: String,
    #[arg(long, env)]
    pub templating_port: u32,
}
