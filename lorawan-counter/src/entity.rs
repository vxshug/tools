
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Identifiers {
    pub identifiers: Vec<Identifier>,
    pub names: Vec<String>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Identifier {
    pub device_ids: DeviceId
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeviceId {
    pub device_id: String,
    pub application_ids: ApplictaionID
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EndDevices {
    pub end_devices: Vec<Device>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub ids: DeviceId
}

