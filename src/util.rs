pub trait BoolToOption<T> {
    fn option(self, val: T) -> Option<T>;
    fn option_with(self, val_func: impl Fn() -> T) -> Option<T>;
}

impl<T, U: Into<bool>> BoolToOption<T> for U {
    fn option(self, val: T) -> Option<T> {
        if self.into() {
            val.into()
        } else {
            None
        }
    }

    fn option_with(self, val_func: impl Fn() -> T) -> Option<T> {
        if self.into() {
            val_func().into()
        } else {
            None
        }
    }
}
