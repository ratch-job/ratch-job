# ratch-job

ratch-job 是一个rust实现的分布式任务调度平台服务。
完全兼容xxl-job协议，同时增强一些任务调度平台能力。


### 特性

1. 自带raft分布式储存,不依赖外部数据库，可直接运行服务。
2. 使用raft唯一主节点替代mysql锁，可大幅提升调度并发效率。
3. 轻量、高性能，每秒运行1000任务持续超过6分钟，cpu使用单核38%，内存占用85M；
4. 完全兼容xxl-job协议，支持使用xxl-job服务的应用平滑迁移到ratch-job；
5. 支持open-api管理任务。

## 架构

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/ratch-job_L2_v0.1.5.png)

说明：

1. ratch-job内部有应用、任务、任务调度3个核心功能模块；
2. 内部使用raft管理的分布式数据库持久化；
3. 集群部署时通过raft主节点统一管理任务调度，避免任务调度冲突.
4. ratch-job提供web控制台，方便用户通过ui管理应用任务。
5. ratch-job提供open-api,支持以api的方式管理应用任务。
6. ratch-job完全兼容xxl-job协议，支持xxl-job client接入。


## 核心调度流程

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250423083517.png)

任务调度可分3个流程，流程之间可并行，第3个流程依赖前两个流程才能成功。

1. 执行器通过xxl-job sdk反应用实例注册到ratch-job中；
2. 管理员通过控制台(或open-api)创建应用任务；
3. ratch-job调度器会自动根据任务的调度配置，在指定时间触发应用任务;


### 一、 安装运行 ratch-job

【单机部署】

#### 方式1：下载二进制包运行

从 [github release](https://github.com/ratch-job/ratch-job/releases) 下载对应系统的应用包，解压后即可运行。

linux 或 MacOS

```shell
# 解压
tar -xvf ratchjob-x86_64-apple-darwin-v0.1.4.tar.gz
# 运行
./ratchjob
```

windows 解压后直接运行 ratchjob.exe 即可。

**注:** 默认配置启动后通过`http://127.0.0.1:8825/ratchjob/`可以访问ratch-job控制台。


#### 方式2: 通过docker 运行


```
docker pull qingpan/ratchjob:stable
docker run --name myratchjob -p 8725:8725 -p 8825:8825 -p 8925:8925 -d qingpan/ratchjob:stable
```


docker 的容器运行目录是`/io`，会从这个目录读写数据文件

##### docker 版本说明

应用每次打包都会同时打对应版本的docker包 ，qingpan/ratchjob:$tag 。

每个版本会打两类docker包

|docker包类型|tag 格式| 示例 |说明 |
|--|--|--|--|
|gnu debian包|$version| qingpan/ratchjob:v0.1.4 | docker包基于debian-slim,体积比较大,运行性能相对较高;|
|musl alpine包|$version-alpine| qingpan/ratchjob:v0.1.4-alpine | docker包基于alpine,体积比较小,运行性能相对较低;|

支持使用  `qingpan/ratchjob:stable` 和 `qingpan/ratchjob:stable-alpine`


#### 方式3: 通过docker-compose 运行

单机部署样列:

[docker-compose.yaml](https://github.com/ratch-job/ratch-job/blob/master/docker/docker-compose/ratch-job-simple/docker-compose.yaml)

```yaml
# 集群部署样例,数据目录: ./data
version: '3.8'

services:
  ratchjob:
    image: qingpan/ratchjob:latest
    container_name: ratchjob
    ports:
      - "8725:8725"
      - "8825:8825"
      - "8925:8925"
    volumes:
      - ./data:/io:rw
    environment:
      - RATCH_HTTP_API_PORT=8725
      - RATCH_XXL_DEFAULT_ACCESS_TOKEN=default_token
      - DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH=/xxl-job-admin
    restart: always
```

集群部署样列: 

参考 [ratch-job-cluster docker-compose.yaml](https://github.com/ratch-job/ratch-job/blob/master/docker/docker-compose/ratch-job-cluster/docker-compose.yaml)


#### 方式4：通过 cargo 编译安装

```
# 安装
cargo install ratchjob
# 运行
ratchjob
```

#### 方式5: 下载源码编译运行

```
git clone https://github.com/ratch-job/ratch-job.git
cd ratch-job
cargo build --release
cargo run --release
```

#### 方式6: MacOS支持通过brew安装


```shell
# 把r-nacos加入taps
brew tap ratch-job/ratch-job 

# brew 安装 ratch-job
brew install ratch-job

# 运行
ratchjob

# 后续可以直接通过以下命令更新到最新版本
# brew upgrade ratch-job
```


#### 方式7: 部署到k8s

待补充

#### 启动配置: 

| 参数KEY|内容描述|默认值|示例|开始支持的版本|
|--|--|--|--|--|
|RUST_LOG|日志等级:debug,info,warn,error;所有http,grpc请求都会打info日志,如果不观注可以设置为error减少日志量|info|error|0.1.x|
|RATCH_HTTP_API_PORT|http open api端口|8725|8725|0.1.x|
|RATCH_HTTP_CONSOLE_PORT|独立控制台端口|OpenApi+100|8825|0.1.x|
|RATCH_GRPC_CLUSTER_PORT|grpc端口(用于raft集群通信)|OpenApi+200|8925|0.1.x|
|RATCH_DATA_DIR|本地数据库文件夹, 会在系统运行时自动创建|linux,MacOS默认为~/.local/share/ratchjob/ratch_db;windows,docker默认为ratch_db|ratch_db|0.1.x|
|DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH|自定义xxl-job api路径|/xxl-job-admin|/xxl-job-admin|0.1.x|
|RATCH_XXL_DEFAULT_ACCESS_TOKEN|xxl-job全局token|default_token|default_token|0.1.x|
|RATCH_RAFT_NODE_ID|节点id|1|1|0.1.x|
|RATCH_RAFT_NODE_ADDR|节点地址Ip:GrpcPort,单节点运行时每次启动都会生效；多节点集群部署时，只取加入集群时配置的值|127.0.0.1:GrpcPort|127.0.0.1:8925|0.1.x|
|RATCH_RAFT_AUTO_INIT|是否当做主节点初始化,(只在每一次启动时生效)|节点1时默认为true,节点非1时为false|true|0.1.x|
|RATCH_RAFT_JOIN_ADDR|是否当做节点加入对应的主节点,LeaderIp:GrpcPort；只在第一次启动时生效|空|127.0.0.1:8925|0.1.x|
|RATCH_CLUSTER_TOKEN|集群间的通信请求校验token，空表示不开启校验，设置后只有相同token的节点间才可通讯|空字符串|1234567890abcdefg|0.1.x|
|RATCH_GMT_OFFSET_HOURS|日志时间的时区，单位小时；默认为本机时区，运行在docker时需要指定|local|8(东8区),-5(西5区)|0.1.x|
| RATCH_HTTP_WORKERS | HTTP服务线程数，空表示自动分配 | 空 | 8 | 0.1.x |
| RATCH_INSTANCE_HEALTH_TIMEOUT | 实例健康检查超时时间(秒) | 90 | 120 | 0.1.x |
| RATCH_RAFT_SNAPSHOT_LOG_SIZE | Raft触发快照的日志条数阈值 | 10000 | 20000 | 0.1.x |
| RATCH_ENABLE_METRICS | 是否启用指标收集 | true | true | 0.1.x |
| RATCH_METRICS_COLLECT_INTERVAL_SECOND | 指标收集间隔(秒) | 15 | 30 | 0.1.x |
| RATCH_METRICS_ENABLE_LOG | 是否记录指标日志 | false | false | 0.1.x |
| RATCH_METRICS_LOG_INTERVAL_SECOND | 指标日志记录间隔(秒)，最小5秒 | 60 | 30 | 0.1.x |
| RATCH_TASK_REQUEST_PARALLEL | 任务请求(协程)并行处理数 | 20 | 50 | 0.1.x |
| RATCH_CONSOLE_ENABLE_CAPTCHA | 控制台登陆是否启用验证码 | true | true | 0.1.5 |
| RATCH_CONSOLE_LOGIN_TIMEOUT | 控制台登陆session过期时间(秒) | 86400 | 86400 | 0.1.5 |
| RATCH_INIT_ADMIN_USERNAME | 初始化管理员用户名 | admin | admin | 0.1.5 |
| RATCH_INIT_ADMIN_PASSWORD | 初始化管理员密码 | admin | admin | 0.1.5 |




### 二、 运行xxl-job执行器


#### 1、 java执行器

参考[xxl-job执行样例](https://github.com/xuxueli/xxl-job/tree/2.4.2/xxl-job-executor-samples/xxl-job-executor-sample-springboot)

关键配置`application.properties`信息：

```properties
xxl.job.admin.addresses=http://127.0.0.1:8725/xxl-job-admin
xxl.job.accessToken=default_token
```

把两项设置为ratch-job对应的信息，打包运行，如果正常可在ratch-job控制台-> 执行器管理查看到执行器与其实例信息。


![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331003904.png)


#### 2、rust执行器

参考作者写的rust xxl-job sdk对应样例[xxljob-sdk-rs examples](https://github.com/heqingpan/xxljob-sdk-rs/blob/master/examples/src/registry.rs)


#### 其它语言

待补充


### 三、创建设置任务

ratch-job支持通过控制台或open-api创建任务。

1、方式一：任务管理中创建任务

系统会默认创建一个名为`admin`的用户，密码为`admin`(也可以通过环境变量 RATCH_INIT_ADMIN_USERNAME 和 RATCH_INIT_ADMIN_PASSWORD 修改默认账号的账户名和密码)。 

使用默认账户登陆控制台并创建任务：


![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331004847.png)

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331005302.png)


2、方式二：通过open-api创建任务


```sh
curl -X POST "http://127.0.0.1:8725/ratch/v1/job/create" -H 'Content-Type: application/json' -d '{"appName":"xxl-job-executor-sample","namespace":"xxl","handleName":"demoJobHandler","scheduleType":"CRON","cronValue":"0/15 * * * * *","blockingStrategy":"SERIAL_EXECUTION"}'
```

响应信息为:

```json
{"content":{"id":2,"enable":true,"appName":"xxl-job-executor-sample","namespace":"xxl","description":"","scheduleType":"CRON","cronValue":"0/15 * * * * *","delaySecond":0,"intervalSecond":0,"runMode":"BEAN","handleName":"demoJobHandler","triggerParam":"","routerStrategy":"ROUND_ROBIN","pastDueStrategy":"DEFAULT","blockingStrategy":"SERIAL_EXECUTION","timeoutSecond":0,"tryTimes":0,"versionId":0,"lastModifiedMillis":1743353817624,"createTime":1743353817624,"retryInterval":0},"code":200,"msg":null}%
```

创建任务后，如果是启用状态ratch-job会按配置的规则触发任务


3、手动触发任务

支持选择指定执行器运行任务。


![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331011411.png)


### 四、任务相关其它功能

#### 1. 查看任务执行记录

任务对应执行记录，每个任务默认只保留最近100条记录。

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331011445.png)


#### 2. 全局最近执行记录

全局默认保留最近10000条记录.

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250331011503.png)



### 五、系统监控

ratch-job控制台自带服务监控，包含服务应用cpu,内存水位、任务调度rps、状态、数量等关键指标。


![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250329102751.png)


### 六、性能

为了确认ratch-job性能，对几个使用水位分别做的压测，以下为压测结果：


|环境|任务数|任务调度tps|cpu使用率(单核占比)|内存(M)|
|--|--|--|--|--|
|docker|100|100|5.2%|20M|
|ubuntu|1000|1000|32%|80M|
|mac m1|2000|2000|18%|90M|
|ubuntu|5000|5000|107%|270M|
|ubuntu|10000|10000|220%|810M|

+ 100tps

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250330222620.png)

+ 1000tps

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250329004040.png)

+ 2000tps

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250328090942.png)

+ 5000tps

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250326010712.png)

+ 10000tps

![](https://github.com/ratch-job/ratch-job/raw/master/doc/assets/imgs/20250329102751.png)


### 七、open api


待补充说明

