use std::collections::HashSet;
use std::sync::Arc;

use hunspell_rs::{CheckResult, Hunspell};
use lexer::Lexer;
use log::{debug, info};
use pipeline::Pipeline;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard};
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

struct SafeLocalDictionary(Arc<Mutex<HashSet<String>>>);

impl SafeLocalDictionary {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    async fn take(&self) -> MutexGuard<'_, HashSet<String>> {
        self.0.lock().await
    }
}

struct Backend {
    client: Client,
    local_dict: SafeLocalDictionary,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
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

    // TODO: Implement this
    // - Initialize a parser based on filetype
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let language_id = params.text_document.language_id;
        debug!("file opened {language_id}");
        let src = params.text_document.text;
        let lexer = Lexer::new(src);
        let tokens = Pipeline::new(&language_id).run(lexer);

        let local_set = self.local_dict.take().await;

        let misspelled_words = tokens
            .iter()
            .filter(|t| {
                SPELL_CHECKERS.with(|checkers| {
                    checkers
                        .iter()
                        .all(|c| c.check(&t.lexeme) == CheckResult::MissingInDictionary)
                })
            })
            .filter(|t| !local_set.contains(&t.lexeme))
            .map(|t| {
                Diagnostic::new(
                    Range {
                        start: Position::new(t.start.line(), t.start.column() - 1),
                        end: Position::new(t.end.line(), t.end.column() - 1),
                    },
                    Some(DiagnosticSeverity::INFORMATION),
                    Some(NumberOrString::String(t.lexeme.to_string())),
                    None,
                    format!("Unknown word {}", t.lexeme),
                    None,
                    None,
                )
            })
            .collect::<Vec<_>>();

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                misspelled_words,
                Some(params.text_document.version),
            )
            .await;
    }

    // TODO: perhaps later
    async fn did_change(&self, _params: DidChangeTextDocumentParams) {
        debug!("file changed");
    }

    // TODO: Implement this
    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        debug!("file saved");
    }

    // TODO: Implement this
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
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
                arguments: Some(vec![Value::String(word.to_string())]),
            }),
            ..Default::default()
        }));

        Ok(Some(suggestions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("command executed!");
        let mut dict = self.local_dict.take().await;
        match params.command.as_str() {
            "add.to.dict" => match params.arguments.first() {
                Some(a) => match a {
                    Value::String(s) => {
                        dict.insert(s.to_string());
                        return Ok(None);
                    }
                    _ => return Ok(None),
                },
                None => return Ok(None),
            },
            "replace.with.word" => return Ok(None),
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
        local_dict: SafeLocalDictionary::new(),
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
