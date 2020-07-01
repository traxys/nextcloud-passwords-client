#[doc(hidden)]
#[macro_export]
macro_rules! create_details {
    (
        pub struct $struct:ident {
        $(
            pub $name:ident : bool
        ),*
        $(,)?
    }) => {
        /// Amount of optional details requested
        #[derive(Default, Debug)]
        pub struct $struct {
            $(
                pub $name: bool,
            )*
        }

        impl $struct {
            pub(crate) fn to_string(&self) -> String {
                let mut s = "model".into();
                $(
                    if self.$name {
                        s += concat!("+", stringify!($name));
                    }
                )*
                s
            }

            pub fn new() -> Self {
                Default::default()
            }

            $(
                pub fn $name(self) -> Self {
                    Self {
                        $name: true,
                        ..self
                    }
                }
            )*

        }
    };
}


// Tags: versioned, create(optional | required), update(optional | required)

#[doc(hidden)]
#[macro_export]
macro_rules! create_binding {
    (
        $(#[$s_attr:meta])*
        pub
        struct $name:ident {
            $(
                $(#[$f_attr:meta])*
                pub
                $field:ident : $type:ty [$($tags:tt)*]
            ),* $(,)?
        }
    ) => (
        create_binding! {
            @name $name
            @meta ($($s_attr)*)
            @create_new ()
            @create ()
            @update_new ()
            @update ()
            @versioned ()
            @not_versioned ()
            $(
                (
                    $(#[$f_attr])*
                    $field : $type
                ) [$($tags)*]
            )*
        }
    );

    (
        @name $name:ident
        @meta (
            $($s_attr:tt)*
        )
        @create_new (
            $(
                (
                    $(#[$cn_attr:tt])*
                    $cn_field:ident : $cn_type:ty
                )
            )*
        )
        @create (
            $(
                (
                    $(#[$c_attr:tt])*
                    $c_field:ident : $c_type:ty
                )
            )*
        )
        @update_new (
            $(
                (
                    $(#[$un_attr:tt])*
                    $un_field:ident : $un_type:ty
                )
            )*
        )
        @update (
            $(
                (
                    $(#[$u_attr:tt])*
                    $u_field:ident : $u_type:ty
                )
            )*
        )
        @versioned (
            $(
                (
                    $(#[$v_attr:tt])*
                    $v_field:ident : $v_type:ty
                )
            )*
        )
        @not_versioned (
            $(
                (
                    $(#[$n_attr:tt])*
                    $n_field:ident : $n_type:ty
                )
            )*
        )
        // nothing left to parse
    ) => (
        ::paste::item! {
            $(#[$s_attr])*
            pub
            struct $name {
                $(
                    $(#[$n_attr])*
                    pub
                    $n_field : $n_type,
                )*
                #[serde(flatten)]
                pub
                versioned : [<Versioned $name>],
            }
            ::doc_comment::doc_comment! { concat!("versioned properties of [", stringify!($name), "]") }
            $(#[$s_attr])*
            pub
            struct [<Versioned $name>] {
                $(
                    $(#[$v_attr])*
                    pub
                    $v_field : $v_type,
                )*
            }

            ::doc_comment::doc_comment!{ concat!("Builder to create [", stringify!($name) , "], the values in the builder are optional values") }
            $(#[$s_attr])*
            pub
            struct [<Create $name>] {
                $(
                    $(#[$cn_attr])*
                    $cn_field: $cn_type,
                )*
                $(
                    $(#[$c_attr])*
                    pub
                    $c_field : Option<$c_type>,
                )*
            }

            impl [<Create $name>] {
                pub
                fn new ($($cn_field: $cn_type,)*)
                  -> Self
                {
                    Self {
                        $(
                            $cn_field,
                        )*
                        $(
                            $c_field: None,
                        )*
                    }
                }

                $(
                    pub
                    fn $c_field (self: Self, $c_field: $c_type)
                      -> Self
                    {
                        Self { $c_field: Some($c_field), ..self }
                    }
                )*
            }

            ::doc_comment::doc_comment! {
                concat!("Builder to update [", stringify!($name), "], the values in the builder are optional values")
            }
            $(#[$s_attr])*
            pub
            struct [<Update $name>] {
                $(
                    $(#[$un_attr])*
                    $un_field: $un_type,
                )*
                $(
                    $(#[$u_attr])*
                    pub
                    $u_field : Option<$u_type>,
                )*
            }

            impl [<Update $name>] {
                pub
                fn new ($($un_field: $un_type,)*)
                  -> Self
                {
                    Self {
                        $(
                            $un_field,
                        )*
                        $(
                            $u_field: None,
                        )*
                    }
                }

                $(
                    pub
                    fn $u_field (self: Self, $u_field: $u_type)
                      -> Self
                    {
                        Self { $u_field: Some($u_field), ..self }
                    }
                )*
            }
        }
    );

    // Create new 
    (
        @name $name:ident
        @meta $meta:tt
        @create_new ($($create_new:tt)*)
        @create $create:tt
        @update_new $update_new:tt
        @update $update:tt
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
            $current:tt [create(required) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new ( $($create_new)* $current )
            @create $create
            @update_new $update_new
            @update $update
            @versioned $versioned
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Create
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create ($($create:tt)*)
        @update_new $update_new:tt
        @update $update:tt
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
            $current:tt [create(optional) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create ($($create)* $current)
            @update_new $update_new
            @update $update
            @versioned $versioned
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Update new
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new ($($update_new:tt)*)
        @update $update:tt
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
            $current:tt [update(required) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new ($($update_new)* $current)
            @update $update
            @versioned $versioned
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Update 
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new $update_new:tt
        @update ($($update:tt)*)
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
            $current:tt [update(optional) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update ($($update)* $current)
            @versioned $versioned
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Versioned
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new $update_new:tt
        @update $update:tt
        @versioned ( $($versioned:tt)* )
        @not_versioned $not_versioned:tt
            $current:tt [versioned $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update $update
            @versioned ( $($versioned)* $current )
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Nothing
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new $update_new:tt
        @update $update:tt
        @versioned $versioned:tt
        @not_versioned ( $($not_versioned:tt)* )
            $current:tt []
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update $update
            @versioned $versioned
            @not_versioned ( $($not_versioned)* $current )
                $($rest)*
        }
    );
}
