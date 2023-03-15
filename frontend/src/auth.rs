use std::rc::Rc;

use base64::{Engine, engine::general_purpose};
use gloo_net::http::Request;
use log::{error, info};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::{AppContext, AppRoute};

pub struct AuthPage {
    context: Rc<AppContext>,
    _listener: ContextHandle<Rc<AppContext>>,
    user: String,
    password: String,
    redirect: bool,
    auth_err: bool,
}

pub enum AuthPageMessage {
    ContextChanged(Rc<AppContext>),
    UpdateUser(String),
    UpdatePassword(String),
    Redirect,
    AuthFailed,
    TryAuth,
}

impl Component for AuthPage {
    type Message = AuthPageMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _listener) = ctx
            .link()
            .context::<Rc<AppContext>>(ctx.link().callback(AuthPageMessage::ContextChanged))
            .unwrap();

        if context.back_url == None || context.retransmit == None {
            ctx.link().callback(move |()| AuthPageMessage::Redirect).emit(());
        }

        Self {
            context,
            _listener,
            user: "".to_string(),
            password: "".to_string(),
            redirect: false,
            auth_err: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AuthPageMessage::ContextChanged(context) => self.context = context,
            AuthPageMessage::UpdateUser(user) => self.user = user,
            AuthPageMessage::UpdatePassword(password) => self.password = password,
            AuthPageMessage::Redirect => self.redirect = true,
            AuthPageMessage::AuthFailed => self.auth_err = true,
            AuthPageMessage::TryAuth => {
                let redirect_callback = ctx.link().callback(move |()| AuthPageMessage::Redirect);
                let auth_failed_callback = ctx.link().callback(move |()| AuthPageMessage::AuthFailed);
                let context = &self.context;
                let json = context.retransmit.clone();
                let base64_engine = general_purpose::STANDARD;
                let credentials = base64_engine.encode(format!("{}:{}", self.user, self.password));
                if let Some(json) = json {
                    spawn_local(async move {
                        let json = json.clone();
                        let request = Request::post("http://127.0.0.1:8088/api/part/create")
                            .header("Content-Type", "text/html; charset=utf-8")
                            .header("Authorization", &format!("Basic {}", credentials))
                            .json(&json)
                            .unwrap()
                            .send()
                            .await
                            .unwrap();

                        if request.ok() {
                            redirect_callback.emit(());
                            info!("Redirecting");
                        } else {
                            auth_failed_callback.emit(());
                            info!("Auth failed");
                        }
                    });
                } else {
                    error!("Failed to get json to retransmit!");
                }
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let user_callback = ctx.link().callback(move |event: InputEvent| { 
            let event: Event = event.dyn_into().unwrap();
            let event_target = event.target().unwrap();
            let html_element: HtmlInputElement = event_target.dyn_into().unwrap();
            AuthPageMessage::UpdateUser(html_element.value())
        });

        let password_callback = ctx.link().callback(move |event: InputEvent| { 
            let event: Event = event.dyn_into().unwrap();
            let event_target = event.target().unwrap();
            let html_element: HtmlInputElement = event_target.dyn_into().unwrap();
            AuthPageMessage::UpdatePassword(html_element.value())
        });

        let try_auth_callback = ctx.link().callback(move |()| AuthPageMessage::TryAuth);
        let submit_callback = {
            let try_auth_callback = try_auth_callback.clone();
            Callback::from(move |_| {
                try_auth_callback.emit(());
            })
        };

        let route = self.context.back_url.clone();

        html! {
            <div class={classes!("login-form")}>
                <h4 class={classes!("login-form-text")}>{"User"}</h4>
                <input type="text" oninput={user_callback} value={self.user.clone()} />
                <h4 class={classes!("login-form-text")}>{"Password"}</h4>
                <input type="password" oninput={password_callback} value={self.password.clone()} />
                <div onclick={submit_callback} class={classes!("login-form-submit")}>
                    <h2>{"Submit"}</h2>
                </div>
                if self.auth_err {
                    <h3 class={classes!("login-form-auth-failed")}>{"Authentication failed"}</h3>
                }

                if self.redirect {
                    if let Some(route) = route {
                        <Redirect<AppRoute> to={route} />
                    } else {
                        <Redirect<AppRoute> to={AppRoute::Home} />
                    }
                }
            </div>
        }
    }
}
