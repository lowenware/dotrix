//! Entity Component System
use core::ops::{Deref, DerefMut};

use crate::application::{
    services::Services,
    Service
};

/// Entity structure has only id field and represent an agregation of components
pub struct Entity(u64);

/// Any data structure can be a component
pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

/// Wrapper for system functions
///
/// Read [`crate::systems`] for more information
pub struct System {
    data: Box<dyn Systemized>,
    run_level: RunLevel,
}

/// Defines when and how often a system should run.
pub enum RunLevel {
    /// Before rendering stage
    Standard,
    /// Only once on application startup
    Startup,
    /// Rendering stage
    Render,
}

impl System {
    /// Constructs a system from function
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

    /// Adds an option to the system
    pub fn with<T>(mut self, option: T) -> Self
    where
        Self: SystemOption<T>,
    {
        self.set_option(option);
        self
    }

    pub(crate) fn tuple(self) -> (Box<dyn Systemized>, RunLevel) {
        (self.data, self.run_level)
    }
}

/// System option abstract interface
pub trait SystemOption<T> {
    /// customize the syystem with an option
    fn set_option(&mut self, option: T);
}

/// [`RunLevel`] option implementation
impl SystemOption<RunLevel> for System {
    fn set_option(&mut self, option: RunLevel) {
        self.run_level = option;
    }
}

struct SystemData<Run, Ctx> 
where
    Run: FnMut(&mut Ctx, &mut Services) + Send + Sync,
{
    name: &'static str,
    run: Run,
    ctx: Ctx,
}

/// Abstraction for [`System`] prepared to be integrated into engine
pub trait Systemized: Send + Sync {
    /// Returns name of the system
    fn name(&mut self) -> &'static str;
    /// Executes system cylce
    fn run(&mut self, app: &mut Services);
}

impl<Run, Ctx> Systemized for SystemData<Run, Ctx>
where
    Run: FnMut(&mut Ctx, &mut Services) + Send + Sync,
    Ctx: SystemContext,
{
    fn name(&mut self) -> &'static str {
        self.name
    }

    fn run(&mut self, app: &mut Services) {
        (self.run)(&mut self.ctx, app);
    }
}

/// Abstraction for a function that can be turned into a [`System`]
pub trait IntoSystem<C, S> {
    /// Converts a function into [`System`]
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

/// Abstraction for [`System`] context
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

/// Accessor for [`System`] context to privately store data between runs
pub struct Context<T> {
    value: *mut T,
}

impl<T> Context<T> {
    /// Constructs an accessor instance
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

/// Mutable accessor for [`Service`] instance
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

/// Imutable accessor for [`Service`] instance
pub struct Const<T> {
    value: *const T,
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

unsafe impl<T: Service> Send for Const<T> {}
unsafe impl<T: Service> Sync for Const<T> {}

/// Abstraction to access [`Service`] in their storage
pub trait Accessor: Send + Sync {
    /// Type of [`Service`] to be accessed
    type Item: Service;
    /// Fetches the [`Service`] from the storage
    fn fetch(services: &mut Services) -> Self;
}

impl<T> Accessor for Mut<T>
where
    T: Service,
{
    type Item = T;
    fn fetch(services: &mut Services) -> Self {
        let service: &mut T = services.get_mut::<T>()
            .unwrap_or_else(|| panic!("Service {} does not exist", std::any::type_name::<T>()));
        Mut {
            value: service as *mut T
        }
    }
}

impl<T> Accessor for Const<T>
where
    T: Service,
{
    type Item = T;
    fn fetch(service: &mut Services) -> Self {
        let service: &T = service.get::<T>()
            .unwrap_or_else(|| panic!("Service {} does not exist", std::any::type_name::<T>()));
        Const {
            value: service as *const T
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::services::Services,
        ecs::{
            Context,
            System,
            Mut,
        },
        world::World,
    };

    struct MyComponent(u64);

    fn my_system(mut world: Mut<World>) {
        world.spawn(Some((MyComponent(123),)));
    }

    #[test]
    fn world_system() {
        let mut services = Services::new();
        services.add(World::new());
        let mut s = System::from(my_system);
        s.data.run(&mut services);
        assert_eq!(services.get::<World>().unwrap().counter(), 1);
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
        let mut services = Services::new();
        services.add(MyService { data: 123 });
        let mut s = System::from(my_system_with_context);
        s.data.run(&mut services);
        assert_eq!(services.get::<MyService>().unwrap().data, 0);
    }

}
