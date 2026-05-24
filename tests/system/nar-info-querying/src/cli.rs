use std::path::PathBuf;

use anyhow::{Context, Result as AnyhowResult, bail};
use clap::Parser;

#[derive(Parser)]
#[command(name = "selector4nix-system-test-nar-info-querying")]
struct Cli {
    #[arg(long = "selector4nix", env = "SELECTOR4NIX_BIN")]
    selector4nix: Option<PathBuf>,

    #[arg(long = "nix", env = "NIX_BIN")]
    nix: Option<PathBuf>,

    #[arg(long = "nix-serve", env = "NIX_SERVE_BIN")]
    nix_serve: Option<PathBuf>,
}

pub struct ResolvedPaths {
    pub selector4nix: PathBuf,
    pub nix: PathBuf,
    pub nix_serve: PathBuf,
}

pub fn resolve() -> AnyhowResult<ResolvedPaths> {
    let cli = Cli::parse();

    let selector4nix = resolve_binary(cli.selector4nix, "selector4nix")
        .context("failed to resolve selector4nix binary")?;
    let nix = resolve_binary(cli.nix, "nix").context("failed to resolve nix binary")?;
    let nix_serve =
        resolve_binary(cli.nix_serve, "nix-serve").context("failed to resolve nix-serve binary")?;

    Ok(ResolvedPaths {
        selector4nix,
        nix,
        nix_serve,
    })
}

fn resolve_binary(explicit: Option<PathBuf>, name: &str) -> AnyhowResult<PathBuf> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(path);
        }
        bail!(
            "explicitly specified {name} binary not found: {}",
            path.display()
        );
    }
    which::which(name).with_context(|| {
        format!(
            "{name} not found on PATH; specify --{name} or set {} env var",
            env_var_for(name)
        )
    })
}

fn env_var_for(name: &str) -> &'static str {
    match name {
        "selector4nix" => "SELECTOR4NIX_BIN",
        "nix" => "NIX_BIN",
        "nix-serve" => "NIX_SERVE_BIN",
        _ => "UNKNOWN",
    }
}
