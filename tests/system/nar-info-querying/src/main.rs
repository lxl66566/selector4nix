mod assertions;
mod cli;
mod context;
mod fixture;

use anyhow::{Context, Result as AnyhowResult};

use assertions::*;
use context::TestContext;
use fixture::INVALID_HASH;

use crate::fixture::TestFixtures;

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let paths = cli::resolve()?;
    let fixtures = TestFixtures::new();
    let context = TestContext::init(fixtures, &paths).await?;

    single_nar_info_fetch(&context).await?;
    eprintln!("testcase `single_nar_info_fetch`: ok");
    batch_nar_info_fetch(&context).await?;
    eprintln!("testcase `batch_nar_info_fetch`: ok");
    not_found_nar_info(&context).await?;
    eprintln!("testcase `not_found_nar_info`: ok");
    cached_nar_info(&context).await?;
    eprintln!("testcase `cached_nar_info`: ok");
    batch_mixed(&context).await?;
    eprintln!("testcase `batch_mixed`: ok");

    eprintln!("all testcases passed");
    Ok(())
}

async fn single_nar_info_fetch(context: &TestContext) -> AnyhowResult<()> {
    let hash = context.fixtures().populated()[0].hash.as_str();
    let response = fetch_nar_info(context.client(), context.proxy_base_url(), hash).await?;
    assert_nar_info_ok(response, hash)
        .await
        .context("`single_nar_info_fetch` failed")
}

async fn batch_nar_info_fetch(context: &TestContext) -> AnyhowResult<()> {
    let hashes = context.fixtures().valid_hashes();
    let mut tasks = Vec::with_capacity(hashes.len());

    for hash in &hashes {
        let client = context.client().clone();
        let base_url = context.proxy_base_url().to_string();
        let hash = hash.to_string();
        tasks.push(tokio::spawn(async move {
            let response = fetch_nar_info(&client, &base_url, &hash).await?;
            assert_nar_info_ok(response, &hash).await
        }));
    }

    for task in tasks {
        task.await
            .context("batch task panicked")?
            .context("`batch_nar_info_fetch` failed")?;
    }
    Ok(())
}

async fn not_found_nar_info(context: &TestContext) -> AnyhowResult<()> {
    let response = fetch_nar_info(context.client(), context.proxy_base_url(), INVALID_HASH).await?;
    assert_nar_info_not_found(response, INVALID_HASH)
        .await
        .context("`not_found_nar_info` failed")
}

async fn cached_nar_info(context: &TestContext) -> AnyhowResult<()> {
    let hash = context.fixtures().populated()[0].hash.as_str();

    let response1 = fetch_nar_info(context.client(), context.proxy_base_url(), hash).await?;
    assert_nar_info_ok(response1, hash).await?;

    let response2 = fetch_nar_info(context.client(), context.proxy_base_url(), hash).await?;
    assert_nar_info_ok(response2, hash).await?;

    Ok(())
}

async fn batch_mixed(context: &TestContext) -> AnyhowResult<()> {
    let valid_hash = context.fixtures().populated()[0].hash.clone();
    let invalid_hash = INVALID_HASH.to_string();
    let client = context.client().clone();
    let base_url = context.proxy_base_url().to_string();

    let valid_client = client.clone();
    let valid_base_url = base_url.clone();
    let valid_task = tokio::spawn(async move {
        let response = fetch_nar_info(&valid_client, &valid_base_url, &valid_hash).await?;
        assert_nar_info_ok(response, &valid_hash).await
    });

    let invalid_task = tokio::spawn(async move {
        let response = fetch_nar_info(&client, &base_url, &invalid_hash).await?;
        assert_nar_info_not_found(response, &invalid_hash).await
    });

    valid_task
        .await
        .context("valid task panicked")?
        .context("valid hash case failed")?;
    invalid_task
        .await
        .context("invalid task panicked")?
        .context("invalid hash case failed")?;
    Ok(())
}
