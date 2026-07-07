use axum::Extension;
use sdkwork_iam_context_service::{IamAppContext, LoginScope};

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

/// Ensures backend write commands target an owner the caller may act on.
///
/// Organization-scoped integrators (payment/order callbacks) may write for any user
/// within the authenticated tenant. Tenant-scoped tokens must match `owner_user_id`.
pub(crate) fn ensure_backend_owner_user_allowed(
    context: &IamAppContext,
    owner_user_id: &str,
) -> Result<(), String> {
    let owner_user_id = owner_user_id.trim();
    if owner_user_id.is_empty() {
        return Err("owner_user_id is required".to_owned());
    }

    if context.login_scope == LoginScope::Tenant && context.user_id.trim() != owner_user_id {
        return Err(
            "owner_user_id must match authenticated user for tenant-scoped backend calls"
                .to_owned(),
        );
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use sdkwork_iam_context_service::{
        AuthLevel, DeploymentMode, Environment, IamAppContext, LoginScope,
    };

    use super::ensure_backend_owner_user_allowed;

    fn context_with_scope(login_scope: LoginScope, user_id: &str) -> IamAppContext {
        let mut context = IamAppContext::new(
            "tenant-1",
            None,
            user_id,
            "session-1",
            "app-1",
            Environment::Dev,
            DeploymentMode::Private,
            AuthLevel::Password,
            Vec::new(),
            Vec::new(),
        );
        context.login_scope = login_scope;
        context
    }

    #[test]
    fn organization_scope_allows_any_owner_in_tenant() {
        let context = context_with_scope(LoginScope::Organization, "service-bot");
        ensure_backend_owner_user_allowed(&context, "user-42").expect("org scope allows");
    }

    #[test]
    fn tenant_scope_requires_matching_owner() {
        let context = context_with_scope(LoginScope::Tenant, "user-42");
        ensure_backend_owner_user_allowed(&context, "user-42").expect("same user ok");
        let error =
            ensure_backend_owner_user_allowed(&context, "user-99").expect_err("cross-user blocked");
        assert!(error.contains("owner_user_id"));
    }
}
