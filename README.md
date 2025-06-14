# Rustproof

A fast, extensible code checker written in Rust. Rustproof uses the Language Server Protocol (LSP) to communicate with your editor and detect spelling mistakes in your code. It handles a multitude of casings by breaking words into individual components. It supports multiple natural and programming languages. Rustproof is built on top of [Hunspell](https://hunspell.github.io/), the spellchecker that powers LibreOffice. While Rustproof is primarily built for Neovim, it can be used in any editor that supports the Language Server Protocol.

![rustproof](https://github.com/user-attachments/assets/ad313dcb-fac7-4df7-afbb-47f95ecf2e2f)

## Why use this and not [cspell](https://cspell.org/)?

Since Rustproof is written in rust and the implementation is fairly simple it makes the server quite fast. It's also much easier to add more dictionaries when using an editor such as Neovim.

## Installation

The easiest is probably to clone the repository and build from source:

```sh
git clone https://github.com/redsuperbat/rustproof.git
cd rustproof
cargo build --release
```

There are also binaries available which are built when a new release happens. Those can be found under the releases section on [github](https://github.com/redsuperbat/rustproof/releases).

## How It Works

The concept is simple: split camelCase, PascalCase, and snake_case words before checking them against a list of known words.

- **camelCase** → camel case
- **PascalCase** → pascal case
- **snake_case_words** → snake case words
- **kebab-case-words** → kebab case words
- **JSONParser** → json parser
- **DataJSON** → data json
- **DataJSONParser** → data json parser

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

| Name                  | Type                     | Default                                   | Description                                                                                                                                                                          |
| --------------------- | ------------------------ | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `dict_path`           | `string`                 | `<system-config-path>/rustproof/dict.txt` | Specifies the path to a local dictionary file. Words added via LSP actions (like "add to dictionary") will be saved here.                                                            |
| `diagnostic_severity` | `string`                 | `error`                                   | Sets the severity level reported for spelling diagnostics in the editor. Values: `"error"`, `"warning"`, `"info"`, `"hint"`.                                                         |
| `dictionaries`        | `table` (list of tables) | _See default below_                       | A list of dictionaries to load for spellchecking. Each dictionary requires a `language`, `aff` (affix file URL), and `dic` (dictionary file URL). LSP will download/cache as needed. |

**Default dictionaries**:

```lua
[
  {
    language = "en-code",
    aff = "https://raw.githubusercontent.com/redsuperbat/rustproof/refs/heads/main/dictionaries/en-code/index.aff",
    dic = "https://raw.githubusercontent.com/redsuperbat/rustproof/refs/heads/main/dictionaries/en-code/index.dic",
  },
  {
    language = "en",
    aff = "https://raw.githubusercontent.com/redsuperbat/rustproof/refs/heads/main/dictionaries/en/index.aff",
    dic = "https://raw.githubusercontent.com/redsuperbat/rustproof/refs/heads/main/dictionaries/en/index.dic",
  },
]
```

---

## Example Neovim configuration

> [!NOTE]
> The lsp configuration file can be placed in /lsp/rustproof.lua

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
        language = "sv",
        aff = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/sv/index.aff",
        dic = "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/sv/index.dic",
      },
    },
  },
}
```
