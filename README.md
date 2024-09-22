# dprint-plugin-imporg

JavaScript/TypeScript import organizer plugin for dprint.

## Install

This plugin must be used together with the typescript plugin and listed before it in the plugins list.

```jsonc
{
  "plugins": [
    "https://github.com/balthild/dprint-plugin-imporg/releases/download/0.1.1/dprint_plugin_imporg.wasm",
    "https://plugins.dprint.dev/typescript-0.93.0.wasm"
  ]
}
```

## Config

```jsonc
{
  "imporg": {
    // The regex patterns for the <alias> rule.
    // Default: ["^[@~]/"]
    "aliases": ["^@/", "^virtual:"],

    // Groups are matched in order. If an import statement could be matched by two groups, it will
    // be placed in the one appears first in the config.
    "groups": [
      // Defaults
      { "include": ["<effect>"] },
      { "include": ["<builtin>"] },
      { "include": ["<npm>"] },
      { "include": ["<alias>"] },
      { "include": ["<relative>"] },

      // Custom
      {
        // Regex patterns or special rules. See above the defaults for examples of special rules.
        // The import statements matching `incluce` but not `exclude` will be placed in the group.
        // For example, this group will exclude "@components/Sidebar.tsx", even though the <npm>
        // rule matches it.
        "include": ["<npm>"],
        "exclude": ["^@(pages|components|hooks)/"]
      }
    ]
  }
}
```
