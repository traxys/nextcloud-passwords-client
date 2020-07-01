#[doc(hidden)]
#[macro_export]
macro_rules! create_binding {
    (
    $(#[$s_attr:meta])?
    pub struct $name:ident {
        $(
            $(#[$f_attr:meta])*
            pub $field:ident: $type:ty [$($tags:tt)*]
        ),*
        $(,)?
    }
    ) => {
        create_binding! {
            @impl $($s_attr)*, $name
            @writable ()
            @versioned ()
            @not_versioned ()
            @rem [$($field, $type, $($f_attr)? [$($tags)*];)*] 
        }
    };

    (@impl $($s_attr:meta)?, $name:ident
     @writable ($($w_field:ident, $w_type:ty, $($w_attr:meta)?;)*)
     @versioned ($($v_field:ident, $v_type:ty, $($v_attr:meta)?;)*)
     @not_versioned ($($n_field:ident, $n_type:ty, $($n_attr:meta)?;)*)
     @rem []
    ) => {
        paste::item! {
            $(#[$s_attr])?
            pub struct $name {
                $(
                    $(#[$n_attr])?
                    pub $n_field: $n_type,
                )*
                #[serde(flatten)]
                pub versioned: [<Versioned $name>],
            }

            $(#[$s_attr])?
            pub struct [<Versioned $name>] {
                $(
                    $(#[$v_attr])?
                    pub $v_field: $v_type,
                )*
            }

            #[derive(Default)]
            $(#[$s_attr])?
            pub struct [<$name Writer>] {
                $(
                    $(#[$w_attr])?
                    pub $w_field: Option<$w_type>,
                )*
            }

            impl [<$name Writer>] {
                pub fn new() -> Self {
                    Default::default()
                }
                $(
                    pub fn $w_field(mut self, value: $w_type) -> Self {
                        self.$w_field = value;
                        self
                    }
                )*
            }
        }
    };

    (@impl $($s_attr:meta)?, $name:ident
     @writable ($($w_field:ident, $w_type:ty, $($w_attr:meta)?;)*)
     @versioned ($($v_field:ident, $v_type:ty, $($v_attr:meta)?;)*)
     @not_versioned ($($n_field:ident, $n_type:ty, $($n_attr:meta)?;)*)
     @rem [$field:ident, $type:ty, $($attr:meta)? [writable versioned]; $($rest:tt)* ]
    ) => {
        create_binding!(
            @impl $($s_attr)?, $name
            @writable ($field, $type, $($attr)?; $($w_field, $w_type, $($w_attr)?;)*)
            @versioned ($field, $type, $($attr)?; $($v_field, $v_type, $($v_attr)?;)*)
            @not_versioned ($($n_field, $n_type, $($n_attr)?;)*)
            @rem [$($rest)*]
        );
    };
    (@impl $($s_attr:meta)?, $name:ident
     @writable ($($w_field:ident, $w_type:ty, $($w_attr:meta)?;)*)
     @versioned ($($v_field:ident, $v_type:ty, $($v_attr:meta)?;)*)
     @not_versioned ($($n_field:ident, $n_type:ty, $($n_attr:meta)?;)*)
     @rem [$field:ident, $type:ty, $($attr:meta)? [versioned]; $($rest:tt)* ]
    ) => {
        create_binding!(
            @impl $($s_attr)?, $name
            @writable ($($w_field, $w_type, $($w_attr)?;)*)
            @versioned ($field, $type, $($attr)?; $($v_field, $v_type, $($v_attr)?;)*)
            @not_versioned ($($n_field, $n_type, $($n_attr)?;)*)
            @rem [$($rest)*]
        );
    };
    (@impl $($s_attr:meta)?, $name:ident
     @writable ($($w_field:ident, $w_type:ty, $($w_attr:meta)?;)*)
     @versioned ($($v_field:ident, $v_type:ty, $($v_attr:meta)?;)*)
     @not_versioned ($($n_field:ident, $n_type:ty, $($n_attr:meta)?;)*)
     @rem [$field:ident, $type:ty, $($attr:meta)? [writable]; $($rest:tt)* ]
    ) => {
        create_binding!(
            @impl $($s_attr)?, $name
            @writable ($field, $type, $($attr)?; $($w_field, $w_type, $($w_attr)?;)*)
            @versioned ($($v_field, $v_type, $($v_attr)?;)*)
            @not_versioned ($field, $type, $($attr)?; $($n_field, $n_type, $($n_attr)?;)*)
            @rem [$($rest)*]
        );
        
    };
    (@impl $($s_attr:meta)?, $name:ident
     @writable ($($w_field:ident, $w_type:ty, $($w_attr:meta)?;)*)
     @versioned ($($v_field:ident, $v_type:ty, $($v_attr:meta)?;)*)
     @not_versioned ($($n_field:ident, $n_type:ty, $($n_attr:meta)?;)*)
     @rem [$field:ident, $type:ty, $($attr:meta)? []; $($rest:tt)* ]
    ) => {
        create_binding!(
            @impl $($s_attr)?, $name
            @writable ($($w_field, $w_type, $($w_attr)?;)*)
            @versioned ($($v_field, $v_type, $($v_attr)?;)*)
            @not_versioned ($field, $type, $($attr)? ;$($n_field, $n_type, $($n_attr)?;)*)
            @rem [$($rest)*]
        );
    };
}
