use std::net::SocketAddr;
use std::path::PathBuf;

use kroma_backend_core::api::server::serve;
use kroma_backend_core::pipeline::config_validation::{
    validate_pipeline_config_stack, PipelineConfigValidationRequest,
};
use kroma_backend_core::pipeline::runtime::default_app_root_from_manifest_dir;
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
