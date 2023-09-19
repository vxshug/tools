
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApplictaionResp {
    pub applications: Vec<Applictaion>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Applictaion {
    pub ids: ApplictaionID
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApplictaionID {
    pub application_id: String
}
