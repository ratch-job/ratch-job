use std::sync::Arc;

const DEFAULT_DB_PATH: &str = "ratch_db";
const DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH: &str = "/xxl-job-admin";
#[derive(Default, Clone, Debug)]
pub struct AppConfig {
    pub local_db_dir: String,
    pub http_api_port: u16,
    pub xxl_job_prefix_path: String,
    pub http_console_port: u16,
    pub http_workers: Option<usize>,
    pub grpc_cluster_port: u16,
    pub run_in_docker: bool,
    pub gmt_fixed_offset_hours: Option<i32>,
}

impl AppConfig {
    pub fn init_from_env() -> Self {
        let run_in_docker = std::env::var("RATCH_RUN_IN_DOCKER")
            .unwrap_or("".to_owned())
            .eq_ignore_ascii_case("true");
        let local_db_dir = Self::get_data_dir(run_in_docker);
        let xxl_job_prefix_path = Self::get_xxl_job_prefix_path();
        let http_api_port = std::env::var("RATCH_HTTP_API_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(8725);
        let http_console_port = std::env::var("RATCH_HTTP_CONSOLE_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(8825);
        let grpc_cluster_port = std::env::var("RATCH_GRPC_CLUSTER_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(8925);
        let http_workers = std::env::var("RATCH_HTTP_WORKERS")
            .unwrap_or("".to_owned())
            .parse()
            .ok();
        let gmt_fixed_offset_hours = std::env::var("RATCH_GMT_OFFSET_HOURS")
            .unwrap_or_default()
            .parse()
            .ok();
        Self {
            local_db_dir,
            http_api_port,
            xxl_job_prefix_path,
            http_console_port,
            http_workers,
            grpc_cluster_port,
            run_in_docker,
            gmt_fixed_offset_hours,
        }
    }

    /// 获取数据目录
    fn get_data_dir(run_in_docker: bool) -> String {
        if let Ok(v) = std::env::var("RATCH_DATA_DIR") {
            v
        } else if run_in_docker {
            // 运行在docker，默认值保持一致
            DEFAULT_DB_PATH.to_owned()
        } else {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            {
                if let Some(mut home) = dirs::home_dir() {
                    home.push(".local/share/ratchjob/ratch_db");
                    return home.to_string_lossy().to_string();
                }
            }
            // windows系统默认值保持一致
            DEFAULT_DB_PATH.to_owned()
        }
    }

    fn get_xxl_job_prefix_path() -> String {
        if let Ok(v) = std::env::var("RATCH_DATA_DIR") {
            if v.len() < 2 {
                DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH.to_owned()
            } else {
                v
            }
        } else {
            DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH.to_owned()
        }
    }

    pub fn get_grpc_cluster_addr(&self) -> String {
        format!("0.0.0.0:{}", &self.grpc_cluster_port)
    }

    pub fn get_http_api_addr(&self) -> String {
        format!("0.0.0.0:{}", &self.http_api_port)
    }

    pub fn get_http_console_addr(&self) -> String {
        format!("0.0.0.0:{}", &self.http_console_port)
    }
}
