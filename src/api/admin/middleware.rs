use std::future::{Ready, ready};

use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
    web::Data,
};
use futures_util::future::{LocalBoxFuture, ok};
use log::error;
use redis::Commands;

use crate::{AppState, util::cache::get_prefix_key};

pub struct CheckLogin;

impl<S, B> Transform<S, ServiceRequest> for CheckLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckLoginMiddleware { service }))
    }
}
pub struct CheckLoginMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        if cfg!(debug_assertions) {
            // 测试跳过验证
            request.extensions_mut().insert(1i32);
            let res = self.service.call(request);
            return Box::pin(async move {
                // forwarded responses map to "left" body
                res.await.map(ServiceResponse::map_into_left_body)
            });
        }

        let (request, is_login) = check_login(request);
        if is_login {
            let res = self.service.call(request);
            Box::pin(async move {
                // forwarded responses map to "left" body
                res.await.map(ServiceResponse::map_into_left_body)
            })
        } else {
            let response = HttpResponse::Unauthorized().finish().map_into_right_body();
            Box::pin(ok(request.into_response(response)))
        }
    }
}

fn check_login(request: ServiceRequest) -> (ServiceRequest, bool) {
    let path = request.path().to_string();
    if path == "/api/admin/login" {
        return (request, true);
    }

    // 默认值
    let default_val = HeaderValue::from_str("").unwrap();
    // 永远不会是None
    let app_state = request.app_data::<Data<AppState>>().unwrap();

    let token = request
        .headers()
        .get("Authorization")
        .unwrap_or(&default_val)
        .to_str()
        .unwrap_or("");

    // 读取登录状态
    let mut redis = match app_state.redis.get_connection() {
        Ok(redis) => redis,
        Err(err) => {
            error!("{}", err);
            return (request, false);
        }
    };

    let token_key = get_prefix_key(token);

    let user_id: Option<i32> = match redis.get(token_key) {
        Ok(user_id) => user_id,
        Err(err) => {
            error!("{}", err);
            return (request, false);
        }
    };

    if let Some(user_id) = user_id {
        request.extensions_mut().insert(user_id);
        (request, true)
    } else {
        (request, false)
    }
}
