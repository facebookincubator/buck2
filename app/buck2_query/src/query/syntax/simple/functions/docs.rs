/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::borrow::Cow;
use std::fmt::Write;

use indexmap::IndexMap;
use itertools::Itertools;

use crate::query::syntax::simple::functions::helpers::QueryArgType;

pub struct MarkdownOptions {
    /// Termimad doesn't support links, see <https://github.com/Canop/termimad/issues/48>.
    /// Flag to disable generation of those for terminal / rendered outputs.
    pub links_enabled: bool,
}

// Instances created by #[query_module]
pub struct ArgDescription {
    pub name: String,
    pub repr_format: String,
    pub arg_type: QueryArgType,
}

impl ArgDescription {
    pub fn render_markdown(&self, options: &MarkdownOptions) -> String {
        format!(
            "{}: {}",
            &self.name,
            self.repr_format
                .replace("{}", &self.arg_type.rendered_reference(options))
        )
    }
}

// Instances created by #[query_module]
pub struct FunctionDescription {
    pub name: &'static str,
    pub args: Vec<ArgDescription>,

    pub short_help: Option<String>,
    pub details: Option<String>,
}

impl FunctionDescription {
    fn rendered_reference(&self, options: &MarkdownOptions) -> Cow<'static, str> {
        if options.links_enabled {
            Cow::Owned(format!("[{}](#{})", self.name, self.name))
        } else {
            Cow::Borrowed(self.name)
        }
    }

    pub fn render_short_markdown(&self, options: &MarkdownOptions) -> String {
        format!(
            " - {}({}){}",
            self.rendered_reference(options),
            self.args
                .iter()
                .map(|v| v.render_markdown(options))
                .join(", "),
            &match &self.short_help {
                None => "".to_owned(),
                Some(v) => format!(": {}", v),
            }
        )
    }

    pub fn render_markdown(&self, options: &MarkdownOptions) -> String {
        let anchor = if options.links_enabled {
            &format!("{{#{}}}", self.name)
        } else {
            ""
        };
        let mut rendered = format!(
            "### {}({}){}\n\n",
            self.name,
            self.args
                .iter()
                .map(|v| v.render_markdown(options))
                .join(", "),
            anchor,
        );
        if let Some(v) = &self.short_help {
            writeln!(rendered, "{}\n", v).unwrap();
        }

        if let Some(v) = &self.details {
            writeln!(rendered, "{}\n", v).unwrap();
        }
        rendered
    }
}

// Instances created by #[query_module]
pub struct ModuleDescription {
    pub functions: IndexMap<&'static str, FunctionDescription>,

    pub short_help: Option<String>,
    pub details: Option<String>,
}

impl ModuleDescription {
    pub fn render_markdown(&self, options: &MarkdownOptions) -> String {
        let mut rendered = format!(
            "## {}\n\n",
            match &self.short_help {
                Some(v) => v,
                None => "Query functions",
            }
        );
        if let Some(v) = &self.details {
            writeln!(rendered, "{}\n", v).unwrap();
        }

        for (_, func) in &self.functions {
            writeln!(rendered, "{}", func.render_short_markdown(options)).unwrap();
        }

        for (_, func) in &self.functions {
            writeln!(rendered, "{}\n", func.render_markdown(options)).unwrap();
        }

        rendered
    }
}

pub struct QueryEnvironmentDescription {
    pub name: String,
    pub mods: Vec<ModuleDescription>,
}

impl QueryEnvironmentDescription {
    pub fn render_markdown(&self, options: &MarkdownOptions) -> String {
        let functions = self
            .mods
            .iter()
            .map(|v| v.render_markdown(options))
            .join("\n\n");
        let value_types = enum_iterator::all::<QueryArgType>()
            .map(|v| render_arg_type_markdown(v, options))
            .join("\n\n");
        format!(
            indoc::indoc! {r#"
            # {}

            {}

            ## Value Types
            {}
            "#},
            &self.name, functions, value_types
        )
    }
}

fn render_arg_type_markdown(v: QueryArgType, options: &MarkdownOptions) -> String {
    let anchor = if options.links_enabled {
        &format!("<a name=\"{}\"></a>", v.internal_link_id())
    } else {
        ""
    };
    let mut rendered = format!("- *{}*{}: ", v.repr(), anchor);
    if let Some(short_description) = v.short_description(options) {
        rendered.push_str(short_description.as_ref());
    }
    if let Some(description) = v.description() {
        rendered.push_str(&format!("\n\n  {}", description));
    }
    rendered
}
