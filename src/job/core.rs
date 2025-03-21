use crate::common::byte_utils::id_to_bin;
use crate::common::constant::{JOB_TABLE_NAME, JOB_TASK_TABLE_NAME, SEQUENCE_TABLE_NAME};
use crate::common::datetime_utils::now_millis;
use crate::common::pb::data_object::{JobDo, JobTaskDo};
use crate::job::job_index::JobQueryParam;
use crate::job::model::actor_model::{
    JobManagerRaftReq, JobManagerRaftResult, JobManagerReq, JobManagerResult,
};
use crate::job::model::job::{JobInfo, JobInfoDto, JobParam, JobTaskLogQueryParam, JobWrap};
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::{RaftApplyDataRequest, RaftApplyDataResponse};
use crate::raft::store::raftsnapshot::{SnapshotWriterActor, SnapshotWriterRequest};
use crate::schedule::core::ScheduleManager;
use crate::schedule::model::actor_model::ScheduleManagerReq;
use crate::task::model::actor_model::TaskHistoryManagerReq;
use crate::task::model::task::JobTaskInfo;
use crate::task::task_history::TaskHistoryManager;
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use quick_protobuf::{BytesReader, Writer};
use std::collections::HashMap;
use std::sync::Arc;

#[bean(inject)]
pub struct JobManager {
    job_map: HashMap<u64, JobWrap>,
    schedule_manager: Option<Addr<ScheduleManager>>,
    job_task_log_limit: usize,
}

impl JobManager {
    pub fn new() -> Self {
        JobManager {
            job_map: HashMap::new(),
            schedule_manager: None,
            job_task_log_limit: 200,
        }
    }

    fn create_job(&mut self, job_param: JobParam) -> anyhow::Result<Arc<JobInfo>> {
        let id = job_param.id.unwrap_or_default();
        if id == 0 {
            return Err(anyhow::anyhow!("CreateJob JobParam.id==0 is invalid!"));
        }
        if self.job_map.contains_key(&id) {
            return Err(anyhow::anyhow!(
                "CreateJob，The job already exists and is repeatedly created"
            ));
        }
        let mut job_info: JobInfo = job_param.into();
        job_info.check_valid()?;
        let now = now_millis();
        job_info.last_modified_millis = now;
        job_info.create_time = now;
        let value = Arc::new(job_info);
        self.job_map.insert(value.id, JobWrap::new(value.clone()));
        if let Some(schedule_manager) = self.schedule_manager.as_ref() {
            schedule_manager.do_send(ScheduleManagerReq::UpdateJob(value.clone()));
        }
        Ok(value)
    }

    fn update_job(&mut self, job_param: JobParam) -> anyhow::Result<()> {
        let id = job_param.id.unwrap_or_default();
        if id == 0 {
            return Err(anyhow::anyhow!("UpdateJob JobParam.id==0 is invalid!"));
        }
        if let Some(job_wrap) = self.job_map.get_mut(&id) {
            let job_info = &job_wrap.job;
            let mut new_job = job_info.as_ref().clone();
            new_job.update_param(job_param);
            job_info.check_valid()?;
            let value = Arc::new(new_job);
            job_wrap.job = value.clone();
            if let Some(schedule_manager) = self.schedule_manager.as_ref() {
                schedule_manager.do_send(ScheduleManagerReq::UpdateJob(value.clone()));
            }
        } else {
            return Err(anyhow::anyhow!("UpdateJob,Nonexistent Job"));
        }
        Ok(())
    }

    fn remove_job(&mut self, id: u64) {
        self.job_map.remove(&id);
        if let Some(schedule_manager) = self.schedule_manager.as_ref() {
            schedule_manager.do_send(ScheduleManagerReq::RemoveJob(id));
        }
    }

    fn do_update_job(&mut self, job: Arc<JobInfo>) {
        if let Some(job_wrap) = self.job_map.get_mut(&job.id) {
            job_wrap.job = job;
        } else {
            self.job_map.insert(job.id, JobWrap::new(job.clone()));
            if let Some(schedule_manager) = self.schedule_manager.as_ref() {
                schedule_manager.do_send(ScheduleManagerReq::UpdateJob(job));
            }
        }
    }

    fn update_job_task(&mut self, task_log: Arc<JobTaskInfo>) {
        if let Some(schedule_manager) = self.schedule_manager.as_ref() {
            schedule_manager.do_send(ScheduleManagerReq::UpdateTask(task_log.clone()));
        }
        if let Some(job_wrap) = self.job_map.get_mut(&task_log.job_id) {
            job_wrap.update_task_log(task_log, self.job_task_log_limit);
        }
    }

    fn query_jobs(&self, query_param: &JobQueryParam) -> (usize, Vec<JobInfoDto>) {
        let mut rlist = Vec::new();
        let end_index = query_param.offset + query_param.limit;
        let mut index = 0;

        for job_wrap in self.job_map.values() {
            let job_info = &job_wrap.job;
            if query_param.match_namespace(&job_info.namespace)
                && query_param.match_app_name(&job_info.app_name)
                && query_param.match_description(&job_info.description)
                && query_param.match_handle_name(&job_info.handle_name)
            {
                if index >= query_param.offset && index < end_index {
                    rlist.push(JobInfoDto::new_from(job_info));
                }
                index += 1;
            }
        }

        (index, rlist)
    }

    fn query_job_task_logs(
        &self,
        query_param: &JobTaskLogQueryParam,
    ) -> (usize, Vec<Arc<JobTaskInfo>>) {
        //log::info!("query_job_task_logs,query_param={:?}", query_param);
        let mut rlist = Vec::new();
        let end_index = query_param.offset + query_param.limit;
        let mut index = 0;

        if let Some(job_wrap) = self.job_map.get(&query_param.job_id) {
            for (_task_id, task_log) in job_wrap.task_log_map.iter().rev() {
                if index >= query_param.offset && index < end_index {
                    rlist.push(task_log.clone());
                }
                index += 1;
            }
        }
        (index, rlist)
    }

    fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        //任务
        for (key, job_wrap) in &self.job_map {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf);
                let value_do = job_wrap.job.as_ref().to_do();
                writer.write_message(&value_do)?;
            }
            let record = SnapshotRecordDto {
                tree: JOB_TABLE_NAME.clone(),
                key: id_to_bin(*key),
                value: buf,
                op_type: 0,
            };
            writer.do_send(SnapshotWriterRequest::Record(record));
        }
        //任务运行实例
        for (_, job_wrap) in &self.job_map {
            for (task_id, task_log) in job_wrap.task_log_map.iter() {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf);
                    let value_do = task_log.as_ref().to_do();
                    writer.write_message(&value_do)?;
                }
                let record = SnapshotRecordDto {
                    tree: JOB_TASK_TABLE_NAME.clone(),
                    key: id_to_bin(*task_id),
                    value: buf,
                    op_type: 0,
                };
                writer.do_send(SnapshotWriterRequest::Record(record));
            }
        }
        Ok(())
    }

    fn load_snapshot_record(&mut self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        if record.tree.as_str() == JOB_TABLE_NAME.as_str() {
            let mut reader = BytesReader::from_bytes(&record.value);
            let value_do: JobDo = reader.read_message(&record.value)?;
            let value = Arc::new(value_do.into());
            self.do_update_job(value);
        } else if record.tree.as_str() == JOB_TASK_TABLE_NAME.as_str() {
            let mut reader = BytesReader::from_bytes(&record.value);
            let value_do: JobTaskDo = reader.read_message(&record.value)?;
            let value: Arc<JobTaskInfo> = Arc::new(value_do.into());
            if let Some(job_wrap) = self.job_map.get_mut(&value.job_id) {
                job_wrap.update_task_log(value, self.job_task_log_limit);
            }
        }
        Ok(())
    }

    fn load_completed(&mut self, _ctx: &mut Context<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Actor for JobManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("JobManager started");
    }
}

impl Inject for JobManager {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.schedule_manager = factory_data.get_actor();
    }
}

impl Handler<JobManagerReq> for JobManager {
    type Result = anyhow::Result<JobManagerResult>;

    fn handle(&mut self, msg: JobManagerReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            JobManagerReq::GetJob(id) => {
                let job_info = if let Some(job_wrap) = self.job_map.get(&id) {
                    Some(job_wrap.job.clone())
                } else {
                    None
                };
                return Ok(JobManagerResult::JobInfo(job_info));
            }
            JobManagerReq::UpdateTask(task_info) => {
                self.update_job_task(task_info);
            }
            JobManagerReq::QueryJob(query_param) => {
                let (size, list) = self.query_jobs(&query_param);
                return Ok(JobManagerResult::JobPageInfo(size, list));
            }
            JobManagerReq::QueryJobTaskLog(query_param) => {
                let (size, list) = self.query_job_task_logs(&query_param);
                return Ok(JobManagerResult::JobTaskLogPageInfo(size, list));
            }
        }
        Ok(JobManagerResult::None)
    }
}

impl Handler<JobManagerRaftReq> for JobManager {
    type Result = anyhow::Result<JobManagerRaftResult>;
    fn handle(&mut self, msg: JobManagerRaftReq, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            JobManagerRaftReq::AddJob(job_param) => {
                let value = self.create_job(job_param)?;
                return Ok(JobManagerRaftResult::JobInfo(value));
            }
            JobManagerRaftReq::UpdateJob(job_param) => {
                self.update_job(job_param)?;
            }
            JobManagerRaftReq::Remove(id) => {
                self.remove_job(id);
            }
            JobManagerRaftReq::UpdateTask(task_info) => {
                self.update_job_task(task_info);
            }
            JobManagerRaftReq::UpdateTaskList(tasks) => {
                for tasks in tasks {
                    self.update_job_task(tasks);
                }
            }
        }
        Ok(JobManagerRaftResult::None)
    }
}

impl Handler<RaftApplyDataRequest> for JobManager {
    type Result = anyhow::Result<RaftApplyDataResponse>;

    fn handle(&mut self, msg: RaftApplyDataRequest, ctx: &mut Self::Context) -> Self::Result {
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
