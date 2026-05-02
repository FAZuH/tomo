use crate::repo::model::*;

type RepoResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub trait ProjectRepo {}

pub trait TagRepo {}

pub trait TaskRepo {
    fn add(&self, name: String) -> RepoResult<Task>;
}

pub trait SessionRepo {
    fn new_session(&self, task_id: Option<i32>, state: PomodoroState) -> RepoResult<Session>;

    fn update(&self, id: i32) -> RepoResult<usize>;

    fn end_session(&self, id: i32) -> RepoResult<usize>;

    fn close_all_sessions(&self) -> RepoResult<()>;
}

pub trait Repos {
    fn project(&self) -> Box<dyn ProjectRepo>;
    fn tag(&self) -> Box<dyn TagRepo>;
    fn task(&self) -> Box<dyn TaskRepo>;
    fn session(&self) -> Box<dyn SessionRepo>;
}
