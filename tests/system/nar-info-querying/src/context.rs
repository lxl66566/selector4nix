use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result as AnyhowResult, bail};
use reqwest::Client;
use selector4nix_system_test_common::nix_serve::NixServeInstance;
use selector4nix_system_test_common::selector4nix::Selector4NixInstance;
use tempfile::TempDir;
use url::Url;

use crate::cli::ResolvedPaths;
use crate::fixture::TestFixtures;

pub struct SharedContext {
    _tempdir: TempDir,
    nix_serve: NixServeInstance,
    client: Client,
    fixtures: TestFixtures,
    selector4nix_bin: PathBuf,
}

impl SharedContext {
    pub async fn init(mut fixtures: TestFixtures, paths: &ResolvedPaths) -> AnyhowResult<Self> {
        let tempdir = TempDir::new().context("failed to create temp directory")?;
        let cache_dir = tempdir.path().join("cache");
        std::fs::create_dir(&cache_dir).context("failed to create cache directory")?;

        populate_cache(&mut fixtures, &cache_dir, &paths.nix)?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("failed to build HTTP client")?;

        let nix_serve =
            NixServeInstance::start(&paths.nix_serve, &cache_dir, client.clone()).await?;

        Ok(Self {
            _tempdir: tempdir,
            nix_serve,
            client,
            fixtures,
            selector4nix_bin: paths.selector4nix.clone(),
        })
    }

    pub async fn start_proxy(&self) -> AnyhowResult<Selector4NixInstance> {
        let upstream_url =
            Url::parse(&format!("http://127.0.0.1:{}/", self.nix_serve.port())).unwrap();
        Selector4NixInstance::builder(self.selector4nix_bin.clone(), self.client.clone())
            .substituter(upstream_url)
            .start()
            .await
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn fixtures(&self) -> &TestFixtures {
        &self.fixtures
    }
}

fn populate_cache(
    fixtures: &mut TestFixtures,
    cache_dir: &Path,
    nix_bin: &Path,
) -> AnyhowResult<()> {
    let contents: Vec<Vec<u8>> = fixtures.contents().to_vec();
    for (i, content) in contents.into_iter().enumerate() {
        let file_path = cache_dir.join(format!("input-{i}"));
        std::fs::write(&file_path, content)
            .with_context(|| format!("failed to write test file `{i}`"))?;

        let store_uri = format!("file://{}?compression=none", cache_dir.display());
        let output = Command::new(nix_bin)
            .args(["store", "add-file", "--store", &store_uri])
            .arg(&file_path)
            .env("NIX_CONFIG", "experimental-features = nix-command")
            .output()
            .with_context(|| format!("failed to spawn `nix store add-file` for file `{i}`"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("`nix store add-file` failed for file `{i}`: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let store_path = stdout.trim();
        let hash = store_path
            .strip_prefix("/nix/store/")
            .and_then(|s| s.split_once('-'))
            .map(|(h, _)| h)
            .context(format!(
                "unexpected `nix store add-file` output for file `{i}`: {store_path}"
            ))?;

        fixtures.add_populated(hash.to_string());
    }
    Ok(())
}
