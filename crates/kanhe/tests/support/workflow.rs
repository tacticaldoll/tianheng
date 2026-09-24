//! A GitHub Actions workflow, parsed once and held as the structure the checks judge.
//!
//! **Parsed, not read by indentation.** Each check that judged `ci.yml` used to decide the structure itself,
//! line by line, and each decided it differently: one read every `env:` in the file as a job's and expanded
//! it into every job's run lines, one matched `node-version:` at any depth, one derived depths from the first
//! line it met. What a value *belongs to* — which job, which step — is the grammar's answer, so it is asked of
//! a parser and the checks read the answer.
//!
//! **A shape this model cannot hold is refused, never skipped.** An anchor, an alias, a merge key, a tag, a
//! second document, a duplicated key, or a block whose kind the workflow schema does not admit makes
//! [`parse`] return `Err`, because the alternative is a check reporting clean over a value it never read.
//! Each refusal names what was met and where.
//!
//! **What this holds is structure, and two things a workflow carries are not.** A comment is dropped by the
//! grammar, and what a `run:` body says *as shell* is not YAML. A check about either reads the workflow's text,
//! and says so where it does — the model is not the only reader of this file, and is not meant to be.

use std::collections::BTreeMap;

use yaml_rust2::Event;
use yaml_rust2::parser::{MarkedEventReceiver, Parser};
use yaml_rust2::scanner::Marker;

/// One YAML node, with the line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Scalar {
        value: String,
        line: usize,
    },
    Sequence {
        items: Vec<Node>,
        line: usize,
    },
    /// `column` is the zero-based column its first key starts at. The parser marks a mapping's start after
    /// that key rather than at it — measured, `      - name: one` marks column 12, the end of `name` — so the
    /// column is taken from the key's own scalar, which it marks at the token's start.
    Mapping {
        entries: Vec<Entry<Node>>,
        line: usize,
        column: usize,
    },
}

/// A mapping's key, the line it was written on, and its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<V> {
    pub key: String,
    pub line: usize,
    pub value: V,
}

impl Node {
    pub fn line(&self) -> usize {
        match self {
            Node::Scalar { line, .. }
            | Node::Sequence { line, .. }
            | Node::Mapping { line, .. } => *line,
        }
    }

    /// The value of `key`, when this node is a mapping that has it.
    pub fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Mapping { entries, .. } => entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|e| &e.value),
            _ => None,
        }
    }

    /// Every mapping key at or below this node, with the line it sits on.
    pub fn keys_below(&self) -> Vec<(String, usize)> {
        let mut found = Vec::new();
        self.collect_keys(&mut found);
        found
    }

    fn collect_keys(&self, found: &mut Vec<(String, usize)>) {
        match self {
            Node::Scalar { .. } => {}
            Node::Sequence { items, .. } => items.iter().for_each(|item| item.collect_keys(found)),
            Node::Mapping { entries, .. } => {
                for entry in entries {
                    found.push((entry.key.clone(), entry.line));
                    entry.value.collect_keys(found);
                }
            }
        }
    }
}

/// A workflow: its trigger block, its workflow-level environment and shell, and its jobs in order.
#[derive(Debug)]
pub struct Workflow {
    pub on: Option<Node>,
    pub env: Vec<Entry<String>>,
    pub defaults_shell: Option<Entry<String>>,
    pub jobs: Vec<Job>,
}

/// One job: its id, the keys written on it, its own environment and shell, and its steps.
#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub line: usize,
    pub keys: Vec<(String, usize)>,
    pub env: Vec<Entry<String>>,
    pub defaults_shell: Option<Entry<String>>,
    pub steps: Vec<Step>,
}

/// One step and the lines it spans — from its first key to the line before the next step, or before whatever
/// follows `steps` in its job — and the column its keys sit at.
#[derive(Debug)]
pub struct Step {
    pub first_line: usize,
    pub last_line: usize,
    pub column: usize,
    pub uses: Option<Entry<String>>,
    pub with: Vec<Entry<String>>,
    pub env: Vec<Entry<String>>,
    pub shell: Option<Entry<String>>,
    pub run: Option<Entry<String>>,
}

impl Step {
    /// Whether the comment `text` on `line` is this step's: within its lines, and indented at least as deep as
    /// its keys. YAML attaches a comment to nothing, so both halves are needed — a comment between the last step
    /// and a job key written after `steps` is within the lines by position and is the job's by its depth.
    pub fn holds_comment_at(&self, line: usize, text: &str) -> bool {
        let indent = text.len() - text.trim_start().len();
        (self.first_line..=self.last_line).contains(&line) && indent >= self.column
    }
}

impl Workflow {
    /// The environment a step's `run:` sees: the workflow's, then the job's, then the step's own, each
    /// overriding the one before — GitHub's own scoping, so a value is read only where it is in force.
    pub fn env_for(&self, job: &Job, step: &Step) -> BTreeMap<String, String> {
        self.env
            .iter()
            .chain(&job.env)
            .chain(&step.env)
            .map(|entry| (entry.key.clone(), entry.value.clone()))
            .collect()
    }
}

/// Parse a workflow, refusing any shape the model cannot hold.
///
/// # Errors
///
/// A description of what was met and on which line, for YAML that does not parse and for every shape named
/// in the module header.
pub fn parse(text: &str) -> Result<Workflow, String> {
    let root = document(text)?;
    let Node::Mapping { entries, .. } = &root else {
        return Err(format!(
            "line {}: a workflow is a mapping, and this is not one",
            root.line()
        ));
    };
    let lines = text.lines().count();
    let top = |key: &str| entries.iter().find(|entry| entry.key == key);

    let env = match top("env") {
        Some(entry) => scalar_mapping(&entry.value, "env")?,
        None => Vec::new(),
    };
    let defaults_shell = match top("defaults") {
        Some(entry) => shell_of(&entry.value)?,
        None => None,
    };
    let jobs = match top("jobs") {
        Some(entry) => jobs_of(&entry.value, lines)?,
        None => Vec::new(),
    };
    Ok(Workflow {
        on: top("on").map(|entry| entry.value.clone()),
        env,
        defaults_shell,
        jobs,
    })
}

/// Whether line `line` of `text` is a comment or blank rather than part of a value.
///
/// **Asked of the grammar, not of the line.** YAML drops a comment and keeps data, so blanking the line and
/// parsing again answers it: the tree is unchanged exactly when the line carried nothing the grammar keeps.
/// That covers every way a line can be data — a literal or folded block scalar, a quoted scalar spanning lines,
/// a plain one continued — where reading the line's text, or a scalar's folded value, cannot: folding joins a
/// content line to its neighbour, so a value no longer shows where its lines began. Blanking keeps the line
/// count, so every other node keeps its position and the trees compare as they are.
///
/// # Errors
///
/// Where `text` itself does not parse into a tree this model holds.
pub fn is_comment_line(text: &str, line: usize) -> Result<bool, String> {
    let before = document(text)?;
    let blanked: String = text
        .lines()
        .enumerate()
        .map(|(index, content)| if index + 1 == line { "" } else { content })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(document(&blanked).is_ok_and(|after| after == before))
}

/// `defaults: run: shell:`, read on a workflow or on a job.
fn shell_of(defaults: &Node) -> Result<Option<Entry<String>>, String> {
    let Some(run) = defaults.get("run") else {
        return Ok(None);
    };
    match run.get("shell") {
        Some(Node::Scalar { value, line }) => Ok(Some(Entry {
            key: "shell".to_string(),
            line: *line,
            value: value.clone(),
        })),
        Some(other) => Err(format!(
            "line {}: `defaults.run.shell` is not a scalar",
            other.line()
        )),
        None => Ok(None),
    }
}

fn jobs_of(jobs: &Node, lines: usize) -> Result<Vec<Job>, String> {
    let Node::Mapping { entries, .. } = jobs else {
        return Err(format!("line {}: `jobs` is not a mapping", jobs.line()));
    };
    let mut read = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let Node::Mapping { entries: body, .. } = &entry.value else {
            return Err(format!(
                "line {}: job `{}` is not a mapping",
                entry.line, entry.key
            ));
        };
        // A job's last line is the line before the next job's id, or the end of the document.
        let ends = entries.get(index + 1).map_or(lines, |next| next.line - 1);
        let field = |key: &str| body.iter().find(|field| field.key == key);
        // `steps` ends before whichever of the job's own keys follows it, or where the job does.
        let steps = match body.iter().position(|field| field.key == "steps") {
            Some(at) => {
                let steps_end = body.get(at + 1).map_or(ends, |next| next.line - 1);
                steps_of(&body[at].value, steps_end)?
            }
            None => Vec::new(),
        };
        read.push(Job {
            id: entry.key.clone(),
            line: entry.line,
            keys: body
                .iter()
                .map(|field| (field.key.clone(), field.line))
                .collect(),
            env: match field("env") {
                Some(env) => scalar_mapping(&env.value, "env")?,
                None => Vec::new(),
            },
            defaults_shell: match field("defaults") {
                Some(defaults) => shell_of(&defaults.value)?,
                None => None,
            },
            steps,
        });
    }
    Ok(read)
}

fn steps_of(steps: &Node, steps_end: usize) -> Result<Vec<Step>, String> {
    let Node::Sequence { items, .. } = steps else {
        return Err(format!("line {}: `steps` is not a sequence", steps.line()));
    };
    let mut read = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let Node::Mapping {
            entries,
            line,
            column,
        } = item
        else {
            return Err(format!("line {}: a step is not a mapping", item.line()));
        };
        let last_line = items
            .get(index + 1)
            .map_or(steps_end, |next| next.line() - 1);
        let field = |key: &str| entries.iter().find(|field| field.key == key);
        let scalar = |key: &str| -> Result<Option<Entry<String>>, String> {
            match field(key) {
                Some(Entry {
                    value: Node::Scalar { value, line },
                    ..
                }) => Ok(Some(Entry {
                    key: key.to_string(),
                    line: *line,
                    value: value.clone(),
                })),
                Some(other) => Err(format!("line {}: step `{key}` is not a scalar", other.line)),
                None => Ok(None),
            }
        };
        read.push(Step {
            first_line: *line,
            last_line,
            column: *column,
            uses: scalar("uses")?,
            with: match field("with") {
                Some(with) => scalar_mapping(&with.value, "with")?,
                None => Vec::new(),
            },
            env: match field("env") {
                Some(env) => scalar_mapping(&env.value, "env")?,
                None => Vec::new(),
            },
            shell: scalar("shell")?,
            run: scalar("run")?,
        });
    }
    Ok(read)
}

/// A mapping every value of which is a scalar — `env:` and `with:`.
fn scalar_mapping(node: &Node, what: &str) -> Result<Vec<Entry<String>>, String> {
    let Node::Mapping { entries, .. } = node else {
        return Err(format!("line {}: `{what}` is not a mapping", node.line()));
    };
    entries
        .iter()
        .map(|entry| match &entry.value {
            Node::Scalar { value, .. } => Ok(Entry {
                key: entry.key.clone(),
                line: entry.line,
                value: value.clone(),
            }),
            other => Err(format!(
                "line {}: `{what}.{}` is not a scalar, which a workflow {what} value is",
                other.line(),
                entry.key
            )),
        })
        .collect()
}

/// The one document in `text`, as a tree of [`Node`]s.
fn document(text: &str) -> Result<Node, String> {
    let mut builder = Builder::default();
    Parser::new_from_str(text)
        .load(&mut builder, true)
        .map_err(|error| format!("the workflow is not YAML this parser reads: {error}"))?;
    if let Some(refusal) = builder.refusal {
        return Err(refusal);
    }
    match (builder.documents, builder.finished.pop()) {
        (1, Some(root)) if builder.finished.is_empty() => Ok(root),
        (0, _) | (_, None) => Err("the workflow holds no document".to_string()),
        (count, _) => Err(format!(
            "the workflow holds {count} documents, and a check reading one would judge part of it"
        )),
    }
}

/// Builds [`Node`]s from the parser's events, and records the first shape it will not hold.
#[derive(Default)]
struct Builder {
    stack: Vec<Open>,
    finished: Vec<Node>,
    documents: usize,
    refusal: Option<String>,
}

enum Open {
    Sequence {
        items: Vec<Node>,
        line: usize,
    },
    Mapping {
        entries: Vec<Entry<Node>>,
        pending: Option<(String, usize)>,
        line: usize,
        column: Option<usize>,
    },
}

impl Builder {
    fn refuse(&mut self, why: String) {
        self.refusal.get_or_insert(why);
    }

    /// Hand a completed node to whatever is open around it; `column` is where the parser marked it.
    fn place(&mut self, node: Node, at_column: usize) {
        match self.stack.last_mut() {
            None => self.finished.push(node),
            Some(Open::Sequence { items, .. }) => items.push(node),
            Some(Open::Mapping {
                entries,
                pending,
                column,
                ..
            }) => match pending.take() {
                None => match node {
                    // A key: YAML allows a mapping or sequence here, and a workflow never writes one.
                    Node::Scalar { value, line } => {
                        let duplicated = entries.iter().any(|entry| entry.key == value);
                        if value == "<<" {
                            self.refusal.get_or_insert(format!(
                                "line {line}: a merge key copies a mapping in by reference, and this model \
                                 reads what is written where it is written"
                            ));
                        } else if duplicated {
                            self.refusal.get_or_insert(format!(
                                "line {line}: `{value}` is written twice in one mapping, so which value \
                                 applies is not something a reader can decide"
                            ));
                        }
                        column.get_or_insert(at_column);
                        *pending = Some((value, line));
                    }
                    other => {
                        self.refusal.get_or_insert(format!(
                            "line {}: a mapping key that is not a scalar",
                            other.line()
                        ));
                        *pending = Some((String::new(), other.line()));
                    }
                },
                Some((key, line)) => entries.push(Entry {
                    key,
                    line,
                    value: node,
                }),
            },
        }
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, event: Event, mark: Marker) {
        let line = mark.line();
        match event {
            Event::DocumentStart => self.documents += 1,
            // An alias names an anchor, and the anchor is refused where it is written, which comes first — so
            // this arm is reached only behind that refusal and keeps the event from being read as a value.
            Event::Alias(_) => self.refuse(format!(
                "line {line}: an alias reads a value written elsewhere, and this model reads what is written \
                 where it is written"
            )),
            // The parser marks a block scalar at its first content line rather than at its `|` or `>`, so every
            // scalar's line is the mark's — measured, and held by `block_scalars_read_as_yaml_reads_them`.
            Event::Scalar(value, _, anchor, tag) => {
                if anchor != 0 || tag.is_some() {
                    self.refuse(format!("line {line}: an anchor or a tag on a scalar"));
                }
                self.place(Node::Scalar { value, line }, mark.col());
            }
            Event::SequenceStart(anchor, tag) => {
                if anchor != 0 || tag.is_some() {
                    self.refuse(format!("line {line}: an anchor or a tag on a sequence"));
                }
                self.stack.push(Open::Sequence { items: Vec::new(), line });
            }
            Event::MappingStart(anchor, tag) => {
                if anchor != 0 || tag.is_some() {
                    self.refuse(format!("line {line}: an anchor or a tag on a mapping"));
                }
                self.stack.push(Open::Mapping {
                    entries: Vec::new(),
                    pending: None,
                    line,
                    column: None,
                });
            }
            Event::SequenceEnd | Event::MappingEnd => match self.stack.pop() {
                Some(Open::Sequence { items, line }) => {
                    self.place(Node::Sequence { items, line }, mark.col());
                }
                Some(Open::Mapping {
                    entries,
                    line,
                    column,
                    ..
                }) => self.place(
                    Node::Mapping {
                        entries,
                        line,
                        column: column.unwrap_or(mark.col()),
                    },
                    mark.col(),
                ),
                None => self.refuse(format!("line {line}: a collection closed that never opened")),
            },
            Event::Nothing | Event::StreamStart | Event::StreamEnd | Event::DocumentEnd => {}
        }
    }
}
