use anymap::AnyMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};
use std::{any::TypeId, collections::HashMap};

pub trait ComponentsRef<'a> {
    type Result;

    /// Returns a tuple of references to the components
    fn get_components(
        entities_and_components: &'a EntitiesAndComponents,
        entity: Entity,
    ) -> Self::Result;
}

macro_rules! impl_components {
    ($($generic_name: ident),*) => {
        impl<'b, $($generic_name: 'static),*> ComponentsRef<'b> for ($($generic_name,)*) {
            type Result = ($(&'b $generic_name,)*);

            fn get_components(entities_and_components: &'b EntitiesAndComponents, entity: Entity) -> Self::Result {
                let components = entities_and_components
                    .components
                    .get(entity.entity_id)
                    .unwrap_or_else(||{
                        panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
                    });

                (
                    $(
                        components
                            .get::<Box<$generic_name>>()
                            .unwrap_or_else(||{
                                let type_name = std::any::type_name::<$generic_name>();
                                panic!(
                                    "Component {type_name} does not exist on the object, was the Component added to the entity?"
                                )
                            }),
                    )*
                )
            }
        }
    };
}

pub trait TryComponentsRef<'a> {
    type Result;

    /// Returns a tuple of references to the components
    fn try_get_components(
        entities_and_components: &'a EntitiesAndComponents,
        entity: Entity,
    ) -> Self::Result;
}

macro_rules! impl_try_components {
    ($($generic_name: ident),*) => {
        impl<'b, $($generic_name: 'static),*> TryComponentsRef<'b> for ($($generic_name,)*) {
            type Result = ($(Option<&'b $generic_name>,)*);
            fn try_get_components(entities_and_components: &'b EntitiesAndComponents, entity: Entity) -> ($(Option<&'b $generic_name>,)*) {
                let components = entities_and_components
                    .components
                    .get(entity.entity_id)
                    .unwrap_or_else(||{
                        panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
                });


                (
                    $(
                        components
                            .get::<Box<$generic_name>>().map(|boxed_t1|{ &**boxed_t1}),
                    )*
                )
            }
        }
    };
}

pub trait ComponentsMut<'a> {
    type Result;

    /// Returns a tuple of mutable references to the components
    fn get_components_mut(
        entities_and_components: &'a mut EntitiesAndComponents,
        entity: Entity,
    ) -> Self::Result;
}

macro_rules! impl_components_mut {
    ($($generic_name: ident),*) => {
        impl<'b, $($generic_name: 'static),*> ComponentsMut<'b> for ($($generic_name,)*) {
            type Result = ($(&'b mut $generic_name,)*);

            fn get_components_mut(entities_and_components: &'b mut EntitiesAndComponents, entity: Entity) -> Self::Result {

                // make sure that the same component is not borrowed mutably more than once
                let mut all_types = [
                    $(
                        std::any::TypeId::of::<$generic_name>(),
                    )*
                ];

                for i in 0..all_types.len() {
                    for j in i+1..all_types.len() {
                        assert_ne!(all_types[i], all_types[j], "You cannot borrow the same component mutably more than once!");
                    }
                }

                let components = entities_and_components
                    .components
                    .get_mut(entity.entity_id)
                    .unwrap_or_else(||{
                        panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
                });

                (
                    $(
                        {
                            let pointer: *mut $generic_name = &mut **components
                                .get_mut::<Box<$generic_name>>()
                                .unwrap_or_else(||{
                                    let type_name = std::any::type_name::<$generic_name>();
                                    panic!(
                                        "Component {type_name} does not exist on the object, was the Component added to the entity?"
                                    )
                                });
                            // SAFETY: We just checked that the component exists
                            // and that the component is not borrowed mutably more than once
                            // and lifetimes are checked at compile time to make sure that the component still exists
                            // so it is safe to return a mutable reference to the component
                            let reference = unsafe { &mut *pointer };
                            reference
                        },
                    )*
                )
            }
        }
    };
}

pub trait TryComponentsMut<'a> {
    type Result;

    /// Returns a tuple of mutable references to the components
    fn try_get_components_mut(
        entities_and_components: &'a mut EntitiesAndComponents,
        entity: Entity,
    ) -> Self::Result;
}

macro_rules! impl_try_components_mut {
    ($($generic_name: ident),*) => {
        impl<'b, $($generic_name: 'static),*> TryComponentsMut<'b> for ($($generic_name,)*) {
            type Result = ($(Option<&'b mut $generic_name>,)*);

            fn try_get_components_mut(entities_and_components: &'b mut EntitiesAndComponents, entity: Entity) -> Self::Result {

                // make sure that the same component is not borrowed mutably more than once
                let mut all_types = [
                    $(
                        std::any::TypeId::of::<$generic_name>(),
                    )*
                ];

                for i in 0..all_types.len() {
                    for j in i+1..all_types.len() {
                        assert_ne!(all_types[i], all_types[j], "You cannot borrow the same component mutably more than once!");
                    }
                }

                let components = entities_and_components
                    .components
                    .get_mut(entity.entity_id)
                    .unwrap_or_else(||{
                        panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
                });

                (
                    $(
                        {
                            let original_reference = components
                                .get_mut::<Box<$generic_name>>();
                            match original_reference {
                                Some(reference) => {
                                    let pointer: *mut $generic_name = &mut **reference;
                                    // SAFETY: We just checked that the component exists
                                    // and that the component is not borrowed mutably more than once
                                    // and lifetimes are checked at compile time to make sure that the component still exists
                                    // so it is safe to return a mutable reference to the component
                                    let reference = unsafe { &mut *pointer };
                                    Some(reference)
                                },
                                None => None,
                            }
                        },
                    )*
                )
            }
        }
    };
}

pub trait OwnedComponents {
    type Input;

    /// Returns a tuple of owned components
    fn make_entity_with_components(
        entities_and_components: &mut EntitiesAndComponents,
        components: Self::Input,
    ) -> Entity;
}

macro_rules! impl_owned_components {
    ($($generic_name: ident, $component_num: tt),*) => {
        impl<$($generic_name: 'static),*> OwnedComponents for ($($generic_name,)*) {
            type Input = ($($generic_name,)*);

            fn make_entity_with_components(
                entities_and_components: &mut EntitiesAndComponents,
                components: Self::Input,
            ) -> Entity {
                let entity = entities_and_components.add_entity();

                $(
                    entities_and_components.add_component_to(entity, (components.$component_num));
                )*

                entity
            }
        }
    };
}

// it would be nice to have a macro that generates this code
// but I don't know how to do that
impl_components!(T1);
impl_components!(T1, T2);
impl_components!(T1, T2, T3);
impl_components!(T1, T2, T3, T4);
impl_components!(T1, T2, T3, T4, T5);
impl_components!(T1, T2, T3, T4, T5, T6);
impl_components!(T1, T2, T3, T4, T5, T6, T7);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

impl_try_components!(T1);
impl_try_components!(T1, T2);
impl_try_components!(T1, T2, T3);
impl_try_components!(T1, T2, T3, T4);
impl_try_components!(T1, T2, T3, T4, T5);
impl_try_components!(T1, T2, T3, T4, T5, T6);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_try_components!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_try_components!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

impl_components_mut!(T1);
impl_components_mut!(T1, T2);
impl_components_mut!(T1, T2, T3);
impl_components_mut!(T1, T2, T3, T4);
impl_components_mut!(T1, T2, T3, T4, T5);
impl_components_mut!(T1, T2, T3, T4, T5, T6);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

impl_try_components_mut!(T1);
impl_try_components_mut!(T1, T2);
impl_try_components_mut!(T1, T2, T3);
impl_try_components_mut!(T1, T2, T3, T4);
impl_try_components_mut!(T1, T2, T3, T4, T5);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_try_components_mut!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_try_components_mut!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

impl_owned_components!(T1, 0);
impl_owned_components!(T1, 0, T2, 1);
impl_owned_components!(T1, 0, T2, 1, T3, 2);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8);
impl_owned_components!(T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26, T28, 27
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26, T28, 27, T29, 28
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26, T28, 27, T29, 28, T30, 29
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26, T28, 27, T29, 28, T30, 29, T31, 30
);
impl_owned_components!(
    T1, 0, T2, 1, T3, 2, T4, 3, T5, 4, T6, 5, T7, 6, T8, 7, T9, 8, T10, 9, T11, 10, T12, 11, T13,
    12, T14, 13, T15, 14, T16, 15, T17, 16, T18, 17, T19, 18, T20, 19, T21, 20, T22, 21, T23, 22,
    T24, 23, T25, 24, T26, 25, T27, 26, T28, 27, T29, 28, T30, 29, T31, 30, T32, 31
);

// The Entity will just be an ID that can be
// indexed into arrays of components for now...
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Entity {
    pub entity_id: DefaultKey,
}

pub struct EntitiesAndComponents {
    // Maybe there should be an object that takes a component
    // and has a list of which entities have that component?
    // This would make it easier to iterate over all entities with a certain component,
    // without having to iterate over all entities
    entities: SlotMap<DefaultKey, Entity>,
    pub(crate) components: SlotMap<DefaultKey, AnyMap>, // where components[entity_id][component_id]
    entities_with_components: HashMap<TypeId, Vec<Entity>>,
    type_ids_on_entity: SecondaryMap<DefaultKey, Vec<TypeId>>,
}

impl EntitiesAndComponents {
    pub fn new() -> Self {
        EntitiesAndComponents {
            entities: SlotMap::new(),
            components: SlotMap::new(),
            entities_with_components: HashMap::new(),
            type_ids_on_entity: SecondaryMap::new(),
        }
    }

    /// Adds an entity to the game engine
    /// Returns the entity
    pub fn add_entity(&mut self) -> Entity {
        let entity_id = self.components.insert(AnyMap::new());
        self.entities.insert(Entity { entity_id });
        self.type_ids_on_entity.insert(entity_id, vec![]);

        Entity { entity_id }
    }

    pub fn add_entity_with<T: OwnedComponents<Input = T>>(&mut self, components: T) -> Entity {
        let entity = <T>::make_entity_with_components(self, components);
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for type_id in self.type_ids_on_entity[entity.entity_id].clone() {
            match self.entities_with_components.get_mut(&type_id) {
                Some(entities) => {
                    entities.retain(|e| *e != entity);
                }
                None => {}
            }
        }
        self.type_ids_on_entity.remove(entity.entity_id);
        self.components.remove(entity.entity_id);
        self.entities.remove(entity.entity_id);
    }

    /// Gets a reference to all the entities in the game engine
    /// Should rarely if ever be used
    pub fn get_entities(&self) -> Vec<Entity> {
        // clone the entities vector
        self.entities.values().cloned().collect::<Vec<Entity>>()
    }

    /// Gets a copy of an entity at a certain index
    pub fn get_nth_entity(&self, index: usize) -> Option<Entity> {
        // get the nth entity
        if let Some(entity) = self.entities.values().nth(index) {
            Some(entity.clone())
        } else {
            None
        }
    }

    /// Gets the number of entities in the game engine
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Gets a reference to all the components on an entity
    /// Returns an AnyMap, which can be used to get a reference to a component
    /// This should rarely if ever be used
    pub fn get_all_components(&self, entity: Entity) -> &AnyMap {
        self.components.get(entity.entity_id).unwrap_or_else(|| {
            panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
        })
    }

    /// Gets a mutable reference to the components on an entity
    /// If the entity does not exist, it will panic
    pub fn get_all_components_mut(&mut self, entity: Entity) -> &mut AnyMap {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component<T: 'static>(&self, entity: Entity) -> Option<&Box<T>> {
        self.components
            .get(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get::<Box<T>>()
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will return None
    pub fn try_get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut Box<T>> {
        self.components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            })
            .get_mut::<Box<T>>()
    }

    /// Gets a reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_components<'a, T: ComponentsRef<'a> + 'static>(
        &'a self,
        entity: Entity,
    ) -> T::Result {
        <T>::get_components(self, entity)
    }

    /// Gets a mutable reference to a component on an entity
    /// If the component does not exist on the entity, it will panic
    pub fn get_components_mut<'a, T: ComponentsMut<'a> + 'static>(
        &'a mut self,
        entity: Entity,
    ) -> T::Result {
        <T>::get_components_mut(self, entity)
    }

    pub fn try_get_components<'a, T: TryComponentsRef<'a> + 'static>(
        &'a self,
        entity: Entity,
    ) -> T::Result {
        <T>::try_get_components(self, entity)
    }

    pub fn try_get_components_mut<'a, T: TryComponentsMut<'a> + 'static>(
        &'a mut self,
        entity: Entity,
    ) -> T::Result {
        <T>::try_get_components_mut(self, entity)
    }

    /// Adds a component to an entity
    /// If the component already exists on the entity, it will be overwritten
    pub fn add_component_to<T: Component>(&mut self, entity: Entity, component: T) {
        // add the component to the entity
        let components = self
            .components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            });
        components.insert(Box::new(component));

        // add the entity to the list of entities with the component
        match self.entities_with_components.entry(TypeId::of::<T>()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(entity);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![entity]);
            }
        }
        self.type_ids_on_entity[entity.entity_id].push(TypeId::of::<T>());
    }

    pub fn remove_component_from<T: Component>(&mut self, entity: Entity) {
        // remove the component from the entity
        let components = self
            .components
            .get_mut(entity.entity_id)
            .unwrap_or_else(|| {
                panic!("Entity ID {entity:?} does not exist, was the Entity ID edited?");
            });
        components.remove::<Box<T>>();

        // remove the entity from the list of entities with the component
        match self.entities_with_components.get_mut(&TypeId::of::<T>()) {
            Some(entities) => {
                entities.retain(|e| *e != entity);
            }
            None => {}
        }
        self.type_ids_on_entity[entity.entity_id].retain(|t| *t != TypeId::of::<T>());
    }

    /// returns a vector of all the entities that have a certain component
    /// if no entities have the component, it will return an empty vector
    /// clones the vector, so it is not very efficient
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<Entity> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => entities.clone(),
            None => vec![],
        }
    }

    pub fn get_entity_count_with_component<T: Component>(&self) -> usize {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => entities.len(),
            None => 0,
        }
    }

    /// gets the nth entity with a certain component
    /// doesn't use nth so not O(n)
    pub fn get_entity_with_component<T: Component>(&self, index: usize) -> Option<Entity> {
        match self.entities_with_components.get(&TypeId::of::<T>()) {
            Some(entities) => {
                if let Some(entity) = entities.get(index) {
                    Some(entity.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

pub struct GameEngine {
    pub entities_and_components: EntitiesAndComponents,
    systems: Vec<Box<dyn System>>,
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            entities_and_components: EntitiesAndComponents::new(),
            systems: vec![],
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn run(&mut self) {
        for system in &mut self.systems {
            // not sure what to do about the mutability here...
            // maybe seperate the systems and the entities and components?
            system.run(&mut self.entities_and_components);
        }
    }
}

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

/// Systems access and change components on objects
pub trait System {
    fn run(&mut self, engine: &mut EntitiesAndComponents);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }

    //impl Component for Position {}

    struct Velocity {
        x: f32,
        y: f32,
    }

    //impl Component for Velocity {}

    struct MovementSystem {}

    impl System for MovementSystem {
        fn run(&mut self, engine: &mut EntitiesAndComponents) {
            for i in 0..engine.entities.len() {
                let entity = engine.get_nth_entity(i).unwrap(); // this should never panic

                // be very careful when using this macro like this
                // using it this way could cause a data race if you are not careful
                //let (velocity,) = get_components!(engine, entity, Velocity);
                let (position, velocity) =
                    <(Position, Velocity)>::get_components_mut(engine, entity);

                position.x += velocity.x;
                position.y += velocity.y;

                println!("Position: {}, {}", position.x, position.y);
            }
        }
    }

    #[test]
    fn test_components_mut() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        engine.add_system(Box::new(MovementSystem {}));

        for _ in 0..5 {
            engine.run();
        }
    }

    #[test]
    fn test_try_get_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        let (position, velocity) =
            <(Position, Velocity)>::try_get_components(entities_and_components, entity);

        assert_eq!(position.unwrap().x, 0.0);
        assert_eq!(position.unwrap().y, 0.0);
        assert_eq!(velocity.unwrap().x, 1.0);
        assert_eq!(velocity.unwrap().y, 1.0);
    }

    #[test]
    fn test_overriding_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Position { x: 6.0, y: 1.0 });

        let (position,) = entities_and_components.get_components::<(Position,)>(entity);
        assert_eq!(position.x, 6.0);
        assert_eq!(position.y, 1.0);
    }

    #[test]
    fn test_multiple_entities() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();
        let entity_2 = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        entities_and_components.add_component_to(entity_2, Position { x: 0.0, y: 0.0 });
        entities_and_components.add_component_to(entity_2, Velocity { x: 1.0, y: 1.0 });

        // this should compile but, currently you can't borrow from two different entities mutably at the same time
        let (position,) = entities_and_components.get_components_mut::<(Position,)>(entity);
        //let position_2 = entities_and_components.get_component_mut::<Position>(entity_2);

        println!("Position: {}, {}", position.x, position.y);
        //println!("Position: {}, {}", position_2.x, position_2.y);
    }

    #[test]
    fn test_add_entity_with_components() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components
            .add_entity_with((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

        let (position, velocity) =
            entities_and_components.get_components::<(Position, Velocity)>(entity);

        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 0.0);
        assert_eq!(velocity.x, 1.0);
        assert_eq!(velocity.y, 1.0);
    }

    // this test should not compile
    /*#[test]
    fn test_compile_fail_multiple_muts() {
        let mut engine = GameEngine::new();
        let entities_and_components = &mut engine.entities_and_components;

        let entity = entities_and_components.add_entity();

        entities_and_components.add_component_to(entity, Position { x: 1.0, y: 0.0 });
        entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

        let (position, velocity) =
            get_components_mut!(engine.entities_and_components, entity, Position, Velocity);

        let (position_2, velocity_2) =
            get_components_mut!(engine.entities_and_components, entity, Position, Velocity);

        position.x += position_2.x;
        position.y += position_2.y;

        println!("Position: {}, {}", position.x, position_2.y);
    }*/

    // this test should not compile
    /*#[test]
    fn test_lifetimes() {
        let (position, velocity): (&mut Position, &mut Velocity);
        {
            let mut engine = GameEngine::new();
            let entities_and_components = &mut engine.entities_and_components;

            let entity = entities_and_components.add_entity();

            entities_and_components.add_component_to(entity, Position { x: 1.0, y: 0.0 });
            entities_and_components.add_component_to(entity, Velocity { x: 1.0, y: 1.0 });

            let (position, velocity) =
                <(Position, Velocity)>::get_components(entities_and_components, entity);

            //(position, velocity) =
            //    get_components_mut!(engine.entities_and_components, entity, Position, Velocity);
        }

        // should not be possible, but the lifetimes aren't linked
        position.x += velocity.x;
        position.y += velocity.y;

        println!("Position: {}, {}", position.x, position.y);
    }*/
}
