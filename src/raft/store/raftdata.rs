use crate::app::core::AppManager;
use crate::cache::core::CacheManager;
use crate::common::constant::{
    APP_INFO_TABLE_NAME, CACHE_TABLE_NAME, JOB_TABLE_NAME, JOB_TASK_HISTORY_TABLE_NAME,
    JOB_TASK_RUNNING_TABLE_NAME, JOB_TASK_TABLE_NAME, SEQUENCE_TABLE_NAME, USER_TABLE_NAME,
};
use crate::job::core::JobManager;
use crate::raft::store::model::SnapshotRecordDto;
use crate::raft::store::raftapply::RaftApplyDataRequest;
use crate::raft::store::raftindex::{RaftIndexManager, RaftIndexRequest};
use crate::raft::store::raftsnapshot::SnapshotWriterActor;
use crate::raft::store::{ClientRequest, ClientResponse};
use crate::schedule::core::ScheduleManager;
use crate::sequence::core::SequenceDbManager;
use crate::user::core::UserManager;
use actix::prelude::*;

#[derive(Clone)]
pub struct RaftDataHandler {
    pub sequence_db: Addr<SequenceDbManager>,
    pub app_manager: Addr<AppManager>,
    pub job_manager: Addr<JobManager>,
    pub schedule_manager: Addr<ScheduleManager>,
    pub cache_manager: Addr<CacheManager>,
    pub user_manager: Addr<UserManager>,
}

impl RaftDataHandler {
    /// 构建raft快照
    pub async fn build_snapshot(&self, writer: Addr<SnapshotWriterActor>) -> anyhow::Result<()> {
        log::info!("RaftDataHandler|build_snapshot");
        self.sequence_db
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        self.app_manager
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        self.job_manager
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        self.schedule_manager
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        self.cache_manager
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        self.user_manager
            .send(RaftApplyDataRequest::BuildSnapshot(writer.clone()))
            .await??;
        Ok(())
    }

    /// 加载raft快照
    pub async fn load_snapshot(&self, record: SnapshotRecordDto) -> anyhow::Result<()> {
        match record.tree.as_str() {
            ref tree if *tree == SEQUENCE_TABLE_NAME.as_str() => {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.sequence_db.send(req).await??;
            }
            ref tree if *tree == APP_INFO_TABLE_NAME.as_str() => {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.app_manager.send(req).await??;
            }
            ref tree
                if *tree == JOB_TABLE_NAME.as_str() || *tree == JOB_TASK_TABLE_NAME.as_str() =>
            {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.job_manager.send(req).await??;
            }
            ref tree
                if *tree == JOB_TASK_RUNNING_TABLE_NAME.as_str()
                    || *tree == JOB_TASK_HISTORY_TABLE_NAME.as_str() =>
            {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.schedule_manager.send(req).await??;
            }
            ref tree if *tree == CACHE_TABLE_NAME.as_str() => {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.cache_manager.send(req).await??;
            }
            ref tree if *tree == USER_TABLE_NAME.as_str() => {
                let req = RaftApplyDataRequest::LoadSnapshotRecord(record);
                self.user_manager.send(req).await??;
            }
            _ => {
                log::warn!("RaftDataHandler|load_snapshot|ignore_data|tree={}", &record.tree);
            }
        }
        Ok(())
    }

    pub fn load_complete(&self) -> anyhow::Result<()> {
        log::info!("RaftDataHandler|load_complete");
        self.sequence_db
            .do_send(RaftApplyDataRequest::LoadCompleted);
        self.app_manager
            .do_send(RaftApplyDataRequest::LoadCompleted);
        self.job_manager
            .do_send(RaftApplyDataRequest::LoadCompleted);
        self.schedule_manager
            .do_send(RaftApplyDataRequest::LoadCompleted);
        self.cache_manager
            .do_send(RaftApplyDataRequest::LoadCompleted);
        self.user_manager
            .do_send(RaftApplyDataRequest::LoadCompleted);
        Ok(())
    }

    /// 启动时加载日志
    pub async fn load_log(
        &self,
        req: ClientRequest,
        index_manager: &Addr<RaftIndexManager>,
    ) -> anyhow::Result<()> {
        match req {
            ClientRequest::NodeAddr { id, addr } => {
                index_manager
                    .send(RaftIndexRequest::AddNodeAddr(id, addr))
                    .await
                    .ok();
            }
            ClientRequest::Members(member) => {
                index_manager
                    .send(RaftIndexRequest::SaveMember {
                        member: member.clone(),
                        member_after_consensus: None,
                        node_addr: None,
                    })
                    .await
                    .ok();
            }
            ClientRequest::SequenceReq { req } => {
                self.sequence_db.send(req).await.ok();
            }
            ClientRequest::AppReq { req } => {
                self.app_manager.send(req).await.ok();
            }
            ClientRequest::JobReq { req } => {
                self.job_manager.send(req).await.ok();
            }
            ClientRequest::ScheduleReq { req } => {
                self.schedule_manager.send(req).await.ok();
            }
            ClientRequest::CacheReq { req } => {
                self.cache_manager.send(req).await.ok();
            }
            ClientRequest::UserReq { req } => {
                self.user_manager.send(req).await.ok();
            }
        }
        Ok(())
    }

    /// 接收raft请求到状态机，需要返回结果到调用端
    pub async fn apply_log_to_state_machine(
        &self,
        req: ClientRequest,
        index_manager: &Addr<RaftIndexManager>,
    ) -> anyhow::Result<ClientResponse> {
        match req {
            ClientRequest::NodeAddr { id, addr } => {
                index_manager.do_send(RaftIndexRequest::AddNodeAddr(id, addr));
                Ok(ClientResponse::Success)
            }
            ClientRequest::Members(member) => {
                index_manager.do_send(RaftIndexRequest::SaveMember {
                    member: member.clone(),
                    member_after_consensus: None,
                    node_addr: None,
                });
                Ok(ClientResponse::Success)
            }
            ClientRequest::SequenceReq { req } => {
                let r = self.sequence_db.send(req).await??;
                Ok(ClientResponse::SequenceResp { resp: r })
            }
            ClientRequest::AppReq { req } => {
                let r = self.app_manager.send(req).await??;
                Ok(ClientResponse::AppResp { resp: r })
            }
            ClientRequest::JobReq { req } => {
                let r = self.job_manager.send(req).await??;
                Ok(ClientResponse::JobResp { resp: r })
            }
            ClientRequest::ScheduleReq { req } => {
                let r = self.schedule_manager.send(req).await??;
                //Ok(ClientResponse::ScheduleReq { resp: r })
                Ok(ClientResponse::ScheduleResp { resp: r })
            }
            ClientRequest::CacheReq { req } => {
                let r = self.cache_manager.send(req).await??;
                Ok(ClientResponse::CacheResp { resp: r })
            }
            ClientRequest::UserReq { req } => {
                let r = self.user_manager.send(req).await??;
                Ok(ClientResponse::UserResp { resp: r })
            }
        }
    }

    ///批量处理请求，一般是由主节点到从节点，不需要返回值
    pub fn do_send_log(
        &self,
        req: ClientRequest,
        index_manager: &Addr<RaftIndexManager>,
    ) -> anyhow::Result<()> {
        match req {
            ClientRequest::NodeAddr { id, addr } => {
                index_manager.do_send(RaftIndexRequest::AddNodeAddr(id, addr));
            }
            ClientRequest::Members(member) => {
                index_manager.do_send(RaftIndexRequest::SaveMember {
                    member: member.clone(),
                    member_after_consensus: None,
                    node_addr: None,
                });
            }
            ClientRequest::SequenceReq { req } => {
                self.sequence_db.do_send(req);
            }
            ClientRequest::AppReq { req } => {
                self.app_manager.do_send(req);
            }
            ClientRequest::JobReq { req } => {
                self.job_manager.do_send(req);
            }
            ClientRequest::ScheduleReq { req } => {
                self.schedule_manager.do_send(req);
            }
            ClientRequest::CacheReq { req } => {
                self.cache_manager.do_send(req);
            }
            ClientRequest::UserReq { req } => {
                self.user_manager.do_send(req);
            }
        }
        Ok(())
    }
}
