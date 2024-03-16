//! Contains code related specifically to Bevy's type registration.

use crate::derive_data::ReflectMeta;
use crate::serialization::SerializationDataDef;
use crate::utility::WhereClauseOptions;
use quote::quote;
use syn::Type;

/// Creates the `GetTypeRegistration` impl for the given type data.
#[allow(clippy::too_many_arguments)]
pub(crate) fn impl_get_type_registration<'a>(
    meta: &ReflectMeta,
    where_clause_options: &WhereClauseOptions,
    serialization_data: Option<&SerializationDataDef>,
    type_dependencies: Option<impl Iterator<Item = &'a Type>>,
) -> proc_macro2::TokenStream {
    let type_path = meta.type_path();
    let bevy_reflect_path = meta.bevy_reflect_path();
    let registration_data = meta.attrs().idents();

    let type_deps_fn = type_dependencies.map(|deps| {
        quote! {
            #[inline(never)]
            fn register_type_dependencies(registry: &mut #bevy_reflect_path::TypeRegistry) {
                #(<#deps as #bevy_reflect_path::__macro_exports::RegisterForReflection>::__register(registry);)*
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = type_path.generics().split_for_impl();
    let where_reflect_clause = where_clause_options.extend_where_clause(where_clause);

    let from_reflect_data = if meta.from_reflect().should_auto_derive() {
        Some(quote! {
            registration.insert::<#bevy_reflect_path::ReflectFromReflect>(#bevy_reflect_path::FromType::<Self>::from_type());
        })
    } else {
        None
    };

    let serialization_data = serialization_data.map(|data| {
        let serialization_data = data.as_serialization_data(bevy_reflect_path);
        quote! {
            registration.insert::<#bevy_reflect_path::serde::SerializationData>(#serialization_data);
        }
    });

    let spec_register = get_specialized_registrations(meta);

    quote! {
        #[allow(unused_mut)]
        impl #impl_generics #bevy_reflect_path::GetTypeRegistration for #type_path #ty_generics #where_reflect_clause {
            fn get_type_registration() -> #bevy_reflect_path::TypeRegistration {
                let mut registration = #bevy_reflect_path::TypeRegistration::of::<Self>();
                registration.insert::<#bevy_reflect_path::ReflectFromPtr>(#bevy_reflect_path::FromType::<Self>::from_type());
                #from_reflect_data
                #serialization_data
                #(registration.insert::<#registration_data>(#bevy_reflect_path::FromType::<Self>::from_type());)*

                use #bevy_reflect_path::autoref_helpers::{spec, RegisterSpec};
                #(spec::<Self, #spec_register>().register_spec(&mut registration);)*

                registration
            }

            #type_deps_fn
        }
    }
}

fn get_specialized_registrations(meta: &ReflectMeta) -> Vec<proc_macro2::TokenStream> {
    let bevy_reflect_path = meta.bevy_reflect_path();

    #[allow(unused_mut)]
    let mut spec_register = vec![
        quote!(#bevy_reflect_path::std_traits::ReflectDefault),
        quote!(#bevy_reflect_path::ReflectSerialize),
        quote!(#bevy_reflect_path::ReflectDeserialize),
    ];

    // TODO: If these are enabled they will be enabled in all crates that depend on bevy_reflect_derive,
    // not just those that also depend on bevy_ecs and bevy_asset, which will cause build failures.
    // The most prominent example is bevy_reflect itself.

    #[cfg(feature = "bevy_ecs")]
    {
        let bevy_ecs_path = bevy_macro_utils::BevyManifest::get_path_direct("bevy_ecs");
        spec_register.extend([
            quote!(#bevy_ecs_path::reflect::ReflectBundle),
            quote!(#bevy_ecs_path::reflect::ReflectComponent),
            quote!(#bevy_ecs_path::reflect::ReflectFromWorld),
            quote!(#bevy_ecs_path::reflect::ReflectMapEntities),
            quote!(#bevy_ecs_path::reflect::ReflectResource),
        ]);
    }

    #[cfg(feature = "bevy_asset")]
    {
        let bevy_asset_path = bevy_macro_utils::BevyManifest::get_path_direct("bevy_asset");
        spec_register.push(quote!(#bevy_asset_path::ReflectAsset));
    }

    spec_register
}
