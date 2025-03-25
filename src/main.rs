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

    async fn add(&mut self, word: &str) {
        self.0.lock().await.insert(word.to_string());
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
                    commands: vec!["replace.with.word".to_string()],
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
        let word = match params.context.diagnostics.first() {
            Some(t) => match &t.code {
                Some(t) => match t {
                    NumberOrString::String(s) => s,
                    NumberOrString::Number(_) => return Ok(None),
                },
                None => return Ok(None),
            },
            None => return Ok(None),
        };

        let suggestions = SPELL_CHECKERS
            .with(|checkers| {
                checkers
                    .iter()
                    .flat_map(|c| c.suggest(&word))
                    .collect::<Vec<_>>()
            })
            .iter()
            .collect::<HashSet<_>>()
            .iter()
            .map(|w| {
                let title = format!("Replace with \"{}\"", w);
                CodeActionOrCommand::CodeAction(CodeAction {
                    kind: Some(CodeActionKind::REFACTOR_INLINE),
                    title: title.to_string(),
                    command: Some(Command {
                        title,
                        command: "replace.with.word".to_string(),
                        arguments: None,
                    }),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();
        Ok(Some(suggestions))
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("command executed!");

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
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
