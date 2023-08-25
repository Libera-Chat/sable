pub trait FlattenResult<T, E> {
    fn flatten(self) -> Result<T, E>;
}

impl<T, OuterE, InnerE: From<OuterE>> FlattenResult<T, InnerE>
    for Result<Result<T, InnerE>, OuterE>
{
    fn flatten(self) -> Result<T, InnerE> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e.into()),
        }
    }
}
