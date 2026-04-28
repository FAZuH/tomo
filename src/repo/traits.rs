pub trait ProjectRepo {}

pub trait TagRepo {}

pub trait TaskRepo {}

pub trait SessionRepo {}

pub trait Repos {
    fn project(&self) -> impl ProjectRepo;
    fn tag(&self) -> impl TagRepo;
    fn task(&self) -> impl TaskRepo;
    fn session(&self) -> impl SessionRepo;
}
