use crate::common::datetime_utils::now_millis;
use crate::common::get_app_version;
use crate::common::model::privilege::PrivilegeGroup;
use crate::common::model::{ApiResult, UserSession};
use crate::common::namespace_util::{get_namespace, get_namespace_by_option};
use crate::common::share_data::ShareData;
use crate::console::model::job::{
    JobExportInfo, JobImportForm, JobImportResult, JobQueryListRequest, RatchMeta,
};
use crate::console::v1::job_api::{do_create_job, do_update_job};
use crate::console::v1::{ERROR_CODE_NO_PERMISSION, ERROR_CODE_SYSTEM_ERROR};
use crate::job::job_index::JobQueryParam;
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::enum_type::{
    ExecutorBlockStrategy, JobRunMode, PastDueStrategy, RouterStrategy, ScheduleType,
};
use crate::job::model::job::{JobInfoDto, JobKey, JobParam};
use actix_http::HttpMessage;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::http::header::{ContentDisposition, ContentType};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::io::{Cursor, Read, Write};
use std::sync::Arc;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

/// 导出包元数据文件名
pub(crate) const RATCH_META_FILE_NAME: &str = ".ratch_meta";
/// 导出模块标识，导入时校验使用
pub(crate) const JOB_EXPORT_MODULE: &str = "joblist";

/// 解析 .ratch_meta 内容并校验导出模块标识，模块不匹配时返回错误
pub(crate) fn parse_ratch_meta(content: &str) -> anyhow::Result<RatchMeta> {
    let meta: RatchMeta = serde_yaml::from_str(content)?;
    if meta.module != JOB_EXPORT_MODULE {
        return Err(anyhow::anyhow!(
            "invalid ratch meta module:{},expected module:{}",
            meta.module,
            JOB_EXPORT_MODULE
        ));
    }
    Ok(meta)
}

/// 把任务列表按 id 升序打包为导出 zip，每个任务一个 `{index:04}.yaml`，末尾追加 `.ratch_meta`
pub(crate) fn build_export_zip(mut list: Vec<JobInfoDto>) -> anyhow::Result<Vec<u8>> {
    list.sort_by_key(|dto| dto.id);
    let mut buf = Vec::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default();
        for (index, dto) in list.iter().enumerate() {
            let export_info = JobExportInfo::from(dto);
            writer.start_file(format!("{:04}.yaml", index + 1), options)?;
            writer.write_all(serde_yaml::to_string(&export_info)?.as_bytes())?;
        }
        let meta = RatchMeta {
            version: get_app_version().to_string(),
            module: JOB_EXPORT_MODULE.to_string(),
        };
        writer.start_file(RATCH_META_FILE_NAME, options)?;
        writer.write_all(serde_yaml::to_string(&meta)?.as_bytes())?;
        writer.finish()?;
    }
    Ok(buf)
}

/// 解析导入 zip 为任务列表，`namespace` 为上传时指定的目标命名空间；
/// 返回（任务列表，失败明细），yaml 条目解析失败计入明细且不中断；zip 非法或模块不匹配时整体报错
pub(crate) fn parse_import_zip(
    bytes: &[u8],
    namespace: &str,
) -> anyhow::Result<(Vec<JobInfoDto>, Vec<String>)> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| anyhow::anyhow!("import file is not a valid zip,{}", e))?;
    let mut dto_list = Vec::new();
    let mut errors = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if name == RATCH_META_FILE_NAME {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            parse_ratch_meta(&content)?;
            continue;
        }
        if !name.ends_with(".yaml") {
            continue;
        }
        let mut content = String::new();
        if let Err(e) = entry.read_to_string(&mut content) {
            errors.push(format!("{}: read entry error,{}", name, e));
            continue;
        }
        match serde_yaml::from_str::<JobExportInfo>(&content) {
            Ok(export_info) => dto_list.push(build_import_job_dto(export_info, namespace)),
            Err(e) => errors.push(format!("{}: yaml parse error,{}", name, e)),
        }
    }
    Ok((dto_list, errors))
}

/// 把导出内容转换为导入任务，id 归零、命名空间统一取上传时指定值
fn build_import_job_dto(export_info: JobExportInfo, namespace: &str) -> JobInfoDto {
    JobInfoDto {
        id: 0,
        enable: export_info.enable,
        app_name: export_info.app_name,
        key: export_info.key,
        namespace: Arc::new(namespace.to_owned()),
        description: export_info.description,
        schedule_type: export_info.schedule_type,
        cron_value: export_info.cron_value,
        delay_second: export_info.delay_second,
        interval_second: export_info.interval_second,
        run_mode: export_info.run_mode,
        handle_name: export_info.handle_name,
        trigger_param: export_info.trigger_param,
        glue_source: export_info.glue_source,
        router_strategy: export_info.router_strategy,
        past_due_strategy: export_info.past_due_strategy,
        blocking_strategy: export_info.blocking_strategy,
        timeout_second: export_info.timeout_second,
        try_times: export_info.try_times,
        version_id: export_info.version_id,
        last_modified_millis: export_info.last_modified_millis,
        register_time: export_info.register_time,
        retry_interval: export_info.retry_interval,
    }
}

/// 导入任务的单条 upsert：已存在（JobId 命中）走更新分支，否则走创建分支，与页面创建/更新行为一致
async fn upsert_job(
    share_data: &Data<Arc<ShareData>>,
    app_privilege: &PrivilegeGroup<Arc<String>>,
    dto: JobInfoDto,
    existing_job_id: Option<u64>,
) -> anyhow::Result<()> {
    if dto.key.is_empty() {
        return Err(anyhow::anyhow!("job key is empty"));
    }
    if !app_privilege.check_permission(&dto.app_name) {
        return Err(anyhow::anyhow!("user no app permission:{}", &dto.app_name));
    }
    let mut param = JobParam {
        id: None,
        enable: Some(dto.enable),
        app_name: Some(dto.app_name.clone()),
        namespace: Some(dto.namespace.clone()),
        key: Some(dto.key.clone()),
        description: Some(dto.description.clone()),
        schedule_type: Some(ScheduleType::from_str(&dto.schedule_type)),
        cron_value: Some(dto.cron_value.clone()),
        delay_second: Some(dto.delay_second),
        interval_second: Some(dto.interval_second),
        run_mode: Some(JobRunMode::from_str(&dto.run_mode).unwrap_or(JobRunMode::Bean)),
        handle_name: Some(dto.handle_name.clone()),
        trigger_param: Some(dto.trigger_param.clone()),
        glue_source: Some(dto.glue_source.clone()),
        router_strategy: Some(
            RouterStrategy::from_str(&dto.router_strategy).unwrap_or(RouterStrategy::RoundRobin),
        ),
        past_due_strategy: Some(PastDueStrategy::from_str(&dto.past_due_strategy)),
        blocking_strategy: Some(ExecutorBlockStrategy::from_str(&dto.blocking_strategy)),
        timeout_second: Some(dto.timeout_second),
        try_times: Some(dto.try_times),
        update_time: Some(now_millis()),
        retry_interval: Some(dto.retry_interval),
    };
    param.migrate_legacy_glue_source();
    match existing_job_id {
        Some(job_id) if job_id != 0 => {
            param.id = Some(job_id);
            do_update_job(share_data.clone(), app_privilege, param).await
        }
        _ => do_create_job(share_data.clone(), param).await.map(|_| ()),
    }
}

/// 导入任务：解析 zip 后经批量 JobKey 查询逐条 upsert，返回导入结果统计（有失败项仍返回成功并携带明细）
pub(crate) async fn import_jobs(
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    MultipartForm(form): MultipartForm<JobImportForm>,
) -> impl Responder {
    let session = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    let namespace = match form.namespace {
        Some(Text(namespace)) => get_namespace(&Arc::new(namespace)),
        None => get_namespace_by_option(&None),
    };
    if !session.namespace_privilege.check_permission(&namespace) {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_NO_PERMISSION.to_string(),
            Some(format!("user no namespace permission:{}", namespace)),
        ));
    }
    let file_bytes = match std::fs::read(form.file.file.path()) {
        Ok(bytes) => bytes,
        Err(e) => {
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(format!("import_jobs read upload file error,{}", e)),
            ));
        }
    };
    let (dto_list, parse_errors) = match parse_import_zip(&file_bytes, &namespace) {
        Ok(result) => result,
        Err(e) => {
            log::error!("import_jobs parse zip error,{}", e);
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some(format!("import_jobs error,{}", e)),
            ));
        }
    };
    let keys: Vec<JobKey> = dto_list
        .iter()
        .map(|dto| JobKey::new(&namespace, &dto.app_name, &dto.key))
        .collect();
    let id_map = match share_data
        .job_manager
        .send(JobManagerReq::GetJobIdsByKeys(keys))
        .await
    {
        Ok(Ok(JobManagerResult::JobIdMap(map))) => map,
        _ => {
            return HttpResponse::Ok().json(ApiResult::<()>::error(
                ERROR_CODE_SYSTEM_ERROR.to_string(),
                Some("import_jobs query job ids by keys error".to_string()),
            ));
        }
    };

    let mut result = JobImportResult {
        total_count: (dto_list.len() + parse_errors.len()) as u64,
        ..Default::default()
    };
    for error in parse_errors {
        result.fail_count += 1;
        result.errors.push(error);
    }
    for dto in dto_list {
        let job_key = JobKey::new(&namespace, &dto.app_name, &dto.key);
        let existing_job_id = id_map.get(&job_key).copied();
        match upsert_job(&share_data, &session.app_privilege, dto, existing_job_id).await {
            Ok(()) => result.success_count += 1,
            Err(e) => {
                result.fail_count += 1;
                result.errors.push(format!(
                    "job {}/{}: {}",
                    job_key.app_name, job_key.job_key, e
                ));
            }
        }
    }
    HttpResponse::Ok().json(ApiResult::success(Some(result)))
}

/// 按查询条件全量导出任务为 zip 下载
pub(crate) async fn export_jobs(
    req: actix_web::HttpRequest,
    share_data: Data<Arc<ShareData>>,
    web::Query(request): web::Query<JobQueryListRequest>,
) -> impl Responder {
    let session = if let Some(session) = req.extensions().get::<Arc<UserSession>>() {
        session.clone()
    } else {
        return HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("user session is invalid".to_string()),
        ));
    };
    let mut param: JobQueryParam = request.to_param_with_session(&session);
    param.offset = 0;
    param.limit = 0xffff_ffff;
    match share_data
        .job_manager
        .send(JobManagerReq::QueryJob(param))
        .await
    {
        Ok(Ok(JobManagerResult::JobPageInfo(_, list))) => match build_export_zip(list) {
            Ok(bytes) => {
                let filename = format!("ratchjob_job_export_{}.zip", now_millis());
                HttpResponse::Ok()
                    .insert_header(ContentType::octet_stream())
                    .insert_header(ContentDisposition::attachment(filename))
                    .body(bytes)
            }
            Err(e) => {
                log::error!("export_jobs build zip error,{}", e);
                HttpResponse::Ok().json(ApiResult::<()>::error(
                    ERROR_CODE_SYSTEM_ERROR.to_string(),
                    Some(format!("export_jobs error,{}", e)),
                ))
            }
        },
        _ => HttpResponse::Ok().json(ApiResult::<()>::error(
            ERROR_CODE_SYSTEM_ERROR.to_string(),
            Some("export_jobs error".to_string()),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::model::job::JobExportInfo;
    use std::io::{Cursor, Read};

    fn build_test_job_dto(id: u64, key: &str) -> JobInfoDto {
        JobInfoDto {
            id,
            enable: true,
            app_name: Arc::new("demo-app".to_string()),
            key: Arc::new(key.to_string()),
            namespace: Arc::new("xxl".to_string()),
            description: Arc::new(format!("job description {}", key)),
            schedule_type: "CRON".to_string(),
            cron_value: Arc::new("0 0 * * * ?".to_string()),
            delay_second: 0,
            interval_second: 0,
            run_mode: "BEAN".to_string(),
            handle_name: Arc::new("demoHandler".to_string()),
            trigger_param: Arc::new("".to_string()),
            glue_source: Arc::new("".to_string()),
            router_strategy: "ROUND".to_string(),
            past_due_strategy: "DEFAULT".to_string(),
            blocking_strategy: "SERIAL_EXECUTION".to_string(),
            timeout_second: 0,
            try_times: 0,
            version_id: 1,
            last_modified_millis: 1000,
            register_time: 2000,
            retry_interval: 0,
        }
    }

    #[test]
    fn test_job_export_info_yaml_roundtrip() {
        let dto = build_test_job_dto(100, "job-001");
        let export_info = JobExportInfo::from(&dto);
        let yaml = serde_yaml::to_string(&export_info).unwrap();

        assert!(yaml.contains("appName: demo-app"));
        assert!(yaml.contains("key: job-001"));
        assert!(yaml.contains("scheduleType: CRON"));
        assert!(!yaml.contains("id:"));
        assert!(!yaml.contains("namespace:"));

        let parsed: JobExportInfo = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(serde_yaml::to_string(&parsed).unwrap(), yaml);
        assert_eq!(parsed.app_name.as_str(), "demo-app");
        assert_eq!(parsed.key.as_str(), "job-001");
        assert_eq!(parsed.handle_name.as_str(), "demoHandler");
        assert_eq!(parsed.description.as_str(), "job description job-001");
        assert_eq!(parsed.schedule_type, "CRON");
        assert_eq!(parsed.cron_value.as_str(), "0 0 * * * ?");
        assert_eq!(parsed.run_mode, "BEAN");
        assert_eq!(parsed.enable, true);
        assert_eq!(parsed.version_id, 1);
        assert_eq!(parsed.last_modified_millis, 1000);
        assert_eq!(parsed.register_time, 2000);
    }

    #[test]
    fn test_ratch_meta_serialize_and_module_check() {
        let meta = RatchMeta {
            version: get_app_version().to_string(),
            module: JOB_EXPORT_MODULE.to_string(),
        };
        let yaml = serde_yaml::to_string(&meta).unwrap();
        assert!(yaml.contains("version: 0.3.0"));
        assert!(yaml.contains("module: joblist"));

        let parsed = parse_ratch_meta("version: 0.3.0\nmodule: joblist").unwrap();
        assert_eq!(parsed.module, "joblist");
        assert_eq!(parsed.version, "0.3.0");

        let err = parse_ratch_meta("version: 0.3.0\nmodule: other").unwrap_err();
        assert!(err.to_string().contains("joblist"));
    }

    #[test]
    fn test_build_export_zip_meta_only_when_list_empty() {
        let bytes = build_export_zip(vec![]).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 1);
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert_eq!(names, vec![RATCH_META_FILE_NAME.to_string()]);
        let mut content = String::new();
        archive
            .by_name(RATCH_META_FILE_NAME)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        let meta = parse_ratch_meta(&content).unwrap();
        assert_eq!(meta.module, JOB_EXPORT_MODULE);
        assert_eq!(meta.version, get_app_version());
    }

    #[test]
    fn test_build_export_zip_orders_by_id() {
        let list = vec![
            build_test_job_dto(300, "job-a"),
            build_test_job_dto(100, "job-b"),
            build_test_job_dto(200, "job-c"),
        ];
        let bytes = build_export_zip(list).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 4);
        assert_eq!(archive.by_index(0).unwrap().name(), "0001.yaml");
        assert_eq!(archive.by_index(1).unwrap().name(), "0002.yaml");
        assert_eq!(archive.by_index(2).unwrap().name(), "0003.yaml");
        assert_eq!(archive.by_index(3).unwrap().name(), RATCH_META_FILE_NAME);

        let mut yaml = String::new();
        archive
            .by_name("0001.yaml")
            .unwrap()
            .read_to_string(&mut yaml)
            .unwrap();
        assert!(yaml.contains("key: job-b"));
        yaml.clear();
        archive
            .by_name("0002.yaml")
            .unwrap()
            .read_to_string(&mut yaml)
            .unwrap();
        assert!(yaml.contains("key: job-c"));
        yaml.clear();
        archive
            .by_name("0003.yaml")
            .unwrap()
            .read_to_string(&mut yaml)
            .unwrap();
        assert!(yaml.contains("key: job-a"));
    }

    #[test]
    fn test_export_zip_import_zip_roundtrip() {
        let list = vec![
            build_test_job_dto(300, "job-a"),
            build_test_job_dto(100, "job-b"),
            build_test_job_dto(200, "job-c"),
        ];
        let bytes = build_export_zip(list).unwrap();
        let (dto_list, errors) = parse_import_zip(&bytes, "target-ns").unwrap();
        assert!(errors.is_empty());
        assert_eq!(dto_list.len(), 3);
        assert_eq!(dto_list[0].id, 0);
        assert_eq!(dto_list[0].namespace.as_str(), "target-ns");
        assert_eq!(dto_list[0].key.as_str(), "job-b");
        assert_eq!(dto_list[1].key.as_str(), "job-c");
        assert_eq!(dto_list[2].key.as_str(), "job-a");
        for dto in dto_list.iter() {
            assert_eq!(dto.app_name.as_str(), "demo-app");
            assert_eq!(
                dto.description.as_str(),
                format!("job description {}", dto.key)
            );
            assert_eq!(dto.schedule_type, "CRON");
            assert_eq!(dto.cron_value.as_str(), "0 0 * * * ?");
            assert_eq!(dto.handle_name.as_str(), "demoHandler");
            assert_eq!(dto.run_mode, "BEAN");
            assert_eq!(dto.router_strategy, "ROUND");
            assert_eq!(dto.past_due_strategy, "DEFAULT");
            assert_eq!(dto.blocking_strategy, "SERIAL_EXECUTION");
        }

        let empty_bytes = build_export_zip(vec![]).unwrap();
        let (empty_list, empty_errors) = parse_import_zip(&empty_bytes, "target-ns").unwrap();
        assert!(empty_list.is_empty());
        assert!(empty_errors.is_empty());
    }

    #[test]
    fn test_parse_import_zip_invalid_bytes() {
        let err = parse_import_zip(&[0x50, 0x4b, 0x01, 0x02], "target-ns").unwrap_err();
        assert!(err.to_string().contains("zip"));
    }

    #[test]
    fn test_parse_import_zip_invalid_module() {
        let yaml = build_test_yaml("job-001");
        let bytes = build_test_zip(
            &[(
                RATCH_META_FILE_NAME,
                "version: 0.3.0\nmodule: other".as_bytes(),
            )],
            &[("0001.yaml", yaml.as_bytes())],
        );
        let err = parse_import_zip(&bytes, "target-ns").unwrap_err();
        assert!(err.to_string().contains("joblist"));
    }

    #[test]
    fn test_parse_import_zip_invalid_yaml_entry() {
        let yaml = build_test_yaml("job-001");
        let bytes = build_test_zip(
            &[(
                RATCH_META_FILE_NAME,
                "version: 0.3.0\nmodule: joblist".as_bytes(),
            )],
            &[
                ("0001.yaml", yaml.as_bytes()),
                ("0002.yaml", ": : 非法yaml".as_bytes()),
            ],
        );
        let (dto_list, errors) = parse_import_zip(&bytes, "target-ns").unwrap();
        assert_eq!(dto_list.len(), 1);
        assert_eq!(dto_list[0].key.as_str(), "job-001");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("0002.yaml"));
        assert!(errors[0].contains("yaml parse error"));
    }

    fn build_test_yaml(key: &str) -> String {
        let dto = build_test_job_dto(100, key);
        serde_yaml::to_string(&JobExportInfo::from(&dto)).unwrap()
    }

    fn build_test_zip(meta_entries: &[(&str, &[u8])], yaml_entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default();
            for (name, content) in meta_entries.iter().chain(yaml_entries.iter()) {
                writer.start_file(*name, options).unwrap();
                writer.write_all(content).unwrap();
            }
            writer.finish().unwrap();
        }
        buf
    }
}
