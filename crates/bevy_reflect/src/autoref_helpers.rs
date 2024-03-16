use std::marker::PhantomData;

use crate::{FromType, TypeData, TypeRegistration};

pub trait RegisterSpec: Sized {
    fn register_spec(self, _registration: &mut TypeRegistration) {}
}

pub fn spec<T, D>() -> Spec<T, D> {
    Spec(PhantomData)
}

pub struct Spec<T, D>(PhantomData<(T, D)>);

impl<T, D> RegisterSpec for Spec<T, D>
where
    D: FromType<T> + TypeData,
{
    fn register_spec(self, registration: &mut TypeRegistration) {
        registration.insert::<D>(FromType::<T>::from_type());
    }
}

impl<T, D> RegisterSpec for &Spec<T, D> {}
