# 任务列表

- [x] 1. 定义 Protobuf 持久化对象
  - 在 `src/common/pb/data_object.proto` 中添加 `NamespaceDo` message 定义（包含 id、name、type 字段）
  - 切换到 src/common/pb目录执行`pb-rs data_object.proto`生成代码，更新对应的 Rust 生成文件
  - _需求: 1.2_

- [x] 2. 定义命名空间数据模型
  - 在 `src/namespace/model/namespace.rs` 中定义 `Namespace` 内存对象（id, name, type）
  - 定义 `NamespaceParam` 参数对象（id 为 Option, name 和 type 必填）
  - 定义 `NamespaceInfo` API 响应对象（id 为 Arc\<String\>）
  - 定义 `NamespaceQueryParam` 查询参数对象（page, page_size, type 过滤）
  - 定义 `NamespaceWrap` 包装结构（含 Arc\<Namespace\> 和索引字段）
  - _需求: 1.1, 1.3_

- [x] 3. 定义 Actor 消息类型
  - 在 `src/namespace/model/actor_model.rs` 中定义 `NamespaceManagerRaftReq` 枚举（AddNamespace, UpdateNamespace, Remove）
  - 定义 `NamespaceManagerRaftResult` 枚举（NamespaceInfo, None）
  - 定义 `NamespaceManagerReq` 枚举（GetNamespace, QueryNamespace）
  - 定义 `NamespaceManagerResult` 枚举（NamespaceInfo, NamespacePageInfo, None）
  - _需求: 3.1, 6.1_

- [x] 4. 实现 Protobuf 序列化与反序列化
  - 在 `Namespace` 对象中实现 `to_do()` 方法，转换为 `NamespaceDo` protobuf 对象
  - 实现 `From<NamespaceDo<'_>>` trait，支持从 protobuf 转换为内存对象
  - 使用 `Cow::Borrowed` 避免不必要的字符串复制
  - _需求: 7.1, 7.2_

- [x] 5. 实现 NamespaceManager Actor 基础框架
  - 在 `src/namespace/core.rs` 中定义 `NamespaceManager` 结构体
  - 使用 `BTreeMap<String, NamespaceWrap>` 作为内存存储（以 id 为 key）
  - 实现 `Actor` trait，提供 Actor 生命周期管理
  - 实现 `Inject` trait，支持依赖注入
  - 在 Actor 启动时记录日志
  - 创建 `src/namespace/mod.rs` 模块入口文件
  - _需求: 2.1, 2.2_

- [x] 6. 实现命名空间 CRUD 业务逻辑
  - 实现创建逻辑：检查 id 重复（重复则当更新处理）、验证 name/type 非空、id 为空时自动生成 UUID、插入内存存储
  - 实现更新逻辑：根据 ID 查找命名空间、验证存在性、更新 name 和 type 字段
  - 实现删除逻辑：根据 ID 从 BTreeMap 中删除命名空间记录
  - _需求: 2.3, 2.4, 2.5_

- [x] 7. 实现 Raft 消息处理 Handler
  - 实现 `Handler<NamespaceManagerRaftReq>` trait
  - 处理 AddNamespace 请求：调用创建逻辑，返回 NamespaceInfo
  - 处理 UpdateNamespace 请求：调用更新逻辑，返回 NamespaceInfo
  - 处理 Remove 请求：调用删除逻辑，返回 None
  - 处理过程中只修改内存状态，不触发额外 IO
  - _需求: 3.2_

- [x] 8. 实现普通业务消息处理 Handler
  - 实现 `Handler<NamespaceManagerReq>` trait
  - 处理 GetNamespace：根据 id 从 BTreeMap 查询并返回
  - 处理 QueryNamespace：根据 type 过滤、分页计算、返回列表和总数
  - 直接从内存读取，不触发 Raft 复制
  - _需求: 6.2, 6.3_

- [x] 9. 实现快照构建与加载
  - 实现 `build_snapshot()` 方法：遍历 namespace_map，调用 to_do() 序列化，创建 SnapshotRecordDto（tree=T_NAMESPACE），发送到 SnapshotWriterActor
  - 实现 `load_snapshot_record()` 方法：识别 tree 为 T_NAMESPACE 的记录，使用 BytesReader 反序列化为 NamespaceDo，转换并重建内存状态
  - 实现 `load_completed()` 方法：执行加载后初始化逻辑
  - 实现 `Handler<RaftApplyDataRequest>` trait：路由 BuildSnapshot、LoadSnapshotRecord、LoadCompleted 请求
  - _需求: 3.3, 3.4, 3.5, 3.6, 7.3, 7.4_

- [x] 10. RaftDataHandler 集成注册
  - 在 `src/common/constant.rs` 中定义 `NAMESPACE_TABLE_NAME` 常量（值为 "T_NAMESPACE"）
  - 在 `RaftDataHandler` 结构体中添加 `namespace_manager: Addr<NamespaceManager>` 字段
  - 在 `ClientRequest` 枚举中添加 `NamespaceReq { req: NamespaceManagerRaftReq }` 分支
  - 在 `ClientResponse` 枚举中添加 `NamespaceResp { resp: NamespaceManagerRaftResult }` 分支
  - 在 `apply_log_to_state_machine()` 中添加 NamespaceReq 请求路由
  - 在 `do_send_log()` 中添加命名空间异步发送逻辑
  - 在 `build_snapshot()` 中添加命名空间快照构建调用
  - 在 `load_snapshot()` 中添加命名空间快照加载路由
  - 在 `load_complete()` 中添加命名空间加载完成通知
  - _需求: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 11. 实现 Console API 接口
  - 在 `src/console/v1/namespace_api.rs` 中实现命名空间 API 路由注册
  - 实现 `GET /ratchjob/api/console/v1/namespace/list` 接口：分页查询、按 type 过滤
  - 实现 `GET /ratchjob/api/console/v1/namespace/info` 接口：根据 id 查询详情
  - 实现 `POST /ratchjob/api/console/v1/namespace/create` 接口：参数验证、通过 Raft 提交创建请求
  - 实现 `POST /ratchjob/api/console/v1/namespace/update` 接口：验证存在性、通过 Raft 提交更新请求
  - 实现 `POST /ratchjob/api/console/v1/namespace/remove` 接口：检查关联任务、通过 Raft 提交删除请求
  - _需求: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 12. 实现错误处理和参数验证
  - 在创建接口中验证 name 和 type 不为空，返回清晰错误提示
  - 在更新接口中验证命名空间是否存在，返回"命名空间不存在"错误
  - 在删除接口中检查命名空间下是否有关联任务，返回"命名空间下存在任务，无法删除"错误
  - 在 Raft 操作中记录详细错误日志，返回"系统错误"提示，不影响其他操作
  - _需求: 8.1, 8.2, 8.3, 8.4_

- [ ] 13. 兼容性与迁移保障
  - 确保默认命名空间（`xxl`）的兼容性，系统启动时自动创建默认命名空间
  - 确保 protobuf 定义支持版本兼容，支持旧版本快照加载
  - 保持现有 API 接口不变，新增 API 遵循 RESTful 规范
  - _需求: 10.1, 10.2, 10.3_
