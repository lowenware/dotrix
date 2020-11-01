use core::ops::{Deref, DerefMut};

use crate::{
    application::{Application, Service},
};

/// Entity structure has only id field and represent an agregation of components
pub struct Entity(u64);

/// Any data structure can be a component
pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

pub struct System {
    data: Box<dyn Systemized>,
    run_level: RunLevel,
}

pub enum RunLevel {
    Standard,
    Startup,
    Render,
}

impl System {
    pub fn from<Fun, Ctx, Srv>(func: Fun) -> Self
    where
        Fun: IntoSystem<Ctx, Srv> + Sync + Send,
        Ctx: Sync + Send,
        Srv: Sync + Send,
    {
        Self {
            data: func.into_system(),
            run_level: RunLevel::Standard,
        }
    }

    pub fn with<T>(mut self, option: T) -> Self
    where
        Self: SystemOption<T>,
    {
        self.set_option(option);
        self
    }

    pub fn tuple(mut self) -> (Box<dyn Systemized>, RunLevel) {
        (self.data, self.run_level)
    }
}

pub trait SystemOption<T> {
    fn set_option(&mut self, option: T);
}

impl SystemOption<RunLevel> for System {
    fn set_option(&mut self, option: RunLevel) {
        self.run_level = option;
    }
}

struct SystemData<Run, Ctx> 
where
    Run: FnMut(&mut Ctx, &mut Application) + Send + Sync,
{
    name: &'static str,
    run: Run,
    ctx: Ctx,
}

pub trait Systemized: Send + Sync {
    fn name(&mut self) -> &'static str;
    fn run(&mut self, app: &mut Application);
}

impl<Run, Ctx> Systemized for SystemData<Run, Ctx>
where
    Run: FnMut(&mut Ctx, &mut Application) + Send + Sync,
    Ctx: SystemContext,
{
    fn name(&mut self) -> &'static str {
        self.name
    }

    fn run(&mut self, app: &mut Application) {
        (self.run)(&mut self.ctx, app);
    }
}

pub trait IntoSystem<C, S> {
    fn into_system(self) -> Box<dyn Systemized>;
}

macro_rules! impl_into_system {
    (($($context: ident),*), ($($i: ident),*)) => {

        impl<Fun, $($context,)* $($i,)*> IntoSystem<($($context,)*), ($($i,)*)> for Fun
        where
            Fun: FnMut($(Context<$context>,)* $($i,)*) + Send + Sync + 'static,
            $($context: SystemContext,)*
            $($i: Accessor,)*
        {
            #[allow(non_snake_case)]
            #[allow(unused)]
            fn into_system(mut self: Fun) -> Box<dyn Systemized>
            {
                let data: SystemData<_, ($($context)*)> = SystemData {
                    name: std::any::type_name::<Fun>(),
                    run: move |ctx, app| {
                        // context (none or one)
                        $(
                            let $context = Context::new(ctx);
                        )*
                        // services
                        $(
                            let $i = $i::fetch(app);
                        )*
                        (self)($($context,)* $($i,)*);
                    },
                    ctx: ($($context::default())*),
                };
                Box::new(data)
            }
        }
    }
}

pub trait SystemContext: Default + Send + Sync + 'static {}
impl<T: Default + Send + Sync + 'static> SystemContext for T {}

impl_into_system!((), (A));
impl_into_system!((), (A, B));
impl_into_system!((), (A, B, C));
impl_into_system!((), (A, B, C, D));
impl_into_system!((), (A, B, C, D, E));
impl_into_system!((), (A, B, C, D, E, F));
impl_into_system!((), (A, B, C, D, E, F, G));
impl_into_system!((), (A, B, C, D, E, F, G, H));

impl_into_system!((CTX), (A));
impl_into_system!((CTX), (A, B));
impl_into_system!((CTX), (A, B, C));
impl_into_system!((CTX), (A, B, C, D));
impl_into_system!((CTX), (A, B, C, D, E));
impl_into_system!((CTX), (A, B, C, D, E, F));
impl_into_system!((CTX), (A, B, C, D, E, F, G));
impl_into_system!((CTX), (A, B, C, D, E, F, G, H));

pub struct Context<T> {
    value: *mut T,
}

impl<T> Context<T> {
    fn new(ctx: &mut T) -> Self {
        Context {
            value: ctx as *mut T,
        }
    }
}

impl<T> Deref for Context<T>
where
    T: SystemContext,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

impl<T> DerefMut for Context<T>
where
    T: SystemContext,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value
        }
    }
}

pub struct Mut<T>
where
    T: Service
{
    value: *mut T,
}

impl<T> Deref for Mut<T>
where
    T: Service,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

impl<T> DerefMut for Mut<T>
where
    T: Service,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.value
        }
    }
}

unsafe impl<T: Service> Send for Mut<T> {}
unsafe impl<T: Service> Sync for Mut<T> {}

pub struct Const<T> {
    pub value: *const T,
}

impl<T> Deref for Const<T>
where
    T: Service,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.value
        }
    }
}

pub trait Accessor: Send + Sync {
    type Item: Service;
    fn fetch(app: &mut Application) -> Self;
}

impl<T> Accessor for Mut<T>
where
    T: Service,
{
    type Item = T;
    fn fetch(app: &mut Application) -> Self {
        let service: &mut T = app.service::<T>();
        Mut {
            value: service as *mut T
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::{Application},
        ecs::{
            Context,
            System,
            Mut,
        },
        world::{ World },
    };

    struct MyComponent(u64);

    fn my_system(mut world: Mut<World>) {
        world.spawn(Some((MyComponent(123),)));
    }

    #[test]
    fn world_system() {
        let mut app = Application::new("Test App");
        app.add_service(World::new());
        let mut s = System::from(my_system);
        s.data.run(&mut app);
        assert_eq!(app.service::<World>().counter(), 1);
    }

    #[derive(Default)]
    struct MyContext(u64);

    struct MyService {
        data: u64
    }

    fn my_system_with_context(ctx: Context<MyContext>, mut service: Mut<MyService>) {
        service.data = ctx.0;
    }

    #[test]
    fn custom_system() {
        let mut app = Application::new("Test App");
        app.add_service(MyService { data: 123 });
        let mut s = System::from(my_system_with_context);
        s.data.run(&mut app);
        assert_eq!(app.service::<MyService>().data, 0);
    }

}
