use crate::common::constant;
use std::sync::Arc;

const DEFAULT_DB_PATH: &str = "ratch_db";
const DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH: &str = "/xxl-job-admin";
#[derive(Default, Clone, Debug)]
pub struct AppConfig {
    pub local_db_dir: String,
    pub http_api_port: u16,
    pub xxl_job_prefix_path: String,
    pub xxl_default_access_token: String,
    pub app_instance_health_timeout: u32,
    pub http_console_port: u16,
    pub http_workers: Option<usize>,
    pub grpc_cluster_port: u16,
    pub run_in_docker: bool,
    pub gmt_fixed_offset_hours: Option<i32>,
    pub raft_node_id: u64,
    pub raft_node_addr: String,
    pub raft_auto_init: bool,
    pub raft_join_addr: String,
    pub raft_snapshot_log_size: u64,
    pub cluster_token: Arc<String>,
    pub metrics_enable: bool,
    pub metrics_collect_interval_second: u64,
    pub metrics_log_interval_second: u64,
    pub metrics_log_enable: bool,
}

impl AppConfig {
    pub fn init_from_env() -> Self {
        let run_in_docker = std::env::var("RATCH_RUN_IN_DOCKER")
            .unwrap_or("".to_owned())
            .eq_ignore_ascii_case("true");
        let local_db_dir = Self::get_data_dir(run_in_docker);
        let xxl_job_prefix_path = Self::get_xxl_job_prefix_path();
        let xxl_default_access_token =
            std::env::var("RATCH_XXL_DEFAULT_ACCESS_TOKEN").unwrap_or("default_token".to_string());
        let http_api_port = std::env::var("RATCH_HTTP_API_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(8725);
        let http_console_port = std::env::var("RATCH_HTTP_CONSOLE_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(http_api_port + 100);
        let grpc_cluster_port = std::env::var("RATCH_GRPC_CLUSTER_PORT")
            .unwrap_or_default()
            .parse()
            .unwrap_or(http_api_port + 200);
        let app_instance_health_timeout = std::env::var("RATCH_INSTANCE_HEALTH_TIMEOUT")
            .unwrap_or("90".to_owned())
            .parse()
            .unwrap_or(90);
        let http_workers = std::env::var("RATCH_HTTP_WORKERS")
            .unwrap_or("".to_owned())
            .parse()
            .ok();
        let gmt_fixed_offset_hours = std::env::var("RATCH_GMT_OFFSET_HOURS")
            .unwrap_or_default()
            .parse()
            .ok();
        let cluster_token = std::env::var("RATCH_CLUSTER_TOKEN")
            .map(Arc::new)
            .unwrap_or(constant::EMPTY_ARC_STR.clone());
        let raft_node_id = std::env::var("RATCH_RAFT_NODE_ID")
            .unwrap_or("1".to_owned())
            .parse()
            .unwrap_or(1);
        let raft_node_addr = std::env::var("RATCH_RAFT_NODE_ADDR")
            .unwrap_or(format!("127.0.0.1:{}", &grpc_cluster_port));
        let raft_auto_init = std::env::var("RATCH_RAFT_AUTO_INIT")
            .unwrap_or("".to_owned())
            .parse()
            .unwrap_or(raft_node_id == 1);
        let raft_join_addr = std::env::var("RATCH_RAFT_JOIN_ADDR").unwrap_or_default();
        let raft_snapshot_log_size = std::env::var("RATCH_RAFT_SNAPSHOT_LOG_SIZE")
            .unwrap_or("10000".to_owned())
            .parse()
            .unwrap_or(10000);
        let metrics_log_enable = std::env::var("RATCH_METRICS_ENABLE_LOG")
            .unwrap_or("false".to_owned())
            .parse()
            .unwrap_or(false);
        let mut metrics_log_interval_second = std::env::var("RATCH_METRICS_LOG_INTERVAL_SECOND")
            .unwrap_or("60".to_owned())
            .parse()
            .unwrap_or(60);
        if metrics_log_interval_second < 5 {
            metrics_log_interval_second = 5;
        }
        let metrics_enable = std::env::var("RATCH_ENABLE_METRICS")
            .unwrap_or("true".to_owned())
            .parse()
            .unwrap_or(true);
        let mut metrics_collect_interval_second =
            std::env::var("RATCH_METRICS_COLLECT_INTERVAL_SECOND")
                .unwrap_or("15".to_owned())
                .parse()
                .unwrap_or(15);
        if metrics_log_interval_second < metrics_collect_interval_second {
            metrics_collect_interval_second = metrics_log_interval_second;
        }
        Self {
            local_db_dir,
            http_api_port,
            xxl_job_prefix_path,
            xxl_default_access_token,
            app_instance_health_timeout,
            http_console_port,
            http_workers,
            grpc_cluster_port,
            run_in_docker,
            gmt_fixed_offset_hours,
            cluster_token,
            raft_node_id,
            raft_node_addr,
            raft_auto_init,
            raft_join_addr,
            raft_snapshot_log_size,
            metrics_enable,
            metrics_log_enable,
            metrics_collect_interval_second,
            metrics_log_interval_second,
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
        if let Ok(v) = std::env::var("DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH") {
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
