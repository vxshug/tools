use futures_util::StreamExt;
use reqwest::header;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use tracing::{error, info, trace, warn, Level};

pub(crate) mod entity;
const APPLITIONS: &'static str = "https://lora.heltec.org/api/v3/applications";
const DEVICES: &'static str = "https://lora.heltec.org/api/v3/applications/{}/devices";
const EVENTS: &'static str = "https://lora.heltec.org/api/v3/events";

slint::include_modules!();

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::hourly("./", "counter.log");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_ansi(false)
        .with_writer(file_appender)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    info!("开始计数");
    let app = MainWindow::new().unwrap();
    let weak1 = app.as_weak();
    let weak2 = app.as_weak();
    let weak3 = app.as_weak();

    let mut handler = None::<slint::JoinHandle<()>>;

    app.global::<State>().on_enter_key(move |v| {
        let app1 = weak1.clone();
        let mut headers = header::HeaderMap::new();
        let token = v.trim();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap(),
        );
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        slint::spawn_local(async move {
            let a = app1.clone();
            let s: entity::ApplictaionResp = builder
                .build()
                .unwrap()
                .get(APPLITIONS)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            info!("获取到的应用: {:?}", s);
            slint::invoke_from_event_loop(move || {
                let app = a.unwrap();
                let mut v = vec![];
                for pp in s.applications {
                    let ss = SharedString::from(pp.ids.application_id);
                    v.push(ss);
                }
                let the_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(v));
                let the_model_rc = ModelRc::from(the_model.clone());
                app.set_model(the_model_rc);
            })
            .unwrap();
        })
        .unwrap();
    });

    app.global::<State>().on_listen(move |token, application| {
        let app = weak2.clone();
        let app = app.unwrap();
        let mut headers = header::HeaderMap::new();
        let token = token.trim();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap(),
        );
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        let h = slint::spawn_local(async move {
            let client = builder.build().unwrap();
            let url = DEVICES.replace("{}", &application);
            let old_devices: Vec<_> = app.get_devices().iter().collect();

            let devices: entity::EndDevices =
                client.get(url).send().await.unwrap().json().await.unwrap();
            let devices: Vec<_> = devices
                .end_devices
                .into_iter()
                .map(|d| entity::Identifier { device_ids: d.ids })
                .collect();
            let list: Vec<_> = devices
                .iter()
                .map(|e| DeviceItem {
                    name: SharedString::from(&e.device_ids.device_id),
                    count: 0,
                })
                .collect();

            let list: Vec<_> = list
                .into_iter()
                .map(|mut dev| {
                    for old_device in &old_devices {
                        if old_device.name == dev.name {
                            dev.count = old_device.count;
                        }
                    }
                    dev
                })
                .collect();

            let list = Rc::new(slint::VecModel::from(list));
            app.set_devices(list.into());
            let idents = entity::Identifiers {
                identifiers: devices,
                names: vec![String::from("as.up.data.forward")],
            };
            loop {
                let mut stream = client
                    .post(EVENTS)
                    .json(&idents)
                    .send()
                    .await
                    .unwrap()
                    .bytes_stream();
                info!("连接事件成功");
                while let Some(item) = stream.next().await {
                    let devices: Vec<_> = app.get_devices().iter().collect();
                    if let Ok(byte) = item {
                        let res = String::from_utf8_lossy(byte.as_ref());
                        if res.find("events.stream.start").is_some() {
                            continue;
                        }
                        if res.find("error").is_some() {
                            error!("事件出现错误: {}", res);
                            break;
                        }
                        let v: Vec<_> = devices
                            .into_iter()
                            .map(|mut device| {
                                let s = format!("\"device_id\":\"{}\"", device.name);
                                match res.find(&s) {
                                    None => device,
                                    Some(_) => {
                                        trace!(
                                            "设备: {} -- current: {} --- +1",
                                            device.name,
                                            device.count
                                        );
                                        device.count += 1;
                                        device
                                    }
                                }
                            })
                            .collect();
                        let list = Rc::new(slint::VecModel::from(v));
                        app.set_devices(list.into());
                    }
                }
                warn!("事件连接断开");
            }
        })
        .unwrap();
        let old = handler.take();
        old.map(|o| o.abort());
        handler.replace(h);
    });

    app.global::<State>().on_zero(move |name| {
        let app3 = weak3.clone();
        let app = app3.unwrap();
        info!("设备: {}, 计数归零", name);
        let devices: Vec<_> = app
            .get_devices()
            .iter()
            .map(|mut device| {
                if device.name == name {
                    device.count = 0;
                }
                device
            })
            .collect();
        let devices = Rc::new(slint::VecModel::from(devices));
        app.set_devices(devices.into());
    });
    app.run().unwrap();
}
