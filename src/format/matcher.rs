use std::collections::HashSet;
use std::sync::LazyLock;

use crate::config::{Configuration, CustomRule, ImportGroup, Rule, RuleSet};
use crate::re;

use super::ImportElement;

pub struct Matcher<'a> {
    config: &'a Configuration,
}

impl<'a> Matcher<'a> {
    pub fn new(config: &'a Configuration) -> Self {
        Self { config }
    }

    pub fn matches_group(&self, group: &ImportGroup, element: &ImportElement<'a>) -> bool {
        if !self.matches_rules(&group.include, element) {
            return false;
        }

        !self.matches_rules(&group.exclude, element)
    }

    pub fn matches_rules(&self, rules: &RuleSet, element: &ImportElement<'a>) -> bool {
        for rule in rules.iter() {
            if self.matches_rule(rule, element) {
                return true;
            }
        }

        false
    }

    pub fn matches_rule(&self, rule: &Rule, element: &ImportElement<'a>) -> bool {
        match rule {
            Rule::Effect => self.matches_effect(element),
            Rule::Builtin => self.matches_builtin(element),
            Rule::Relative => self.matches_relative(element),
            Rule::Alias => self.matches_alias(element),
            Rule::Npm => self.matches_npm(element),
            Rule::Custom(custom) => self.matches_custom(custom, element),
        }
    }

    fn matches_effect(&self, element: &ImportElement<'a>) -> bool {
        element.decl.specifiers.is_none()
    }

    fn matches_builtin(&self, element: &ImportElement<'a>) -> bool {
        // node -e 'console.log(JSON.stringify(require("node:module").builtinModules))'
        static BUILTINS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
            HashSet::from([
                "_http_agent",
                "_http_client",
                "_http_common",
                "_http_incoming",
                "_http_outgoing",
                "_http_server",
                "_stream_duplex",
                "_stream_passthrough",
                "_stream_readable",
                "_stream_transform",
                "_stream_wrap",
                "_stream_writable",
                "_tls_common",
                "_tls_wrap",
                "assert",
                "assert/strict",
                "async_hooks",
                "buffer",
                "child_process",
                "cluster",
                "console",
                "constants",
                "crypto",
                "dgram",
                "diagnostics_channel",
                "dns",
                "dns/promises",
                "domain",
                "events",
                "fs",
                "fs/promises",
                "http",
                "http2",
                "https",
                "inspector",
                "inspector/promises",
                "module",
                "net",
                "os",
                "path",
                "path/posix",
                "path/win32",
                "perf_hooks",
                "process",
                "punycode",
                "querystring",
                "readline",
                "readline/promises",
                "repl",
                "stream",
                "stream/consumers",
                "stream/promises",
                "stream/web",
                "string_decoder",
                "sys",
                "timers",
                "timers/promises",
                "tls",
                "trace_events",
                "tty",
                "url",
                "util",
                "util/types",
                "v8",
                "vm",
                "wasi",
                "worker_threads",
                "zlib",
            ])
        });

        let module = element.module();
        module.starts_with("node:") || BUILTINS.contains(&module)
    }

    fn matches_relative(&self, element: &ImportElement<'a>) -> bool {
        element.module().starts_with('.')
    }

    fn matches_alias(&self, element: &ImportElement<'a>) -> bool {
        let module = element.module();

        for alias in &self.config.aliases {
            if alias.matches(module) {
                return true;
            }
        }

        false
    }

    fn matches_npm(&self, element: &ImportElement<'a>) -> bool {
        if self.matches_alias(element) {
            return false;
        }

        re!(r"^@?[0-9A-Za-z\-]").is_match(element.module())
    }

    fn matches_custom(&self, custom: &CustomRule, element: &ImportElement<'a>) -> bool {
        custom.matches(element.module())
    }
}
