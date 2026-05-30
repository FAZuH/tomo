use crate::model::Mode;
use crate::model::Session;
use crate::model::Task;
use crate::repo::RepoError;

pub type RepoResult<T> = Result<T, RepoError>;

pub trait ProjectRepo {}

pub trait TagRepo {}

pub trait TaskRepo {
    fn add(&self, name: String, description: Option<String>) -> RepoResult<Task>;

    fn find_by_name(&self, name: String) -> RepoResult<Task>;

    fn all_tasks(&self) -> RepoResult<Vec<Task>>;
}

pub trait SessionRepo {
    fn new_session(&self, task_id: Option<i32>, mode: Mode) -> RepoResult<Session>;

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
