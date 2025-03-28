# Rustproof

A fast extensible code checker written in rust.

Rustproof uses the language server protocol to communicate with your editor and inform you about spelling mistakes in your code.

It handles camel and pascal case by breaking up the words. Supports a multitude of languages and programming languages.

Rustproof is built on top of Hunspell which is the spellchecker which powers libre office.

Rustproof is primarily built for neovim but can be used in any editor which supports the language server protocol.

## How it works

The concept is simple, split camelCase and snake_case words before checking them against a list of known words.

- camelCase -> camel case
- PascalCase -> pascal case
- snake_case_words -> snake case words
- kebab-case-words -> snake case words

## Things to note

- The local spellchecker is case insensitive. It will not catch errors like english which should be English.
- The spellchecker uses dictionaries stored locally. It does not send anything outside your machine.
- The words in the dictionaries can and do contain errors.
- There are missing words.
- Only words longer than 3 characters are checked. "jsj" is ok, while "jsja" is not.
- All symbols and punctuation are ignored.

## LSP Configuration 

### Neovim

**/lsp/rustproof.lua**

```lua
return {
  cmd = { "rustproof" },
  name = "rustproof",
  -- Specify which filetypes to spellcheck
  filetypes = { "rust", "lua", "ruby", "javascript", "toml", "vue" },
  root_dir = vim.fn.getcwd(),
  init_options = {
    -- You can add words to a local dictionary, here you can specify a path for that dictionary
    dict_path = "~/.config/nvim/lua/max/plugins/lsp/rustproof/dict.txt",
    -- Severity of the diagnostics in the editor
    diagnostic_severity = "warning",
    -- List of dictionaries to include in your spellchecker.
    -- Rustproof will automatically download and cache the dictionaries when you first start the lsp
    dictionaries = {
      {
        language = "en",
        aff = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.aff",
        dic = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.dic",
      },
      {
        language = "en_code",
        aff = "https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.aff",
        dic = "https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.dic",
      },
      {
        language = "sv",
        aff = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/sv/index.aff",
        dic = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/sv/index.dic",
      },
    },
  },
}
```
