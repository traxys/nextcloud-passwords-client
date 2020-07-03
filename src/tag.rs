use crate::{create_binding, create_details, AuthenticatedApi, Color};
use serde::{Serialize, Deserialize};

pub struct TagApi<'a> {
    pub(crate) api: &'a AuthenticatedApi,
}

create_details! {
    pub struct Details {
        pub revisions: bool,
        pub passwords: bool,
    }
}

create_binding! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Tag {
        pub id: String [],
        pub label: String [versioned],
        pub color: Color [],
    }
}
