use std::rc::Rc;
use reqwest::header;
use slint::{VecModel, SharedString, ModelRc};

pub(crate) mod entity;
const APPLITIONS: &'static str = "https://lora.heltec.org/api/v3/applications";

slint::include_modules!();

#[tokio::main]
async fn main() {
    let app = MainWindow::new().unwrap();
    let weak = app.as_weak();
    slint::invoke_from_event_loop(||{
        println!("123");
    });
    app.global::<State>().on_enter_key(move |v| {
        let app1 = weak.clone();
        let app = weak.unwrap();
        let model = app.get_model();
        let mut headers = header::HeaderMap::new();
        let token = v.trim(); 
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap());
        let builder = reqwest::ClientBuilder::new().default_headers(headers);
        tokio::spawn(async move {
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
            });
        });
    });
    app.run().unwrap();
}
