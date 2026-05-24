use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result as AnyhowResult, bail};
use reqwest::Client;
use tempfile::TempDir;

use crate::cli::ResolvedPaths;
use crate::fixture::TestFixtures;

const READINESS_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

struct SubprocessGuard {
    child: Child,
}

impl Drop for SubprocessGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct TestContext {
    _tempdir: TempDir,
    _nix_serve: SubprocessGuard,
    _selector4nix: SubprocessGuard,
    proxy_base_url: String,
    client: Client,
    fixtures: TestFixtures,
}

impl TestContext {
    pub async fn init(mut fixtures: TestFixtures, paths: &ResolvedPaths) -> AnyhowResult<Self> {
        let tempdir = TempDir::new().context("failed to create temp directory")?;
        let cache_dir = tempdir.path().join("cache");
        std::fs::create_dir(&cache_dir).context("failed to create cache directory")?;

        populate_cache(&mut fixtures, &cache_dir, &paths.nix)?;

        let upstream_port = find_free_port();
        let proxy_port = find_free_port();

        let nix_serve = start_nix_serve(&paths.nix_serve, &cache_dir, upstream_port)?;
        let selector4nix =
            start_selector4nix(&paths.selector4nix, &tempdir, upstream_port, proxy_port)?;

        let proxy_base_url = format!("http://127.0.0.1:{proxy_port}");
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("failed to build HTTP client")?;

        let context = Self {
            _tempdir: tempdir,
            _nix_serve: nix_serve,
            _selector4nix: selector4nix,
            proxy_base_url,
            client,
            fixtures,
        };

        context.wait_ready().await?;
        Ok(context)
    }

    pub fn proxy_base_url(&self) -> &str {
        &self.proxy_base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn fixtures(&self) -> &TestFixtures {
        &self.fixtures
    }

    async fn wait_ready(&self) -> AnyhowResult<()> {
        let start = Instant::now();
        let url = format!("{}/nix-cache-info", self.proxy_base_url);
        loop {
            match self.client.get(&url).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                _ => {
                    if start.elapsed() > READINESS_TIMEOUT {
                        bail!("`selector4nix` did not become ready within {READINESS_TIMEOUT:?}");
                    }
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }
        }
    }
}

const TEST_FILES: &[(&str, &[u8])] = &[
    ("hello.txt", b"Hello, system test!"),
    ("empty.txt", b""),
    ("multiline.txt", b"line one\nline two\nline three\n"),
    ("binary.dat", &[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]),
    ("unicode.txt", "系统测试 🦀\n".as_bytes()),
];

fn populate_cache(
    fixtures: &mut TestFixtures,
    cache_dir: &Path,
    nix_bin: &Path,
) -> AnyhowResult<()> {
    for (name, content) in TEST_FILES {
        let file_path = cache_dir.join(format!("input-{name}"));
        std::fs::write(&file_path, content)
            .with_context(|| format!("failed to write test file `{name}`"))?;

        let store_uri = format!("file://{}?compression=none", cache_dir.display());
        let output = Command::new(nix_bin)
            .args(["store", "add-file", "--store", &store_uri])
            .arg(&file_path)
            .env("NIX_CONFIG", "experimental-features = nix-command")
            .output()
            .with_context(|| format!("failed to spawn `nix store add-file` for `{name}`"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("`nix store add-file` failed for {name}: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let store_path = stdout.trim();
        let hash = store_path
            .strip_prefix("/nix/store/")
            .and_then(|s| s.split_once('-'))
            .map(|(h, _)| h)
            .context(format!(
                "unexpected `nix store add-file` output for {name}: {store_path}"
            ))?;

        fixtures.add_populated(hash.to_string());
    }
    Ok(())
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    listener
        .local_addr()
        .expect("failed to get local address")
        .port()
}

fn start_nix_serve(
    nix_serve_bin: &Path,
    cache_dir: &Path,
    port: u16,
) -> AnyhowResult<SubprocessGuard> {
    let store_uri = format!("file://{}", cache_dir.display());
    let child = Command::new(nix_serve_bin)
        .args(["--store", &store_uri, "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn `nix-serve`")?;

    Ok(SubprocessGuard { child })
}

fn start_selector4nix(
    selector4nix_bin: &Path,
    tempdir: &TempDir,
    upstream_port: u16,
    proxy_port: u16,
) -> AnyhowResult<SubprocessGuard> {
    let config_content = format!(
        r#"[server]
ip = "127.0.0.1"
port = {proxy_port}

[network]
periodic_probing = false

[[substituters]]
url = "http://127.0.0.1:{upstream_port}/"
"#,
    );
    let config_path = tempdir.path().join("selector4nix.toml");
    std::fs::write(&config_path, &config_content).context("failed to write config")?;

    let child = Command::new(selector4nix_bin)
        .args(["--config-file", &config_path.to_string_lossy()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn `selector4nix`")?;

    Ok(SubprocessGuard { child })
}
