use std::rc::Rc;
use futures_util::StreamExt;
use reqwest::header;
use slint::{VecModel, SharedString, ModelRc};

pub(crate) mod entity;
const APPLITIONS: &'static str = "https://lora.heltec.org/api/v3/applications";
const DEVICES: &'static str = "https://lora.heltec.org/api/v3/applications/{}/devices";
const EVENTS: &'static str = "https://lora.heltec.org/api/v3/events";

slint::include_modules!();

#[tokio::main]
async fn main() {
    let app = MainWindow::new().unwrap();
    let weak = app.as_weak();

    app.global::<State>().on_enter_key(move |v| {
        let app1 = weak.clone();
        let app = weak.unwrap();
        let model = app.get_model();
        let mut headers = header::HeaderMap::new();
        let token = v.trim(); 
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap());
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        slint::spawn_local(async move {
            let a = app1.clone();
            let s: entity::ApplictaionResp = builder.build().unwrap().get(APPLITIONS).send().await.unwrap().json().await.unwrap();
            slint::invoke_from_event_loop(move || {
                let app = a.unwrap(); 
                let mut v = vec![];
                for pp in s.applications {
                    let ss = SharedString::from(pp.ids.application_id);
                    v.push(ss);
                } 
                let the_model : Rc<VecModel<SharedString>> = Rc::new(VecModel::from(v));
                let the_model_rc = ModelRc::from(the_model.clone());
                app.set_model(the_model_rc);
            }).unwrap();
        }).unwrap();
    });

    app.global::<State>().on_listen(move |token, application| {
        let mut headers = header::HeaderMap::new();
        let token = token.trim();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap());
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        slint::spawn_local(async move {
            let client = builder.build().unwrap();
            let url = DEVICES.replace("{}", &application);
            let devices: entity::EndDevices  = client.get(url).send().await.unwrap().json().await.unwrap();
            println!("{:?}", devices);
            let devices: Vec<_> = devices.end_devices.into_iter().map(|d| entity::Identifier {device_ids: d.ids}).collect();
            let idents = entity::Identifiers { identifiers: devices, names: vec![String::from("as.up.data.forward")] };
            let mut stream = client.post(EVENTS).json(&idents).send().await.unwrap()
                .bytes_stream();

            while let Some(item) = stream.next().await {
                println!("Chunk: {:?}", item);
            }
        }).unwrap();
    });
    app.run().unwrap();
}
