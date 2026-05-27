use retailer_guild::io::RetailerCode;

pub struct ActiveRetailers;

impl ActiveRetailers {
    pub fn list() -> Vec<RetailerCode> {
        vec![
            RetailerCode::MINISFORUM_EU,
            RetailerCode::MINISFORUM_US,
            RetailerCode::MINISFORUM_UK,
            RetailerCode::MINISFORUM_FR,
            RetailerCode::MINISFORUM_CA,
            RetailerCode::MINISFORUM_AU,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_returns_all_known_minisforum_codes() {
        let codes = ActiveRetailers::list();
        assert_eq!(
            codes,
            vec![
                RetailerCode::MINISFORUM_EU,
                RetailerCode::MINISFORUM_US,
                RetailerCode::MINISFORUM_UK,
                RetailerCode::MINISFORUM_FR,
                RetailerCode::MINISFORUM_CA,
                RetailerCode::MINISFORUM_AU,
            ],
        );
    }
}
