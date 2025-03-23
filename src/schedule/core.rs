use crate::common::byte_utils::id_to_bin;
use crate::common::constant::{
    EMPTY_ARC_STR, ERR_MSG_TASK_TIMEOUT, JOB_TASK_HISTORY_TABLE_NAME, JOB_TASK_RUNNING_TABLE_NAME,
    JOB_TASK_TABLE_NAME,
};
use crate::common::datetime_utils::{
    get_datetime_by_second, get_datetime_millis, get_local_offset, now_millis, now_millis_i64,
    now_second_u32,
};
use crate::common::pb::data_object::JobTaskDo;
use crate::job::model::actor_model::{JobManagerRaftReq, JobManagerReq};
use crate::job::model::enum_type::ScheduleType;
use crate::job::model::job::{JobInfo, JobTaskLogQueryParam};
use crate::metrics::core::MetricsManager;
use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsRecord, MetricsRequest};
use crate::raft::cluster::model::{VoteChangeRequest, VoteChangeResponse, VoteInfo};
use crate::raft::cluster::route::RaftRequestRoute;
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use crate::raft::store::ClientRequest;
use crate::schedule::job_task::JobTaskLogGroup;
use crate::schedule::model::actor_model::{
    ScheduleManagerRaftReq, ScheduleManagerRaftResult, ScheduleManagerReq, ScheduleManagerResult,
};
use crate::schedule::model::{JobRunState, RedoInfo, RedoType, TriggerInfo};
use crate::task::core::TaskManager;
use crate::task::model::actor_model::{RedoTaskItem, TaskManagerReq, TriggerItem};
use crate::task::model::enum_type::TaskStatusType;
use crate::task::model::task::{JobTaskInfo, TaskCallBackParam, UpdateTaskMetricsInfo};
use actix::prelude::*;
use actix_web::cookie::time::macros::datetime;
use anyhow::anyhow;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset, TimeZone, Utc};
use futures_util::TryFutureExt;
use inner_mem_cache::TimeoutSet;
use quick_protobuf::{BytesReader, Writer};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct ScheduleManager {
    job_run_state: HashMap<u64, JobRunState>,
    active_time_set: TimeoutSet<TriggerInfo>,
    fixed_offset: FixedOffset,
    task_manager: Option<Addr<TaskManager>>,
    raft_request_route: Option<Arc<RaftRequestRoute>>,
    metrics_manager: Option<Addr<MetricsManager>>,
    /// 运行中的任务实例
    pub(crate) running_task: HashMap<u64, Arc<JobTaskInfo>>,
    /// 失败重试集
    redo_set: TimeoutSet<RedoInfo>,
    /// 任务实例历史记录
    history_task: JobTaskLogGroup,
    history_task_log_limit: usize,
    last_vote_info: VoteInfo,
    local_is_master: bool,
    app_start_second: u32,
    last_trigger_time: u32,
    last_retry_time: u32,
    running_heartbeat: bool,
    default_timeout_second: u32,
}

impl Actor for ScheduleManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("ScheduleManager started");
    }
}

impl Inject for ScheduleManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.task_manager = factory_data.get_actor();
        self.raft_request_route = factory_data.get_bean();
        self.metrics_manager = factory_data.get_actor();
    }
}

impl ScheduleManager {
    pub fn new(offset_seconds: Option<i32>) -> Self {
        let fixed_offset = if let Some(offset_value) = offset_seconds {
            FixedOffset::east_opt(offset_value).unwrap_or(get_local_offset())
        } else {
            get_local_offset()
        };
        ScheduleManager {
            job_run_state: HashMap::new(),
            active_time_set: TimeoutSet::new(),
            fixed_offset,
            task_manager: None,
            raft_request_route: None,
            metrics_manager: None,
            running_task: Default::default(),
            history_task: JobTaskLogGroup::new(),
            history_task_log_limit: 10000,
            last_vote_info: VoteInfo::default(),
            local_is_master: false,
            app_start_second: now_second_u32(),
            last_trigger_time: 0,
            last_retry_time: 0,
            redo_set: TimeoutSet::new(),
            running_heartbeat: false,
            default_timeout_second: 24 * 60 * 60, // 默认24小时
        }
    }

    fn active_job(&mut self, job_id: u64, time: u32, version: u32) {
        // 前期如果不是leader节点，则不执行调度任务
        if !self.local_is_master || time == 0 {
            return;
        }
        self.active_time_set
            .add(time as u64, TriggerInfo::new(job_id, time, version));
    }

    fn active_retry_task(&mut self, task_id: u64, time: u32, redo_type: RedoType) {
        if !self.local_is_master {
            return;
        }
        self.redo_set
            .add(time as u64, RedoInfo::new(task_id, redo_type));
    }

    fn update_job_trigger_time(&mut self, job_id: u64, last_time: u32, next_time: u32) {
        if let Some(job) = self.job_run_state.get_mut(&job_id) {
            job.pre_trigger_time = last_time;
            job.next_trigger_time = next_time;
        }
    }

    fn update_job(&mut self, job_info: Arc<JobInfo>) {
        let job_id = job_info.id;
        if job_info.schedule_type == ScheduleType::None || job_info.enable == false {
            self.job_run_state.remove(&job_id);
            return;
        }
        let mut active_job_param = None;
        if let Some(job_run_state) = self.job_run_state.get_mut(&job_id) {
            let change_schedule = job_run_state.update_job(job_info);
            if change_schedule {
                let now_second = now_second_u32();
                let reset_time = if job_run_state.pre_trigger_time == now_second {
                    //从下一秒开始
                    now_second
                } else {
                    now_second - 1
                };
                if let Some(now_datetime) = get_datetime_by_second(reset_time, &self.fixed_offset) {
                    let next_trigger_time =
                        job_run_state.calculate_first_trigger_time(&now_datetime);
                    job_run_state.next_trigger_time = next_trigger_time;
                    active_job_param = Some((next_trigger_time, job_run_state.version))
                }
            }
        } else {
            let mut job_run_state = JobRunState::new(job_info);
            if let Some(now_datetime) = get_datetime_by_second(now_second_u32(), &self.fixed_offset)
            {
                let next_trigger_time = job_run_state.calculate_first_trigger_time(&now_datetime);
                job_run_state.next_trigger_time = next_trigger_time;
                active_job_param = Some((next_trigger_time, job_run_state.version))
            }
            self.job_run_state.insert(job_id, job_run_state);
        }
        if let Some((next_trigger_time, version)) = active_job_param {
            self.active_job(job_id, next_trigger_time, version);
        }
    }

    fn remove_job(&mut self, job_id: u64) {
        self.job_run_state.remove(&job_id);
    }

    fn trigger_job(&mut self, seconds: u32) {
        if let Some(task_manager) = self.task_manager.clone() {
            let date_time = get_datetime_by_second(seconds, &self.fixed_offset).unwrap();
            let mut trigger_list = Vec::new();
            for item in self.active_time_set.timeout(seconds as u64) {
                if let Some(job) = self.job_run_state.get(&item.job_id) {
                    if job.version != item.version {
                        //ignore
                        log::info!("job version change ignore,id:{}", &item.job_id);
                        continue;
                    }
                    /*
                    log::info!(
                        "prepare job,id:{},run_mode:{:?},handler_name:{}",
                        &job.id,
                        &job.source_job.run_mode,
                        &job.source_job.handle_name
                    );
                    */
                    trigger_list.push(TriggerItem::new(item.trigger_time, job.source_job.clone()));
                    let next_trigger_time = job.calculate_next_trigger_time(&date_time);
                    if next_trigger_time > 0 {
                        self.active_job(item.job_id, next_trigger_time, job.version);
                    } else {
                        log::info!("job next trigger is none,id:{}", &item.job_id);
                    }
                    self.update_job_trigger_time(item.job_id, item.trigger_time, next_trigger_time);
                } else {
                    log::info!(
                        "ScheduleManager|job run state not exist,job id:{}",
                        &item.job_id
                    );
                }
            }
            task_manager.do_send(TaskManagerReq::TriggerTaskList(trigger_list));
        } else {
            log::error!("ScheduleManager|task manager is none");
        }
    }

    fn trigger_redo_job(&mut self, seconds: u32) {
        if let Some(task_manager) = self.task_manager.clone() {
            let mut retry_items = Vec::new();
            let mut tasks = Vec::new();
            for redo_info in self.redo_set.timeout(seconds as u64) {
                let task_id = redo_info.task_id;
                if let Some(old_task) = self.running_task.get(&task_id) {
                    match &redo_info.redo_type {
                        RedoType::Retry => {
                            if !old_task.can_retry() {
                                continue;
                            }
                        }
                        RedoType::Timeout => {
                            if old_task.status != TaskStatusType::Running {
                                continue;
                            }
                        }
                        RedoType::Redo => {
                            if old_task.status != TaskStatusType::Init {
                                continue;
                            }
                        }
                    }
                    let job = self
                        .job_run_state
                        .get(&old_task.job_id)
                        .map(|e| e.source_job.clone());
                    #[cfg(feature = "debug")]
                    log::info!(
                        "ScheduleManager|redo task,id:{},{:?},job is none:{}",
                        task_id,
                        &redo_info.redo_type,
                        job.is_none()
                    );
                    tasks.push((old_task.as_ref().clone(), redo_info.redo_type, job));
                }
            }
            if tasks.is_empty() {
                return;
            }
            log::info!("ScheduleManager|redo task count:{}", tasks.len());
            for (mut task, redo_type, mut job) in tasks {
                let fail_reason = match redo_type {
                    RedoType::Retry | RedoType::Timeout => {
                        if task.can_retry() {
                            task.push_next_try();
                            self.running_task
                                .insert(task.task_id, Arc::new(task.clone()));
                            EMPTY_ARC_STR.clone()
                        } else {
                            job = None;
                            ERR_MSG_TASK_TIMEOUT.clone()
                        }
                    }
                    RedoType::Redo => EMPTY_ARC_STR.clone(),
                };
                let item = RedoTaskItem {
                    trigger_time: seconds,
                    task_info: task,
                    job_info: job,
                    fail_reason,
                };
                retry_items.push(item);
            }
            task_manager.do_send(TaskManagerReq::RedoTaskList(retry_items));
        }
    }

    fn heartbeat(&mut self, ctx: &mut Context<Self>) {
        // 前期只支持主节点发起调度
        if !self.local_is_master {
            self.running_heartbeat = false;
            return;
        }
        let now = now_second_u32();
        self.trigger_job(now);
        self.trigger_redo_job(now);
        let later_millis = 1000 - now_millis() % 1000;
        ctx.run_later(
            std::time::Duration::from_millis(later_millis),
            move |act, ctx| {
                act.heartbeat(ctx);
            },
        );
    }

    fn task_callback(
        &mut self,
        params: Vec<TaskCallBackParam>,
        ctx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let mut list = Vec::with_capacity(params.len());
        let now = now_millis_i64();
        let mut metrics_info = UpdateTaskMetricsInfo::default();
        let mut metrics_request = vec![];
        for param in params {
            if let Some(task_instance) = self.running_task.remove(&param.task_id) {
                let duration_ms = now - (task_instance.trigger_time as i64) * 1000;
                metrics_request.push(MetricsItem::new(
                    MetricsKey::TaskFinishRtHistogram,
                    MetricsRecord::HistogramRecord(duration_ms as f32),
                ));
                let mut task_instance = task_instance.as_ref().clone();
                task_instance.finish_time = (now / 1000) as u32;
                if param.success {
                    task_instance.status = TaskStatusType::Success;
                    metrics_info.success_count += 1;
                } else {
                    task_instance.status = TaskStatusType::Fail;
                    metrics_info.fail_count += 1;
                    if let Some(msg) = param.handle_msg {
                        task_instance.callback_message = Arc::new(msg);
                    }
                }
                list.push(Arc::new(task_instance));
            }
        }
        Self::append_update_metrics_request(&metrics_info, &mut metrics_request);
        if !metrics_request.is_empty() {
            self.do_send_metrics_request(MetricsRequest::BatchRecord(metrics_request));
        }
        let raft_request_route = self.raft_request_route.clone();
        if let Some(raft_request_route) = raft_request_route {
            Self::notify_update_task(raft_request_route, list)
                .into_actor(self)
                .map(|_, _, _| {})
                .spawn(ctx);
        }
        Ok(())
    }

    async fn notify_update_task(
        raft_request_route: Arc<RaftRequestRoute>,
        tasks: Vec<Arc<JobTaskInfo>>,
    ) -> anyhow::Result<()> {
        raft_request_route
            .request(ClientRequest::JobReq {
                req: JobManagerRaftReq::UpdateTaskList(tasks),
            })
            .await?;
        Ok(())
    }

    fn update_task_log(&mut self, task_log: Arc<JobTaskInfo>) {
        let mut metrics_request = vec![];
        let (task_log, metrics_info) = self.update_running_task(task_log);
        self.history_task
            .update_task_log(task_log, self.history_task_log_limit);
        Self::append_update_metrics_request(&metrics_info, &mut metrics_request);
        if !metrics_request.is_empty() {
            self.do_send_metrics_request(MetricsRequest::BatchRecord(metrics_request));
        }
    }

    fn update_task_logs(&mut self, task_logs: Vec<Arc<JobTaskInfo>>) {
        let mut metrics_request = vec![];
        let mut metrics_info = UpdateTaskMetricsInfo::default();
        for item in task_logs {
            let (task_log, tmp_metrics_info) = self.update_running_task(item);
            self.history_task
                .update_task_log(task_log, self.history_task_log_limit);
            metrics_info.add(&tmp_metrics_info);
        }
        Self::append_update_metrics_request(&metrics_info, &mut metrics_request);
        if !metrics_request.is_empty() {
            self.do_send_metrics_request(MetricsRequest::BatchRecord(metrics_request));
        }
    }

    fn update_running_task(
        &mut self,
        task_log: Arc<JobTaskInfo>,
    ) -> (Arc<JobTaskInfo>, UpdateTaskMetricsInfo) {
        if self.last_trigger_time < task_log.trigger_time {
            self.last_trigger_time = task_log.trigger_time;
        }
        let mut metrics_info = UpdateTaskMetricsInfo::default();
        match &task_log.status {
            TaskStatusType::Init => {
                self.running_task.insert(task_log.task_id, task_log.clone());
                self.active_retry_task(task_log.task_id, now_second_u32() + 15, RedoType::Redo);
            }
            TaskStatusType::Running => {
                self.running_task.insert(task_log.task_id, task_log.clone());
                self.active_retry_task(
                    task_log.task_id,
                    now_second_u32() + task_log.get_timeout_second(self.default_timeout_second),
                    RedoType::Timeout,
                );
            }
            TaskStatusType::Success => {
                if let Some(_v) = self.running_task.remove(&task_log.task_id) {
                    if task_log.finish_time >= self.app_start_second {
                        metrics_info.success_count += 1;
                    }
                }
            }
            TaskStatusType::Fail => {
                if task_log.can_retry() {
                    self.running_task.insert(task_log.task_id, task_log.clone());
                    self.active_retry_task(
                        task_log.task_id,
                        now_second_u32() + task_log.get_retry_interval(),
                        RedoType::Retry,
                    );
                } else {
                    if let Some(_v) = self.running_task.remove(&task_log.task_id) {
                        if task_log.finish_time >= self.app_start_second {
                            metrics_info.fail_count += 1;
                        }
                    }
                }
            }
        };
        (task_log, metrics_info)
    }

    fn query_latest_history_task_logs(
        &self,
        query_param: &JobTaskLogQueryParam,
    ) -> (usize, Vec<Arc<JobTaskInfo>>) {
        let mut rlist = Vec::new();
        let end_index = query_param.offset + query_param.limit;
        let mut index = 0;

        for (_task_id, task_log) in self.history_task.task_log_map.iter().rev() {
            if index >= query_param.offset && index < end_index {
                rlist.push(task_log.clone());
            }
            index += 1;
        }
        (index, rlist)
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        //运行实例历史记录
        for (task_id, task_log) in self.history_task.task_log_map.iter() {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = task_log.as_ref().to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: JOB_TASK_HISTORY_TABLE_NAME.clone(),
                key: id_to_bin(*task_id),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        //运行中任务实例
        for (task_id, task_log) in self.running_task.iter() {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = task_log.as_ref().to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: JOB_TASK_RUNNING_TABLE_NAME.clone(),
                key: id_to_bin(*task_id),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        if record.tree.as_str() == JOB_TASK_HISTORY_TABLE_NAME.as_str() {
            let mut reader = BytesReader::from_bytes(&record.value);
            let value_do: JobTaskDo = reader.read_message(&record.value)?;
            let value = Arc::new(value_do.into());
            self.history_task
                .update_task_log(value, self.history_task_log_limit);
        } else if record.tree.as_str() == JOB_TASK_RUNNING_TABLE_NAME.as_str() {
            let mut reader = BytesReader::from_bytes(&record.value);
            let value_do: JobTaskDo = reader.read_message(&record.value)?;
            let value: Arc<JobTaskInfo> = Arc::new(value_do.into());
            self.running_task.insert(value.task_id, value);
        }
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        Ok(())
    }

    fn update_vote(&mut self, vote_info: VoteInfo, local_is_master: bool, ctx: &mut Context<Self>) {
        if self.last_vote_info.term < vote_info.term {
            let last_local_is_master = self.local_is_master;
            self.last_vote_info = vote_info;
            self.local_is_master = local_is_master;
            if !last_local_is_master && local_is_master {
                self.init_run_job();
                if !self.running_heartbeat {
                    self.running_heartbeat = true;
                    self.heartbeat(ctx);
                }
            }
            if !local_is_master {
                // 从节点清理任务
                self.active_time_set.clear();
                self.redo_set.clear();
            }
        }
    }

    fn init_run_job(&mut self) {
        // 初始化任务调度
        let now = now_second_u32();
        let start_second = std::cmp::min(
            std::cmp::max(self.last_trigger_time, self.app_start_second),
            now - 1,
        );
        let mut active_jobs: Vec<(u64, u32, u32)> = Vec::new();
        if let Some(now_datetime) = get_datetime_by_second(start_second, &self.fixed_offset) {
            for (_, job_run_state) in &mut self.job_run_state {
                let next_trigger_time = job_run_state.calculate_first_trigger_time(&now_datetime);
                if next_trigger_time > 0 {
                    job_run_state.next_trigger_time = next_trigger_time;
                    active_jobs.push((job_run_state.id, next_trigger_time, job_run_state.version));
                }
            }
        }
        let mut retry_list = Vec::new();
        for task in self.running_task.values() {
            match task.status {
                TaskStatusType::Init => {
                    // 十分钟之内支持重试
                    if task.trigger_time + 600 > now {
                        retry_list.push((task.task_id, task.trigger_time + 15, RedoType::Redo));
                    }
                }
                TaskStatusType::Running => {
                    let timeout = if task.timeout_second > 0 {
                        task.timeout_second
                    } else {
                        self.default_timeout_second
                    };
                    retry_list.push((task.task_id, task.trigger_time + timeout, RedoType::Timeout));
                }
                TaskStatusType::Fail => {
                    if task.can_retry() {
                        retry_list.push((
                            task.task_id,
                            now + task.get_retry_interval(),
                            RedoType::Retry,
                        ));
                    }
                }
                _ => {}
            }
        }
        for (job_id, next_trigger_time, version) in active_jobs {
            self.active_job(job_id, next_trigger_time, version);
        }
        for (task_id, time, redo_type) in retry_list {
            self.active_retry_task(task_id, time, redo_type);
        }
    }

    fn append_update_metrics_request(
        metrics_info: &UpdateTaskMetricsInfo,
        metrics_request: &mut Vec<MetricsItem>,
    ) {
        if metrics_info.success_count > 0 {
            metrics_request.push(MetricsItem::new(
                MetricsKey::TaskSuccessSize,
                MetricsRecord::CounterInc(metrics_info.success_count),
            ));
            metrics_request.push(MetricsItem::new(
                MetricsKey::TaskFinishTotalCount,
                MetricsRecord::CounterInc(metrics_info.success_count),
            ));
        }
        if metrics_info.fail_count > 0 {
            metrics_request.push(MetricsItem::new(
                MetricsKey::TaskFailSize,
                MetricsRecord::CounterInc(metrics_info.fail_count),
            ));
            metrics_request.push(MetricsItem::new(
                MetricsKey::TaskFinishTotalCount,
                MetricsRecord::CounterInc(metrics_info.fail_count),
            ));
        }
    }

    fn do_send_metrics_request(&self, req: MetricsRequest) {
        if let Some(addr) = self.metrics_manager.as_ref() {
            addr.do_send(req);
        }
    }
}

impl Handler<ScheduleManagerReq> for ScheduleManager {
    type Result = anyhow::Result<ScheduleManagerResult>;

    fn handle(&mut self, msg: ScheduleManagerReq, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            ScheduleManagerReq::UpdateJob(job) => {
                //log::info!("ScheduleManagerReq::UpdateJob,job_id:{}", &job.id);
                self.update_job(job);
            }
            ScheduleManagerReq::RemoveJob(job_id) => {
                self.remove_job(job_id);
            }
            ScheduleManagerReq::UpdateTask(task) => {
                self.update_task_log(task);
            }
            ScheduleManagerReq::UpdateTaskList(task_list) => {
                self.update_task_logs(task_list);
            }
            ScheduleManagerReq::QueryJobTaskLog(param) => {
                let (total, list) = self.query_latest_history_task_logs(&param);
                return Ok(ScheduleManagerResult::JobTaskLogPageInfo(total, list));
            }
        }
        Ok(ScheduleManagerResult::None)
    }
}

impl Handler<ScheduleManagerRaftReq> for ScheduleManager {
    type Result = anyhow::Result<ScheduleManagerRaftResult>;

    fn handle(&mut self, msg: ScheduleManagerRaftReq, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            ScheduleManagerRaftReq::TaskCallBacks(params) => {
                self.task_callback(params, ctx)?;
            }
        }
        Ok(ScheduleManagerRaftResult::None)
    }
}

impl Handler<RaftApplyDataRequest> for ScheduleManager {
    type Result = anyhow::Result<RaftApplyDataResponse>;

    fn handle(&mut self, msg: RaftApplyDataRequest, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RaftApplyDataRequest::BuildSnapshot(writer) => {
                self.build_snapshot(writer)?;
            }
            RaftApplyDataRequest::LoadSnapshotRecord(record) => {
                self.load_snapshot_record(record)?;
            }
            RaftApplyDataRequest::LoadCompleted => {
                self.load_completed(ctx)?;
            }
        }
        Ok(RaftApplyDataResponse::None)
    }
}

impl Handler<VoteChangeRequest> for ScheduleManager {
    type Result = anyhow::Result<VoteChangeResponse>;

    fn handle(&mut self, msg: VoteChangeRequest, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            VoteChangeRequest::VoteChange {
                vote_info,
                local_is_master,
            } => {
                self.update_vote(vote_info, local_is_master, ctx);
            }
        }
        Ok(VoteChangeResponse::None)
    }
}
