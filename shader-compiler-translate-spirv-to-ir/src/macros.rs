// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

macro_rules! impl_spirv_enum_partition {
    (
        $(#[doc = $class_enum_doc:expr])*
        $vis:vis enum $class_enum:ident($enum:ident) {
            $(
                $(#[doc = $class_enumerant_doc:expr])*
                $class_enumerant:ident($class_value:ident {
                    $(
                        $enumerant:ident($value:ident),
                    )+
                }),
            )+
        }
    ) => {
        $(
            $(#[doc = $class_enumerant_doc])*
            #[derive(Clone, Debug)]
            $vis enum $class_value {
                $(
                    $enumerant(spirv_parser::$value),
                )+
            }

            impl From<$class_value> for $class_enum {
                fn from(v: $class_value) -> Self {
                    $class_enum::$class_enumerant(v)
                }
            }

            impl Into<spirv_parser::$enum> for $class_value {
                fn into(self) -> spirv_parser::$enum {
                    match self {
                        $(
                            $class_value::$enumerant(v) => v.into(),
                        )+
                    }
                }
            }
        )+

        $(#[doc = $class_enum_doc])*
        #[derive(Clone, Debug)]
        $vis enum $class_enum {
            $(
                $(#[doc = $class_enumerant_doc])*
                $class_enumerant($class_value),
            )+
        }

        impl Into<spirv_parser::$enum> for $class_enum {
            fn into(self) -> spirv_parser::$enum {
                match self {
                    $(
                        $class_enum::$class_enumerant(v) => v.into(),
                    )+
                }
            }
        }

        impl From<spirv_parser::$enum> for $class_enum {
            fn from(v: spirv_parser::$enum) -> Self {
                match v {
                    $($(
                        spirv_parser::$enum::$enumerant(v) => $class_enum::$class_enumerant($class_value::$enumerant(v)),
                    )+)+
                }
            }
        }
    };
}

macro_rules! impl_decoration_aspect_members {
    (struct $name:ty {
        $(
            $member:ident: $member_ty:ty,
        )+
    }) => {
        $(
            impl crate::decorations::GetDecorationAspect<$member_ty> for $name {
                fn get_decoration_aspect_impl(&self) -> &$member_ty {
                    &self.$member
                }
            }

            impl crate::decorations::GetDecorationAspectMut<$member_ty> for $name {
                fn get_decoration_aspect_mut_impl(&mut self) -> &mut $member_ty {
                    &mut self.$member
                }
            }
        )+
    };
}

macro_rules! decl_translation_state {
    (
        $vis:vis struct $state_name:ident<$($l:lifetime),+> {
            base: $base_type:ty,
            $(
                $member_name:ident: $member_type:ty,
            )*
        }
    ) => {
        $vis struct $state_name<$($l),+> {
            $vis base: $base_type,
            $(
                $vis $member_name: $member_type,
            )*
        }

        impl<$($l),+> core::ops::Deref for $state_name<$($l),+> {
            type Target = $base_type;
            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }

        impl<$($l),+> core::ops::DerefMut for $state_name<$($l),+> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }
    };
}
