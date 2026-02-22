use std::collections::BTreeMap;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{
    fetch_project_by_slug, normalize_optional_storage_field, normalize_provider_code,
    normalize_required_text, normalize_slug, now_iso, parse_json_value, CharacterSummary,
    ProjectsRepoError, ProjectsStore, PromptTemplateSummary, ProviderAccountSummary,
    ReferenceSetItemSummary, ReferenceSetSummary, SecretSummary, StyleGuideSummary,
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
    pub characters: Vec<CharacterSummary>,
    pub reference_sets: Vec<ProjectBootstrapReferenceSet>,
    pub secrets: Vec<ProjectBootstrapSecret>,
    pub prompt_templates: Vec<PromptTemplateSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapReferenceSet {
    pub name: String,
    pub description: String,
    pub items: Vec<ProjectBootstrapReferenceSetItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapReferenceSetItem {
    pub label: String,
    pub content_uri: String,
    pub content_text: String,
    pub sort_order: i64,
    pub metadata_json: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapSecret {
    pub provider_code: String,
    pub secret_name: String,
    pub has_value: bool,
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
    pub characters: usize,
    pub reference_sets: usize,
    pub secrets: usize,
    pub prompt_templates: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapProjectChangeSummary {
    pub provided: bool,
    pub updated: bool,
    pub name_changed: bool,
    pub description_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapSectionChangeSummary {
    pub provided: bool,
    pub replaced: bool,
    pub before_count: usize,
    pub after_count: usize,
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapImportChangeSummary {
    pub project: BootstrapProjectChangeSummary,
    pub provider_accounts: BootstrapSectionChangeSummary,
    pub style_guides: BootstrapSectionChangeSummary,
    pub characters: BootstrapSectionChangeSummary,
    pub reference_sets: BootstrapSectionChangeSummary,
    pub secrets: BootstrapSectionChangeSummary,
    pub prompt_templates: BootstrapSectionChangeSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectBootstrapImportResult {
    pub schema_version: String,
    pub mode: String,
    pub dry_run: bool,
    pub project_updated: bool,
    pub applied: BootstrapAppliedSummary,
    pub changes: BootstrapImportChangeSummary,
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
    pub characters: Option<Vec<ProjectBootstrapCharacterInput>>,
    #[serde(default)]
    pub reference_sets: Option<Vec<ProjectBootstrapReferenceSetInput>>,
    #[serde(default)]
    pub secrets: Option<Vec<ProjectBootstrapSecretInput>>,
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapCharacterInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub prompt_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapReferenceSetInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<ProjectBootstrapReferenceSetItemInput>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapReferenceSetItemInput {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub content_uri: Option<String>,
    #[serde(default)]
    pub content_text: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub metadata_json: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProjectBootstrapSecretInput {
    #[serde(default)]
    pub provider_code: String,
    #[serde(default)]
    pub secret_name: String,
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
struct NormalizedCharacter {
    name: String,
    description: Option<String>,
    prompt_text: Option<String>,
}

#[derive(Debug, Clone)]
struct NormalizedSettings {
    project: Option<NormalizedProjectPatch>,
    has_provider_accounts_section: bool,
    has_style_guides_section: bool,
    has_characters_section: bool,
    has_reference_sets_section: bool,
    has_secrets_section: bool,
    has_prompt_templates_section: bool,
    provider_accounts: Vec<NormalizedProviderAccount>,
    style_guides: Vec<NormalizedStyleGuide>,
    characters: Vec<NormalizedCharacter>,
    reference_sets: Vec<NormalizedReferenceSet>,
    secrets: Vec<NormalizedSecret>,
    prompt_templates: Vec<NormalizedPromptTemplate>,
}

#[derive(Debug, Clone)]
struct NormalizedReferenceSet {
    name: String,
    description: Option<String>,
    items: Vec<NormalizedReferenceSetItem>,
}

#[derive(Debug, Clone)]
struct NormalizedReferenceSetItem {
    label: String,
    content_uri: Option<String>,
    content_text: Option<String>,
    sort_order: i64,
    metadata_json: Value,
}

#[derive(Debug, Clone)]
struct NormalizedSecret {
    provider_code: String,
    secret_name: String,
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
        let before_snapshot = self.load_bootstrap_snapshot(slug)?;
        let applied = BootstrapAppliedSummary {
            provider_accounts: settings.provider_accounts.len(),
            style_guides: settings.style_guides.len(),
            characters: settings.characters.len(),
            reference_sets: settings.reference_sets.len(),
            secrets: settings.secrets.len(),
            prompt_templates: settings.prompt_templates.len(),
        };

        if dry_run {
            let preview = preview_snapshot(before_snapshot.clone(), &settings, mode);
            let changes = compute_bootstrap_import_changes(
                &before_snapshot,
                &preview.project,
                &preview.settings,
                &settings,
                mode,
                preview.project_updated,
            );
            return Ok(ProjectBootstrapImportResult {
                schema_version: String::from(BOOTSTRAP_SCHEMA_VERSION),
                mode: String::from(mode.as_str()),
                dry_run: true,
                project_updated: preview.project_updated,
                applied,
                changes,
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
            if matches!(mode, BootstrapImportMode::Replace) && settings.has_characters_section {
                tx.execute(
                    "DELETE FROM characters WHERE project_id = ?1",
                    [&project.id],
                )?;
            }
            if matches!(mode, BootstrapImportMode::Replace) && settings.has_reference_sets_section {
                tx.execute(
                    "DELETE FROM reference_set_items WHERE project_id = ?1",
                    [&project.id],
                )?;
                tx.execute(
                    "DELETE FROM reference_sets WHERE project_id = ?1",
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

            for character in &settings.characters {
                tx.execute(
                    "
                    INSERT INTO characters (
                        id,
                        project_id,
                        name,
                        description,
                        prompt_text,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                    ON CONFLICT(project_id, name) DO UPDATE SET
                        description = excluded.description,
                        prompt_text = excluded.prompt_text,
                        updated_at = excluded.updated_at
                ",
                    params![
                        Uuid::new_v4().to_string(),
                        project.id.as_str(),
                        character.name.as_str(),
                        character.description.as_deref(),
                        character.prompt_text.as_deref(),
                        now
                    ],
                )?;
            }

            for reference_set in &settings.reference_sets {
                tx.execute(
                    "
                    INSERT INTO reference_sets (
                        id,
                        project_id,
                        name,
                        description,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                    ON CONFLICT(project_id, name) DO UPDATE SET
                        description = excluded.description,
                        updated_at = excluded.updated_at
                ",
                    params![
                        Uuid::new_v4().to_string(),
                        project.id.as_str(),
                        reference_set.name.as_str(),
                        reference_set.description.as_deref(),
                        now
                    ],
                )?;

                let reference_set_id: String = tx.query_row(
                    "
                    SELECT id
                    FROM reference_sets
                    WHERE project_id = ?1 AND name = ?2
                ",
                    params![project.id.as_str(), reference_set.name.as_str()],
                    |row| row.get(0),
                )?;

                // Nested item lists are treated as authoritative for each provided set.
                tx.execute(
                    "
                    DELETE FROM reference_set_items
                    WHERE project_id = ?1 AND reference_set_id = ?2
                ",
                    params![project.id.as_str(), reference_set_id.as_str()],
                )?;

                for item in &reference_set.items {
                    let metadata_json = serde_json::to_string(&item.metadata_json)
                        .unwrap_or_else(|_| String::from("{}"));
                    tx.execute(
                        "
                        INSERT INTO reference_set_items (
                            id,
                            project_id,
                            reference_set_id,
                            label,
                            content_uri,
                            content_text,
                            sort_order,
                            metadata_json,
                            created_at,
                            updated_at
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
                    ",
                        params![
                            Uuid::new_v4().to_string(),
                            project.id.as_str(),
                            reference_set_id.as_str(),
                            item.label.as_str(),
                            item.content_uri.as_deref(),
                            item.content_text.as_deref(),
                            item.sort_order,
                            metadata_json,
                            now
                        ],
                    )?;
                }
            }

            for secret in &settings.secrets {
                // Bootstrap secrets are metadata-only: never import or overwrite secret values.
                tx.execute(
                    "
                    INSERT INTO project_secrets (
                        project_id,
                        provider_code,
                        secret_name,
                        secret_value,
                        created_at,
                        updated_at
                    )
                    VALUES (?1, ?2, ?3, '', ?4, ?4)
                    ON CONFLICT(project_id, provider_code, secret_name) DO UPDATE SET
                        updated_at = excluded.updated_at
                ",
                    params![
                        project.id.as_str(),
                        secret.provider_code.as_str(),
                        secret.secret_name.as_str(),
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
        let changes = compute_bootstrap_import_changes(
            &before_snapshot,
            &snapshot.project,
            &snapshot.settings,
            &settings,
            mode,
            project_updated,
        );
        Ok(ProjectBootstrapImportResult {
            schema_version: String::from(BOOTSTRAP_SCHEMA_VERSION),
            mode: String::from(mode.as_str()),
            dry_run: false,
            project_updated,
            applied,
            changes,
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
            let characters = load_characters(conn, project.id.as_str())?;
            let reference_sets = load_reference_sets(conn, project.id.as_str())?;
            let secrets = load_secrets(conn, project.id.as_str())?;
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
                    characters,
                    reference_sets,
                    secrets,
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

    let has_characters_section = input.characters.is_some();
    let mut characters = BTreeMap::new();
    for character in input.characters.unwrap_or_default() {
        let name = normalize_required_text(character.name.as_str(), "characters[].name")?;
        let description = character
            .description
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let prompt_text = character
            .prompt_text
            .as_deref()
            .and_then(normalize_optional_storage_field);
        characters.insert(
            name.to_ascii_lowercase(),
            NormalizedCharacter {
                name,
                description,
                prompt_text,
            },
        );
    }

    let has_reference_sets_section = input.reference_sets.is_some();
    let mut reference_sets = BTreeMap::new();
    for reference_set in input.reference_sets.unwrap_or_default() {
        let name = normalize_required_text(reference_set.name.as_str(), "reference_sets[].name")?;
        let description = reference_set
            .description
            .as_deref()
            .and_then(normalize_optional_storage_field);
        let raw_items = reference_set.items.ok_or_else(|| {
            ProjectsRepoError::Validation(String::from(
                "Field 'reference_sets[].items' is required (use [] to provide an empty set)",
            ))
        })?;

        let mut items = Vec::new();
        for item in raw_items {
            let label =
                normalize_required_text(item.label.as_str(), "reference_sets[].items[].label")?;
            let content_uri = item
                .content_uri
                .as_deref()
                .and_then(normalize_optional_storage_field);
            let content_text = item
                .content_text
                .as_deref()
                .and_then(normalize_optional_storage_field);
            if content_uri.is_none() && content_text.is_none() {
                return Err(ProjectsRepoError::Validation(String::from(
                    "Field 'reference_sets[].items[]' requires at least one of: content_uri, content_text",
                )));
            }
            items.push(NormalizedReferenceSetItem {
                label,
                content_uri,
                content_text,
                sort_order: item.sort_order.unwrap_or(0),
                metadata_json: item
                    .metadata_json
                    .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
            });
        }

        reference_sets.insert(
            name.to_ascii_lowercase(),
            NormalizedReferenceSet {
                name,
                description,
                items,
            },
        );
    }

    let has_secrets_section = input.secrets.is_some();
    let mut secrets = BTreeMap::new();
    for secret in input.secrets.unwrap_or_default() {
        let provider_code = normalize_provider_code(secret.provider_code.as_str())?;
        let secret_name =
            normalize_required_text(secret.secret_name.as_str(), "secrets[].secret_name")?;
        secrets.insert(
            format!("{provider_code}\u{0}{secret_name}"),
            NormalizedSecret {
                provider_code,
                secret_name,
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
        && !has_characters_section
        && !has_reference_sets_section
        && !has_secrets_section
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
        has_characters_section,
        has_reference_sets_section,
        has_secrets_section,
        has_prompt_templates_section,
        provider_accounts: providers.into_values().collect(),
        style_guides: style_guides.into_values().collect(),
        characters: characters.into_values().collect(),
        reference_sets: reference_sets.into_values().collect(),
        secrets: secrets.into_values().collect(),
        prompt_templates: prompt_templates.into_values().collect(),
    })
}

fn compute_bootstrap_import_changes(
    before: &BootstrapSnapshot,
    after_project: &ProjectBootstrapProject,
    after_settings: &ProjectBootstrapSettings,
    normalized: &NormalizedSettings,
    mode: BootstrapImportMode,
    project_updated: bool,
) -> BootstrapImportChangeSummary {
    BootstrapImportChangeSummary {
        project: BootstrapProjectChangeSummary {
            provided: normalized.project.is_some(),
            updated: project_updated,
            name_changed: before.project.name != after_project.name,
            description_changed: before.project.description != after_project.description,
        },
        provider_accounts: summarize_provider_account_changes(
            before.settings.provider_accounts.as_slice(),
            after_settings.provider_accounts.as_slice(),
            normalized.has_provider_accounts_section,
            matches!(mode, BootstrapImportMode::Replace)
                && normalized.has_provider_accounts_section,
        ),
        style_guides: summarize_style_guide_changes(
            before.settings.style_guides.as_slice(),
            after_settings.style_guides.as_slice(),
            normalized.has_style_guides_section,
            matches!(mode, BootstrapImportMode::Replace) && normalized.has_style_guides_section,
        ),
        characters: summarize_character_changes(
            before.settings.characters.as_slice(),
            after_settings.characters.as_slice(),
            normalized.has_characters_section,
            matches!(mode, BootstrapImportMode::Replace) && normalized.has_characters_section,
        ),
        reference_sets: summarize_reference_set_changes(
            before.settings.reference_sets.as_slice(),
            after_settings.reference_sets.as_slice(),
            normalized.has_reference_sets_section,
            matches!(mode, BootstrapImportMode::Replace) && normalized.has_reference_sets_section,
        ),
        secrets: summarize_secret_changes(
            before.settings.secrets.as_slice(),
            after_settings.secrets.as_slice(),
            normalized.has_secrets_section,
            false,
        ),
        prompt_templates: summarize_prompt_template_changes(
            before.settings.prompt_templates.as_slice(),
            after_settings.prompt_templates.as_slice(),
            normalized.has_prompt_templates_section,
            matches!(mode, BootstrapImportMode::Replace) && normalized.has_prompt_templates_section,
        ),
    }
}

fn summarize_provider_account_changes(
    before: &[ProviderAccountSummary],
    after: &[ProviderAccountSummary],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<&str, &ProviderAccountSummary> = before
        .iter()
        .map(|item| (item.provider_code.as_str(), item))
        .collect();
    let after_map: BTreeMap<&str, &ProviderAccountSummary> = after
        .iter()
        .map(|item| (item.provider_code.as_str(), item))
        .collect();
    summarize_section_changes(
        before_map,
        after_map,
        provided,
        replaced,
        provider_accounts_equal,
    )
}

fn summarize_style_guide_changes(
    before: &[StyleGuideSummary],
    after: &[StyleGuideSummary],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<String, &StyleGuideSummary> = before
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    let after_map: BTreeMap<String, &StyleGuideSummary> = after
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    summarize_section_changes(
        before_map,
        after_map,
        provided,
        replaced,
        style_guides_equal,
    )
}

fn summarize_prompt_template_changes(
    before: &[PromptTemplateSummary],
    after: &[PromptTemplateSummary],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<String, &PromptTemplateSummary> = before
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    let after_map: BTreeMap<String, &PromptTemplateSummary> = after
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    summarize_section_changes(
        before_map,
        after_map,
        provided,
        replaced,
        prompt_templates_equal,
    )
}

fn summarize_character_changes(
    before: &[CharacterSummary],
    after: &[CharacterSummary],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<String, &CharacterSummary> = before
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    let after_map: BTreeMap<String, &CharacterSummary> = after
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    summarize_section_changes(before_map, after_map, provided, replaced, characters_equal)
}

fn summarize_reference_set_changes(
    before: &[ProjectBootstrapReferenceSet],
    after: &[ProjectBootstrapReferenceSet],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<String, &ProjectBootstrapReferenceSet> = before
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    let after_map: BTreeMap<String, &ProjectBootstrapReferenceSet> = after
        .iter()
        .map(|item| (item.name.to_ascii_lowercase(), item))
        .collect();
    summarize_section_changes(
        before_map,
        after_map,
        provided,
        replaced,
        reference_sets_equal,
    )
}

fn summarize_secret_changes(
    before: &[ProjectBootstrapSecret],
    after: &[ProjectBootstrapSecret],
    provided: bool,
    replaced: bool,
) -> BootstrapSectionChangeSummary {
    let before_map: BTreeMap<String, &ProjectBootstrapSecret> = before
        .iter()
        .map(|item| {
            (
                format!("{}\u{0}{}", item.provider_code, item.secret_name),
                item,
            )
        })
        .collect();
    let after_map: BTreeMap<String, &ProjectBootstrapSecret> = after
        .iter()
        .map(|item| {
            (
                format!("{}\u{0}{}", item.provider_code, item.secret_name),
                item,
            )
        })
        .collect();
    summarize_section_changes(before_map, after_map, provided, replaced, secrets_equal)
}

fn summarize_section_changes<K, T, F>(
    before_map: BTreeMap<K, &T>,
    after_map: BTreeMap<K, &T>,
    provided: bool,
    replaced: bool,
    equals: F,
) -> BootstrapSectionChangeSummary
where
    K: Ord + Clone,
    F: Fn(&T, &T) -> bool,
{
    let before_count = before_map.len();
    let after_count = after_map.len();

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut unchanged = 0usize;

    for (key, after_item) in &after_map {
        match before_map.get(key) {
            None => created += 1,
            Some(before_item) => {
                if equals(before_item, after_item) {
                    unchanged += 1;
                } else {
                    updated += 1;
                }
            }
        }
    }

    let mut deleted = 0usize;
    for key in before_map.keys() {
        if !after_map.contains_key(key) {
            deleted += 1;
        }
    }

    BootstrapSectionChangeSummary {
        provided,
        replaced,
        before_count,
        after_count,
        created,
        updated,
        deleted,
        unchanged,
    }
}

fn provider_accounts_equal(left: &ProviderAccountSummary, right: &ProviderAccountSummary) -> bool {
    left.provider_code == right.provider_code
        && left.display_name == right.display_name
        && left.account_ref == right.account_ref
        && left.base_url == right.base_url
        && left.enabled == right.enabled
        && left.config_json == right.config_json
}

fn style_guides_equal(left: &StyleGuideSummary, right: &StyleGuideSummary) -> bool {
    left.name == right.name && left.instructions == right.instructions && left.notes == right.notes
}

fn prompt_templates_equal(left: &PromptTemplateSummary, right: &PromptTemplateSummary) -> bool {
    left.name == right.name && left.template_text == right.template_text
}

fn characters_equal(left: &CharacterSummary, right: &CharacterSummary) -> bool {
    left.name == right.name
        && left.description == right.description
        && left.prompt_text == right.prompt_text
}

fn reference_sets_equal(
    left: &ProjectBootstrapReferenceSet,
    right: &ProjectBootstrapReferenceSet,
) -> bool {
    left.name == right.name
        && left.description == right.description
        && left.items.len() == right.items.len()
        && left
            .items
            .iter()
            .zip(right.items.iter())
            .all(|(a, b)| reference_set_items_equal(a, b))
}

fn reference_set_items_equal(
    left: &ProjectBootstrapReferenceSetItem,
    right: &ProjectBootstrapReferenceSetItem,
) -> bool {
    left.label == right.label
        && left.content_uri == right.content_uri
        && left.content_text == right.content_text
        && left.sort_order == right.sort_order
        && left.metadata_json == right.metadata_json
}

fn secrets_equal(left: &ProjectBootstrapSecret, right: &ProjectBootstrapSecret) -> bool {
    left.provider_code == right.provider_code
        && left.secret_name == right.secret_name
        && left.has_value == right.has_value
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
    let characters = preview_characters(
        snapshot.settings.characters,
        settings,
        mode,
        project.id.as_str(),
        now.as_str(),
    );
    let reference_sets = preview_reference_sets(snapshot.settings.reference_sets, settings, mode);
    let secrets = preview_secrets(snapshot.settings.secrets, settings, mode);
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
            characters,
            reference_sets,
            secrets,
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

fn preview_characters(
    existing: Vec<CharacterSummary>,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
    project_id: &str,
    now: &str,
) -> Vec<CharacterSummary> {
    let mut map: BTreeMap<String, CharacterSummary> =
        if matches!(mode, BootstrapImportMode::Replace) && settings.has_characters_section {
            BTreeMap::new()
        } else {
            existing
                .into_iter()
                .map(|item| (item.name.to_ascii_lowercase(), item))
                .collect()
        };

    if settings.has_characters_section {
        for character in &settings.characters {
            let key = character.name.to_ascii_lowercase();
            let existing_summary = map.get(key.as_str()).cloned();
            map.insert(
                key,
                CharacterSummary {
                    id: existing_summary
                        .as_ref()
                        .map(|item| item.id.clone())
                        .unwrap_or_else(|| format!("preview_character_{}", Uuid::new_v4())),
                    project_id: project_id.to_string(),
                    name: character.name.clone(),
                    description: character.description.clone().unwrap_or_default(),
                    prompt_text: character.prompt_text.clone().unwrap_or_default(),
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

fn preview_reference_sets(
    existing: Vec<ProjectBootstrapReferenceSet>,
    settings: &NormalizedSettings,
    mode: BootstrapImportMode,
) -> Vec<ProjectBootstrapReferenceSet> {
    let mut map: BTreeMap<String, ProjectBootstrapReferenceSet> =
        if matches!(mode, BootstrapImportMode::Replace) && settings.has_reference_sets_section {
            BTreeMap::new()
        } else {
            existing
                .into_iter()
                .map(|item| (item.name.to_ascii_lowercase(), item))
                .collect()
        };

    if settings.has_reference_sets_section {
        for reference_set in &settings.reference_sets {
            map.insert(
                reference_set.name.to_ascii_lowercase(),
                ProjectBootstrapReferenceSet {
                    name: reference_set.name.clone(),
                    description: reference_set.description.clone().unwrap_or_default(),
                    items: reference_set
                        .items
                        .iter()
                        .map(|item| ProjectBootstrapReferenceSetItem {
                            label: item.label.clone(),
                            content_uri: item.content_uri.clone().unwrap_or_default(),
                            content_text: item.content_text.clone().unwrap_or_default(),
                            sort_order: item.sort_order,
                            metadata_json: item.metadata_json.clone(),
                        })
                        .collect(),
                },
            );
        }
    }

    map.into_values().collect()
}

fn preview_secrets(
    existing: Vec<ProjectBootstrapSecret>,
    settings: &NormalizedSettings,
    _mode: BootstrapImportMode,
) -> Vec<ProjectBootstrapSecret> {
    let mut map: BTreeMap<String, ProjectBootstrapSecret> = existing
        .into_iter()
        .map(|item| {
            (
                format!("{}\u{0}{}", item.provider_code, item.secret_name),
                item,
            )
        })
        .collect();

    if settings.has_secrets_section {
        for secret in &settings.secrets {
            let key = format!("{}\u{0}{}", secret.provider_code, secret.secret_name);
            map.entry(key).or_insert_with(|| ProjectBootstrapSecret {
                provider_code: secret.provider_code.clone(),
                secret_name: secret.secret_name.clone(),
                has_value: false,
            });
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

fn load_characters(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<CharacterSummary>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT id, project_id, name, description, prompt_text, created_at, updated_at
        FROM characters
        WHERE project_id = ?1
        ORDER BY COALESCE(updated_at, '') DESC, id DESC
    ",
    )?;
    let mut rows = stmt.query([project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(CharacterSummary {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            name: row.get("name")?,
            description: row
                .get::<_, Option<String>>("description")?
                .unwrap_or_default(),
            prompt_text: row
                .get::<_, Option<String>>("prompt_text")?
                .unwrap_or_default(),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        });
    }
    Ok(out)
}

fn load_reference_sets(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectBootstrapReferenceSet>, ProjectsRepoError> {
    let mut set_stmt = conn.prepare(
        "
        SELECT id, name, description
        FROM reference_sets
        WHERE project_id = ?1
        ORDER BY COALESCE(updated_at, '') DESC, id DESC
    ",
    )?;
    let mut set_rows = set_stmt.query([project_id])?;
    let mut out = Vec::new();

    while let Some(set_row) = set_rows.next()? {
        let reference_set = ReferenceSetSummary {
            id: set_row.get("id")?,
            project_id: project_id.to_string(),
            name: set_row.get("name")?,
            description: set_row
                .get::<_, Option<String>>("description")?
                .unwrap_or_default(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        let mut item_stmt = conn.prepare(
            "
            SELECT
              id,
              project_id,
              reference_set_id,
              label,
              content_uri,
              content_text,
              sort_order,
              metadata_json,
              created_at,
              updated_at
            FROM reference_set_items
            WHERE project_id = ?1 AND reference_set_id = ?2
            ORDER BY sort_order ASC, COALESCE(updated_at, '') DESC, id DESC
        ",
        )?;
        let item_rows = item_stmt.query_map(
            params![project_id, reference_set.id.as_str()],
            |row| -> rusqlite::Result<ReferenceSetItemSummary> {
                Ok(ReferenceSetItemSummary {
                    id: row.get("id")?,
                    project_id: row.get("project_id")?,
                    reference_set_id: row.get("reference_set_id")?,
                    label: row.get("label")?,
                    content_uri: row
                        .get::<_, Option<String>>("content_uri")?
                        .unwrap_or_default(),
                    content_text: row
                        .get::<_, Option<String>>("content_text")?
                        .unwrap_or_default(),
                    sort_order: row.get("sort_order")?,
                    metadata_json: parse_json_value(row.get::<_, Option<String>>("metadata_json")?),
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                })
            },
        )?;
        let mut items = Vec::new();
        for row in item_rows {
            let item = row?;
            items.push(ProjectBootstrapReferenceSetItem {
                label: item.label,
                content_uri: item.content_uri,
                content_text: item.content_text,
                sort_order: item.sort_order,
                metadata_json: item.metadata_json,
            });
        }

        out.push(ProjectBootstrapReferenceSet {
            name: reference_set.name,
            description: reference_set.description,
            items,
        });
    }

    Ok(out)
}

fn load_secrets(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectBootstrapSecret>, ProjectsRepoError> {
    let mut stmt = conn.prepare(
        "
        SELECT project_id, provider_code, secret_name, secret_value, updated_at
        FROM project_secrets
        WHERE project_id = ?1
        ORDER BY provider_code ASC, secret_name ASC
    ",
    )?;
    let mut rows = stmt.query([project_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let secret = SecretSummary {
            project_id: row.get("project_id")?,
            provider_code: row.get("provider_code")?,
            secret_name: row.get("secret_name")?,
            has_value: !row
                .get::<_, Option<String>>("secret_value")?
                .unwrap_or_default()
                .trim()
                .is_empty(),
            updated_at: row.get("updated_at")?,
        };
        out.push(ProjectBootstrapSecret {
            provider_code: secret.provider_code,
            secret_name: secret.secret_name,
            has_value: secret.has_value,
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
            "characters": [
                {
                    "name": "Hero Character",
                    "description": "[OPTIONAL_CHARACTER_DESCRIPTION]",
                    "prompt_text": "Describe the character and visual constraints."
                }
            ],
            "reference_sets": [
                {
                    "name": "Hero Faces",
                    "description": "[OPTIONAL_REFERENCE_SET_DESCRIPTION]",
                    "items": [
                        {
                            "label": "Hero closeup",
                            "content_uri": "[OPTIONAL_FILE_OR_HTTP_URI]",
                            "content_text": "Short text reference notes for this item.",
                            "sort_order": 0,
                            "metadata_json": {
                                "source": "manual"
                            }
                        }
                    ]
                }
            ],
            "secrets": [
                {
                    "provider_code": "openai",
                    "secret_name": "api_key"
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
        "characters": &snapshot.settings.characters,
        "reference_sets": &snapshot.settings.reference_sets,
        "secrets": &snapshot.settings.secrets,
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
- Never include secret values or API keys.\n\
- If you include a `secrets` section, provide metadata only (`provider_code`, `secret_name`).\n\
- `secrets` imports are merge-only for safety; omitted secrets are not deleted.\n\
- If you include `reference_sets`, each set must include an explicit `items` array (use `[]` for an empty set).\n\
- For each provided reference set, the `items` array is treated as authoritative (omitted items for that set are removed).\n\
\n\
Current project settings:\n\
{current_settings_pretty}\n"
    )
}
