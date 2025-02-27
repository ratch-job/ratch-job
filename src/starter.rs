use crate::app::core::AppManager;
use crate::common::actor_utils::{create_actor_at_thread, create_actor_at_thread2};
use crate::common::app_config::AppConfig;
use crate::common::share_data::ShareData;
use crate::grpc::handler::RAFT_ROUTE_REQUEST;
use crate::grpc::payload_utils::PayloadUtils;
use crate::job::core::JobManager;
use crate::raft::cluster::model::RouterRequest;
use crate::raft::cluster::route::{RaftAddrRouter, RaftRequestRoute};
use crate::raft::network::core::RaftRouter;
use crate::raft::network::factory::{RaftClusterRequestSender, RaftConnectionFactory};
use crate::raft::store::core::Store;
use crate::raft::store::raftapply::StateApplyManager;
use crate::raft::store::raftdata::RaftDataWrap;
use crate::raft::store::raftindex::RaftIndexManager;
use crate::raft::store::raftlog::RaftLogManager;
use crate::raft::store::raftsnapshot::RaftSnapshotManager;
use crate::raft::store::ClientRequest;
use crate::raft::RatchRaft;
use crate::schedule::core::ScheduleManager;
use crate::sequence::core::SequenceDbManager;
use crate::sequence::SequenceManager;
use crate::task::core::TaskManager;
use crate::task::task_history::TaskHistoryManager;
use actix::Actor;
use async_raft_ext::raft::ClientWriteRequest;
use async_raft_ext::{Config, Raft, RaftStorage};
use bean_factory::{BeanDefinition, BeanFactory, FactoryData};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

pub async fn config_factory(app_config: Arc<AppConfig>) -> anyhow::Result<FactoryData> {
    std::fs::create_dir_all(app_config.local_db_dir.as_str())?;
    let base_path = Arc::new(app_config.local_db_dir.clone());
    //let base_path = Arc::new(app_config.local_db_dir.clone());
    let factory = BeanFactory::new();
    factory.register(BeanDefinition::from_obj(app_config.clone()));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        AppManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        JobManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        SequenceManager::new().start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        ScheduleManager::new(app_config.gmt_fixed_offset_hours.map(|v| v * 60 * 60)).start(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        TaskManager::new(app_config.clone()).start(),
    ));
    factory.register(BeanDefinition::actor_from_obj(
        TaskHistoryManager::new().start(),
    ));
    let sequence_db_addr = SequenceDbManager::new().start();
    factory.register(BeanDefinition::actor_from_obj(sequence_db_addr.clone()));

    // raft begin
    let index_manager = RaftIndexManager::new(base_path.clone()).start();
    let log_manager = RaftLogManager::new(base_path.clone(), Some(index_manager.clone()));
    let log_manager = create_actor_at_thread(log_manager);
    let snapshot_manager = RaftSnapshotManager::new(base_path.clone(), Some(index_manager.clone()));
    let apply_manager = StateApplyManager::new();
    let (snapshot_manager, apply_manager) =
        create_actor_at_thread2(snapshot_manager, apply_manager);
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        log_manager.clone(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        index_manager.clone(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        snapshot_manager.clone(),
    ));
    factory.register(BeanDefinition::actor_with_inject_from_obj(
        apply_manager.clone(),
    ));

    let store = Arc::new(Store::new(
        app_config.raft_node_id.to_owned(),
        index_manager,
        snapshot_manager,
        log_manager,
        apply_manager,
    ));
    factory.register(BeanDefinition::from_obj(store.clone()));
    let conn_factory = RaftConnectionFactory::new(60).start();
    let cluster_sender = Arc::new(RaftClusterRequestSender::new(
        conn_factory,
        app_config.clone(),
    ));
    let raft_data_wrap = Arc::new(RaftDataWrap {
        sequence_db: sequence_db_addr,
    });
    factory.register(BeanDefinition::from_obj(raft_data_wrap.clone()));
    let raft = build_raft(&app_config, store.clone(), cluster_sender.clone()).await?;
    factory.register(BeanDefinition::from_obj(raft.clone()));
    let raft_addr_router = Arc::new(RaftAddrRouter::new(
        raft.clone(),
        store.clone(),
        app_config.raft_node_id.to_owned(),
    ));
    let raft_request_route = Arc::new(RaftRequestRoute::new(
        raft_addr_router.clone(),
        cluster_sender.clone(),
        raft.clone(),
    ));
    factory.register(BeanDefinition::from_obj(raft_request_route));
    // raft end
    Ok(factory.init().await)
}

async fn build_raft(
    sys_config: &Arc<AppConfig>,
    store: Arc<Store>,
    cluster_sender: Arc<RaftClusterRequestSender>,
) -> anyhow::Result<Arc<RatchRaft>> {
    match store.get_last_log_index().await {
        Ok(last_log) => log::info!(
            "[PEEK_RAFT_LOG] raft last log,index:{} term:{}",
            last_log.index,
            last_log.term
        ),
        Err(e) => log::warn!("[PEEK_RAFT_LOG] raft last log is empty,error:{}", e),
    };
    let config = Config::build("ratch raft".to_owned())
        .heartbeat_interval(1000)
        .election_timeout_min(2500)
        .election_timeout_max(5000)
        .snapshot_policy(async_raft_ext::SnapshotPolicy::LogsSinceLast(
            sys_config.raft_snapshot_log_size,
        ))
        .snapshot_max_chunk_size(3 * 1024 * 1024)
        .validate()
        .unwrap();
    let config = Arc::new(config);
    let network = Arc::new(RaftRouter::new(store.clone(), cluster_sender.clone()));
    let raft = Arc::new(Raft::new(
        sys_config.raft_node_id.to_owned(),
        config,
        network,
        store.clone(),
    ));
    if sys_config.raft_auto_init {
        tokio::spawn(auto_init_raft(store, raft.clone(), sys_config.clone()));
    } else if !sys_config.raft_join_addr.is_empty() {
        tokio::spawn(auto_join_raft(store, sys_config.clone(), cluster_sender));
    }
    Ok(raft)
}

pub fn build_share_data(factory_data: FactoryData) -> anyhow::Result<Arc<ShareData>> {
    let app_config: Arc<AppConfig> = factory_data.get_bean().unwrap();
    let app_data = Arc::new(ShareData {
        app_config,
        app_manager: factory_data.get_actor().unwrap(),
        job_manager: factory_data.get_actor().unwrap(),
        sequence_manager: factory_data.get_actor().unwrap(),
        schedule_manager: factory_data.get_actor().unwrap(),
        task_manager: factory_data.get_actor().unwrap(),
        task_history_manager: factory_data.get_actor().unwrap(),
        raft: factory_data.get_bean().unwrap(),
        raft_store: factory_data.get_bean().unwrap(),
        raft_request_route: factory_data.get_bean().unwrap(),
        factory_data,
    });
    Ok(app_data)
}

async fn auto_init_raft(
    store: Arc<Store>,
    raft: Arc<RatchRaft>,
    sys_config: Arc<AppConfig>,
) -> anyhow::Result<()> {
    let state = store.get_initial_state().await?;
    if state.last_log_term == 0 {
        log::info!(
            "auto init raft. node_id:{},addr:{}",
            &sys_config.raft_node_id,
            &sys_config.raft_node_addr
        );
        let mut members = HashSet::new();
        members.insert(sys_config.raft_node_id.to_owned());
        raft.initialize(members).await.ok();
        raft.client_write(ClientWriteRequest::new(ClientRequest::NodeAddr {
            id: sys_config.raft_node_id,
            addr: Arc::new(sys_config.raft_node_addr.to_owned()),
        }))
        .await
        .ok();
        raft.client_write(ClientWriteRequest::new(ClientRequest::Members(vec![
            sys_config.raft_node_id,
        ])))
        .await
        .ok();
    } else if state.membership.all_nodes().len() < 2 {
        // 单节点支持更新集群ip地址
        tokio::time::sleep(Duration::from_millis(5000)).await;
        if let Some(node_id) = raft.current_leader().await {
            if node_id == sys_config.raft_node_id {
                if let Ok(addr) = store.get_target_addr(node_id).await {
                    if addr.as_str() == sys_config.raft_node_addr.as_str() {
                        // 如果当前节点与集群ip相同则不用更新
                        return Ok(());
                    }
                }
                raft.client_write(ClientWriteRequest::new(ClientRequest::NodeAddr {
                    id: sys_config.raft_node_id,
                    addr: Arc::new(sys_config.raft_node_addr.to_owned()),
                }))
                .await
                .ok();
            }
        }
    }
    Ok(())
}

async fn auto_join_raft(
    store: Arc<Store>,
    sys_config: Arc<AppConfig>,
    cluster_sender: Arc<RaftClusterRequestSender>,
) -> anyhow::Result<()> {
    let state = store.get_initial_state().await?;
    if state.last_log_term == 0 {
        //wait for self raft network started
        tokio::time::sleep(Duration::from_millis(500)).await;
        let req = RouterRequest::JoinNode {
            node_id: sys_config.raft_node_id.to_owned(),
            node_addr: Arc::new(sys_config.raft_node_addr.to_owned()),
        };
        let request = serde_json::to_vec(&req).unwrap_or_default();
        let payload = PayloadUtils::build_payload(RAFT_ROUTE_REQUEST, request);
        cluster_sender
            .send_request(Arc::new(sys_config.raft_join_addr.to_owned()), payload)
            .await?;
        log::info!(
            "auto join raft,join_addr:{}.node_id:{},addr:{}",
            &sys_config.raft_join_addr,
            &sys_config.raft_node_id,
            &sys_config.raft_node_addr
        );
    }
    Ok(())
}
