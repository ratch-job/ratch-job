# Ratch Job Open API 文档

## 概述

本文档描述了 Ratch Job 系统的 HTTP Open API 接口，所有接口都以 `/ratch/v1/job/` 开头。

基础地址：`http://127.0.0.1:8725`

---

## 1. 创建任务

**接口地址：** `POST /ratch/v1/job/create`

**接口描述：** 创建一个新的定时任务

### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| appName | string | 是 | 应用名称 |
| namespace | string | 是 | 命名空间 |
| handleName | string | 是 | 任务处理器名称（当runMode为BEAN时必填） |
| scheduleType | string | 否 | 调度类型：CRON、INTERVAL、DELAY、NONE |
| cronValue | string | 否 | CRON表达式（当scheduleType为CRON时必填） |
| delaySecond | number | 否 | 延迟秒数（当scheduleType为DELAY时必填） |
| intervalSecond | number | 否 | 间隔秒数（当scheduleType为INTERVAL时必填） |
| runMode | string | 否 | 运行模式：BEAN、GLUE_GROOVY、GLUE_SHELL、GLUE_PYTHON、GLUE_PHP、GLUE_NODEJS、GLUE_POWERSHELL |
| description | string | 否 | 任务描述 |
| triggerParam | string | 否 | 触发参数 |
| routerStrategy | string | 否 | 路由策略：FIRST、LAST、ROUND_ROBIN、RANDOM、CONSISTENT_HASH、SHARDING_BROADCAST |
| pastDueStrategy | string | 否 | 过期策略：DEFAULT、IGNORE、EXECUTE |
| blockingStrategy | string | 否 | 阻塞策略：SERIAL_EXECUTION、DISCARD_LATER、COVER_EARLY、OTHER |
| timeoutSecond | number | 否 | 超时秒数 |
| tryTimes | number | 否 | 重试次数 |
| retryInterval | number | 否 | 重试间隔 |
| enable | boolean | 否 | 是否启用，默认true |

### 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| content | object | 任务信息对象 |
| content.id | number | 任务ID |
| content.enable | boolean | 是否启用 |
| content.appName | string | 应用名称 |
| content.namespace | string | 命名空间 |
| content.description | string | 任务描述 |
| content.scheduleType | string | 调度类型 |
| content.cronValue | string | CRON表达式 |
| content.delaySecond | number | 延迟秒数 |
| content.intervalSecond | number | 间隔秒数 |
| content.runMode | string | 运行模式 |
| content.handleName | string | 任务处理器名称 |
| content.triggerParam | string | 触发参数 |
| content.routerStrategy | string | 路由策略 |
| content.pastDueStrategy | string | 过期策略 |
| content.blockingStrategy | string | 阻塞策略 |
| content.timeoutSecond | number | 超时秒数 |
| content.tryTimes | number | 重试次数 |
| content.versionId | number | 版本ID |
| content.lastModifiedMillis | number | 最后修改时间戳 |
| content.createTime | number | 创建时间戳 |
| content.retryInterval | number | 重试间隔 |
| code | number | 响应码，200表示成功 |
| msg | string | 错误信息 |

### 示例

通过open-api创建任务

```sh
curl -X POST "http://127.0.0.1:8725/ratch/v1/job/create" -H 'Content-Type: application/json' -d '{"appName":"xxl-job-executor-sample","namespace":"xxl","handleName":"demoJobHandler","scheduleType":"CRON","cronValue":"0/15 * * * * *","blockingStrategy":"SERIAL_EXECUTION"}'
```

响应信息为:

```json
{"content":{},"code":200,"msg":null}
```

### 注意事项

- 通过 `enable` 参数可以设置任务是否启用：
  - `enable: true` - 启用任务，任务会按照调度配置正常执行
  - `enable: false` - 禁用任务，任务不会执行但配置会保留
- 更新任务时，只需要传入需要修改的字段，其他字段会保持原值不变
- 任务ID (`id`) 是必填参数，用于指定要更新的任务

---

## 2. 更新任务

**接口地址：** `POST /ratch/v1/job/update`

**接口描述：** 更新已存在的任务信息

### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | number | 是 | 任务ID |
| appName | string | 否 | 应用名称 |
| namespace | string | 否 | 命名空间 |
| handleName | string | 否 | 任务处理器名称 |
| scheduleType | string | 否 | 调度类型 |
| cronValue | string | 否 | CRON表达式 |
| delaySecond | number | 否 | 延迟秒数 |
| intervalSecond | number | 否 | 间隔秒数 |
| runMode | string | 否 | 运行模式 |
| description | string | 否 | 任务描述 |
| triggerParam | string | 否 | 触发参数 |
| routerStrategy | string | 否 | 路由策略 |
| pastDueStrategy | string | 否 | 过期策略 |
| blockingStrategy | string | 否 | 阻塞策略 |
| timeoutSecond | number | 否 | 超时秒数 |
| tryTimes | number | 否 | 重试次数 |
| retryInterval | number | 否 | 重试间隔 |
| enable | boolean | 否 | 是否启用 |

### 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| content | object | 响应内容，更新成功时为空对象 |
| code | number | 响应码，200表示成功 |
| msg | string | 错误信息 |

### 示例

```sh
curl -X POST "http://127.0.0.1:8725/ratch/v1/job/update" -H 'Content-Type: application/json' -d '{"id":2,"appName":"xxl-job-executor-sample","namespace":"xxl","handleName":"demoJobHandler","scheduleType":"CRON","cronValue":"0/30 * * * * *","blockingStrategy":"SERIAL_EXECUTION","enable":true}'
```

响应信息为:

```json
{"content":{},"code":200,"msg":null}
```

---

## 3. 删除任务

**接口地址：** `POST /ratch/v1/job/remove`

**接口描述：** 删除指定的任务

### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | number | 是 | 任务ID |

### 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| content | object | 响应内容，删除成功时为空对象 |
| code | number | 响应码，200表示成功 |
| msg | string | 错误信息 |

### 示例

```sh
curl -X POST "http://127.0.0.1:8725/ratch/v1/job/remove" -H 'Content-Type: application/json' -d '{"id":2}'
```

响应信息为:

```json
{"content":{},"code":200,"msg":null}
```

---

## 4. 获取任务详情

**接口地址：** `GET /ratch/v1/job/info`

**接口描述：** 获取指定任务的详细信息

### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | number | 是 | 任务ID |

### 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data | object | 任务信息对象 |
| data.id | number | 任务ID |
| data.enable | boolean | 是否启用 |
| data.appName | string | 应用名称 |
| data.namespace | string | 命名空间 |
| data.description | string | 任务描述 |
| data.scheduleType | string | 调度类型 |
| data.cronValue | string | CRON表达式 |
| data.delaySecond | number | 延迟秒数 |
| data.intervalSecond | number | 间隔秒数 |
| data.runMode | string | 运行模式 |
| data.handleName | string | 任务处理器名称 |
| data.triggerParam | string | 触发参数 |
| data.routerStrategy | string | 路由策略 |
| data.pastDueStrategy | string | 过期策略 |
| data.blockingStrategy | string | 阻塞策略 |
| data.timeoutSecond | number | 超时秒数 |
| data.tryTimes | number | 重试次数 |
| data.versionId | number | 版本ID |
| data.lastModifiedMillis | number | 最后修改时间戳 |
| data.createTime | number | 创建时间戳 |
| data.retryInterval | number | 重试间隔 |
| success | boolean | 是否成功 |
| code | string | 错误码 |
| message | string | 错误信息 |

### 示例

```sh
curl -X GET "http://127.0.0.1:8725/ratch/v1/job/info?id=2"
```

响应信息为:

```json
{"data":{"id":2,"enable":true,"appName":"xxl-job-executor-sample","namespace":"xxl","description":"","scheduleType":"CRON","cronValue":"0/15 * * * * *","delaySecond":0,"intervalSecond":0,"runMode":"BEAN","handleName":"demoJobHandler","triggerParam":"","routerStrategy":"ROUND_ROBIN","pastDueStrategy":"DEFAULT","blockingStrategy":"SERIAL_EXECUTION","timeoutSecond":0,"tryTimes":0,"versionId":0,"lastModifiedMillis":1743353817624,"createTime":1743353817624,"retryInterval":0},"success":true,"code":null,"message":null}
```

---

## 5. 查询任务列表

**接口地址：** `GET /ratch/v1/job/list`

**接口描述：** 分页查询任务列表

### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| namespace | string | 否 | 命名空间 |
| appName | string | 否 | 应用名称 |
| likeDescription | string | 否 | 任务描述模糊查询 |
| likeHandleName | string | 否 | 处理器名称模糊查询 |
| pageNo | number | 否 | 页码，默认1 |
| pageSize | number | 否 | 每页大小，默认所有 |

### 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| data | object | 分页结果对象 |
| data.totalCount | number | 总记录数 |
| data.list | array | 任务列表 |
| success | boolean | 是否成功 |
| code | string | 错误码 |
| message | string | 错误信息 |

### 示例

```sh
curl -X GET "http://127.0.0.1:8725/ratch/v1/job/list?namespace=xxl&pageNo=1&pageSize=10"
```

响应信息为:

```json
{"data":{"totalCount":1,"list":[{"id":2,"enable":true,"appName":"xxl-job-executor-sample","namespace":"xxl","description":"","scheduleType":"CRON","cronValue":"0/15 * * * * *","delaySecond":0,"intervalSecond":0,"runMode":"BEAN","handleName":"demoJobHandler","triggerParam":"","routerStrategy":"ROUND_ROBIN","pastDueStrategy":"DEFAULT","blockingStrategy":"SERIAL_EXECUTION","timeoutSecond":0,"tryTimes":0,"versionId":0,"lastModifiedMillis":1743353817624,"createTime":1743353817624,"retryInterval":0}]},"success":true,"code":null,"message":null}
```

---

## 枚举值说明

### 调度类型 (scheduleType)
- `CRON`: CRON表达式调度
- `INTERVAL`: 固定间隔调度
- `DELAY`: 延迟调度
- `NONE`: 无调度

### 运行模式 (runMode)
- `BEAN`: Spring Bean模式
- `GLUE_GROOVY`: Groovy脚本模式
- `GLUE_SHELL`: Shell脚本模式
- `GLUE_PYTHON`: Python脚本模式
- `GLUE_PHP`: PHP脚本模式
- `GLUE_NODEJS`: Node.js脚本模式
- `GLUE_POWERSHELL`: PowerShell脚本模式

### 路由策略 (routerStrategy)
- `FIRST`: 第一个
- `LAST`: 最后一个
- `ROUND_ROBIN`: 轮询
- `RANDOM`: 随机
- `CONSISTENT_HASH`: 一致性哈希
- `SHARDING_BROADCAST`: 分片广播

### 过期策略 (pastDueStrategy)
- `DEFAULT`: 默认策略
- `IGNORE`: 忽略过期任务
- `EXECUTE`: 执行过期任务

### 阻塞策略 (blockingStrategy)
- `SERIAL_EXECUTION`: 串行执行
- `DISCARD_LATER`: 丢弃后续任务
- `COVER_EARLY`: 覆盖之前任务
- `OTHER`: 其他策略

---

## 错误码说明

- `200`: 成功
- `500`: 系统内部错误

当接口调用失败时，会返回相应的错误信息和错误码。
