use config::{expand_tilde, Config};
use dashmap::DashMap;
use hunspell_rs::{CheckResult, Hunspell};
use lexer::Lexer;
use local_dictionary::LocalDictionary;
use parking_lot::RwLock;
use pipeline::Pipeline;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod config;
mod expander;
mod keywords;
mod lexer;
mod local_dictionary;
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
    config: RwLock<Config>,
    local_dict: LocalDictionary,
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
            .filter(|t| !self.local_dict.contains(&t.lexeme))
            .map(|t| {
                Diagnostic::new(
                    Range {
                        start: Position::new(t.start.line, t.start.col),
                        end: Position::new(t.end.line, t.end.col),
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

    async fn replace_with_word(&self, params: ExecuteCommandParams) {
        self.log_info("Replacing word").await;
        let [Value::String(uri)] = &params.arguments.as_slice() else {
            return;
        };
        let Ok(uri) = Url::from_str(uri) else { return };
        self.spell_check_uri(uri).await;
    }

    async fn add_to_dict(&self, params: ExecuteCommandParams) {
        self.log_info("Adding word to local dictionary").await;
        let [Value::String(word), Value::String(uri)] = &params.arguments.as_slice() else {
            return;
        };
        self.insert_into_local_dict(word);
        let Ok(uri) = Url::from_str(uri) else { return };
        self.spell_check_uri(uri).await;
    }

    async fn spell_check_uri(&self, uri: Url) {
        let Some(source) = self.sources.get(&uri.to_string()) else {
            return;
        };
        let misspelled_words = self.spell_check_code(&source.1, &source.0).await;
        self.client
            .publish_diagnostics(uri.clone(), misspelled_words, None)
            .await;
    }

    fn load_local_dict_from_file(&self) {
        let config = &self.config.read();
        if !config.dict_path.exists() {
            return;
        };
        let file = fs::read_to_string(&config.dict_path).expect("Unable to read dict");
        for w in file.split("\n") {
            self.local_dict.insert(w.to_string());
        }
    }

    fn insert_into_local_dict(&self, word: &str) {
        self.local_dict.insert(word.to_string());
        let path = &self.config.read().dict_path;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Unable to create config dir");
            }
        };

        let mut file = OpenOptions::new()
            .append(true)
            .create(true) // Create if it doesn't exist
            .open(path)
            .expect("Unable to open local dictionary");

        writeln!(file, "{word}").expect("Unable to append to local dictionary");
    }

    async fn load_config(&self, init: InitializeParams) {
        let Some(options) = init.initialization_options else {
            return;
        };
        let mut options: Config = match serde_json::from_value(options) {
            Ok(o) => o,
            Err(e) => {
                self.log_error(e).await;
                return;
            }
        };
        options.dict_path = expand_tilde(options.dict_path).expect("Invalid dict path");
        let mut config = self.config.write();
        *config = options
    }

    async fn log_error<T: Display>(&self, v: T) {
        self.client.log_message(MessageType::ERROR, v).await
    }
    async fn log_info<T: Display>(&self, v: T) {
        self.client.log_message(MessageType::INFO, v).await
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, init: InitializeParams) -> Result<InitializeResult> {
        self.load_config(init).await;
        self.load_local_dict_from_file();

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
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log_info("initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.log_info("shutdown").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info("opened file").await;
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
        let cursor_line = params.range.start.line;
        let cursor_col = params.range.start.character;
        let diagnostic_under_cursor = params.context.diagnostics.iter().find(|d| {
            d.range.start.line == cursor_line
                && d.range.start.character <= cursor_col
                && cursor_col < d.range.end.character
        });
        let Some(diagnostic_under_cursor) = diagnostic_under_cursor else {
            return Ok(None);
        };

        let Some(NumberOrString::String(word)) = diagnostic_under_cursor.code.as_ref() else {
            return Ok(None);
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
            // remove duplicates
            .collect::<HashSet<_>>()
            .iter()
            .map(|w| {
                let title = format!("Replace with \"{}\"", w);
                let mut changes = HashMap::new();
                changes.insert(
                    uri.clone(),
                    vec![TextEdit {
                        range: diagnostic_under_cursor.range,
                        new_text: w.to_string(),
                    }],
                );
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: title.to_string(),
                    command: Some(Command {
                        title,
                        command: "replace.with.word".to_string(),
                        arguments: Some(vec![Value::String(uri.to_string())]),
                    }),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
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
        match params.command.as_str() {
            "add.to.dict" => self.add_to_dict(params).await,
            "replace.with.word" => self.replace_with_word(params).await,
            _ => {}
        };
        return Ok(None);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        local_dict: LocalDictionary::new(),
        config: RwLock::new(Config::default()),
        sources: DashMap::new(),
    });

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
