pub mod io {
    pub use crate::models::RetailerCode;
}

mod models {
    use serde::{Deserialize, Serialize};

    #[allow(non_camel_case_types)]
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(into = "String", try_from = "String")]
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

        pub fn homepage_url(&self) -> &'static str {
            match self {
                RetailerCode::MINISFORUM_EU => "https://minisforumpc.eu/",
                RetailerCode::MINISFORUM_US => "https://store.minisforum.com/",
                RetailerCode::MINISFORUM_UK => "https://www.minisforum.uk/",
                RetailerCode::MINISFORUM_FR => "https://minisforumpc.fr/",
                RetailerCode::MINISFORUM_CA => "https://ca.minisforum.com/",
                RetailerCode::MINISFORUM_AU => "https://au.minisforum.com/",
            }
        }

        pub fn shop_homepage_url(&self) -> &'static str {
            self.homepage_url()
        }
    }

    impl TryFrom<String> for RetailerCode {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            match value.as_str() {
                "MinisForumEU" => Ok(RetailerCode::MINISFORUM_EU),
                "MinisForumUS" => Ok(RetailerCode::MINISFORUM_US),
                "MinisForumUK" => Ok(RetailerCode::MINISFORUM_UK),
                "MinisForumFR" => Ok(RetailerCode::MINISFORUM_FR),
                "MinisForumCA" => Ok(RetailerCode::MINISFORUM_CA),
                "MinisForumAU" => Ok(RetailerCode::MINISFORUM_AU),
                other => Err(format!("unknown RetailerCode: {other}")),
            }
        }
    }

    impl From<RetailerCode> for String {
        fn from(code: RetailerCode) -> Self {
            code.as_string().to_string()
        }
    }
}
