// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tower_lsp::lsp_types::*;

/// Andromeda Language Server capabilities
pub fn create_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![
                ".".to_string(),
                " ".to_string(),
                "(".to_string(),
                "\"".to_string(),
                "'".to_string(),
            ]),
            work_done_progress_options: Default::default(),
            all_commit_characters: None,
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
        }),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![
                CodeActionKind::QUICKFIX,
                CodeActionKind::SOURCE_FIX_ALL,
                CodeActionKind::REFACTOR,
            ]),
            work_done_progress_options: Default::default(),
            resolve_provider: Some(false),
        })),
        document_formatting_provider: Some(OneOf::Left(true)),
        document_range_formatting_provider: Some(OneOf::Left(true)),
        document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
            first_trigger_character: ";".to_string(),
            more_trigger_character: Some(vec!["}".to_string(), "\n".to_string()]),
        }),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![
                "andromeda.applyAutoFix".to_string(),
                "andromeda.fixAll".to_string(),
                "andromeda.organizeImports".to_string(),
            ],
            work_done_progress_options: Default::default(),
        }),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
            retrigger_characters: None,
            work_done_progress_options: Default::default(),
        }),
        experimental: Some(serde_json::json!({
            "supportedDiagnosticTags": ["unnecessary", "deprecated"],
            "providesDiagnostics": true,
            "providesAutoFix": true,
        })),
        // All other capabilities are set to None since they're not implemented yet
        selection_range_provider: None,
        definition_provider: None,
        type_definition_provider: None,
        implementation_provider: None,
        references_provider: None,
        document_highlight_provider: None,
        document_symbol_provider: None,
        workspace_symbol_provider: None,
        code_lens_provider: None,
        rename_provider: None,
        document_link_provider: None,
        color_provider: None,
        folding_range_provider: None,
        declaration_provider: None,
        call_hierarchy_provider: None,
        semantic_tokens_provider: None,
        linked_editing_range_provider: None,
        moniker_provider: None,
        inline_value_provider: None,
        inlay_hint_provider: None,
        diagnostic_provider: None,
    }
}
