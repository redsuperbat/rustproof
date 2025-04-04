use clap::Parser;
use config::{expand_tilde, Config};
use crop::Rope;
use dashmap::DashMap;
use expander::Expandable;
use hunspell_rs::{CheckResult, Hunspell};
use lexer::{Lexer, Token};
use local_dictionary::LocalDictionary;
use log::info;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod config;
mod expander;
mod lexer;
mod local_dictionary;

type SourceCode = Rope;

struct Backend {
    client: Client,
    config: RwLock<Config>,
    local_dict: LocalDictionary,
    sources: DashMap<Url, SourceCode>,
    checker: RwLock<Option<mpsc::Sender<(String, oneshot::Sender<bool>)>>>,
    suggester: RwLock<Option<mpsc::Sender<(String, oneshot::Sender<Vec<String>>)>>>,
}

impl Backend {
    fn misspelled_tokens(&self, code: &SourceCode) -> Vec<Token> {
        Lexer::new(code.chars())
            .into_iter()
            // We ignore tokens with a lexeme shorter than 4 characters
            // Those are not relevant for spelling mistakes
            .filter(|t| t.lexeme.len() > 3)
            // Expand camelCase and PascalCase
            .flat_map(|t| {
                if let Some(t) = t.expand() {
                    return t;
                }
                return vec![t];
            })
            // After expansion the tokens could be broken into smaller ones
            // therefore we filter again the first is just a performance optimization
            .filter(|t| t.lexeme.len() > 3)
            // Hunspell spell-check
            .filter(|t| !self.spell_check(&t.lexeme))
            // Check against our local dictionary
            .filter(|t| !self.local_dict.contains(&t.lexeme))
            .collect()
    }

    fn spell_check_code(&self, code: &SourceCode) -> Vec<Diagnostic> {
        let severity = { self.config.read().diagnostic_severity.clone() };
        self.misspelled_tokens(code)
            .iter()
            .map(|t| Diagnostic {
                range: Range {
                    start: Position::new(t.start.line, t.start.col),
                    end: Position::new(t.end.line, t.end.col),
                },
                severity: Some(severity.to_lsp_diagnostic()),
                code: Some(NumberOrString::Number(1)),
                message: format!("Unknown word \"{}\"", t.lexeme),
                data: Some(Value::String(t.lexeme.to_string())),
                ..Default::default()
            })
            .collect()
    }

    async fn add_all_to_dict(&self, params: ExecuteCommandParams) {
        info!("Adding all spelling mistakes to local dict");
        let [Value::String(uri)] = &params.arguments.as_slice() else {
            return;
        };
        let Ok(uri) = Url::from_str(uri) else { return };
        let Some(source) = self.sources.get(&uri) else {
            return;
        };
        let misspelled_words = self
            .misspelled_tokens(&source)
            .into_iter()
            .map(|t| t.lexeme)
            .collect::<HashSet<_>>();

        for word in misspelled_words {
            self.insert_into_local_dict(&word);
        }
        self.spell_check_uri(uri).await;
    }

    async fn replace_with_word(&self, params: ExecuteCommandParams) {
        info!("Replacing word");
        let [Value::String(uri), range, Value::String(word)] = &params.arguments.as_slice() else {
            return;
        };
        let range = serde_json::from_value::<Range>(range.to_owned())
            .expect("Could not convert argument to range");
        let Ok(uri) = Url::from_str(uri) else { return };
        self.replace_word_in_source(&uri, &range, word);
        self.spell_check_uri(uri).await
    }

    fn replace_word_in_source(&self, uri: &Url, range: &Range, word: &str) {
        let Some(mut source) = self.sources.get_mut(&uri) else {
            return;
        };
        let start = source.byte_of_line(range.start.line as usize) + range.start.character as usize;
        let end = start + word.bytes().count();
        source.replace(start..end, word);
    }

    async fn add_to_dict(&self, params: ExecuteCommandParams) {
        info!("Adding word to local dictionary");
        let [Value::String(word), Value::String(uri)] = &params.arguments.as_slice() else {
            return;
        };
        self.insert_into_local_dict(word);
        let Ok(uri) = Url::from_str(uri) else { return };
        self.spell_check_uri(uri).await;
    }

    async fn spell_check_uri(&self, uri: Url) {
        let Some(source) = self.sources.get(&uri) else {
            return;
        };
        let diagnostics = self.spell_check_code(&source);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
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
        *self.config.write() = options;
    }

    async fn log_error<T: Display>(&self, v: T) {
        self.client.log_message(MessageType::ERROR, v).await
    }

    async fn start_spellchecker(&self) {
        let (suggester, suggester_tx) = mpsc::channel::<(String, oneshot::Sender<Vec<String>>)>();
        *self.suggester.write() = Some(suggester);
        let dicts = { self.config.read().dictionaries.clone() };

        let mut paths = Vec::with_capacity(dicts.len());

        for dict in dicts {
            let path = dict.resolve().await;
            paths.push(path);
        }

        let suggestion_paths = paths.clone();

        thread::spawn(move || {
            let checkers: Vec<_> = suggestion_paths
                .iter()
                .map(|p| Hunspell::new(p.aff.to_str().unwrap(), p.dic.to_str().unwrap()))
                .collect();

            while let Ok((word, send)) = suggester_tx.recv() {
                let suggestions = checkers
                    .iter()
                    .flat_map(|c| c.suggest(&word))
                    // Suggestions shorter than 2 characters are usually bad
                    .filter(|s| s.len() > 2)
                    // remove duplicates
                    .collect::<HashSet<_>>()
                    .into_iter()
                    // Take at most 6 suggestions
                    // TODO: Make this better
                    .take(6)
                    .collect::<Vec<_>>();

                let _ = send.send(suggestions);
            }
        });
        let (checker, checker_tx) = mpsc::channel::<(String, oneshot::Sender<bool>)>();
        *self.checker.write() = Some(checker);

        thread::spawn(move || {
            let checkers: Vec<_> = paths
                .iter()
                .map(|p| Hunspell::new(p.aff.to_str().unwrap(), p.dic.to_str().unwrap()))
                .collect();
            while let Ok((word, send)) = checker_tx.recv() {
                let result = &checkers
                    .iter()
                    .any(|c| c.check(&word) == CheckResult::FoundInDictionary);
                let _ = send.send(*result);
            }
        });
    }

    fn spell_check(&self, word: &str) -> bool {
        let (rx, tx) = oneshot::channel();
        let checker = self.checker.read();
        let _ = checker.as_ref().unwrap().send((word.to_string(), rx));
        tx.recv().unwrap_or(true)
    }

    fn suggest(&self, word: &str) -> Vec<String> {
        let (rx, tx) = oneshot::channel();
        let suggester = self.suggester.read();
        let _ = suggester.as_ref().unwrap().send((word.to_string(), rx));
        tx.recv().unwrap_or(vec![])
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, init: InitializeParams) -> Result<InitializeResult> {
        self.load_config(init).await;
        self.load_local_dict_from_file();
        self.start_spellchecker().await;

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
                    commands: vec![
                        "replace.with.word".to_string(),
                        "add.to.dict".to_string(),
                        "add.all.to.dict".to_string(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        info!("shutdown");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("opened file");
        let source = Rope::from(params.text_document.text);
        let uri = params.text_document.uri;
        self.sources.insert(uri.clone(), source);
        self.spell_check_uri(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("closed file");
        self.sources.remove(&params.text_document.uri);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let Some(text) = params.text else {
            return;
        };
        let source = Rope::from(text);
        let uri = params.text_document.uri;
        self.sources.insert(uri.clone(), source);
        self.spell_check_uri(uri).await;
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

        let Some(Value::String(word)) = diagnostic_under_cursor.data.as_ref() else {
            return Ok(None);
        };

        let mut code_actions = self
            .suggest(word)
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
                        arguments: Some(vec![
                            Value::String(uri.to_string()),
                            serde_json::to_value(diagnostic_under_cursor.range)
                                .expect("Could not convert range to value"),
                            Value::String(w.to_string()),
                        ]),
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
        code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
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

        let title = format!("Add all misspelled words in current file to local dictionary");
        code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: title.to_string(),
            command: Some(Command {
                title,
                command: "add.all.to.dict".to_string(),
                arguments: Some(vec![Value::String(uri.to_string())]),
            }),
            ..Default::default()
        }));

        Ok(Some(code_actions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        match params.command.as_str() {
            "add.to.dict" => self.add_to_dict(params).await,
            "replace.with.word" => self.replace_with_word(params).await,
            "add.all.to.dict" => self.add_all_to_dict(params).await,
            _ => {}
        };
        return Ok(None);
    }
}

/// A fast, extensible code checker. Rustproof uses the Language Server Protocol (LSP) to communicate with your editor and detect spelling mistakes in your code. It handles a multitude of casings by breaking words into individual components.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() {
    env_logger::init();
    Args::parse();
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        local_dict: LocalDictionary::new(),
        config: RwLock::new(Config::default()),
        sources: DashMap::new(),
        checker: RwLock::new(None),
        suggester: RwLock::new(None),
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
