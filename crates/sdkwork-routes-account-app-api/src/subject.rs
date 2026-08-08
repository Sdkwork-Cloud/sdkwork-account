use axum::Extension;
use sdkwork_account_service::AccountSummarySnapshot;
use sdkwork_iam_context_service::IamAppContext;

#[derive(Debug, Clone)]
pub(crate) struct AppRuntimeSubject {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub user_id: String,
}

pub(crate) fn app_runtime_subject_from_extension(
    context: Option<Extension<IamAppContext>>,
) -> Result<AppRuntimeSubject, String> {
    let Some(Extension(context)) = context else {
        return Err("authenticated runtime context is required".to_owned());
    };
    app_runtime_subject_from_iam(&context)
}

pub(crate) fn app_runtime_subject_from_iam(
    context: &IamAppContext,
) -> Result<AppRuntimeSubject, String> {
    let tenant_id = required_context_text(&context.tenant_id, "tenant_id")?;
    let user_id = required_context_text(&context.user_id, "user_id")?;
    let organization_id = context
        .organization_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    Ok(AppRuntimeSubject {
        tenant_id,
        organization_id,
        user_id,
    })
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

pub(crate) fn enrich_account_summary_from_iam(
    snapshot: &mut AccountSummarySnapshot,
    context: &IamAppContext,
) {
    if snapshot.name.is_empty() {
        let name = context.display_name.trim();
        if !name.is_empty() {
            snapshot.name = name.to_owned();
        }
    }

    if snapshot.email.is_empty() {
        let email = context.email.trim();
        if !email.is_empty() {
            snapshot.email = email.to_owned();
        }
    }

    if !snapshot.is_verified && context.email_verified {
        snapshot.is_verified = true;
    }

    if snapshot.tier.is_empty() {
        let tier = context
            .standard_role_codes
            .iter()
            .map(|code| code.trim())
            .filter(|code| !code.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        if !tier.is_empty() {
            snapshot.tier = tier;
        }
    }
}

#[cfg(test)]
mod tests {
    use sdkwork_account_service::AccountSummarySnapshot;
    use sdkwork_iam_context_service::{AuthLevel, DeploymentMode, Environment, IamAppContext};

    use super::enrich_account_summary_from_iam;

    fn empty_summary() -> AccountSummarySnapshot {
        AccountSummarySnapshot {
            id: "user-1".to_owned(),
            name: String::new(),
            email: String::new(),
            is_verified: false,
            tier: String::new(),
            organization: String::new(),
            available_points: "0".to_owned(),
            est_days_remaining: 0,
            monthly_points_consumed: "0".to_owned(),
            consumption_by_service: Vec::new(),
        }
    }

    fn base_context() -> IamAppContext {
        IamAppContext::new(
            "tenant-1",
            None,
            "user-1",
            "session-1",
            "app-1",
            Environment::Dev,
            DeploymentMode::Private,
            AuthLevel::Password,
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn enrich_summary_sets_profile_and_tier_from_iam_context() {
        let mut snapshot = empty_summary();
        let mut context = base_context();
        context.display_name = "Ada Lovelace".to_owned();
        context.email = "ada@example.com".to_owned();
        context.email_verified = true;
        context.standard_role_codes = vec!["pro".to_owned(), "billing_admin".to_owned()];

        enrich_account_summary_from_iam(&mut snapshot, &context);

        assert_eq!(snapshot.name, "Ada Lovelace");
        assert_eq!(snapshot.email, "ada@example.com");
        assert!(snapshot.is_verified);
        assert_eq!(snapshot.tier, "pro, billing_admin");
    }

    #[test]
    fn enrich_summary_does_not_overwrite_existing_snapshot_fields() {
        let mut snapshot = empty_summary();
        snapshot.name = "Existing".to_owned();
        snapshot.email = "existing@example.com".to_owned();
        snapshot.tier = "enterprise".to_owned();

        let mut context = base_context();
        context.display_name = "Ignored".to_owned();
        context.email = "ignored@example.com".to_owned();
        context.email_verified = true;
        context.standard_role_codes = vec!["pro".to_owned()];

        enrich_account_summary_from_iam(&mut snapshot, &context);

        assert_eq!(snapshot.name, "Existing");
        assert_eq!(snapshot.email, "existing@example.com");
        assert_eq!(snapshot.tier, "enterprise");
        assert!(snapshot.is_verified);
    }
}
