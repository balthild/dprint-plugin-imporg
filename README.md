# dprint-plugin-imporg

JavaScript/TypeScript import organizer plugin for dprint.

## Install

This plugin must be used together with the typescript plugin and listed before it in the plugins list.

```jsonc
{
  "plugins": [
    "https://github.com/balthild/dprint-plugin-imporg/releases/download/0.1.0/dprint_plugin_imporg.wasm",
    "https://plugins.dprint.dev/typescript-0.93.0.wasm"
  ]
}
```

## Config

```jsonc
{
  "imporg": {
    // The regex patterns which will be matched by the <alias> rule.
    // Default: ["^[@~]/"]
    "aliases": ["^@/", "^virtual:"],
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
        // The import statements matching `incluce` but not `exclude` will be placed in this group.
        "include": [],
        "exclude": []
      }
    ]
  }
}
```
