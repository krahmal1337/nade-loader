use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use serde::{Deserialize, Serialize};
use crate::error::LauncherError;
use crate::steam;
const LOCAL_BIN_DIR: &str = "local";

const GITHUB_REPO_NL: &str = "krahmal1337/NeverNade";
const GITHUB_REPO_SKEET: &str = "krahmal1337/otherstuff";

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    prerelease: bool,
    draft: bool,
    published_at: Option<String>,
    updated_at: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Serialize)]
pub struct LauncherGitMetadata {
    pub releases: Vec<LauncherVersion>,
    pub nightlies: Vec<LauncherVersion>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LauncherVersion {
    pub tag: String,
    pub name: String,
    pub changelog: String,
    pub updated_at: String,
    pub url: String,
    pub assets: Vec<LauncherAsset>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LauncherAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

pub fn bins_dir() -> Result<PathBuf, LauncherError> {
    let p = Path::new("C:\\nevernade\\builds");
    eprintln!("[loader] bins_dir: {}", p.display());
    Ok(p.to_path_buf())
}

fn version_has_files(dir: &Path, dll_name: &str) -> bool {
    dir.join(dll_name).exists()
}

fn is_skeet(tag: &str) -> bool {
    tag.to_lowercase().contains("skeet")
}

fn is_nightly(tag: &str) -> bool {
    let lower = tag.to_lowercase();
    lower.contains("test") || lower.contains("nightly") || lower.contains("beta") || lower.contains("alpha")
}

fn make_version_from_tag(tag: String, name: String, changelog: String, updated_at: String, url: String, assets: Vec<GithubAsset>) -> LauncherVersion {
    LauncherVersion {
        tag,
        name,
        changelog,
        updated_at,
        url,
        assets: assets.into_iter().map(|a| LauncherAsset {
            name: a.name,
            url: a.browser_download_url,
            size: a.size,
        }).collect(),
    }
}

fn github_client() -> Result<reqwest::Client, LauncherError> {
    reqwest::Client::builder()
        .user_agent("NeverloseTauriOfficial")
        .build()
        .map_err(|error| LauncherError::Reqwest(format!("failed to create GitHub client: {error}")))
}

async fn fetch_github_releases(repo: &str) -> Result<(Vec<LauncherVersion>, Vec<LauncherVersion>), LauncherError> {
    let url = format!("https://api.github.com/repos/{repo}/releases");
    let client = github_client()?;
    let releases = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<GithubRelease>>()
        .await?;

    let mut stable = Vec::new();
    let mut nightly = Vec::new();

    let is_nl_repo = repo == GITHUB_REPO_NL;
    for release in releases.into_iter().filter(|r| !r.draft && (!is_nl_repo || !is_skeet(&r.tag_name))) {
        let tag_name = release.tag_name.clone();
        let version = make_version_from_tag(
            release.tag_name,
            release.name.unwrap_or_else(|| tag_name),
            release.body.unwrap_or_default(),
            release.published_at.unwrap_or(release.updated_at),
            release.html_url,
            release.assets,
        );

        if release.prerelease || is_nightly(&version.tag) {
            nightly.push(version);
        } else {
            stable.push(version);
        }
    }

    Ok((stable, nightly))
}

fn scan_local_versions() -> (Vec<LauncherVersion>, Vec<LauncherVersion>) {
    let mut releases = Vec::new();
    let mut nightlies = Vec::new();

    let bins = match bins_dir() {
        Ok(d) => d,
        Err(_) => return (releases, nightlies),
    };

    if !bins.exists() {
        return (releases, nightlies);
    }

    let mut dirs: Vec<String> = Vec::new();
    if let Ok(mut entries) = std::fs::read_dir(&bins) {
        while let Some(Ok(entry)) = entries.next() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name != LOCAL_BIN_DIR && !is_skeet(name) && (version_has_files(&entry.path(), "neverlose.dll") || version_has_files(&entry.path(), "skeet.dll")) {
                            dirs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    dirs.sort_by(|a, b| b.cmp(a));

    for tag in dirs {
        let version = LauncherVersion {
            tag: tag.clone(),
            name: tag.clone(),
            changelog: String::new(),
            updated_at: String::new(),
            url: String::new(),
            assets: vec![
                LauncherAsset { name: "neverlose.dll".to_string(), url: String::new(), size: 0 },
                LauncherAsset { name: "injector.exe".to_string(), url: String::new(), size: 0 },
            ],
        };

        if is_nightly(&tag) {
            nightlies.push(version);
        } else {
            releases.push(version);
        }
    }

    (releases, nightlies)
}

pub async fn load_git_metadata(product: &str) -> Result<LauncherGitMetadata, LauncherError> {
    let repo = if product == "skeet" { GITHUB_REPO_SKEET } else { GITHUB_REPO_NL };
    eprintln!("[loader] load_git_metadata: fetching releases from {repo}");
    match fetch_github_releases(repo).await {
        Ok((releases, nightlies)) => {
            eprintln!("[loader] GitHub OK: {} stable, {} nightly", releases.len(), nightlies.len());
            return Ok(LauncherGitMetadata { releases, nightlies });
        }
        Err(error) => {
            eprintln!("[loader] GitHub fetch failed, falling back to local: {error}");
        }
    }

    let (releases, nightlies) = scan_local_versions();
    eprintln!("[loader] local fallback: {} stable, {} nightly", releases.len(), nightlies.len());
    Ok(LauncherGitMetadata { releases, nightlies })
}

async fn download_github_asset(client: &reqwest::Client, repo: &str, tag: &str, asset_name: &str, install_dir: &Path) -> Result<(), LauncherError> {
    eprintln!("[loader] download_github_asset: fetching {tag} release metadata from {repo}");
    let release_url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let release = client
        .get(&release_url)
        .send()
        .await?
        .error_for_status()?
        .json::<GithubRelease>()
        .await?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(asset_name))
        .ok_or_else(|| LauncherError::Validation(format!("release {tag} missing {asset_name}")))?;

    let target = install_dir.join(asset_name);
    if target.exists() {
        eprintln!("[loader] {} already exists, skipping download", asset_name);
        return Ok(());
    }

    eprintln!("[loader] downloading {} ({} bytes)...", asset_name, asset.size);
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    eprintln!("[loader] writing {} to disk ({} bytes)", asset_name, bytes.len());
    tokio::fs::write(&target, bytes)
        .await
        .map_err(|error| LauncherError::Io(format!("failed to write {}: {error}", target.display())))?;

    Ok(())
}

async fn wait_for_csgo_process(timeout_secs: u64) -> Result<u32, LauncherError> {
    eprintln!("[loader] waiting for csgo window (Valve001), timeout={timeout_secs}s");
    let start = std::time::Instant::now();
    loop {
        if let Some(pid) = steam::find_csgo_pid() {
            eprintln!("[loader] csgo process found, PID={pid}");
            return Ok(pid);
        }
        let elapsed = start.elapsed().as_secs();
        if elapsed >= timeout_secs {
            eprintln!("[loader] timed out waiting for csgo process");
            return Err(LauncherError::System("timed out waiting for csgo process".to_string()));
        }
        if elapsed > 0 && elapsed % 5 == 0 {
            eprintln!("[loader] still waiting for csgo... ({elapsed}s)");
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

pub async fn prepare_version(tag: String, dll_name: String) -> Result<String, LauncherError> {
    eprintln!("[loader] prepare_version: tag={tag}, dll={dll_name}");

    if tag.trim().is_empty() || tag == "Unavailable" {
        return Err(LauncherError::Validation("no release version is selected".to_string()));
    }

    let install_dir = bins_dir()?.join(&tag);
    eprintln!("[loader] install_dir: {}", install_dir.display());

    if !install_dir.exists() || !install_dir.join(&dll_name).exists() {
        eprintln!("[loader] files not found locally, downloading from GitHub");
        tokio::fs::create_dir_all(&install_dir)
            .await
            .map_err(|error| LauncherError::Io(format!("failed to create {}: {error}", install_dir.display())))?;

        #[cfg(windows)]
        let _ = kill_background_processes();

        let client = github_client()?;
        let repo = if dll_name == "skeet.dll" { GITHUB_REPO_SKEET } else { GITHUB_REPO_NL };
        download_github_asset(&client, repo, &tag, &dll_name, &install_dir).await?;
    } else {
        eprintln!("[loader] DLL already cached locally");
    }

    let dll_path = install_dir.join(&dll_name);
    let dll_path_str = dll_path.to_str()
        .ok_or_else(|| LauncherError::System("invalid dll path".to_string()))?
        .to_string();
    eprintln!("[loader] DLL ready: {dll_path_str}");
    Ok(dll_path_str)
}

pub async fn wait_and_inject(dll_path: String, dll_name: String) -> Result<(), LauncherError> {
    eprintln!("[loader] wait_and_inject: dll_path={dll_path}, dll_name={dll_name}");
    let pid = wait_for_csgo_process(30).await?;
    eprintln!("[loader] csgo found, PID={pid}");

    if let Some(exe_path) = steam::get_process_image_path(pid) {
        eprintln!("[loader] csgo exe path: {exe_path}");
        if let Some(game_dir) = std::path::Path::new(&exe_path).parent() {
            steam::save_csgo_path_registry(&game_dir.to_string_lossy());
            let nl_cloud = game_dir.join("nl_cloud");
            if !nl_cloud.join("state.json").exists() || !nl_cloud.join("avatar.png").exists() {
                eprintln!("[loader] creating nl_cloud files in {}", nl_cloud.display());
                let _ = std::fs::create_dir_all(&nl_cloud);
                let _ = std::fs::write(nl_cloud.join("state.json"), crate::theme::DEFAULT_STATE_JSON);
                let _ = std::fs::write(nl_cloud.join("avatar.png"), crate::theme::DEFAULT_AVATAR);
            }
        }
    }

    eprintln!("[loader] injecting DLL into PID {pid}");
    steam::inject_dll(pid, &dll_path, dll_name == "skeet.dll")?;
    eprintln!("[loader] injection successful");
    Ok(())
}

pub fn kill_background_processes() -> Result<(), LauncherError> {
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/im", "injector.exe", "/f"])
            .creation_flags(0x08000000)
            .spawn()
            .map(|_| ())
            .map_err(|error| LauncherError::System(format!("failed to kill injector: {error}")))?;
    }
    Ok(())
}
