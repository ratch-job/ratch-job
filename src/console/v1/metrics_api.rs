use crate::common::model::ApiResult;
use crate::common::share_data::ShareData;
use crate::console::model::metrics_model::TimelineQueryRequest;
use crate::grpc::PayloadUtils;
use crate::metrics::model::{MetricsRequest, MetricsResponse};
use crate::metrics::timeline::model::TimelineQueryParam;
use crate::raft::cluster::model::RouterResponse;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

pub async fn query_metrics_timeline(
    app: Data<Arc<ShareData>>,
    web::Query(req): web::Query<TimelineQueryRequest>,
) -> actix_web::Result<impl Responder> {
    let param: TimelineQueryParam = req.into();
    match do_query_metrics_timeline(app, param).await {
        Ok(v) => Ok(v),
        Err(err) => Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_owned(),
            Some(err.to_string()),
        ))),
    }
}

pub async fn query_metrics_timeline_json(
    app: Data<Arc<ShareData>>,
    web::Json(req): web::Json<TimelineQueryRequest>,
) -> actix_web::Result<impl Responder> {
    let param: TimelineQueryParam = req.into();
    match do_query_metrics_timeline(app, param).await {
        Ok(v) => Ok(v),
        Err(err) => Ok(HttpResponse::Ok().json(ApiResult::<()>::error(
            "SYSTEM_ERROR".to_owned(),
            Some(err.to_string()),
        ))),
    }
}

async fn do_query_metrics_timeline(
    app: Data<Arc<ShareData>>,
    param: TimelineQueryParam,
) -> anyhow::Result<HttpResponse> {
    let resp = if param.node_id == 0 || param.node_id == app.app_config.raft_node_id {
        if let MetricsResponse::TimelineResponse(mut resp) = app
            .metrics_manager
            .send(MetricsRequest::TimelineQuery(param))
            .await??
        {
            resp.from_node_id = app.app_config.raft_node_id;
            resp
        } else {
            return Err(anyhow::anyhow!("query timeline error"));
        }
    } else {
        //从其它节点查询
        let node_id = param.node_id;
        let resp = app
            .raft_request_route
            .request_to_target(param.into(), node_id)
            .await?;
        if let RouterResponse::MetricsTimeLineResponse(v) = resp {
            v
        } else {
            return Err(anyhow::anyhow!(
                "query remote timeline error,node:{}",
                node_id,
            ));
        }
    };
    Ok(HttpResponse::Ok().json(ApiResult::success(Some(resp))))
}
