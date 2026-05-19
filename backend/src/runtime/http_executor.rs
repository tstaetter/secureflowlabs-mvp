use reqwest::Client;

pub struct HttpExecutor {
    client: Client,
}

#[async_trait::async_trait]
impl Executor for HttpExecutor {
    async fn execute(
        &self,
        plan: ExecutionPlan,
        context: ExecutionContext,
    ) -> anyhow::Result<ExecutionResult> {
        validate_request(&plan)?;

        enforce_safety(&plan)?;

        let request = build_request(&self.client, &plan, &context).await?;

        let response = execute_with_retry(&self.client, request, &plan.retry).await?;

        normalize_response(response).await
    }
}

pub async fn build_request(
    client: &Client,
    plan: &ExecutionPlan,
    context: &ExecutionContext,
) -> anyhow::Result<reqwest::RequestBuilder> {
    let mut req = client.request(plan.request.method.into(), &plan.request.url);

    req = apply_auth(req, &plan.auth, context).await?;

    for header in &plan.request.headers {
        req = req.header(&header.name, &header.value);
    }

    Ok(req)
}

pub async fn execute_with_retry(
    client: &Client,
    request: reqwest::RequestBuilder,
    retry: &RetryPolicy,
) -> anyhow::Result<reqwest::Response> {
    let mut attempts = 0;

    loop {
        let cloned = request
            .try_clone()
            .ok_or_else(|| anyhow::anyhow!("clone failed"))?;

        let result = cloned.send().await;

        match result {
            Ok(resp) if !retry.retry_on_status.contains(&resp.status().as_u16()) => {
                return Ok(resp);
            }

            _ => {
                attempts += 1;

                if attempts >= retry.max_retries {
                    anyhow::bail!("retry limit exceeded");
                }

                tokio::time::sleep(std::time::Duration::from_millis(retry.backoff_ms)).await;
            }
        }
    }
}
