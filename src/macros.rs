use crate::*;

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
                let all_types = [
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
                let all_types = [
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
