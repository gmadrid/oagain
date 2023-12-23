pub(crate) trait BoolToOption<T> {
    fn option(self, val: T) -> Option<T>;
    fn option_with(self, val_func: impl Fn() -> T) -> Option<T>;
}

impl<T> BoolToOption<T> for bool {
    fn option(self, val: T) -> Option<T> {
        if self {
            val.into()
        } else {
            None
        }
    }

    fn option_with(self, val_func: impl Fn() -> T) -> Option<T> {
        if self {
            val_func().into()
        } else {
            None
        }
    }
}
