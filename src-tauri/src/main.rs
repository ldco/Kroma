use std::net::SocketAddr;
use std::path::PathBuf;

use kroma_backend_core::api::server::serve;
use kroma_backend_core::db::projects::{ProjectsStore, RotateSecretsInput};
use kroma_backend_core::db::{resolve_backend_config, DatabaseBackendConfig};
use kroma_backend_core::pipeline::config_validation::{
    validate_pipeline_config_stack, PipelineConfigValidationRequest,
};
use kroma_backend_core::pipeline::runtime::default_app_root_from_manifest_dir;
use serde_json::json;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let cli_args = std::env::args().skip(1).collect::<Vec<_>>();
    if matches!(
        cli_args.first().map(String::as_str),
        Some("validate-pipeline-config")
    ) {
        run_validate_pipeline_config_cli(cli_args.into_iter().skip(1).collect::<Vec<_>>())?;
        return Ok(());
    }
    if matches!(
        cli_args.first().map(String::as_str),
        Some("secrets-rotation-status")
    ) {
        run_secrets_rotation_status_cli(cli_args.into_iter().skip(1).collect::<Vec<_>>())?;
        return Ok(());
    }
    if matches!(cli_args.first().map(String::as_str), Some("secrets-rotate")) {
        run_secrets_rotate_cli(cli_args.into_iter().skip(1).collect::<Vec<_>>())?;
        return Ok(());
    }

    let bind =
        std::env::var("KROMA_BACKEND_BIND").unwrap_or_else(|_| String::from("127.0.0.1:8788"));
    let addr: SocketAddr = bind.parse()?;

    serve(addr).await?;
    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
}

fn run_validate_pipeline_config_cli(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print_validate_pipeline_config_usage();
        return Ok(());
    }

    let mut app_root = default_app_root_from_manifest_dir();
    let mut project_root = None::<PathBuf>;
    let mut app_settings_path = None::<String>;
    let mut project_settings_path = None::<String>;
    let mut manifest_path_override = None::<String>;
    let mut postprocess_config_path_override = None::<String>;

    let mut i = 0usize;
    while i < args.len() {
        let flag = args[i].as_str();
        let needs_value = |idx: usize| -> Result<String, Box<dyn std::error::Error>> {
            let Some(value) = args.get(idx + 1) else {
                return Err(std::io::Error::other(format!("Missing value for {flag}")).into());
            };
            Ok(value.clone())
        };

        match flag {
            "--app-root" => {
                app_root = PathBuf::from(needs_value(i)?);
                i += 2;
            }
            "--project-root" => {
                project_root = Some(PathBuf::from(needs_value(i)?));
                i += 2;
            }
            "--app-settings" => {
                app_settings_path = Some(needs_value(i)?);
                i += 2;
            }
            "--project-settings" => {
                project_settings_path = Some(needs_value(i)?);
                i += 2;
            }
            "--manifest" => {
                manifest_path_override = Some(needs_value(i)?);
                i += 2;
            }
            "--postprocess-config" => {
                postprocess_config_path_override = Some(needs_value(i)?);
                i += 2;
            }
            unknown => {
                return Err(std::io::Error::other(format!(
                    "Unknown argument: {unknown}\n\nUse --help for usage."
                ))
                .into());
            }
        }
    }

    let summary = validate_pipeline_config_stack(&PipelineConfigValidationRequest {
        app_root,
        project_root,
        app_settings_path,
        project_settings_path,
        manifest_path_override,
        postprocess_config_path_override,
    })?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecretsRotationStatusCliArgs {
    project_slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecretsRotateCliArgs {
    project_slug: String,
    from_key_ref: Option<String>,
    force: bool,
}

fn parse_secrets_rotation_status_cli_args(
    args: &[String],
) -> Result<SecretsRotationStatusCliArgs, Box<dyn std::error::Error>> {
    let mut project_slug = None::<String>;
    let mut i = 0usize;
    while i < args.len() {
        let flag = args[i].as_str();
        let needs_value = |idx: usize| -> Result<String, Box<dyn std::error::Error>> {
            let Some(value) = args.get(idx + 1) else {
                return Err(std::io::Error::other(format!("Missing value for {flag}")).into());
            };
            Ok(value.clone())
        };

        match flag {
            "--project-slug" => {
                project_slug = Some(needs_value(i)?);
                i += 2;
            }
            unknown => {
                return Err(std::io::Error::other(format!(
                    "Unknown argument: {unknown}\n\nUse --help for usage."
                ))
                .into());
            }
        }
    }

    let project_slug = project_slug
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| std::io::Error::other("Missing required --project-slug"))?;
    Ok(SecretsRotationStatusCliArgs { project_slug })
}

fn parse_secrets_rotate_cli_args(
    args: &[String],
) -> Result<SecretsRotateCliArgs, Box<dyn std::error::Error>> {
    let mut project_slug = None::<String>;
    let mut from_key_ref = None::<String>;
    let mut force = false;
    let mut i = 0usize;
    while i < args.len() {
        let flag = args[i].as_str();
        let needs_value = |idx: usize| -> Result<String, Box<dyn std::error::Error>> {
            let Some(value) = args.get(idx + 1) else {
                return Err(std::io::Error::other(format!("Missing value for {flag}")).into());
            };
            Ok(value.clone())
        };

        match flag {
            "--project-slug" => {
                project_slug = Some(needs_value(i)?);
                i += 2;
            }
            "--from-key-ref" => {
                from_key_ref = Some(needs_value(i)?);
                i += 2;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            unknown => {
                return Err(std::io::Error::other(format!(
                    "Unknown argument: {unknown}\n\nUse --help for usage."
                ))
                .into());
            }
        }
    }

    let project_slug = project_slug
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| std::io::Error::other("Missing required --project-slug"))?;
    let from_key_ref = from_key_ref
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    Ok(SecretsRotateCliArgs {
        project_slug,
        from_key_ref,
        force,
    })
}

fn open_projects_store_for_cli() -> Result<ProjectsStore, Box<dyn std::error::Error>> {
    let repo_root = default_app_root_from_manifest_dir();
    let db_path = match resolve_backend_config(repo_root.as_path()) {
        DatabaseBackendConfig::Sqlite(sqlite) => sqlite.app_db_path,
        DatabaseBackendConfig::Postgres(pg) => {
            return Err(std::io::Error::other(format!(
                "PostgreSQL backend is not implemented for CLI operations yet (KROMA_BACKEND_DB_URL={})",
                pg.database_url
            ))
            .into());
        }
    };
    let store = ProjectsStore::new(db_path, repo_root);
    store.initialize()?;
    Ok(store)
}

fn run_secrets_rotation_status_cli(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print_secrets_rotation_status_usage();
        return Ok(());
    }
    let parsed = parse_secrets_rotation_status_cli_args(args.as_slice())?;
    let store = open_projects_store_for_cli()?;
    let status = store.get_project_secret_encryption_status(parsed.project_slug.as_str())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "project_slug": parsed.project_slug,
            "status": status
        }))?
    );
    Ok(())
}

fn run_secrets_rotate_cli(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        print_secrets_rotate_usage();
        return Ok(());
    }
    let parsed = parse_secrets_rotate_cli_args(args.as_slice())?;
    let store = open_projects_store_for_cli()?;
    let rotation = store.rotate_project_secrets(
        parsed.project_slug.as_str(),
        RotateSecretsInput {
            from_key_ref: parsed.from_key_ref.clone(),
            force: parsed.force,
        },
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "project_slug": parsed.project_slug,
            "from_key_ref": parsed.from_key_ref,
            "force": parsed.force,
            "rotation": rotation
        }))?
    );
    Ok(())
}

fn print_validate_pipeline_config_usage() {
    eprintln!(
        concat!(
            "Usage:\n",
            "  cargo run -- validate-pipeline-config ",
            "[--app-root PATH] [--project-root PATH] [--app-settings PATH] [--project-settings PATH] ",
            "[--manifest PATH] [--postprocess-config PATH]\n\n",
            "Defaults:\n",
            "  --app-root defaults to repository app root (derived from Cargo manifest)\n",
            "  app settings default: config/pipeline.settings.toml (fallback: config/pipeline.settings.json)\n",
            "  project settings default: <project_root>/.kroma/pipeline.settings.json\n",
            "  --manifest / --postprocess-config override layered settings for validation only\n"
        )
    );
}

fn print_secrets_rotation_status_usage() {
    eprintln!(concat!(
        "Usage:\n",
        "  cargo run -- secrets-rotation-status --project-slug <slug>\n"
    ));
}

fn print_secrets_rotate_usage() {
    eprintln!(
        concat!(
            "Usage:\n",
            "  cargo run -- secrets-rotate --project-slug <slug> [--from-key-ref <key-ref>] [--force]\n"
        )
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rotation_status_requires_project_slug() {
        let err = parse_secrets_rotation_status_cli_args(&[]).expect_err("slug should be required");
        assert!(err.to_string().contains("--project-slug"));
    }

    #[test]
    fn parse_rotation_status_accepts_project_slug() {
        let parsed = parse_secrets_rotation_status_cli_args(&[
            String::from("--project-slug"),
            String::from("demo"),
        ])
        .expect("parse should succeed");
        assert_eq!(parsed.project_slug, "demo");
    }

    #[test]
    fn parse_rotate_requires_project_slug() {
        let err = parse_secrets_rotate_cli_args(&[]).expect_err("slug should be required");
        assert!(err.to_string().contains("--project-slug"));
    }

    #[test]
    fn parse_rotate_accepts_optional_flags() {
        let parsed = parse_secrets_rotate_cli_args(&[
            String::from("--project-slug"),
            String::from("demo"),
            String::from("--from-key-ref"),
            String::from("legacy-v1"),
            String::from("--force"),
        ])
        .expect("parse should succeed");
        assert_eq!(parsed.project_slug, "demo");
        assert_eq!(parsed.from_key_ref.as_deref(), Some("legacy-v1"));
        assert!(parsed.force);
    }
}
