use axum::Extension;
use sdkwork_iam_context_service::IamAppContext;

#[derive(Debug, Clone)]
pub(crate) struct BackendRuntimeSubject {
    pub tenant_id: String,
}

pub(crate) fn backend_runtime_subject_from_extension(
    context: Option<Extension<IamAppContext>>,
) -> Result<BackendRuntimeSubject, String> {
    let Some(Extension(context)) = context else {
        return Err("authenticated runtime context is required".to_owned());
    };
    backend_runtime_subject_from_iam(&context)
}

pub(crate) fn backend_runtime_subject_from_iam(
    context: &IamAppContext,
) -> Result<BackendRuntimeSubject, String> {
    let tenant_id = required_context_text(&context.tenant_id, "tenant_id")?;
    let _user_id = required_context_text(&context.user_id, "user_id")?;

    Ok(BackendRuntimeSubject { tenant_id })
}

fn required_context_text(value: &str, field_name: &'static str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!(
            "authenticated runtime context {field_name} is required"
        ));
    }
    Ok(value.to_owned())
}
