use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::{
    PageInfo, PageMode, SdkWorkApiResponse, SdkWorkPageData, SdkWorkProblemDetail,
    SdkWorkProblemRouting, SdkWorkResourceData, SdkWorkResultCode,
};
use sdkwork_web_core::WebRequestContext;

pub fn resolve_trace_id(context: Option<&WebRequestContext>) -> String {
    context
        .map(WebRequestContext::resolved_trace_id)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| sdkwork_utils_rust::uuid())
}

fn problem_routing(context: Option<&WebRequestContext>) -> SdkWorkProblemRouting {
    context
        .map(WebRequestContext::problem_routing)
        .unwrap_or_default()
}

fn problem_for_context(
    context: Option<&WebRequestContext>,
    status: StatusCode,
    result_code: SdkWorkResultCode,
    detail: impl Into<String>,
) -> Response {
    let trace_id = resolve_trace_id(context);
    let problem = SdkWorkProblemDetail::platform_enriched(
        result_code,
        detail,
        trace_id.clone(),
        problem_routing(context),
    );
    attach_trace_header((status, Json(problem)).into_response(), &trace_id)
}

pub fn success_item<T: serde::Serialize>(
    context: Option<&WebRequestContext>,
    item: T,
) -> Response {
    let trace_id = resolve_trace_id(context);
    let envelope = SdkWorkApiResponse::success(SdkWorkResourceData { item }, trace_id.clone());
    attach_trace_header((StatusCode::OK, Json(envelope)).into_response(), &trace_id)
}

pub fn success_items<T: serde::Serialize>(
    context: Option<&WebRequestContext>,
    items: Vec<T>,
    page: i64,
    page_size: i64,
) -> Response {
    let trace_id = resolve_trace_id(context);
    let envelope = SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(page as i32),
                page_size: Some(page_size as i32),
                total_items: None,
                total_pages: None,
                next_cursor: None,
                has_more: None,
            },
        },
        trace_id.clone(),
    );
    attach_trace_header((StatusCode::OK, Json(envelope)).into_response(), &trace_id)
}

pub fn map_service_error(
    context: Option<&WebRequestContext>,
    error: CommerceServiceError,
) -> Response {
    let (status, result_code, detail) = match error.code() {
        "validation" => (
            StatusCode::BAD_REQUEST,
            SdkWorkResultCode::ValidationError,
            error.message().to_string(),
        ),
        "not-found" => (
            StatusCode::NOT_FOUND,
            SdkWorkResultCode::NotFound,
            error.message().to_string(),
        ),
        "conflict" => (
            StatusCode::CONFLICT,
            SdkWorkResultCode::Conflict,
            error.message().to_string(),
        ),
        "unauthorized" => (
            StatusCode::UNAUTHORIZED,
            SdkWorkResultCode::AuthenticationRequired,
            error.message().to_string(),
        ),
        "invalid-state" => (
            StatusCode::UNPROCESSABLE_ENTITY,
            SdkWorkResultCode::UnprocessableEntity,
            error.message().to_string(),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            SdkWorkResultCode::InternalError,
            error.message().to_string(),
        ),
    };
    problem_for_context(context, status, result_code, detail)
}

pub fn unauthorized(context: Option<&WebRequestContext>, detail: impl Into<String>) -> Response {
    problem_for_context(
        context,
        StatusCode::UNAUTHORIZED,
        SdkWorkResultCode::AuthenticationRequired,
        detail,
    )
}

pub fn validation(context: Option<&WebRequestContext>, detail: impl Into<String>) -> Response {
    problem_for_context(
        context,
        StatusCode::BAD_REQUEST,
        SdkWorkResultCode::ValidationError,
        detail,
    )
}

pub fn not_found(context: Option<&WebRequestContext>, detail: impl Into<String>) -> Response {
    problem_for_context(
        context,
        StatusCode::NOT_FOUND,
        SdkWorkResultCode::NotFound,
        detail,
    )
}

fn attach_trace_header(response: Response, trace_id: &str) -> Response {
    let mut response = response;
    if let Ok(value) = HeaderValue::from_str(trace_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-sdkwork-trace-id"),
            value,
        );
    }
    response
}
