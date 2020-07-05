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

// Tags: versioned, create(optional | required), update(optional | required), search
//

#[derive(Debug)]
pub struct SearchQuery<T: serde::Serialize> {
    value: T,
    query: QueryKind,
}

#[derive(Debug)]
pub enum QueryKind {
    Exact,
    Equals,
    NotEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum Criteria {
    Value(serde_json::Value),
    Search(&'static str, serde_json::Value),
}

impl<T: serde::Serialize> SearchQuery<T> {
    pub(crate) fn to_criteria(&self) -> Result<Criteria, serde_json::Error> {
        let value = serde_json::to_value(&self.value)?;
        Ok(match self.query {
            QueryKind::Exact => Criteria::Value(value),
            QueryKind::Equals => Criteria::Search("eq", value),
            QueryKind::NotEqual => Criteria::Search("ne", value),
            QueryKind::LessThan => Criteria::Search("lt", value),
            QueryKind::GreaterThan => Criteria::Search("gt", value),
            QueryKind::LessOrEqual => Criteria::Search("le", value),
            QueryKind::GreaterOrEqual => Criteria::Search("ge", value),
        })
    }
}
/*

    TEMPLATE UTILISATION

create_calls! {
    PasswordApi where 
        Endpoint = "1.0/password",
        Details: Details,
        Type: Password,
        Create: CreatePassword,
        Update: UpdatePassword,
        Error: Error,
        Identifier: PasswordIdentifier,
        Trashed: TrashedIdentifier,
        Criteria: PasswordSearch 
    {
        pub async fn list(&self, details: Option<Details>) -> Result<Vec<Type>, Error>;

        pub async fn get(&self, details: Option<Details>, id: uuid::Uuid) -> Result<Type, Error>;

        pub async fn find(&self, criteria: Criteria, details: Option<Details>) -> Result<Vec<Type>, Error>;

        pub async fn create(&self, value: Create) -> Result<Identifier, Error>;

        pub async fn update(&self, value: Update) -> Result<Identifier, Error>;

        pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Trashed, Error>;

        pub async fn restore(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Identifier, Error>;
    }
}

 */

#[doc(hidden)]
#[macro_export]
macro_rules! create_calls {
    (
        $base:ident where
            Endpoint = $endpoint:expr,
            Details: $details:ty,
            Type: $ty:ty,
            Create: $create:ty,
            Update: $update:ty,
            Error: $err:ty,
            Identifier: $ident:ty,
            Trashed: $trashed:ty,
            Criteria: $criteria:ty $(,)?
        {
            $(
            List;
            $(#[$meta_list:meta])*
            pub async fn list(&self, details: Option<Details>) -> Result<Vec<Type>, Error>;
            )?

            $(
            Get;
            $(#[$meta_get:meta])*
            pub async fn get(&self, details: Option<Details>, id: uuid::Uuid) -> Result<Type, Error>;
            )?

            $(
            Find;
            $(#[$meta_find:meta])*
            pub async fn find(&self, criteria: Criteria, details: Option<Details>) -> Result<Vec<Type>, Error>;
            )?

            $(
            Create;
            $(#[$meta_create:meta])*
            pub async fn create(&self, value: Create) -> Result<Identifier, Error>;
            )?

            $(
            Update;
            $(#[$meta_update:meta])*
            pub async fn update(&self, value: Update) -> Result<Identifier, Error>;
            )?

            $(
            Delete;
            $(#[$meta_delete:meta])*
            pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Trashed, Error>;
            )?

            $(
            Restore;
            $(#[$meta_restore:meta])*
            pub async fn restore(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<Identifier, Error>;
            )?
        }
    ) => {
        ::doc_comment::doc_comment! { concat!("Actions on the ", stringify!($base), " API"),
        pub struct $base<'a> {
            pub(crate) api: &'a crate::AuthenticatedApi,
        }}

        impl<'a> $base<'a> {
            $(
            $(#[$meta_list])*
            pub async fn list(&self, details: Option<$details>) -> Result<Vec<$ty>, $err> {
                #[derive(serde::Serialize, serde::Deserialize)]
                struct DetailsStr {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    details: Option<String>,
                }
                self.api
                    .passwords_post(
                        concat!($endpoint, "/list"),
                        DetailsStr {
                            details: details.map(|d| d.to_string()),
                        },
                    )
                    .await
            }
            )?

            $(
            $(#[$meta_get])*
            pub async fn get(&self, details: Option<$details>, id: uuid::Uuid) -> Result<$ty, $err> {
                #[derive(Serialize, Deserialize)]
                struct Show {
                    id: uuid::Uuid,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    details: Option<String>,
                }
                let request = Show {
                    id,
                    details: details.map(|d| d.to_string()),
                };
                self.api
                    .passwords_post(concat!($endpoint, "/show"), request)
                    .await
            }
            )?

            $(
            $(#[$meta_find])*
            pub async fn find(
                &self,
                criteria: $criteria,
                details: Option<$details>,
            ) -> Result<Vec<$ty>, $err> {
                #[derive(Serialize)]
                struct Request {
                    criteria: $criteria,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    details: Option<String>,
                }
                let request = Request {
                    criteria,
                    details: details.map(|d| d.to_string()),
                };
                self.api
                    .passwords_post(concat!($endpoint, "/find"), request)
                    .await
            }
            )?

            $(
            $(#[$meta_create])*
            pub async fn create(&self, value: $create) -> Result<$ident, $err> {
                self.api
                    .passwords_post(concat!($endpoint, "/create"), value)
                    .await
            }
            )?

            $(
            $(#[$meta_update])*
            pub async fn update(&self, folder: $update) -> Result<$ident, $err> {
                self.api
                    .passwords_post(concat!($endpoint, "/update"), folder)
                    .await
            }
            )?

            $(
            $(#[$meta_delete])*
            pub async fn delete(&self, id: uuid::Uuid, revision: Option<uuid::Uuid>) -> Result<$trashed, $err> {
                #[derive(Serialize)]
                struct Request {
                    id: uuid::Uuid,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    revision: Option<uuid::Uuid>,
                }
                self.api
                    .passwords_delete(concat!($endpoint, "/delete"), Request { id, revision })
                    .await
            }
            )?

            $(
            $(#[$meta_restore])*
            pub async fn restore(
                &self,
                id: uuid::Uuid,
                revision: Option<uuid::Uuid>,
            ) -> Result<$ident, $err> {
                #[derive(Serialize)]
                struct Request {
                    id: uuid::Uuid,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    revision: Option<uuid::Uuid>,
                }
                self.api
                    .passwords_patch(concat!($endpoint, "/restore"), Request { id, revision })
                    .await
            }
            )?
        }
    };
}

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
            @search ()
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
            $(
                (
                    $(#[$cn_attr:tt])*
                    $cn_field:ident : $cn_type:ty
                )
            )+
            )?
        )
        @create (
            $(
            $(
                (
                    $(#[$c_attr:tt])*
                    $c_field:ident : $c_type:ty
                )
            )+
            )?
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
        @search (
            $(
            $(
                (
                    $(#[$se_attr:tt])*
                    $se_field:ident : $se_type:ty
                )
            )+)?
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
            ::doc_comment::doc_comment! { concat!("versioned properties of [", stringify!($name), "]"),
            $(#[$s_attr])*
            pub
            struct [<Versioned $name>] {
                $(
                    $(#[$v_attr])*
                    pub
                    $v_field : $v_type,
                )*
            }
            }

            $(
            ::doc_comment::doc_comment!{ concat!("Builder to create [", stringify!($name) , "], the values in the builder are optional values"),
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub
            struct [<Create $name>] {
                $(
                    $(#[$cn_attr])*
                    $cn_field: $cn_type,
                )+
                $(
                    $(#[$c_attr])*
                    pub
                    $c_field : Option<$c_type>,
                )+
            }
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
            )?

            $(
            ::doc_comment::doc_comment! {
                "Builder to add search criterias (see the `find` method)",
            #[derive(serde::Serialize, Default)]
            pub struct [<$name Search>] {
                $(
                    #[serde(skip_serializing_if = "Option::is_none")]
                    $(#[$se_attr])?
                    $se_field: Option<crate::utils::Criteria>,
                )+
            }
            }
            impl [<$name Search>] {
                pub fn new() -> Self {
                    Default::default()
                }
                $(
                    pub fn [<and_ $se_field>](self, query: crate::utils::SearchQuery<$se_type>) -> Result<Self, crate::Error> {
                        Ok(Self {
                            $se_field: Some(query.to_criteria()?),
                            ..self
                        })
                    }
                )+
            }
            )?

            ::doc_comment::doc_comment! {
                concat!("Builder to update [", stringify!($name), "], the values in the builder are optional values"),
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
        @search $search:tt
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
            @search $search
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
        @search $search:tt
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
            @search $search
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
        @search $search:tt
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
            @search $search
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
        @search $search:tt
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
            @search $search
            @versioned $versioned
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );

    // Search
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new $update_new:tt
        @update $update:tt
        @search ($($search:tt)*)
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
            $current:tt [search $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update $update
            @search ($($search)* $current)
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
        @search $search:tt
        @versioned ( $($versioned:tt)* )
        @not_versioned $not_versioned:tt
            $current:tt [versioned(true) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update $update
            @search $search
            @versioned ( $($versioned)* $current )
            @not_versioned $not_versioned
                $($current [$($tags)+])?
                $($rest)*
        }
    );
    // Not Versioned
    (
        @name $name:ident
        @meta $meta:tt
        @create_new $create_new:tt
        @create $create:tt
        @update_new $update_new:tt
        @update $update:tt
        @search $search:tt
        @versioned $versioned:tt
        @not_versioned ( $($not_versioned:tt)* )
            $current:tt [versioned(false) $($($tags:tt)+)?]
            $($rest:tt)*
    ) => (
        create_binding! {
            @name $name
            @meta $meta
            @create_new $create_new
            @create $create
            @update_new $update_new
            @update $update
            @search $search
            @versioned $versioned
            @not_versioned ( $($not_versioned)* $current)
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
        @search $search:tt
        @versioned $versioned:tt
        @not_versioned $not_versioned:tt
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
            @search $search
            @versioned $versioned
            @not_versioned $not_versioned
                $($rest)*
        }
    );
}
