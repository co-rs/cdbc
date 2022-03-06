pub mod statement_cache;
pub mod ustr;
pub mod scan;
pub mod crud;


use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

/// A wrapper for `Fn`s that provides a debug impl that just says "Function"
pub struct DebugFn<F: ?Sized>(pub F);

impl<F: ?Sized> Deref for DebugFn<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F: ?Sized> DerefMut for DebugFn<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F: ?Sized> Debug for DebugFn<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Function").finish()
    }
}

pub fn to_snake_name(name: &str) -> String {
    let chs = name.chars();
    let mut new_name = String::new();
    let mut index = 0;
    let chs_len = name.len();
    for x in chs {
        if x.is_uppercase() {
            if index != 0 && (index + 1) != chs_len {
                new_name.push_str("_");
            }
            new_name.push_str(x.to_lowercase().to_string().as_str());
        } else {
            new_name.push(x);
        }
        index += 1;
    }
    return new_name;
}