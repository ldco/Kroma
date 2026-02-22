use std::collections::BTreeMap;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{
    fetch_project_by_slug, normalize_optional_storage_field, normalize_provider_code,
    normalize_required_text, normalize_slug, now_iso, parse_json_value, ProjectsRepoError,
    ProjectsStore, PromptTemplateSummary, ProviderAccountSummary, StyleGuideSummary,
};

const BOOTSTRAP_SCHEMA_VERSION: &str = "kroma.bootstrap.v1";

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapSettings {
    pub provider_accounts: Vec<ProviderAccountSummary>,
    pub style_guides: Vec<StyleGuideSummary>,
    pub prompt_templates: Vec<PromptTemplateSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapExport {
    pub schema_version: String,
    pub generated_at: String,
    pub project: ProjectBootstrapProject,
    pub settings: ProjectBootstrapSettings,
    pub expected_response: Value,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapAppliedSummary {
    pub provider_accounts: usize,
    pub style_guides: usize,
    pub prompt_templates: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapImportResult {
    pub schema_version: String,
    pub mode: String,
    pub dry_run: bool,
    pub project_updated: bool,
    pub applied: BootstrapAppliedSummary,
    pub project: ProjectBootstrapProject,
    pub settings: ProjectBootstrapSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImportProjectBootstrapInput {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub settings: Option<ProjectBootstrapSettingsInput>,
    #[serde(default)]
    pub ai_response_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapSettingsInput {
    #[serde(default)]
    pub project: Option<ProjectBootstrapProjectInput>,
    #[serde(default)]
    pub provider_accounts: Option<Vec<ProjectBootstrapProviderAccountInput>>,
    #[serde(default)]
    pub style_guides: Option<Vec<ProjectBootstrapStyleGuideInput>>,
    #[serde(default)]
    pub prompt_templates: Option<Vec<ProjectBootstrapPromptTemplateInput>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapProjectInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapProviderAccountInput {
    #[serde(default)]
    pub provider_code: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub account_ref: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapStyleGuideInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapPromptTemplateInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub template_text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BootstrapAiResponseWrapper {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    settings: Option<ProjectBootstrapSettingsInput>,
}

#[derive(Debug, Clone)]
struct BootstrapSnapshot {
    project: ProjectBootstrapProject,
    settings: ProjectBootstrapSettings,
}

#[derive(Debug, Clone)]
struct BootstrapPreview {
    project: ProjectBootstrapProject,
    settings: ProjectBootstrapSettings,
    project_updated: bool,
}

#[derive(Debug, Clone)]
struct NormalizedProjectPatch {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone)]
struct NormalizedProviderAccount {
    provider_code: String,
    display_name: String,
    account_ref: Option<String>,
    base_url: Option<String>,
    enabled: bool,
    config_json: Value,
}

#[derive(Debug, Clone)]
struct NormalizedStyleGuide {
    name: String,
    instructions: String,
    notes: Option<String>,
}

#[derive(Debug, Clone)]
struct NormalizedPromptTemplate {
    name: String,
    template_text: String,
}

#[derive(Debug, Clone)]
struct NormalizedSettings {
    project: Option<NormalizedProjectPatch>,
    has_provider_accounts_section: bool,
    has_style_guides_section: bool,
    has_prompt_templates_section: bool,
    provider_accounts: Vec<NormalizedProviderAccount>,
    style_guides: Vec<NormalizedStyleGuide>,
    prompt_templates: Vec<NormalizedPromptTemplate>,
}

#[derive(Debug, Clone, Copy)]
enum BootstrapImportMode {
    Merge,
    Replace,
}

impl BootstrapImportMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Replace => "replace",
        }
    }
}

impl ProjectsStore {
    pub fn export_project_bootstrap_prompt(
        &self,
        slug: &str,
    ) -> Result<ProjectBootstrapExport, ProjectsRepoError> {
        let snapshot = self.load_bootstrap_snapshot(slug)?;
        let expected_response = bootstrap_response_template();

        Ok(ProjectBootstrapExport {
            schema_version: String::from(BOOTSTRAP_SCHEMA_VERSION),
            generated_at: now_iso(),
            prompt: render_bootstrap_prompt(&snapshot, &expected_response),
            expected_response,
            project: snapshot.project,
            settings: snapshot.settings,
        })
    }

    pub fn import_project_bootstrap(
        &self,
        slug: &str,
        input: ImportProjectBootstrapInput,
    ) -> Result<ProjectBootstrapImportResult, ProjectsRepoError> {
        let (mode_from_payload, settings_input) = resolve_bootstrap_settings_input(&input)?;
        let mode = parse_bootstrap_mode(input.mode.as_deref().or(mode_from_payload.as_deref()))?;
        let dry_run = input.dry_run.unwrap_or(false);
        let settings = normalize_bootstrap_settings(settings_input)?;
        let applied = BootstrapAppliedSummary {
            provider_accounts: settings.provider_accounts.len(),
            style_guides: settings.style_guides.len(),
            prompt_templates: settings.prompt_templates.len(),
        };

        if dry_run {
            let snapshot = self.load_bootstrap_snapshot(slug)?;
            let preview = preview_snapshot(snapshot, &settings, mode);
            return Ok(ProjectBootstrapImportResult {
                schema_version: String::from(BOOTSTRAP_SCHEMA_VERSION),
                mode: String::from(mode.as_str()),
                dry_run: true,
                project_updated: preview.project_updated,
                applied,
                project: preview.project,
                settings: preview.settings,
            });
        }

        let project_updated = self.with_connection_mut(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;

            let tx = conn.transaction()?;
            let now = now_iso();
            let mut project_updated = false;

            if let Some(patch) = settings.project.clone() {
                let next_name = patch.name.unwrap_or_else(|| project.name.clone());
                let next_description = patch
                    .description
                    .unwrap_or_else(|| project.description.clone());
                if next_name != project.name || next_description != project.description {
                    tx.execute(
                        "
                        UPDATE projects
                        SET name = ?1,
                            description = ?2,
                            updated_at = ?3
                        WHERE id = ?4
                    ",
                        params![next_name, next_description, now, project.id.as_str()],
                    )?;
                    project_updated = true;
                }
            }

            if matches!(mode, BootstrapImportMode::Replace)
                && settings.has_provider_accounts_section
            {
                tx.execute(
                    "DELETE FROM provider_accounts WHERE project_id = ?1",
                    [&project.id],
                )?;
            }
            if matches!(mode, BootstrapImportMode::Replace) && settings.has_style_guides_section {
                tx.execute(
                    "DELETE FROM style_guides WHERE project_id = ?1",
                    [&project.id],
                )?;
            }
            if matches!(mode, BootstrapImportMode::Replace) && settings.has_prompt_templates_section
            {
                tx.execute(
                    "DELETE FROM prompt_templates WHERE project_id = ?1",
                    [&project.id],
                )?;
            }

            for provider in &settings.provider_accounts {
                let config_json = serde_json::to_string(&provider.config_json)
                    .unwrap_or_else(|_| String::from("{}"));
                tx.execute(
                    "
                    INSERT INTO provider_accounts (
                        project_id,
                        provider_code,
                        display_name,
                        account_ref,
                        base_url,
                        enabled,
                        config_json,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
                    ON CONFLICT(project_id, provider_code) DO UPDATE SET
                        display_name = excluded.display_name,
                        account_ref = excluded.account_ref,
                        base_url = excluded.base_url,
                        enabled = excluded.enabled,
                        config_json = excluded.config_json,
                        updated_at = excluded.updated_at
                ",
                    params![
                        project.id.as_str(),
                        provider.provider_code.as_str(),
                        provider.display_name.as_str(),
                        provider.account_ref.as_deref(),
                        provider.base_url.as_deref(),
                        if provider.enabled { 1 } else { 0 },
                        config_json,
                        now
                    ],
                )?;
            }

            for style in &settings.style_guides {
                tx.execute(
                    "
                    INSERT INTO style_guides (
                        id,
                        project_id,
                        name,
                        instructions,
                        notes,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                    ON CONFLICT(project_id, name) DO UPDATE SET
                        instructions = excluded.instructions,
                        notes = excluded.notes,
                        updated_at = excluded.updated_at
                ",
                    params![
                        Uuid::new_v4().to_string(),
                        project.id.as_str(),
                        style.name.as_str(),
                        style.instructions.as_str(),
                        style.notes.as_deref(),
                        now
                    ],
                )?;
            }

            for template in &settings.prompt_templates {
                tx.execute(
                    "
                    INSERT INTO prompt_templates (
                        id,
                        project_id,
                        name,
                        template_text,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                    ON CONFLICT(project_id, name) DO UPDATE SET
                        template_text = excluded.template_text,
                        updated_at = excluded.updated_at
                ",
                    params![
                        Uuid::new_v4().to_string(),
                        project.id.as_str(),
                        template.name.as_str(),
                        template.template_text.as_str(),
                        now
                    ],
                )?;
            }

            tx.execute(
                "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
                params![now, project.id.as_str()],
            )?;
            tx.commit()?;

            Ok(project_updated)
        })?;

        let snapshot = self.load_bootstrap_snapshot(slug)?;
        Ok(ProjectBootstrapImportResult {
            schema_version: String::from(BOOTSTRAP_SCHEMA_VERSION),
            mode: String::from(mode.as_str()),
            dry_run: false,
            project_updated,
            applied,
            project: snapshot.project,
            settings: snapshot.settings,
        })
    }

    fn load_bootstrap_snapshot(&self, slug: &str) -> Result<BootstrapSnapshot, ProjectsRepoError> {
        self.with_connection(|conn| {
            let safe_slug = normalize_slug(slug).ok_or(ProjectsRepoError::NotFound)?;
            let project = fetch_project_by_slug(conn, safe_slug.as_str())?
                .ok_or(ProjectsRepoError::NotFound)?;
            let provider_accounts = load_provider_accounts(conn, project.id.as_str())?;
            let style_guides = load_style_guides(conn, project.id.as_str())?;
            let prompt_templates = load_prompt_templates(conn, project.id.as_str())?;

            Ok(BootstrapSnapshot {
                project: ProjectBootstrapProject {
                    id: project.id,
                    slug: project.slug,
                    name: project.name,
                    description: project.description,
                },
                settings: ProjectBootstrapSettings {
                    provider_accounts,
                    style_guides,
                    prompt_templates,
                },
            })
        })
    }
}

fn parse_bootstrap_mode(raw: Option<&str>) -> Result<BootstrapImportMode, ProjectsRepoError> {
    match raw.map(str::trim).map(str::to_ascii_lowercase) {
        None => Ok(BootstrapImportMode::Merge),
        Some(value) if value == "merge" => Ok(BootstrapImportMode::Merge),
        Some(value) if value == "replace" => Ok(BootstrapImportMode::Replace),
        _ => Err(ProjectsRepoError::Validation(String::from(
            "Field 'mode' must be one of: merge, replace",
        ))),
    }
}

fn resolve_bootstrap_settings_input(
    input: &ImportProjectBootstrapInput,
) -> Result<(Option<String>, ProjectBootstrapSettingsInput), ProjectsRepoError> {
    if let Some(settings) = input.settings.clone() {
        return Ok((None, settings));
    }

    let response_text = input
        .ai_response_text
        .as_deref()
        .ok_or_else(|| {
            ProjectsRepoError::Validation(String::from(
                "Provide either 'settings' or 'ai_response_text'",
            ))
        })?
        .trim();

    if response_text.is_empty() {
        return Err(ProjectsRepoError::Validation(String::from(
            "Field 'ai_response_text' must not be empty",
        )));
    }

    let value = parse_json_value_from_text(response_text)?;
    let wrapper: BootstrapAiResponseWrapper =
        serde_json::from_value(value.clone()).unwrap_or(BootstrapAiResponseWrapper {
            mode: None,
            settings: None,
        });
    if let Some(settings) = wrapper.settings {
        return Ok((wrapper.mode, settings));
    }

    let settings =
        serde_json::from_value::<ProjectBootstrapSettingsInput>(value).map_err(|_| {
            ProjectsRepoError::Validation(String::from(
                "Could not parse bootstrap JSON from 'ai_response_text'",
            ))
        })?;
    Ok((None, settings))
}

fn parse_json_value_from_text(raw: &str) -> Result<Value, ProjectsRepoError> {
    let trimmed = raw.trim();
    let mut candidates = vec![trimmed.to_string()];

    if let Some(unfenced) = strip_markdown_code_fence(trimmed) {
        candidates.push(unfenced);
    }

    if let Some((start, end)) =
        trimmed
            .char_indices()
            .find(|(_, ch)| *ch == '{')
            .and_then(|(start, _)| {
                trimmed
                    .char_indices()
                    .rev()
                    .find(|(_, ch)| *ch == '}')
                    .and_then(|(end, _)| (end > start).then_some((start, end)))
            })
    {
        candidates.push(trimmed[start..=end].to_string());
    }

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<Value>(candidate.as_str()) {
            return Ok(value);
        }
    }

    Err(ProjectsRepoError::Validation(String::from(
        "Could not parse JSON from 'ai_response_text'",
    )))
}

fn strip_markdown_code_fence(raw: &str) -> Option<String> {
    if !raw.starts_with("```") {
        return None;
    }

    let mut lines = raw.lines();
    let _opening = lines.next()?;
    let mut body: Vec<&str> = lines.collect();
    if body
        .last()
        .map(|line| line.trim() == "```")
        .unwrap_or(false)
    {
        let _ = body.pop();
    }
    Some(body.join("\n").trim().to_string())
}

fn normalize_bootstrap_settings(
    input: ProjectBootstrapSettingsInput,
) -> Result<NormalizedSettings, ProjectsRepoError> {
    let project = if let Some(project_input) = input.project {
        let name = match project_input.name {
            Some(raw) => Some(normalize_required_text(raw.as_str(), "project.name")?),
            None => None,
        };
        let description = project_input.description.map(|raw| raw.trim().to_string());
        Some(NormalizedProjectPatch { name, description })
    } else {
        None
    };

    let has_provider_accounts_section = input.provider_accounts.is_some();
    let mut providers = BTreeMap::new();
    for provider in input.provider_accounts.unwrap_or_default() {
        let provider_code = normalize_provider_code(provider.provider_code.as_str())?;
        let display_name = provider
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| provider_code.clone());
        let account_ref = provider
            .account_ref
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let base_url = provider
            .base_url
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let enabled = provider.enabled.unwrap_or(true);
        let config_json = provider.config_json.unwrap_or_else(|| json!({}));

        providers.insert(
            provider_code.clone(),
            NormalizedProviderAccount {
                provider_code,
                display_name,
                account_ref,
                base_url,
                enabled,
                config_json,
            },
        );
    }

    let has_style_guides_section = input.style_guides.is_some();
    let mut style_guides = BTreeMap::new();
    for style in input.style_guides.unwrap_or_default() {
        let name = normalize_required_text(style.name.as_str(), "style_guides[].name")?;
        let instructions =
            normalize_required_text(style.instructions.as_str(), "style_guides[].instructions")?;
        let notes = style
            .notes
            .as_deref()
            .and_then(normalize_optional_storage_field);

        style_guides.insert(
            name.to_ascii_lowercase(),
            NormalizedStyleGuide {
                name,
                instructions,
                notes,
            },
        );
    }

    let has_prompt_templates_section = input.prompt_templates.is_some();
    let mut prompt_templates = BTreeMap::new();
    for template in input.prompt_templates.unwrap_or_default() {
        let name = normalize_required_text(template.name.as_str(), "prompt_templates[].name")?;
        let template_text = normalize_required_text(
            template.template_text.as_str(),
            "prompt_templates[].template_text",
        )?;
        prompt_templates.insert(
            name.to_ascii_lowercase(),
            NormalizedPromptTemplate {
                name,
                template_text,
            },
        );
    }

    if project.is_none()
        && !has_provider_accounts_section
        && !has_style_guides_section
        && !has_prompt_templates_section
    {
        return Err(ProjectsRepoError::Validation(String::from(
            "Import payload does not include any settings",
        )));
    }

    Ok(NormalizedSettings {
        project,
        has_provider_accounts_section,
        has_style_guides_section,
        has_prompt_templates_section,
        provider_accounts: providers.into_values().collect(),
        style_guides: style_guides.into_values().collect(),
        prompt_templates: prompt_templates.into_values().collect(),
    })
}

fn preview_snapshot(
    snapshot: BootstrapSnapshot,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
) -> BootstrapPreview {
    let now = now_iso();
    let mut project = snapshot.project.clone();
    let mut project_updated = false;

    if let Some(patch) = settings.project.as_ref() {
        let next_name = patch.name.clone().unwrap_or_else(|| project.name.clone());
        let next_description = patch
            .description
            .clone()
            .unwrap_or_else(|| project.description.clone());
        if next_name != project.name || next_description != project.description {
            project.name = next_name;
            project.description = next_description;
            project_updated = true;
        }
    }

    let provider_accounts = preview_provider_accounts(
        snapshot.settings.provider_accounts,
        settings,
        mode,
        project.id.as_str(),
        now.as_str(),
    );
    let style_guides = preview_style_guides(
        snapshot.settings.style_guides,
        settings,
        mode,
        project.id.as_str(),
        now.as_str(),
    );
    let prompt_templates = preview_prompt_templates(
        snapshot.settings.prompt_templates,
        settings,
        mode,
        project.id.as_str(),
        now.as_str(),
    );

    BootstrapPreview {
        project,
        project_updated,
        settings: ProjectBootstrapSettings {
            provider_accounts,
            style_guides,
            prompt_templates,
        },
    }
}

fn preview_provider_accounts(
    existing: Vec<ProviderAccountSummary>,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
    project_id: &str,
    now: &str,
) -> Vec<ProviderAccountSummary> {
    let mut map: BTreeMap<String, ProviderAccountSummary> =
        if matches!(mode, BootstrapImportMode::Replace) && settings.has_provider_accounts_section {
            BTreeMap::new()
        } else {
            existing
                .into_iter()
                .map(|item| (item.provider_code.clone(), item))
                .collect()
        };

    if settings.has_provider_accounts_section {
        for provider in &settings.provider_accounts {
            let created_at = map
                .get(provider.provider_code.as_str())
                .map(|item| item.created_at.clone())
                .unwrap_or_else(|| now.to_string());
            map.insert(
                provider.provider_code.clone(),
                ProviderAccountSummary {
                    project_id: project_id.to_string(),
                    provider_code: provider.provider_code.clone(),
                    display_name: provider.display_name.clone(),
                    account_ref: provider.account_ref.clone().unwrap_or_default(),
                    base_url: provider.base_url.clone().unwrap_or_default(),
                    enabled: provider.enabled,
                    config_json: provider.config_json.clone(),
                    created_at,
                    updated_at: now.to_string(),
                },
            );
        }
    }

    map.into_values().collect()
}

fn preview_style_guides(
    existing: Vec<StyleGuideSummary>,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
    project_id: &str,
    now: &str,
) -> Vec<StyleGuideSummary> {
    let mut map: BTreeMap<String, StyleGuideSummary> =
        if matches!(mode, BootstrapImportMode::Replace) && settings.has_style_guides_section {
            BTreeMap::new()
        } else {
            existing
                .into_iter()
                .map(|item| (item.name.to_ascii_lowercase(), item))
                .collect()
        };

    if settings.has_style_guides_section {
        for style in &settings.style_guides {
            let key = style.name.to_ascii_lowercase();
            let existing_summary = map.get(key.as_str()).cloned();
            map.insert(
                key,
                StyleGuideSummary {
                    id: existing_summary
                        .as_ref()
                        .map(|item| item.id.clone())
                        .unwrap_or_else(|| format!("preview_style_{}", Uuid::new_v4())),
                    project_id: project_id.to_string(),
                    name: style.name.clone(),
                    instructions: style.instructions.clone(),
                    notes: style.notes.clone().unwrap_or_default(),
                    created_at: existing_summary
                        .as_ref()
                        .map(|item| item.created_at.clone())
                        .unwrap_or_else(|| now.to_string()),
                    updated_at: now.to_string(),
                },
            );
        }
    }

    map.into_values().collect()
}

fn preview_prompt_templates(
    existing: Vec<PromptTemplateSummary>,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
    project_id: &str,
    now: &str,
) -> Vec<PromptTemplateSummary> {
    let mut map: BTreeMap<String, PromptTemplateSummary> =
        if matches!(mode, BootstrapImportMode::Replace) && settings.has_prompt_templates_section {
            BTreeMap::new()
        } else {
            existing
                .into_iter()
                .map(|item| (item.name.to_ascii_lowercase(), item))
                .collect()
        };

    if settings.has_prompt_templates_section {
        for template in &settings.prompt_templates {
            let key = template.name.to_ascii_lowercase();
            let existing_summary = map.get(key.as_str()).cloned();
            map.insert(
                key,
                PromptTemplateSummary {
                    id: existing_summary
                        .as_ref()
                        .map(|item| item.id.clone())
                        .unwrap_or_else(|| format!("preview_prompt_{}", Uuid::new_v4())),
                    project_id: project_id.to_string(),
                    name: template.name.clone(),
                    template_text: template.template_text.clone(),
                    created_at: existing_summary
                        .as_ref()
                        .map(|item| item.created_at.clone())
                        .unwrap_or_else(|| now.to_string()),
                    updated_at: now.to_string(),
                },
            );
        }
    }

    map.into_values().collect()
}

fn load_provider_accounts(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<ProviderAccountSummary>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT
          project_id,
          provider_code,
          display_name,
          account_ref,
          base_url,
          enabled,
          config_json,
          created_at,
          updated_at
        FROM provider_accounts
        WHERE project_id = ?1
        ORDER BY provider_code ASC
    ",
    )?;
    let mut rows = stmt.query([project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(ProviderAccountSummary {
            project_id: row.get("project_id")?,
            provider_code: row.get("provider_code")?,
            display_name: row.get("display_name")?,
            account_ref: row
                .get::<_, Option<String>>("account_ref")?
                .unwrap_or_default(),
            base_url: row
                .get::<_, Option<String>>("base_url")?
                .unwrap_or_default(),
            enabled: row.get::<_, Option<i64>>("enabled")?.unwrap_or(1) != 0,
            config_json: parse_json_value(row.get::<_, Option<String>>("config_json")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        });
    }
    Ok(out)
}

fn load_style_guides(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<StyleGuideSummary>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, project_id, name, instructions, notes, created_at, updated_at
        FROM style_guides
        WHERE project_id = ?1
        ORDER BY COALESCE(updated_at, '') DESC, id DESC
    ",
    )?;
    let mut rows = stmt.query([project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(StyleGuideSummary {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            name: row.get("name")?,
            instructions: row.get("instructions")?,
            notes: row.get::<_, Option<String>>("notes")?.unwrap_or_default(),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        });
    }
    Ok(out)
}

fn load_prompt_templates(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<PromptTemplateSummary>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, project_id, name, template_text, created_at, updated_at
        FROM prompt_templates
        WHERE project_id = ?1
        ORDER BY COALESCE(updated_at, '') DESC, id DESC
    ",
    )?;
    let mut rows = stmt.query([project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(PromptTemplateSummary {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            name: row.get("name")?,
            template_text: row.get("template_text")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        });
    }
    Ok(out)
}

fn bootstrap_response_template() -> Value {
    json!({
        "mode": "merge",
        "settings": {
            "project": {
                "name": "[OPTIONAL_PROJECT_NAME]",
                "description": "[OPTIONAL_PROJECT_DESCRIPTION]"
            },
            "provider_accounts": [
                {
                    "provider_code": "openai",
                    "display_name": "OpenAI Main",
                    "account_ref": "[OPTIONAL_ACCOUNT_REF]",
                    "base_url": "https://api.openai.com/v1",
                    "enabled": true,
                    "config_json": {
                        "model": "gpt-image-1"
                    }
                }
            ],
            "style_guides": [
                {
                    "name": "Core Style",
                    "instructions": "Describe the visual style rules here.",
                    "notes": "[OPTIONAL_NOTES]"
                }
            ],
            "prompt_templates": [
                {
                    "name": "Cover Prompt",
                    "template_text": "Write the reusable generation prompt text here."
                }
            ]
        }
    })
}

fn render_bootstrap_prompt(snapshot: &BootstrapSnapshot, expected_response: &Value) -> String {
    let current_settings = json!({
        "project": {
            "id": &snapshot.project.id,
            "slug": &snapshot.project.slug,
            "name": &snapshot.project.name,
            "description": &snapshot.project.description,
        },
        "provider_accounts": &snapshot.settings.provider_accounts,
        "style_guides": &snapshot.settings.style_guides,
        "prompt_templates": &snapshot.settings.prompt_templates,
    });

    let current_settings_pretty =
        serde_json::to_string_pretty(&current_settings).unwrap_or_else(|_| String::from("{}"));
    let expected_response_pretty =
        serde_json::to_string_pretty(expected_response).unwrap_or_else(|_| String::from("{}"));

    format!(
        "You are configuring a Kroma project bootstrap profile.\n\
Return ONLY valid JSON (no markdown, no comments, no prose).\n\
Use this exact response shape:\n\
{expected_response_pretty}\n\
\n\
Rules:\n\
- Keep provider_code lowercase (letters, numbers, '-', '_').\n\
- Omit sections you do not want to change.\n\
- Keep arrays concise and production-ready.\n\
- Do not include secrets or API keys.\n\
\n\
Current project settings:\n\
{current_settings_pretty}\n"
    )
}
