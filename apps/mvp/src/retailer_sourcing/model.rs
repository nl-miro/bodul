use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Retailer {
    pub id: Uuid,
    pub name: String,
    pub code: RetailerCode,
}

pub struct InMemoryRetailerRepository {
    retailers: Vec<Retailer>,
}

impl InMemoryRetailerRepository {
    pub fn new() -> Self {
        let retailers = vec![
            Retailer {
                id: Uuid::parse_str("718c82c3-1b1e-4796-8f14-a32f3227cc00").unwrap(),
                name: "MinisForumEu".to_string(),
                code: RetailerCode::MinisForumEu,
            },
            Retailer {
                id: Uuid::parse_str("5cfcd46b-7966-4193-94a0-7e9954f14e5c").unwrap(),
                name: "MinisForumUs".to_string(),
                code: RetailerCode::MinisForumUs,
            },
            Retailer {
                id: Uuid::parse_str("b7d63252-3121-4370-af65-bc0b69345680").unwrap(),
                name: "MinisForumUk".to_string(),
                code: RetailerCode::MinisForumUk,
            },
            Retailer {
                id: Uuid::parse_str("2eec5987-ac7d-4fd9-ba84-0aa431cf903d").unwrap(),
                name: "MinisForumFr".to_string(),
                code: RetailerCode::MinisForumFr,
            },
            Retailer {
                id: Uuid::parse_str("4181c457-74c7-40dc-b225-7a4461fdb097").unwrap(),
                name: "MinisForumCa".to_string(),
                code: RetailerCode::MinisForumCa,
            },
            Retailer {
                id: Uuid::parse_str("acb3e558-6c04-43f6-ae0d-4573c784c1a1").unwrap(),
                name: "MinisForumAu".to_string(),
                code: RetailerCode::MinisForumAu,
            },
            Retailer {
                id: Uuid::parse_str("c41f0f3f-d95c-48c3-8896-50348351e549").unwrap(),
                name: "MinisForumKr".to_string(),
                code: RetailerCode::MinisForumKr,
            },
            Retailer {
                id: Uuid::parse_str("c00cc124-7cac-497f-94aa-942e9373cefd").unwrap(),
                name: "MinisForumJp".to_string(),
                code: RetailerCode::MinisForumJp,
            },
            Retailer {
                id: Uuid::parse_str("3c56cb03-15de-4523-99c5-fa96177e7993").unwrap(),
                name: "MinisForumRu".to_string(),
                code: RetailerCode::MinisForumRu,
            },
            Retailer {
                id: Uuid::parse_str("49cf7ebf-6a69-4380-8104-eed6d1916d2b").unwrap(),
                name: "MinisForumHk".to_string(),
                code: RetailerCode::MinisForumHk,
            },
            Retailer {
                id: Uuid::parse_str("61d6e617-f3de-472c-980d-484cf09200ec").unwrap(),
                name: "AnkerCom".to_string(),
                code: RetailerCode::AnkerCom,
            },
            Retailer {
                id: Uuid::parse_str("325747e5-6347-4cab-b712-ea9ca269d363").unwrap(),
                name: "AnkerJapanCom".to_string(),
                code: RetailerCode::AnkerJapanCom,
            },
            Retailer {
                id: Uuid::parse_str("6eb08789-0f96-4798-9521-ecb373d64d55").unwrap(),
                name: "AnkerKr".to_string(),
                code: RetailerCode::AnkerKr,
            },
            Retailer {
                id: Uuid::parse_str("565da80f-677e-4819-8fb0-fb894ec1249c").unwrap(),
                name: "AnkerItalyCom".to_string(),
                code: RetailerCode::AnkerItalyCom,
            },
            Retailer {
                id: Uuid::parse_str("efcbd5cd-50b8-42e0-b4e1-389d717d3300").unwrap(),
                name: "AnkerNordicsCom".to_string(),
                code: RetailerCode::AnkerNordicsCom,
            },
            Retailer {
                id: Uuid::parse_str("70d94200-6a1d-4b80-90cb-91d394bb8ffd").unwrap(),
                name: "AnkerUk".to_string(),
                code: RetailerCode::AnkerUk,
            },
            Retailer {
                id: Uuid::parse_str("12bbd756-4f1d-43b4-b3d8-42491ec847a9").unwrap(),
                name: "AnkerCa".to_string(),
                code: RetailerCode::AnkerCa,
            },
            Retailer {
                id: Uuid::parse_str("faa01ae7-5e55-4dea-9390-1583a92fee0a").unwrap(),
                name: "AnkerEu".to_string(),
                code: RetailerCode::AnkerEu,
            },
            Retailer {
                id: Uuid::parse_str("67749216-875f-46a4-8901-e234118c670d").unwrap(),
                name: "AnkerDe".to_string(),
                code: RetailerCode::AnkerDe,
            },
            Retailer {
                id: Uuid::parse_str("c9494ce9-94ed-4679-9366-72471f713c57").unwrap(),
                name: "AnkerFr".to_string(),
                code: RetailerCode::AnkerFr,
            },
            Retailer {
                id: Uuid::parse_str("5a934f5b-9404-45a8-af5b-c99f11364589").unwrap(),
                name: "AnkerPl".to_string(),
                code: RetailerCode::AnkerPl,
            },
            Retailer {
                id: Uuid::parse_str("b93dbbff-ee94-40c4-bfa7-7ac571d8041e").unwrap(),
                name: "AnkerAu".to_string(),
                code: RetailerCode::AnkerAu,
            },
            Retailer {
                id: Uuid::parse_str("c4557722-5097-4547-bc22-5eac35ead82b").unwrap(),
                name: "AnkerNz".to_string(),
                code: RetailerCode::AnkerNz,
            },
            // Retailer {
            //     id: Uuid::parse_str("fc80b8a2-df02-43db-bf03-3406ea920006").unwrap(),
            //     name: "AnkerMy".to_string(),
            //     code: RetailerCode::AnkerMy,
            // },
            Retailer {
                id: Uuid::parse_str("c36ec54e-e8a8-42a4-bb9e-7c3d4bc55d11").unwrap(),
                name: "AnkerVn".to_string(),
                code: RetailerCode::AnkerVn,
            },
        ];

        Self { retailers }
    }
}

impl InMemoryRetailerRepository {
    pub fn all(&self) -> Vec<Retailer> {
        self.retailers.clone()
    }
}

impl Default for InMemoryRetailerRepository {
    fn default() -> Self {
        Self::new()
    }
}
