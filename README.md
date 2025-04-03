# Rustproof

A fast, extensible code checker written in Rust. Rustproof uses the Language Server Protocol (LSP) to communicate with your editor and detect spelling mistakes in your code. It handles a multitude of casings by breaking words into individual components. It supports multiple natural and programming languages. Rustproof is built on top of [Hunspell](https://hunspell.github.io/), the spellchecker that powers LibreOffice. While Rustproof is primarily built for Neovim, it can be used in any editor that supports the Language Server Protocol.

## Why use this and not [cspell](https://cspell.org/)?

Since Rustproof is written in rust and the implementation is fairly simple it makes the server quite fast. It's also much easier to add more dictionaries when using an editor such as Neovim.

## How It Works

The concept is simple: split camelCase, PascalCase, and snake_case words before checking them against a list of known words.

- **camelCase** → camel case
- **PascalCase** → pascal case
- **snake_case_words** → snake case words
- **kebab-case-words** → kebab case words

## Things to Note

- The local dictionary is **case-insensitive**. It will not catch errors like "english," which should be "English." Hunspell dictionaries will flag these errors though!
- The spellchecker uses **dictionaries stored locally** and does **not** send data outside your machine.
- **Dictionaries may contain errors** and missing words.
- Only words longer than **three characters** are checked. For example, "jsj" is ignored, while "jsja" is checked.
- **Symbols and punctuation are ignored.** Except for single quotes in words such as `it's` and `wouldn't`

## Adding Dictionaries

- Since Rustproof is based on Hunspell, you can add many additional dictionaries. See [this repository](https://github.com/wooorm/dictionaries/tree/main/dictionaries) for more options.
- If Rustproof detects that the dictionaries provided are not available on the local machine it will download and cache them using the reqwest library

## LSP Initialization Options (`init_options`)

Configuration options passed during LSP initialization.

---

### `dict_path`

- **Type**: `string`
- **Default**: `<system-config-path>/rustproof/dict.txt`
- **Description**: Specifies the path to a local dictionary file. Words added via LSP actions (like "add to dictionary") will be saved here.
- **Example**: `dict_path = "~/.config/nvim/lsp/rustproof-dict.txt"`

---

### `diagnostic_severity`

- **Type**: `string`
- **Default**: `error`
- **Description**: Sets the severity level reported for spelling diagnostics in the editor. Common values might include `"error"`, `"warning"`, `"info"`, or `"hint"`.
- **Example**: `diagnostic_severity = "warning"`

---

### `dictionaries`

- **Type**: `table` (list of tables)
- **Description**: A list of dictionaries to load for spellchecking. Each entry in the list is a table specifying the dictionary details. The LSP will automatically download and cache these dictionaries on first use if they are not present locally.
- **Structure of each dictionary entry**:
  - `language`
    - **Type**: `string`
    - **Required**: Yes
    - **Description**: An identifier for the language (e.g., "en", "en_code", "sv"). Used internally and potentially for selecting dictionaries.
    - **Example**: `"en"`
  - `aff`
    - **Type**: `string`
    - **Required**: Yes
    - **Description**: The URL to the Hunspell affix (`.aff`) file for this dictionary.
    - **Example**: `"https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.aff"`
  - `dic`
    - **Type**: `string`
    - **Required**: Yes
    - **Description**: The URL to the Hunspell dictionary (`.dic`) file for this dictionary.
    - **Example**: `"https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.dic"`
- **Default**:
```lua
[
  {
    language = "en_AU_coding",
    aff = "https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.aff",
    dic = "https://raw.githubusercontent.com/maxmilton/hunspell-dictionary/refs/heads/master/en_AU.dic",
  },
  {
    language = "en_US",
    aff = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.aff",
    dic = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/en/index.dic",
  },
]
```


---

## Example neovim configuration

> [!INFO] /lsp/rustproof.lua

```lua
return {
  cmd = { "rustproof" },
  -- Specify which filetypes to spellcheck
  -- This is just a subset, rustproofs lexer supports a range of programming languages
  filetypes = { "rust", "lua", "ruby", "javascript", "toml", "vue" },
  init_options = {
    -- You can add words to a local dictionary, here you can specify a path for that dictionary
    dict_path = "~/.config/nvim/lsp/rustproof-dict.txt",
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
