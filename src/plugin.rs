use std::path::Path;

use anyhow::Result;
use dprint_core::configuration::{
    get_unknown_property_diagnostics, ConfigKeyMap, GlobalConfiguration,
};
use dprint_core::generate_plugin_code;
use dprint_core::plugins::{
    CheckConfigUpdatesMessage, ConfigChange, FileMatchingInfo, FormatResult, PluginInfo,
    PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest, SyncPluginHandler,
};

use crate::config::{resolve_config, Configuration};
use crate::format::format_source;

pub struct ImporgHandler;

impl SyncPluginHandler<Configuration> for ImporgHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "imporg".to_string(),
            help_url: env!("CARGO_PKG_REPOSITORY").to_string(),
            config_schema_url: "".to_string(),
            update_url: Some(
                "https://plugins.dprint.dev/balthild/dprint-plugin-imporg/latest.json".to_string(),
            ),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../LICENSE").to_string()
    }

    fn resolve_config(
        &mut self,
        mut raw: ConfigKeyMap,
        _global: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Configuration> {
        let mut diagnostics = Vec::new();

        let resolved = match resolve_config(&mut raw) {
            Ok(it) => it,
            Err(err) => {
                diagnostics.push(err);
                Configuration::empty()
            }
        };

        diagnostics.extend(get_unknown_property_diagnostics(raw));

        PluginResolveConfigurationResult {
            config: resolved,
            diagnostics,
            file_matching: FileMatchingInfo {
                file_extensions: vec![
                    "js".to_string(),
                    "mjs".to_string(),
                    "jsx".to_string(),
                    "ts".to_string(),
                    "mts".to_string(),
                    "tsx".to_string(),
                ],
                file_names: vec![],
            },
        }
    }

    fn check_config_updates(
        &self,
        _message: CheckConfigUpdatesMessage,
    ) -> Result<Vec<ConfigChange>> {
        Ok(vec![])
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<Configuration>,
        mut format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        let source = match request.range {
            Some(ref range) => &request.file_bytes[range.clone()],
            None => &request.file_bytes,
        };

        let source = std::str::from_utf8(source)?;

        let mut output = format_source(request.config, request.file_path, source)?;
        let mut output_range = None;

        if let Some(range) = request.range {
            let formatted_len = output.len_bytes();
            output_range = Some(range.start..(range.start + formatted_len));

            output.insert(0, &source[..range.start]);
            output.insert(output.len_chars(), &source[range.end..]);
        };

        format_with_host(SyncHostFormatRequest {
            // Hack: Imporg does not format CommonJS files, so this avoids infinite recursion.
            file_path: Path::new("dummy.cts"),
            file_bytes: output.to_string().as_bytes(),
            range: output_range,
            override_config: &ConfigKeyMap::from([(
                "module.sortImportDeclarations".to_string(),
                "maintain".into(),
            )]),
        })
    }
}

generate_plugin_code!(ImporgHandler, ImporgHandler, Configuration);
