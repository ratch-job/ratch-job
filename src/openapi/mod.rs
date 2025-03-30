pub mod metrics;
pub mod middle;
pub mod v1;
pub mod xxljob;

use crate::openapi::metrics::metrics_config;
use crate::openapi::v1::v1_api_config;
use actix_web::web::ServiceConfig;

pub fn openapi_config(config: &mut ServiceConfig) {
    metrics_config(config);
    v1_api_config(config);
}
