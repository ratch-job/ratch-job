# ratch-job

一个rust实现的分布式任务调度平台服务。计划完全兼容xxl-job协议，然后再增强一些任务调度平台能力。



## 快速开始

### 一、 安装运行 ratch-job

【单机部署】

#### 方式1：下载二进制包运行

从 [github release](https://github.com/ratch-job/ratch-job/releases) 下载对应系统的应用包，解压后即可运行。

linux 或 MacOS

```shell
# 解压
tar -xvf ratchjob-x86_64-apple-darwin-v0.1.1-beta.2.tar.gz
# 运行
./ratchjob
```

windows 解压后直接运行 ratchjob.exe 即可。

#### 方式2: 通过docker 运行


```
docker pull qingpan/ratchjob:latest
docker run --name myratchjob -p 8725:8725 -p 8825:8825 -p 8925:8925 -d qingpan/ratchjob:latest
```


docker 的容器运行目录是 /io，会从这个目录读写配置文件

##### docker 版本说明

应用每次打包都会同时打对应版本的docker包 ，qingpan/ratchjob:$tag 。

每个版本会打两类docker包

|docker包类型|tag 格式| 示例 |说明 |
|--|--|--|--|
|gnu debian包|$version| qingpan/ratchjob:v0.1.1-beta.2 | docker包基于debian-slim,体积比较大,运行性能相对较高;|
|musl alpine包|$version-alpine| qingpan/ratchjob:v0.1.1-beta.2-alpine | docker包基于alpine,体积比较小,运行性能相对较低;|

支持使用  `qingpan/ratchjob:latest`


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

集群部署样列: 待补充


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

待补充


#### 方式7: 部署到k8s

待补充

#### 启动配置: 

| 参数KEY|内容描述|默认值|示例|开始支持的版本|
|--|--|--|--|--|
|RUST_LOG|日志等级:debug,info,warn,error;所有http,grpc请求都会打info日志,如果不观注可以设置为error减少日志量|info|error|0.3.0|
|RATCH_HTTP_API_PORT|http open api端口|8725|8725|0.1.x|
|RATCH_HTTP_CONSOLE_PORT|独立控制台端口|OpenApi+100|8825|0.1.x|
|RATCH_GRPC_CLUSTER_PORT|grpc端口(用于raft集群通信)|OpenApi+200|8925|0.1.x|
|RATCH_DATA_DIR|本地数据库文件夹, 会在系统运行时自动创建|linux,MacOS默认为~/.local/share/ratchjob/ratch_db;windows,docker默认为ratch_db|ratch_db|0.1.1|
|DEFAULT_XXL_JOB_ADMIN_PREFIX_PATH|自定义xxl-job api路径|/xxl-job-admin|/xxl-job-admin|0.1.x|
|RATCH_XXL_DEFAULT_ACCESS_TOKEN|xxl-job全局token|default_token|default_token|0.1.x|
|RATCH_RAFT_NODE_ID|节点id|1|1|0.1.1|
|RATCH_RAFT_NODE_ADDR|节点地址Ip:GrpcPort,单节点运行时每次启动都会生效；多节点集群部署时，只取加入集群时配置的值|127.0.0.1:GrpcPort|127.0.0.1:8925|0.1.1|
|RATCH_RAFT_AUTO_INIT|是否当做主节点初始化,(只在每一次启动时生效)|节点1时默认为true,节点非1时为false|true|0.1.1|
|RATCH_RAFT_JOIN_ADDR|是否当做节点加入对应的主节点,LeaderIp:GrpcPort；只在第一次启动时生效|空|127.0.0.1:8925|0.1.1|
|RATCH_CLUSTER_TOKEN|集群间的通信请求校验token，空表示不开启校验，设置后只有相同token的节点间才可通讯|空字符串|1234567890abcdefg|0.1.1|
|RATCH_GMT_OFFSET_HOURS|日志时间的时区，单位小时；默认为本机时区，运行在docker时需要指定|local|8(东8区),-5(西5区)|0.5.7|


