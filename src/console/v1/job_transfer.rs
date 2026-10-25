use crate::common::datetime_utils::now_millis;
use crate::common::get_app_version;
use crate::common::model::{ApiResult, UserSession};
use crate::common::share_data::ShareData;
use crate::console::model::job::{JobExportInfo, JobQueryListRequest, RatchMeta};
use crate::console::v1::ERROR_CODE_SYSTEM_ERROR;
use crate::job::job_index::JobQueryParam;
use crate::job::model::actor_model::{JobManagerReq, JobManagerResult};
use crate::job::model::job::JobInfoDto;
use actix_http::HttpMessage;
use actix_web::http::header::{ContentDisposition, ContentType};
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::io::{Cursor, Write};
use std::sync::Arc;
use zip::write::FileOptions;
use zip::ZipWriter;

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
}
