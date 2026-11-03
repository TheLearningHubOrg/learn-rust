//! Stage 3: model a domain with structs and enums, and fail with typed errors.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Longest title, counted in characters (not bytes).
pub const MAX_TITLE_CHARS: usize = 80;

/// A fieldless enum. `Ord` follows declaration order, so Low < Medium < High.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lets callers write `"high".parse::<Priority>()`.
impl FromStr for Priority {
    type Err = TaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(TaskError::UnknownPriority(other.to_string())),
        }
    }
}

/// An enum whose variants carry different data. Serialised with an internal tag:
/// `{"state":"in_progress","assignee":"ana"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress { assignee: String },
    Done,
}

impl Status {
    fn name(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress { .. } => "in progress",
            Self::Done => "done",
        }
    }

    /// The whole state machine in one `match`. The last arm rejects every pair
    /// not listed. Because that arm is a catch-all, a new variant compiles here
    /// without complaint; the exhaustive `name` methods are where the compiler
    /// flags it.
    pub fn apply(&self, action: Action) -> Result<Status, TaskError> {
        match (self, action) {
            (Status::Todo, Action::Start { assignee }) => Ok(Status::InProgress { assignee }),
            (Status::InProgress { .. }, Action::Complete) => Ok(Status::Done),
            (Status::InProgress { .. } | Status::Done, Action::Reopen) => Ok(Status::Todo),
            (from, action) => Err(TaskError::InvalidTransition {
                from: from.name(),
                action: action.name(),
            }),
        }
    }
}

/// Commands a client can send: `{"action":"start","assignee":"ana"}`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Start { assignee: String },
    Complete,
    Reopen,
}

impl Action {
    fn name(&self) -> &'static str {
        match self {
            Self::Start { .. } => "start",
            Self::Complete => "complete",
            Self::Reopen => "reopen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub priority: Priority,
    pub status: Status,
}

/// The request body for creating a task. `priority` is optional in JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct NewTask {
    pub title: String,
    #[serde(default)]
    pub priority: Priority,
}

/// Every way a task operation can fail. `thiserror` writes the `Display`
/// and `std::error::Error` impls from the `#[error]` attributes.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TaskError {
    #[error("task {0} not found")]
    NotFound(u64),
    #[error("title must be 1 to {max} characters, got {len}")]
    InvalidTitle { len: usize, max: usize },
    #[error("unknown priority {0:?}; expected low, medium or high")]
    UnknownPriority(String),
    #[error("cannot {action} a task that is {from}")]
    InvalidTransition {
        from: &'static str,
        action: &'static str,
    },
}

/// An in-memory store. `BTreeMap` keeps tasks sorted by id, so listings are stable.
#[derive(Debug, Default)]
pub struct TaskStore {
    next_id: u64,
    tasks: BTreeMap<u64, Task>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Validates and inserts a task, returning a borrow of the stored value.
    pub fn create(&mut self, new: NewTask) -> Result<&Task, TaskError> {
        let title = new.title.trim();
        let len = title.chars().count();
        if len == 0 || len > MAX_TITLE_CHARS {
            return Err(TaskError::InvalidTitle {
                len,
                max: MAX_TITLE_CHARS,
            });
        }
        self.next_id += 1;
        let task = Task {
            id: self.next_id,
            title: title.to_string(),
            priority: new.priority,
            status: Status::Todo,
        };
        Ok(self.tasks.entry(task.id).or_insert(task))
    }

    pub fn get(&self, id: u64) -> Result<&Task, TaskError> {
        self.tasks.get(&id).ok_or(TaskError::NotFound(id))
    }

    /// Lists tasks, optionally keeping only one priority.
    pub fn list(&self, priority: Option<Priority>) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| priority.is_none_or(|p| task.priority == p))
            .collect()
    }

    /// Runs one state-machine step. `?` converts a lookup miss or an invalid
    /// transition into an early `Err` return.
    pub fn apply(&mut self, id: u64, action: Action) -> Result<&Task, TaskError> {
        let task = self.tasks.get_mut(&id).ok_or(TaskError::NotFound(id))?;
        task.status = task.status.apply(action)?;
        Ok(task)
    }

    /// Removes a task and hands ownership of it back to the caller.
    pub fn remove(&mut self, id: u64) -> Result<Task, TaskError> {
        self.tasks.remove(&id).ok_or(TaskError::NotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_task(title: &str, priority: Priority) -> NewTask {
        NewTask {
            title: title.to_string(),
            priority,
        }
    }

    #[test]
    fn priority_parses_case_insensitively() {
        assert_eq!(" HIGH ".parse::<Priority>(), Ok(Priority::High));
        assert_eq!(
            "urgent".parse::<Priority>(),
            Err(TaskError::UnknownPriority("urgent".into()))
        );
        assert!(Priority::Low < Priority::High);
        assert_eq!(Priority::default(), Priority::Medium);
    }

    #[test]
    fn create_trims_and_assigns_increasing_ids() {
        let mut store = TaskStore::new();
        let first = store
            .create(new_task("  write docs ", Priority::Low))
            .unwrap();
        assert_eq!((first.id, first.title.as_str()), (1, "write docs"));
        let second = store.create(new_task("ship", Priority::High)).unwrap();
        assert_eq!(second.id, 2);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn create_rejects_blank_and_long_titles() {
        let mut store = TaskStore::new();
        assert_eq!(
            store.create(new_task("   ", Priority::Low)),
            Err(TaskError::InvalidTitle { len: 0, max: 80 })
        );
        let long = "é".repeat(81); // 81 characters but 162 bytes
        assert!(matches!(
            store.create(new_task(&long, Priority::Low)),
            Err(TaskError::InvalidTitle { len: 81, .. })
        ));
        assert!(store.is_empty());
    }

    #[test]
    fn list_filters_by_priority() {
        let mut store = TaskStore::new();
        store.create(new_task("a", Priority::Low)).unwrap();
        store.create(new_task("b", Priority::High)).unwrap();
        store.create(new_task("c", Priority::High)).unwrap();
        let titles: Vec<&str> = store
            .list(Some(Priority::High))
            .iter()
            .map(|t| t.title.as_str())
            .collect();
        assert_eq!(titles, ["b", "c"]);
        assert_eq!(store.list(None).len(), 3);
    }

    #[test]
    fn state_machine_accepts_valid_steps() {
        let mut store = TaskStore::new();
        let id = store.create(new_task("deploy", Priority::High)).unwrap().id;
        let start = Action::Start {
            assignee: "ana".into(),
        };
        assert_eq!(
            store.apply(id, start).unwrap().status,
            Status::InProgress {
                assignee: "ana".into()
            }
        );
        assert_eq!(
            store.apply(id, Action::Complete).unwrap().status,
            Status::Done
        );
        assert_eq!(
            store.apply(id, Action::Reopen).unwrap().status,
            Status::Todo
        );
    }

    #[test]
    fn state_machine_rejects_invalid_steps() {
        let err = Status::Todo.apply(Action::Complete).unwrap_err();
        assert_eq!(err.to_string(), "cannot complete a task that is todo");
        let err = Status::Done
            .apply(Action::Start {
                assignee: "bo".into(),
            })
            .unwrap_err();
        assert_eq!(err.to_string(), "cannot start a task that is done");
    }

    #[test]
    fn missing_ids_are_not_found() {
        let mut store = TaskStore::new();
        assert_eq!(store.get(7), Err(TaskError::NotFound(7)));
        assert_eq!(store.remove(7), Err(TaskError::NotFound(7)));
        assert_eq!(
            store.apply(7, Action::Complete),
            Err(TaskError::NotFound(7))
        );
    }
}
