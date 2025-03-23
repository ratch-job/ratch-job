use crate::common::share_data::ShareData;
use crate::metrics::core::MetricsManager;
use crate::metrics::metrics_key::MetricsKey;
use crate::metrics::model::{MetricsItem, MetricsRecord, MetricsRequest};
use crate::openapi::xxljob::model::XxlApiResult;
use actix::Addr;
use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub struct CheckMiddle {
    share_data: Arc<ShareData>,
}

impl CheckMiddle {
    pub fn new(share_data: Arc<ShareData>) -> Self {
        Self { share_data }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CheckMiddle
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckMiddleware {
            service: Arc::new(service),
            share_data: self.share_data.clone(),
        }))
    }
}

#[derive(Clone)]
pub struct CheckMiddleware<S> {
    service: Arc<S>,
    share_data: Arc<ShareData>,
}

impl<S, B> Service<ServiceRequest> for CheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let start = SystemTime::now();
        let path = request.path();
        let xxl_token_check_success =
            if path.starts_with(self.share_data.app_config.xxl_job_prefix_path.as_str()) {
                let xxl_token = if let Some(v) = request.headers().get("XXL-JOB-ACCESS-TOKEN") {
                    v.to_str().unwrap_or_default().to_owned()
                } else {
                    "".to_owned()
                };
                self.share_data
                    .app_config
                    .xxl_default_access_token
                    .is_empty()
                    || self.share_data.app_config.xxl_default_access_token.as_str()
                        == xxl_token.as_str()
            } else {
                true
            };
        let ignore_metrics = false;
        let service = self.service.clone();
        let share_data = self.share_data.clone();
        Box::pin(async move {
            if xxl_token_check_success {
                let res = service.call(request);
                // forwarded responses map to "left" body
                //res.await.map(ServiceResponse::map_into_left_body)
                res.await.map(move |item| {
                    let success = item.response().status().as_u16() < 400;
                    let duration = SystemTime::now()
                        .duration_since(start)
                        .unwrap_or_default()
                        .as_secs_f64();
                    if !ignore_metrics {
                        record_req_metrics(&share_data.metrics_manager, duration, success);
                    }
                    ServiceResponse::map_into_left_body(item)
                })
            } else {
                //没有token
                let response = HttpResponse::Ok()
                    .json(XxlApiResult::<()>::fail(Some(
                        "access-token is error".to_string(),
                    )))
                    .map_into_right_body();
                let (http_request, _pl) = request.into_parts();
                let res = ServiceResponse::new(http_request, response);
                let duration = SystemTime::now()
                    .duration_since(start)
                    .unwrap_or_default()
                    .as_secs_f64();
                record_req_metrics(&share_data.metrics_manager, duration, false);
                Ok(res)
            }
        })
    }
}

fn record_req_metrics(metrics_manager: &Addr<MetricsManager>, duration: f64, _success: bool) {
    metrics_manager.do_send(MetricsRequest::BatchRecord(vec![
        MetricsItem::new(
            MetricsKey::HttpRequestHandleRtHistogram,
            MetricsRecord::HistogramRecord(duration as f32 * 1000f32),
        ),
        MetricsItem::new(
            MetricsKey::HttpRequestTotalCount,
            MetricsRecord::CounterInc(1),
        ),
    ]));
}
