//! Resend HTTP delivery for deployments that do not use AWS SES.

use anyhow::{Context, Result, bail};

/// Send a rendered HTML email through the Resend API.
pub async fn send_email_resend(
    client: &reqwest::Client,
    api_key: &str,
    base_url: &str,
    from_email: &str,
    to_email: &str,
    subject: &str,
    content: &str,
) -> Result<()> {
    let endpoint = format!("{}/emails", base_url.trim_end_matches('/'));
    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "from": from_email,
            "to": [to_email],
            "subject": subject,
            "html": content,
        }))
        .send()
        .await
        .context("sending via Resend")?;

    let status = response.status();
    let body = response.text().await.context("reading Resend response")?;
    if !status.is_success() {
        bail!("Resend returned {status}: {body}");
    }

    Ok(())
}
