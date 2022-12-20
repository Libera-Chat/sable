use super::*;

pub trait HandlerFn<'ctx, Args>
{
    fn call(&self, ctx: &'ctx impl CommandContext, args: ArgumentListIter<'ctx>) -> CommandResult;
}

pub trait AsyncHandlerFn<'ctx, Args> : Send + Sync
{
    fn call(&'ctx self, ctx: &'ctx impl CommandContext, args: ArgumentListIter<'ctx>) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx;
}

impl<'ctx, T> HandlerFn<'ctx, ()> for T
    where T: Fn() -> CommandResult
{
    fn call(&self, _ctx: &'ctx impl CommandContext, _args: ArgumentListIter<'ctx>) -> CommandResult
    {
        self()
    }
}

impl<'ctx, 'arg, T, F> AsyncHandlerFn<'ctx, ()> for T
    where T: Fn() -> F,
          T: Send + Sync,
          F: Future<Output=CommandResult> + Send + Sync + 'ctx
{
    fn call(&'ctx self, _ctx: &'ctx impl CommandContext, _args: ArgumentListIter<'ctx>) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx
    {
        self()
    }
}

macro_rules! define_handler_fn
{
    ( $($arg:ident),* ) =>
    {
        impl<'ctx, T, $($arg),*> HandlerFn<'ctx, ( $($arg),*, )> for T
            where T: Fn($($arg),*) -> CommandResult,
                  $( $arg: ArgumentType<'ctx> ),*
        {
            fn call(&self, ctx: &'ctx impl CommandContext, mut args: ArgumentListIter<'ctx>) -> CommandResult
            {
                self(
                    $(
                        $arg::parse(ctx, &mut args)?
                    ),*
                )

            }
        }

        impl<'ctx, T, F, $($arg),*> AsyncHandlerFn<'ctx, ( $($arg),*, )> for T
            where T: Fn($($arg),*) -> F,
                  T: Send + Sync,
                  F: Future<Output=CommandResult> + Send + Sync,
                  $( $arg: ArgumentType<'ctx> + Send + Sync + 'ctx ),*
        {
            fn call(&'ctx self, ctx: &'ctx impl CommandContext, mut args: ArgumentListIter<'ctx>) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx
            {
                async move {
                    self(
                        $(
                            $arg::parse(ctx, &mut args)?
                        ),*
                    ).await
                }
            }
        }
    }
}

define_handler_fn!(A1);
define_handler_fn!(A1, A2);
define_handler_fn!(A1, A2, A3);
define_handler_fn!(A1, A2, A3, A4);
define_handler_fn!(A1, A2, A3, A4, A5);
define_handler_fn!(A1, A2, A3, A4, A5, A6);
define_handler_fn!(A1, A2, A3, A4, A5, A6, A7);
define_handler_fn!(A1, A2, A3, A4, A5, A6, A7, A8);
define_handler_fn!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
define_handler_fn!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
