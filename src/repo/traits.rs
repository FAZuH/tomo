pub trait ProjectRepository {}

pub trait TagRepository {}

pub trait TaskRepository {}

pub trait SessionRepository {}

pub trait Repositories {
    fn project(&self) -> impl ProjectRepository;
    fn tag(&self) -> impl TagRepository;
    fn task(&self) -> impl TaskRepository;
    fn session(&self) -> impl SessionRepository;
}
