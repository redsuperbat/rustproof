# Rustproof

A fast, extensible code checker written in Rust.

Rustproof uses the Language Server Protocol (LSP) to communicate with your editor and detect spelling mistakes in your code.

It handles a multitude of casings by breaking words into individual components. It supports multiple natural and programming languages.

Rustproof is built on top of Hunspell, the spellchecker that powers LibreOffice.

While Rustproof is primarily built for Neovim, it can be used in any editor that supports the Language Server Protocol.

## How It Works

The concept is simple: split camelCase, PascalCase, and snake_case words before checking them against a list of known words.

- **camelCase** → camel case
- **PascalCase** → pascal case
- **snake_case_words** → snake case words
- **kebab-case-words** → kebab case words

## Things to Note

- The local spellchecker is **case-insensitive**. It will not catch errors like "english," which should be "English."
- The spellchecker uses **dictionaries stored locally** and does **not** send data outside your machine.
- **Dictionaries may contain errors** and missing words.
- Only words longer than **three characters** are checked. For example, "jsj" is ignored, while "jsja" is checked.
- **Symbols and punctuation are ignored.**

## Adding Dictionaries

Since Rustproof is based on Hunspell, you can add many additional dictionaries. See [this repository](https://github.com/wooorm/dictionaries/tree/main/dictionaries) for more options.

## LSP Configuration

### Neovim (/lsp/rustproof.lua)

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
