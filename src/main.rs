use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use clap::Parser;
use env_logger::TimestampPrecision;
use env_logger_timezone_fmt::{TimeZoneFormat, TimeZoneFormatEnv};
use ratchjob::cli;
use ratchjob::cli::Commands;
use ratchjob::common::app_config::AppConfig;
use ratchjob::common::get_app_version;
use ratchjob::common::share_data::ShareData;
use ratchjob::console::console_config;
use ratchjob::openapi::openapi_config;
use ratchjob::starter::{build_share_data, config_factory};
use ratchjob::web_config::app_config;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_opt = cli::Cli::parse();
    init_env(&cli_opt.env_file);
    let rust_log = std::env::var("RUST_LOG").unwrap_or("info".to_owned());
    std::env::set_var("RUST_LOG", &rust_log);
    let sys_config = Arc::new(AppConfig::init_from_env());
    let timezone_fmt = Arc::new(TimeZoneFormatEnv::new(
        sys_config.gmt_fixed_offset_hours.map(|v| v * 60 * 60),
        Some(TimestampPrecision::Micros),
    ));
    env_logger::Builder::from_default_env()
        .format(move |buf, record| TimeZoneFormat::new(buf, &timezone_fmt).write(record))
        .init();
    if let Some(cmd) = cli_opt.command {
        return run_subcommand(cmd).await;
    }
    // 这里不使用log:info避免日志等级高于info时不打印
    println!("version:{}, RUST_LOG:{}", get_app_version(), &rust_log);
    println!("data dir:{}", sys_config.local_db_dir);
    let factory_data = config_factory(sys_config.clone()).await?;
    let app_data = build_share_data(factory_data.clone())?;
    let http_addr = sys_config.get_http_api_addr();
    let grpc_addr = sys_config.get_grpc_cluster_addr();
    log::info!("http api server addr:{}", &http_addr);
    log::info!("grpc cluster server addr:{}", &grpc_addr);

    let app_console_data = app_data.clone();

    std::thread::spawn(move || {
        actix_rt::System::with_tokio_rt(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
        })
        .block_on(run_console_web(app_console_data));
    });

    let mut server = HttpServer::new(move || {
        let app_data = app_data.clone();
        let app_config_shard = app_data.app_config.clone();
        App::new()
            .app_data(Data::new(app_data))
            .wrap(middleware::Logger::default())
            .configure(app_config(app_config_shard))
    });
    if let Some(num) = sys_config.http_workers {
        server = server.workers(num);
    }
    // 这里不使用log:info避免日志等级高于info时不打印
    println!("ratch-job started");
    server.bind(http_addr)?.run().await?;
    Ok(())
}

async fn run_subcommand(commands: Commands) -> Result<(), Box<dyn Error>> {
    match commands {
        Commands::About => {
            log::info!("version:{}", get_app_version());
        }
    }
    Ok(())
}

fn init_env(env_path: &str) {
    if env_path.is_empty() {
        dotenv::dotenv().ok();
    } else {
        dotenv::from_path(env_path).ok();
    }
}

async fn run_console_web(source_app_data: Arc<ShareData>) {
    let http_console_addr = source_app_data.app_config.get_http_console_addr();
    log::info!("console server http addr:{}", &http_console_addr);
    let app_data = Data::new(source_app_data.clone());
    HttpServer::new(move || {
        let app_data = app_data.clone();
        App::new()
            .app_data(app_data)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(console_config)
    })
    .workers(2)
    .bind(http_console_addr)
    .unwrap()
    .run()
    .await
    .ok();
}
