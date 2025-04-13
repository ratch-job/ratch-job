use crate::app::core::AppManager;
use crate::cache::core::CacheManager;
use crate::common::app_config::AppConfig;
use crate::job::core::JobManager;
use crate::metrics::core::MetricsManager;
use crate::raft::cluster::node_manager::ClusterNodeManager;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::core::Store;
use crate::raft::RatchRaft;
use crate::schedule::batch_call::BatchCallManager;
use crate::schedule::core::ScheduleManager;
use crate::sequence::SequenceManager;
use crate::task::core::TaskManager;
use crate::task::task_history::TaskHistoryManager;
use crate::user::core::UserManager;
use actix::Addr;
use bean_factory::FactoryData;
use chrono::FixedOffset;
use std::sync::Arc;

#[derive(Clone)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub timezone_offset: Arc<FixedOffset>,
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub sequence_manager: Addr<SequenceManager>,
    pub schedule_manager: Addr<ScheduleManager>,
    pub task_manager: Addr<TaskManager>,
    pub task_history_manager: Addr<TaskHistoryManager>,
    pub metrics_manager: Addr<MetricsManager>,
    pub raft_request_route: Arc<RaftRequestRoute>,
    pub factory_data: FactoryData,
    pub raft: Arc<RatchRaft>,
    pub raft_store: Arc<Store>,
    pub cluster_node_manager: Addr<ClusterNodeManager>,
    pub batch_call_manager: Addr<BatchCallManager>,
    pub cache_manager: Addr<CacheManager>,
    pub user_manager: Addr<UserManager>,
}
