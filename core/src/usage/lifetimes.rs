use fnv::FnvHashSet;
use syn::punctuated::Punctuated;
use syn::{self, Lifetime, Type};

use usage::Options;

/// A set of lifetimes.
pub type LifetimeSet = FnvHashSet<Lifetime>;

/// A set of references to lifetimes.
pub type LifetimeRefSet<'a> = FnvHashSet<&'a Lifetime>;

/// Searcher for finding lifetimes in a syntax tree.
/// This can be used to determine which lifetimes must be emitted in generated code.
pub trait UsesLifetimes {
    /// Returns the subset of the queried lifetimes that are used by the implementing syntax element.
    ///
    /// This method only accounts for direct usage by the element; indirect usage via bounds or `where`
    /// predicates are not detected.
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a>;
}

/// Searcher for finding lifetimes in an iterator.
///
/// This trait extends iterators, providing a way to turn a filtered list of fields or variants into a set
/// of lifetimes.
pub trait CollectLifetimes {
    /// Consume an iterator, accumulating all lifetimes in the elements which occur in `lifetimes`.
    fn collect_lifetimes<'a>(
        self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a>;
}

impl<'i, I, T> CollectLifetimes for T
where
    T: IntoIterator<Item = &'i I>,
    I: 'i + UsesLifetimes,
{
    fn collect_lifetimes<'a>(
        self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.into_iter()
            .fold(Default::default(), |mut state, value| {
                state.extend(value.uses_lifetimes(options, lifetimes));
                state
            })
    }
}

impl<T: UsesLifetimes> UsesLifetimes for Vec<T> {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.collect_lifetimes(options, lifetimes)
    }
}

impl<T: UsesLifetimes, U> UsesLifetimes for Punctuated<T, U> {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.collect_lifetimes(options, lifetimes)
    }
}

impl<T: UsesLifetimes> UsesLifetimes for Option<T> {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.as_ref().map_or_else(
            || Default::default(),
            |v| v.uses_lifetimes(options, lifetimes),
        )
    }
}

impl UsesLifetimes for Lifetime {
    fn uses_lifetimes<'a>(&self, _: &Options, lifetimes: &'a LifetimeSet) -> LifetimeRefSet<'a> {
        lifetimes.iter().filter(|lt| *lt == self).collect()
    }
}

uses_lifetimes!(syn::AngleBracketedGenericArguments, args);
uses_lifetimes!(syn::BareFnArg, ty);
uses_lifetimes!(syn::Binding, ty);
uses_lifetimes!(syn::BoundLifetimes, lifetimes);
uses_lifetimes!(syn::DataEnum, variants);
uses_lifetimes!(syn::DataStruct, fields);
uses_lifetimes!(syn::DataUnion, fields);
uses_lifetimes!(syn::Field, ty);
uses_lifetimes!(syn::FieldsNamed, named);
uses_lifetimes!(syn::LifetimeDef, lifetime, bounds);
uses_lifetimes!(syn::ParenthesizedGenericArguments, inputs, output);
uses_lifetimes!(syn::Path, segments);
uses_lifetimes!(syn::PathSegment, arguments);
uses_lifetimes!(syn::QSelf, ty);
uses_lifetimes!(syn::TraitBound, path, lifetimes);
uses_lifetimes!(syn::TypeArray, elem);
uses_lifetimes!(syn::TypeBareFn, inputs, output);
uses_lifetimes!(syn::TypeGroup, elem);
uses_lifetimes!(syn::TypeImplTrait, bounds);
uses_lifetimes!(syn::TypeParen, elem);
uses_lifetimes!(syn::TypePtr, elem);
uses_lifetimes!(syn::TypeReference, lifetime, elem);
uses_lifetimes!(syn::TypeSlice, elem);
uses_lifetimes!(syn::TypeTuple, elems);
uses_lifetimes!(syn::TypeTraitObject, bounds);
uses_lifetimes!(syn::Variant, fields);

impl UsesLifetimes for syn::Data {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            syn::Data::Struct(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::Data::Enum(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::Data::Union(ref v) => v.uses_lifetimes(options, lifetimes),
        }
    }
}

impl UsesLifetimes for Type {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            Type::Slice(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Array(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Ptr(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Reference(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::BareFn(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Tuple(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Path(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Paren(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Group(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::TraitObject(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::ImplTrait(ref v) => v.uses_lifetimes(options, lifetimes),
            Type::Macro(_) | Type::Verbatim(_) | Type::Infer(_) | Type::Never(_) => {
                Default::default()
            }
        }
    }
}

impl UsesLifetimes for syn::Fields {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        self.collect_lifetimes(options, lifetimes)
    }
}

impl UsesLifetimes for syn::TypePath {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        let mut hits = self.path.uses_lifetimes(options, lifetimes);

        if options.include_type_path_qself() {
            hits.extend(self.qself.uses_lifetimes(options, lifetimes));
        }

        hits
    }
}

impl UsesLifetimes for syn::ReturnType {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        if let syn::ReturnType::Type(_, ref ty) = *self {
            ty.uses_lifetimes(options, lifetimes)
        } else {
            Default::default()
        }
    }
}

impl UsesLifetimes for syn::PathArguments {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            syn::PathArguments::None => Default::default(),
            syn::PathArguments::AngleBracketed(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::PathArguments::Parenthesized(ref v) => v.uses_lifetimes(options, lifetimes),
        }
    }
}

impl UsesLifetimes for syn::GenericArgument {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            syn::GenericArgument::Type(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::GenericArgument::Binding(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::GenericArgument::Lifetime(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::GenericArgument::Const(_) => Default::default(),
        }
    }
}

impl UsesLifetimes for syn::TypeParamBound {
    fn uses_lifetimes<'a>(
        &self,
        options: &Options,
        lifetimes: &'a LifetimeSet,
    ) -> LifetimeRefSet<'a> {
        match *self {
            syn::TypeParamBound::Trait(ref v) => v.uses_lifetimes(options, lifetimes),
            syn::TypeParamBound::Lifetime(ref v) => v.uses_lifetimes(options, lifetimes),
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::{self, DeriveInput};

    use super::UsesLifetimes;
    use usage::GenericsExt;
    use usage::Purpose::*;

    fn parse(src: &str) -> DeriveInput {
        syn::parse_str(src).unwrap()
    }

    #[test]
    fn struct_named() {
        let input = parse("struct Foo<'a, 'b: 'a> { parent: &'b Bar, child: &'a Baz, }");
        let omitted = syn::Lifetime::new("'c", Span::call_site());

        let lifetimes = {
            let mut lt = input.generics.declared_lifetimes();
            lt.insert(omitted);
            lt
        };

        let matches = input.data.uses_lifetimes(&BoundImpl.into(), &lifetimes);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn qself() {
        let input = parse(
            "struct Foo<'a, 'b: 'a> { parent: &'b Bar, child: <Bar<'a> as MyIterator>::Item, }",
        );
        let lifetimes = input.generics.declared_lifetimes();
        let matches = input.data.uses_lifetimes(&BoundImpl.into(), &lifetimes);
        assert_eq!(matches.len(), 1);

        let decl_matches = input.data.uses_lifetimes(&Declare.into(), &lifetimes);
        assert_eq!(decl_matches.len(), 2);
    }
}
