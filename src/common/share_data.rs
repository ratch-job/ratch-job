use crate::app::core::AppManager;
use crate::common::app_config::AppConfig;
use crate::job::core::JobManager;
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::core::Store;
use crate::raft::RatchRaft;
use crate::schedule::core::ScheduleManager;
use crate::sequence::SequenceManager;
use crate::task::core::TaskManager;
use crate::task::task_history::TaskHistoryManager;
use actix::Addr;
use bean_factory::FactoryData;
use std::sync::Arc;
use crate::webhook::core::WebHookManager;

#[derive(Clone)]
pub struct ShareData {
    pub app_config: Arc<AppConfig>,
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub sequence_manager: Addr<SequenceManager>,
    pub schedule_manager: Addr<ScheduleManager>,
    pub task_manager: Addr<TaskManager>,
    pub task_history_manager: Addr<TaskHistoryManager>,
    pub webhook_manager: Addr<WebHookManager>,
    pub raft_request_route: Arc<RaftRequestRoute>,
    pub factory_data: FactoryData,
    pub raft: Arc<RatchRaft>,
    pub raft_store: Arc<Store>,
}
