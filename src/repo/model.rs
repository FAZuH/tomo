use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;

use crate::repo::schema::project_default_tags;
use crate::repo::schema::projects;
use crate::repo::schema::sessions;
use crate::repo::schema::tags;
use crate::repo::schema::task_tags;
use crate::repo::schema::tasks;

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = projects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = tags)]
#[diesel(check_for_backend(Sqlite))]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable, Associations,
)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(belongs_to(Task, foreign_key = parent_id))]
#[diesel(belongs_to(Project))]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub deadline: Option<NaiveDateTime>,
    // Optional parent of this subtask
    pub parent_id: Option<i32>,
    // Optional id of the project this task belongs to. A task can only belong to 1 project
    pub project_id: Option<i32>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable, Associations,
)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(Sqlite))]
#[diesel(belongs_to(Task))]
pub struct Session {
    pub id: i32,
    pub task_id: Option<i32>,
    pub start_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub end_at: Option<NaiveDateTime>,
    pub pomodoro_state: PomodoroState,
    pub paused: bool,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable, Associations,
)]
#[diesel(table_name = project_default_tags)]
#[diesel(belongs_to(Project))]
#[diesel(belongs_to(Tag))]
#[diesel(primary_key(project_id, tag_id))]
pub struct ProjectDefaultTag {
    pub project_id: i32,
    pub tag_id: i32,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Queryable, Selectable, Insertable, Identifiable, Associations,
)]
#[diesel(table_name = task_tags)]
#[diesel(belongs_to(Task))]
#[diesel(belongs_to(Tag))]
#[diesel(primary_key(task_id, tag_id))]
pub struct TaskTag {
    pub task_id: i32,
    pub tag_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
#[derive(Default)]
pub enum PomodoroState {
    #[default]
    Focus,
    LongBreak,
    ShortBreak,
}

const POMO_FOCUS: &str = "focus";
const POMO_LONG: &str = "long break";
const POMO_SHORT: &str = "short break";

impl<DB: Backend> FromSql<Text, DB> for PomodoroState
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            POMO_FOCUS => Ok(Self::Focus),
            POMO_LONG => Ok(Self::LongBreak),
            POMO_SHORT => Ok(Self::ShortBreak),
            other => Err(format!("Invalid mode: {}", other).into()),
        }
    }
}

impl<DB: Backend> ToSql<Text, DB> for PomodoroState
where
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        match self {
            Self::Focus => POMO_FOCUS.to_sql(out),
            Self::LongBreak => POMO_LONG.to_sql(out),
            Self::ShortBreak => POMO_SHORT.to_sql(out),
        }
    }
}
