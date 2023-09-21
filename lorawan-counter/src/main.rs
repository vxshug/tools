use std::{cell, fs};
use std::rc::Rc;
use futures_util::{FutureExt, StreamExt};
use reqwest::header;
use slint::{VecModel, SharedString, ModelRc, Model};
use crate::entity::{ApplictaionID, DeviceId, Identifier};

pub(crate) mod entity;
const APPLITIONS: &'static str = "https://lora.heltec.org/api/v3/applications";
const DEVICES: &'static str = "https://lora.heltec.org/api/v3/applications/{}/devices";
const EVENTS: &'static str = "https://lora.heltec.org/api/v3/events";

slint::include_modules!();

#[tokio::main]
async fn main() {
    let app = MainWindow::new().unwrap();
    let weak1 = app.as_weak();
    let weak2 = app.as_weak();

    let mut handler = None::<slint::JoinHandle<()>>;

    app.global::<State>().on_enter_key(move |v| {
        let app1 = weak1.clone();
        let app = weak1.unwrap();
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
        let app = weak2.clone();
        let app = app.unwrap();
        let mut headers = header::HeaderMap::new();
        let token = token.trim();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap());
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        let h = slint::spawn_local(async move {
            let client = builder.build().unwrap();
            let url = DEVICES.replace("{}", &application);
            let old_devices: Vec<_> = app.get_devices().iter().collect();

            let devices: entity::EndDevices  = client.get(url).send().await.unwrap().json().await.unwrap();
            let mut devices: Vec<_> = devices.end_devices.into_iter().map(|d| entity::Identifier {device_ids: d.ids}).collect();
            let list: Vec<_> = devices.iter().map(|e| DeviceItem {name: SharedString::from(&e.device_ids.device_id), count: 0}).collect();

            let list: Vec<_> = list.into_iter().map(| mut dev| {
                for old_device in &old_devices {
                    if old_device.name == dev.name {
                        dev.count = old_device.count;
                    }
                };
                dev
            }).collect();

            let list = Rc::new(slint::VecModel::from(list));
            app.set_devices(list.into());
            let idents = entity::Identifiers { identifiers: devices, names: vec![String::from("as.up.data.forward")] };
            loop {
                let mut stream = client.post(EVENTS).json(&idents).send().await.unwrap()
                    .bytes_stream();
                while let Some(item) = stream.next().await {
                    let mut devices: Vec<_> = app.get_devices().iter().collect();
                    if let Ok(byte) = item {
                        let res = String::from_utf8_lossy(byte.as_ref());
                        if res.find("events.stream.start").is_some() {
                            continue;
                        }
                        println!("Chunk: {:?}", res);
                        if res.find("error").is_some() {
                            fs::write("./log.txt", res.as_bytes()).unwrap();
                            break;
                        }
                        let v: Vec<_> = devices.into_iter().map(|mut device|  {
                            let s = format!("\"device_id\":\"{}\"", device.name);
                            match res.find(&s) {
                                None => {device}
                                Some(_) => {
                                    device.count += 1;
                                    device
                                }
                            }
                        }).collect();
                        let list = Rc::new(slint::VecModel::from(v));
                        app.set_devices(list.into());
                    }
                }
                println!("CLOSE");
            }
        }).unwrap();
        let old = handler.take();
        old.map(|o| o.abort());
        handler.replace(h);

    });
    app.run().unwrap();
}
