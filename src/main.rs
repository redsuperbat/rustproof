use dashmap::{DashMap, DashSet};
use hunspell_rs::{CheckResult, Hunspell};
use lexer::Lexer;
use log::{debug, info};
use pipeline::Pipeline;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::str::FromStr;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod expander;
mod keywords;
mod lexer;
mod pipeline;

thread_local! {
  static SPELL_CHECKERS: Vec<Hunspell> = vec![
     Hunspell::new("./languages/en/en_US.aff", "./languages/en/en_US.dic"),
     Hunspell::new("./languages/en/en_AU.aff", "./languages/en/en_AU.dic")
   ]
}

type LanguageId = String;
type SourceCode = String;

struct Backend {
    client: Client,
    local_dict: DashSet<String>,
    sources: DashMap<String, (LanguageId, SourceCode)>,
}

impl Backend {
    async fn spell_check_code(&self, code: &str, language_id: &str) -> Vec<Diagnostic> {
        let lexer = Lexer::new(code);
        let tokens = Pipeline::new(&language_id).run(lexer);

        tokens
            .iter()
            .filter(|t| {
                SPELL_CHECKERS.with(|checkers| {
                    checkers
                        .iter()
                        .all(|c| c.check(&t.lexeme) == CheckResult::MissingInDictionary)
                })
            })
            .filter(|t| !self.local_dict.contains(&t.lexeme.to_lowercase()))
            .map(|t| {
                Diagnostic::new(
                    Range {
                        start: Position::new(t.start.line(), t.start.column()),
                        end: Position::new(t.end.line(), t.end.column()),
                    },
                    Some(DiagnosticSeverity::INFORMATION),
                    Some(NumberOrString::String(t.lexeme.to_string())),
                    None,
                    format!("Unknown word {}", t.lexeme),
                    None,
                    None,
                )
            })
            .collect::<Vec<_>>()
    }

    async fn spell_check_uri(&self, uri: Url) {
        let source = match self.sources.get(&uri.to_string()) {
            Some(v) => v,
            None => return,
        };
        debug!("{}:{}", source.0.len(), source.1.len());
        let misspelled_words = self.spell_check_code(&source.1, &source.0).await;
        debug!("{:?}", misspelled_words);
        self.client
            .publish_diagnostics(uri.clone(), misspelled_words, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Rustproof".to_string(),
                version: None,
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["replace.with.word".to_string(), "add.to.dict".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        debug!("shutdown!");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let misspelled_words = self
            .spell_check_code(
                &params.text_document.text,
                &params.text_document.language_id,
            )
            .await;
        self.sources.insert(
            params.text_document.uri.to_string(),
            (params.text_document.language_id, params.text_document.text),
        );
        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                misspelled_words,
                Some(params.text_document.version),
            )
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let Some(text) = params.text else {
            return;
        };
        let language_id = match self.sources.get(&params.text_document.uri.to_string()) {
            Some(v) => v.0.to_string(),
            None => return,
        };

        let misspelled_words = self.spell_check_code(&text, &language_id).await;
        self.sources
            .insert(params.text_document.uri.to_string(), (language_id, text));
        self.client
            .publish_diagnostics(params.text_document.uri.clone(), misspelled_words, None)
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        if params.context.diagnostics.is_empty() {
            return Ok(None);
        }
        let cursor_line = params.range.start.line;
        let cursor_col = params.range.start.character;

        let diagnostic_under_cursor = params.context.diagnostics.iter().find(|d| {
            d.range.start.line == cursor_line
                && d.range.start.character <= cursor_col
                && cursor_col < d.range.end.character
        });

        let word = match diagnostic_under_cursor {
            Some(t) => match &t.code {
                Some(t) => match t {
                    NumberOrString::String(s) => s,
                    NumberOrString::Number(_) => return Ok(None),
                },
                None => return Ok(None),
            },
            None => return Ok(None),
        };

        let mut suggestions = SPELL_CHECKERS
            .with(|checkers| {
                checkers
                    .iter()
                    .flat_map(|c| c.suggest(&word))
                    // Suggestions shorter than 2 characters are usually bad
                    .filter(|s| s.len() > 2)
                    // Take first four suggestions
                    .take(4)
                    .collect::<Vec<_>>()
            })
            .iter()
            .collect::<HashSet<_>>()
            .iter()
            .map(|w| {
                let title = format!("Replace with \"{}\"", w);
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: title.to_string(),
                    command: Some(Command {
                        title,
                        command: "replace.with.word".to_string(),
                        arguments: Some(vec![Value::String(w.to_string())]),
                    }),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();

        let title = format!("Add \"{word}\" to dictionary");
        suggestions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: title.to_string(),
            command: Some(Command {
                title,
                command: "add.to.dict".to_string(),
                arguments: Some(vec![
                    Value::String(word.to_string()),
                    Value::String(uri.to_string()),
                ]),
            }),
            ..Default::default()
        }));

        Ok(Some(suggestions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        let dict = &self.local_dict;

        match params.command.as_str() {
            "add.to.dict" => match params.arguments.as_slice() {
                [Value::String(word), Value::String(uri)] => {
                    debug!("Adding word to local dictionary");
                    dict.insert(word.to_string().to_lowercase());
                    let Ok(uri) = Url::from_str(uri) else {
                        return Ok(None);
                    };
                    self.spell_check_uri(uri).await;
                    return Ok(None);
                }
                _ => return Ok(None),
            },
            "replace.with.word" => {
                debug!("Replacing with a new word");

                return Ok(None);
            }
            _ => return Ok(None),
        };
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct InlayHintParams {
    path: String,
}

#[allow(unused)]
enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = InlayHintParams;
    const METHOD: &'static str = "custom/notification";
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        local_dict: DashSet::new(),
        sources: DashMap::new(),
    });

    info!("Started language server");
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod test {
    use hunspell_rs::{CheckResult, Hunspell};

    #[test]
    fn hunspell_works() {
        let spell = Hunspell::new("./languages/en/en_US.aff", "./languages/en/en_US.dic");
        let result = spell.check("hi");
        assert_eq!(result, CheckResult::FoundInDictionary);
    }
}
