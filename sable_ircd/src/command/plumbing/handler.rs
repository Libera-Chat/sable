use super::*;

pub trait HandlerFn<'ctx, Ambient, Positional> {
    fn call(&self, ctx: &'ctx dyn Command, args: ArgListIter<'ctx>) -> CommandResult;
}

pub trait AsyncHandlerFn<'ctx, Ambient, Positional>: Send + Sync {
    fn call(
        &'ctx self,
        ctx: &'ctx dyn Command,
        args: ArgListIter<'ctx>,
    ) -> impl Future<Output = CommandResult> + Send + Sync + 'ctx;
}

macro_rules! define_handler_fn
{
    ( ($($ambient:ident),*), ($($pos:ident),*) ) =>
    {
        impl<'ctx, T, $($ambient,)* $($pos),*> HandlerFn<'ctx, ($($ambient,)*), ($($pos,)*)> for T
            where T: Fn($($ambient,)* $($pos),*) -> CommandResult,
                  $( $ambient: AmbientArgument<'ctx>, )*
                  $( $pos: PositionalArgument<'ctx> ),*
        {
            // When this gets expanded with () as one of the argument lists these warnings will fire
            #[allow(unused_variables,unused_mut)]
            fn call(&self, ctx: &'ctx dyn Command, mut args: ArgListIter<'ctx>) -> CommandResult
            {
                self(
                    $(
                        $ambient::load_from(ctx)?,
                    )*
                    $(
                        $pos::parse(ctx, &mut args)?
                    ),*
                )

            }
        }

        #[allow(clippy::manual_async_fn)]
        impl<'ctx, T, F, $($ambient,)* $($pos),*> AsyncHandlerFn<'ctx, ($($ambient,)*), ($($pos,)*)> for T
            where T: Fn($($ambient,)* $($pos),*) -> F,
                  T: Send + Sync,
                  F: Future<Output=CommandResult> + Send + Sync,
                  $( $ambient: AmbientArgument<'ctx> + Send + Sync, )*
                  $( $pos: PositionalArgument<'ctx> + Send + Sync ),*
        {
            // When this gets expanded with () as one of the argument lists these warnings will fire
            #[allow(unused_variables,unused_mut)]
            fn call(&'ctx self, ctx: &'ctx dyn Command, mut args: ArgListIter<'ctx>) -> impl Future<Output=CommandResult> + Send + Sync + 'ctx
            {
                async move {
                    self(
                        $(
                            $ambient::load_from(ctx)?,
                        )*
                        $(
                            $pos::parse(ctx, &mut args)?
                        ),*
                    ).await
                }
            }
        }
    }
}

macro_rules! define_handlers {
    ( ($a1:ident $(, $arest:ident)*), ( $($pos:ident),* ) ) =>
    {
        define_handlers2!( ($a1 $(, $arest)*), ($( $pos ),*) );
        define_handlers!( ($($arest),*), ($( $pos ),*) );
    };
    ( (), ( $($pos:ident),* ) ) =>
    {
        define_handlers2!((), ($( $pos ),*) );
    };
}

macro_rules! define_handlers2 {
    ( ($( $amb:ident ),*), ($p1:ident $(, $prest:ident)* ) ) =>
    {
        define_handler_fn!(( $( $amb ),* ), ( $p1 $(, $prest)* ));
        define_handlers2!(( $( $amb ),* ), ( $($prest),* ));
    };
    ( ($( $amb:ident ),*), () ) =>
    {
        define_handler_fn!(( $( $amb),* ), ());
    };
    ((), ()) =>
    {
    }
}

define_handlers!((A1, A2, A3, A4, A5, A6), (P1, P2, P3, P4, P5, P6));
