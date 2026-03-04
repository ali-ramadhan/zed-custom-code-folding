use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct CustomCodeFoldingExtension {
    cached_binary_path: Option<String>,
}

impl CustomCodeFoldingExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // Check user-configured binary path
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree);
        if let Ok(settings) = &settings {
            if let Some(binary) = &settings.binary {
                if let Some(path) = &binary.path {
                    return Ok(path.clone());
                }
            }
        }

        // Check PATH
        if let Some(path) = worktree.which("custom-code-folding-server") {
            return Ok(path);
        }

        // Download from GitHub Releases
        self.download_server_binary(language_server_id)
    }

    fn download_server_binary(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "ali-ramadhan/zed-custom-code-folding",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let target = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "aarch64-apple-darwin",
            (zed::Os::Mac, zed::Architecture::X8664) => "x86_64-apple-darwin",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-pc-windows-msvc",
            _ => return Err("unsupported platform".into()),
        };

        let ext = match platform {
            zed::Os::Windows => "zip",
            _ => "tar.gz",
        };

        let asset_name = format!("custom-code-folding-server-{target}.{ext}");

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {asset_name:?}"))?;

        let version_dir = format!("custom-code-folding-server-{}", release.version);

        let binary_name = match platform {
            zed::Os::Windows => "custom-code-folding-server.exe",
            _ => "custom-code-folding-server",
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                    _ => zed::DownloadedFileType::GzipTar,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            // Clean up old versions
            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory: {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry: {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for CustomCodeFoldingExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: binary_path,
            args: vec!["--stdio".to_string()],
            env: vec![],
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)
                .ok()
                .and_then(|s| s.initialization_options),
        )
    }
}

zed::register_extension!(CustomCodeFoldingExtension);
