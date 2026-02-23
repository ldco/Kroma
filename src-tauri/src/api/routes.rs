use std::collections::BTreeSet;

use crate::contract::{HttpMethod, RouteSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteDomain {
    System,
    Auth,
    Projects,
    Storage,
    Runs,
    Assets,
    Analytics,
    Exports,
    PromptTemplates,
    ProviderAccounts,
    StyleGuides,
    Characters,
    ReferenceSets,
    Chat,
    Instructions,
    Secrets,
}

impl RouteDomain {
    pub fn from_path(path: &str) -> Self {
        if path == "/health" {
            return Self::System;
        }
        if path.starts_with("/auth/") {
            return Self::Auth;
        }
        if path.contains("/storage") {
            return Self::Storage;
        }
        if path.contains("/runs") {
            return Self::Runs;
        }
        if path.contains("/asset-links") || path.contains("/assets") {
            return Self::Assets;
        }
        if path.contains("/quality-reports") || path.contains("/cost-events") {
            return Self::Analytics;
        }
        if path.contains("/exports") || path.contains("/export") {
            return Self::Exports;
        }
        if path.contains("/prompt-templates") {
            return Self::PromptTemplates;
        }
        if path.contains("/provider-accounts") {
            return Self::ProviderAccounts;
        }
        if path.contains("/style-guides") {
            return Self::StyleGuides;
        }
        if path.contains("/characters") {
            return Self::Characters;
        }
        if path.contains("/reference-sets") {
            return Self::ReferenceSets;
        }
        if path.contains("/chat/") {
            return Self::Chat;
        }
        if path.contains("/agent/instructions") {
            return Self::Instructions;
        }
        if path.contains("/secrets") {
            return Self::Secrets;
        }
        Self::Projects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDefinition {
    pub spec: RouteSpec,
    pub domain: RouteDomain,
    pub handler_id: String,
}

pub fn route_catalog() -> Vec<RouteDefinition> {
    let mut out = Vec::with_capacity(CONTRACT_ROUTES.len());
    let mut seen = BTreeSet::new();

    for (method, path) in CONTRACT_ROUTES {
        let spec = RouteSpec::new(*method, *path).expect("contract routes must be valid");
        assert!(
            seen.insert(spec.clone()),
            "duplicate route in contract list: {spec}"
        );

        out.push(RouteDefinition {
            domain: RouteDomain::from_path(spec.path.as_str()),
            handler_id: handler_id_for(spec.method, spec.path.as_str()),
            spec,
        });
    }

    out
}

fn handler_id_for(method: HttpMethod, path: &str) -> String {
    let mut tokens = Vec::new();
    tokens.push(method.as_str().to_ascii_lowercase());

    for part in path.trim_matches('/').split('/') {
        let normalized = if part.starts_with('{') && part.ends_with('}') {
            part.trim_matches('{')
                .trim_matches('}')
                .to_ascii_lowercase()
        } else {
            part.chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() {
                        ch.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        };
        tokens.push(normalized);
    }

    tokens.join("_")
}

const CONTRACT_ROUTES: &[(HttpMethod, &str)] = &[
    (HttpMethod::Get, "/health"),
    (HttpMethod::Post, "/auth/token"),
    (HttpMethod::Get, "/auth/tokens"),
    (HttpMethod::Delete, "/auth/tokens/{tokenId}"),
    (HttpMethod::Get, "/api/projects"),
    (HttpMethod::Post, "/api/projects"),
    (HttpMethod::Get, "/api/projects/{slug}"),
    (HttpMethod::Get, "/api/projects/{slug}/bootstrap-prompt"),
    (HttpMethod::Post, "/api/projects/{slug}/bootstrap-import"),
    (HttpMethod::Get, "/api/projects/{slug}/storage"),
    (HttpMethod::Put, "/api/projects/{slug}/storage/local"),
    (HttpMethod::Put, "/api/projects/{slug}/storage/s3"),
    (HttpMethod::Get, "/api/projects/{slug}/runs"),
    (HttpMethod::Post, "/api/projects/{slug}/runs/trigger"),
    (
        HttpMethod::Post,
        "/api/projects/{slug}/runs/validate-config",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}"),
    (HttpMethod::Get, "/api/projects/{slug}/runs/{runId}/jobs"),
    (HttpMethod::Get, "/api/projects/{slug}/assets"),
    (HttpMethod::Get, "/api/projects/{slug}/assets/{assetId}"),
    (HttpMethod::Get, "/api/projects/{slug}/asset-links"),
    (HttpMethod::Post, "/api/projects/{slug}/asset-links"),
    (HttpMethod::Get, "/api/projects/{slug}/asset-links/{linkId}"),
    (HttpMethod::Put, "/api/projects/{slug}/asset-links/{linkId}"),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/asset-links/{linkId}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/quality-reports"),
    (HttpMethod::Get, "/api/projects/{slug}/cost-events"),
    (HttpMethod::Get, "/api/projects/{slug}/exports"),
    (HttpMethod::Get, "/api/projects/{slug}/exports/{exportId}"),
    (HttpMethod::Get, "/api/projects/{slug}/prompt-templates"),
    (HttpMethod::Post, "/api/projects/{slug}/prompt-templates"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/prompt-templates/{templateId}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/prompt-templates/{templateId}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/prompt-templates/{templateId}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/provider-accounts"),
    (HttpMethod::Post, "/api/projects/{slug}/provider-accounts"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/provider-accounts/{providerCode}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/provider-accounts/{providerCode}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/provider-accounts/{providerCode}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/style-guides"),
    (HttpMethod::Post, "/api/projects/{slug}/style-guides"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/style-guides/{styleGuideId}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/style-guides/{styleGuideId}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/style-guides/{styleGuideId}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/characters"),
    (HttpMethod::Post, "/api/projects/{slug}/characters"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/characters/{characterId}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/characters/{characterId}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/characters/{characterId}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/reference-sets"),
    (HttpMethod::Post, "/api/projects/{slug}/reference-sets"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/reference-sets/{referenceSetId}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/reference-sets/{referenceSetId}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/reference-sets/{referenceSetId}",
    ),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/reference-sets/{referenceSetId}/items",
    ),
    (
        HttpMethod::Post,
        "/api/projects/{slug}/reference-sets/{referenceSetId}/items",
    ),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
    ),
    (
        HttpMethod::Put,
        "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
    ),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/chat/sessions"),
    (HttpMethod::Post, "/api/projects/{slug}/chat/sessions"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/chat/sessions/{sessionId}",
    ),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/chat/sessions/{sessionId}/messages",
    ),
    (
        HttpMethod::Post,
        "/api/projects/{slug}/chat/sessions/{sessionId}/messages",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/agent/instructions"),
    (HttpMethod::Post, "/api/projects/{slug}/agent/instructions"),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/agent/instructions/{instructionId}",
    ),
    (
        HttpMethod::Get,
        "/api/projects/{slug}/agent/instructions/{instructionId}/events",
    ),
    (
        HttpMethod::Post,
        "/api/projects/{slug}/agent/instructions/{instructionId}/confirm",
    ),
    (
        HttpMethod::Post,
        "/api/projects/{slug}/agent/instructions/{instructionId}/cancel",
    ),
    (HttpMethod::Get, "/api/projects/{slug}/secrets"),
    (HttpMethod::Post, "/api/projects/{slug}/secrets"),
    (
        HttpMethod::Delete,
        "/api/projects/{slug}/secrets/{providerCode}/{secretName}",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_route_count_is_stable() {
        assert_eq!(CONTRACT_ROUTES.len(), 72);
    }

    #[test]
    fn handler_ids_are_not_empty() {
        for route in route_catalog() {
            assert!(
                !route.handler_id.is_empty(),
                "handler id missing for {}",
                route.spec
            );
        }
    }
}
