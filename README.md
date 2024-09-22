# dprint-plugin-imporg

JavaScript/TypeScript import organizer plugin for dprint.

## Install

This plugin must be used together with the typescript plugin and listed before it in the plugins list.

```jsonc
{
  "plugins": [
    "https://plugins.dprint.dev/balthild/imporg-0.1.3.wasm",
    "https://plugins.dprint.dev/typescript-0.93.0.wasm"
  ]
}
```

## Config

```jsonc
{
  "imporg": {
    // The regex patterns that will be included by <alias> rule and excluded by <npm> rule.
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
        // Regex patterns or predefined rules (see the defaults above for examples).
        // Only the statements matched by `include` but not `exclude` will be placed in the group.
        // For example, this group will exclude "@balthild/a_momorepo_package" although <npm> rule
        // matches it.
        "include": ["<npm>"],
        "exclude": ["^@balthild/"]
      }
    ]
  }
}
```
