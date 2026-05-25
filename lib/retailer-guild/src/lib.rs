pub mod io {
    pub use crate::models::RetailerCode;
}

mod models {
    #[allow(non_camel_case_types)]
    pub enum RetailerCode {
        MINISFORUM_EU,
        MINISFORUM_US,
        MINISFORUM_UK,
        MINISFORUM_FR,
        MINISFORUM_CA,
        MINISFORUM_AU,
    }

    impl RetailerCode {
        pub fn as_string(&self) -> &'static str {
            match self {
                RetailerCode::MINISFORUM_EU => "MinisForumEU",
                RetailerCode::MINISFORUM_US => "MinisForumUS",
                RetailerCode::MINISFORUM_UK => "MinisForumUK",
                RetailerCode::MINISFORUM_FR => "MinisForumFR",
                RetailerCode::MINISFORUM_CA => "MinisForumCA",
                RetailerCode::MINISFORUM_AU => "MinisForumAU",
            }
        }
    }

    impl From<String> for RetailerCode {
        fn from(value: String) -> Self {
            match value.as_str() {
                "MinisForumEU" => RetailerCode::MINISFORUM_EU,
                "MinisForumUS" => RetailerCode::MINISFORUM_US,
                "MinisForumUK" => RetailerCode::MINISFORUM_UK,
                "MinisForumFR" => RetailerCode::MINISFORUM_FR,
                "MinisForumCA" => RetailerCode::MINISFORUM_CA,
                "MinisForumAU" => RetailerCode::MINISFORUM_AU,
                other => panic!("unknown RetailerCode: {other}"),
            }
        }
    }
}
