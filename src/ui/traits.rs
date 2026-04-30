use crate::ui::UiError;

pub trait Runner {
    fn run(&mut self) -> Result<(), UiError>;
}

pub trait Updateable {
    type Msg;
    type Cmd;
    fn update(&mut self, msg: Self::Msg) -> Self::Cmd;
}

pub trait View<C> {
    type Result;
    fn render(self, canvas: C) -> Self::Result;
}

pub trait ViewRef<C> {
    type Result;
    fn render_ref(&self, canvas: C) -> Self::Result;
    fn render_mut(&mut self, canvas: C) -> Self::Result {
        self.render_ref(canvas)
    }
}

pub trait StatefulView<C> {
    type State;
    type Result;
    fn render_stateful(self, canvas: C, state: &mut Self::State) -> Self::Result;
}

pub trait StatefulViewRef<C> {
    type State;
    type Result;
    fn render_stateful_ref(&self, canvas: C, state: &mut Self::State) -> Self::Result;
    fn render_stateful_mut(&mut self, canvas: C, state: &mut Self::State) -> Self::Result {
        self.render_stateful_ref(canvas, state)
    }
}

impl<T, C> View<C> for &T
where
    T: ViewRef<C>,
{
    type Result = <T as ViewRef<C>>::Result;

    fn render(self, canvas: C) -> Self::Result {
        self.render_ref(canvas)
    }
}

impl<T, C> View<C> for &mut T
where
    T: ViewRef<C>,
{
    type Result = <T as ViewRef<C>>::Result;

    fn render(self, canvas: C) -> Self::Result {
        self.render_ref(canvas)
    }
}

impl<T, C> StatefulView<C> for &T
where
    T: StatefulViewRef<C>,
{
    type State = <T as StatefulViewRef<C>>::State;
    type Result = <T as StatefulViewRef<C>>::Result;

    fn render_stateful(self, canvas: C, state: &mut Self::State) -> Self::Result {
        self.render_stateful_ref(canvas, state)
    }
}

impl<T, C> StatefulView<C> for &mut T
where
    T: StatefulViewRef<C>,
{
    type State = <T as StatefulViewRef<C>>::State;
    type Result = <T as StatefulViewRef<C>>::Result;

    fn render_stateful(self, canvas: C, state: &mut Self::State) -> Self::Result {
        self.render_stateful_ref(canvas, state)
    }
}
