//! Async Borders (Study S6's twin): an MCP-style tool
//! surface where the discipline is *schemas first, async only
//! at the border, the core synchronous and deterministic*.
//!
//! A tool is a name, a declared argument schema, and a
//! synchronous handler. The border validates every call
//! against the schema **before** the handler runs — a schema
//! lie (a missing field, a wrong type, an undeclared field)
//! is refused at the border and the handler never sees it.
//! The async part (in a real surface, the transport) is
//! modeled as an explicit queue of pending calls: calls are
//! submitted, then drained in order through the synchronous
//! core. Nothing about the drain is concurrent, so nothing
//! about the result is nondeterministic.

/// The schema types a tool may declare.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldType {
    /// A string argument.
    Text,
    /// An integer argument.
    Int,
    /// A boolean argument.
    Bool,
}

/// One declared argument in a tool's schema.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Field {
    /// Argument name.
    pub name: &'static str,
    /// Argument type.
    pub ty: FieldType,
    /// Whether the call must carry it.
    pub required: bool,
}

/// A call argument value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A string value.
    Text(String),
    /// An integer value.
    Int(i64),
    /// A boolean value.
    Bool(bool),
}

impl Value {
    /// The schema type of this value.
    #[must_use]
    pub fn ty(&self) -> FieldType {
        match self {
            Value::Text(_) => FieldType::Text,
            Value::Int(_) => FieldType::Int,
            Value::Bool(_) => FieldType::Bool,
        }
    }
}

/// Why the border refused a call.
#[derive(Clone, Debug, PartialEq)]
pub enum BorderError {
    /// The tool name is not registered.
    UnknownTool {
        /// The name offered.
        tool: String,
    },
    /// A required argument is absent.
    MissingField {
        /// The argument missing.
        field: &'static str,
    },
    /// An argument's value does not match its declared type.
    WrongType {
        /// The argument.
        field: &'static str,
        /// The declared type.
        expected: FieldType,
        /// The type actually offered.
        got: FieldType,
    },
    /// An argument was offered that the schema never
    /// declared — refused, never ignored (a caller inventing
    /// fields is a caller lying to the schema).
    UndeclaredField {
        /// The undeclared argument.
        field: String,
    },
}

/// One tool: name, schema, handler. The handler is
/// synchronous — the core stays synchronous; only the border
/// is async-shaped.
pub struct Tool {
    // Debug is implemented manually below: the handler is a
    // function pointer, which has no useful Debug of its own.
    /// Tool name.
    pub name: &'static str,
    /// Declared arguments.
    pub schema: Vec<Field>,
    /// The synchronous handler: validated args in, a text
    /// result out.
    pub handler: fn(&[(String, Value)]) -> String,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("schema", &self.schema)
            .field("handler", &"<fn>")
            .finish()
    }
}

/// The border: a registry of tools plus a pending-call queue.
#[derive(Default)]
pub struct ToolBorder {
    tools: Vec<Tool>,
    pending: Vec<(String, Vec<(String, Value)>)>,
}

impl ToolBorder {
    /// An empty border.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Registered tool names, in registration order.
    #[must_use]
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.iter().map(|t| t.name).collect()
    }

    /// Validate a call against its tool's schema. Pure: the
    /// handler is not run.
    ///
    /// # Errors
    /// The first [`BorderError`] the call commits.
    pub fn validate(&self, tool: &str, args: &[(String, Value)]) -> Result<&Tool, BorderError> {
        let Some(tool) = self.tools.iter().find(|t| t.name == tool) else {
            return Err(BorderError::UnknownTool {
                tool: tool.to_string(),
            });
        };
        for (name, value) in args {
            let Some(field) = tool.schema.iter().find(|f| f.name == name) else {
                return Err(BorderError::UndeclaredField {
                    field: name.clone(),
                });
            };
            if field.ty != value.ty() {
                return Err(BorderError::WrongType {
                    field: field.name,
                    expected: field.ty,
                    got: value.ty(),
                });
            }
        }
        for field in &tool.schema {
            if field.required && !args.iter().any(|(n, _)| n == field.name) {
                return Err(BorderError::MissingField { field: field.name });
            }
        }
        Ok(tool)
    }

    /// Submit a call to the pending queue (the async side of
    /// the border). Validation happens at drain time, against
    /// the registry as it then stands — submissions are
    /// cheap and unprivileged.
    pub fn submit(&mut self, tool: &str, args: Vec<(String, Value)>) {
        self.pending.push((tool.to_string(), args));
    }

    /// Pending call count.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    /// Drain the queue through the synchronous core, in
    /// submission order: each call is validated at the border,
    /// then its handler runs. Refusals are returned in place
    /// of results — one bad call refuses itself, never the
    /// batch.
    pub fn drain(&mut self) -> Vec<Result<String, BorderError>> {
        let pending = std::mem::take(&mut self.pending);
        pending
            .iter()
            .map(|(tool, args)| {
                let t = self.validate(tool, args)?;
                Ok((t.handler)(args))
            })
            .collect()
    }
}

/// The exercise's reference tools: a tiny registry that
/// demonstrates the shape (an echo, a counter of beats).
#[must_use]
pub fn reference_border() -> ToolBorder {
    let mut border = ToolBorder::new();
    border.register(Tool {
        name: "echo",
        schema: vec![Field {
            name: "text",
            ty: FieldType::Text,
            required: true,
        }],
        handler: |args| {
            args.iter()
                .find(|(n, _)| n == "text")
                .map(|(_, v)| match v {
                    Value::Text(s) => s.clone(),
                    _ => String::new(),
                })
                .unwrap_or_default()
        },
    });
    border.register(Tool {
        name: "beat_at",
        schema: vec![
            Field {
                name: "index",
                ty: FieldType::Int,
                required: true,
            },
            Field {
                name: "loud",
                ty: FieldType::Bool,
                required: false,
            },
        ],
        handler: |args| {
            let index = args
                .iter()
                .find(|(n, _)| n == "index")
                .map(|(_, v)| match v {
                    Value::Int(i) => *i,
                    _ => 0,
                })
                .unwrap_or(0);
            format!("beat {} lands at tick {}", index, index * 30)
        },
    });
    border
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_args(pairs: &[(&str, Value)]) -> Vec<(String, Value)> {
        pairs
            .iter()
            .map(|(n, v)| (n.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn lawful_calls_validate_and_run() {
        let mut border = reference_border();
        assert_eq!(border.tool_names(), vec!["echo", "beat_at"]);
        border.submit(
            "echo",
            text_args(&[("text", Value::Text("and — four".into()))]),
        );
        border.submit("beat_at", text_args(&[("index", Value::Int(4))]));
        let results = border.drain();
        assert_eq!(results[0], Ok("and — four".to_string()));
        assert_eq!(results[1], Ok("beat 4 lands at tick 120".to_string()));
        assert_eq!(border.pending(), 0);
    }

    #[test]
    fn optional_fields_may_be_omitted() {
        let border = reference_border();
        assert!(
            border
                .validate("beat_at", &text_args(&[("index", Value::Int(1))]))
                .is_ok()
        );
    }

    // ---- adversarial ----

    #[test]
    fn schema_lies_are_refused_at_the_border() {
        let border = reference_border();
        assert_eq!(
            border.validate("echo", &[]).unwrap_err(),
            BorderError::MissingField { field: "text" }
        );
        assert_eq!(
            border
                .validate("echo", &text_args(&[("text", Value::Int(3))]))
                .unwrap_err(),
            BorderError::WrongType {
                field: "text",
                expected: FieldType::Text,
                got: FieldType::Int
            }
        );
        assert_eq!(
            border
                .validate(
                    "echo",
                    &text_args(&[
                        ("text", Value::Text("x".into())),
                        ("extra", Value::Bool(true))
                    ])
                )
                .unwrap_err(),
            BorderError::UndeclaredField {
                field: "extra".to_string()
            }
        );
        assert_eq!(
            border.validate("nope", &[]).unwrap_err(),
            BorderError::UnknownTool {
                tool: "nope".to_string()
            }
        );
    }

    #[test]
    fn one_bad_call_refuses_itself_not_the_batch() {
        let mut border = reference_border();
        border.submit("echo", text_args(&[("text", Value::Text("ok".into()))]));
        border.submit("echo", text_args(&[]));
        border.submit("beat_at", text_args(&[("index", Value::Int(2))]));
        let results = border.drain();
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok(), "the batch survives one caller's lie");
    }

    #[test]
    fn drain_is_ordered_and_deterministic() {
        let run = || {
            let mut border = reference_border();
            for i in 0..5 {
                border.submit("beat_at", text_args(&[("index", Value::Int(i))]));
            }
            border.drain()
        };
        assert_eq!(run(), run());
    }
}
