use log::error;
use patternfly_yew::BackdropViewer;
use patternfly_yew::Nav;
use patternfly_yew::NavItem;
use patternfly_yew::NavRouterItem;
use patternfly_yew::Page;
use patternfly_yew::PageSidebar;
use patternfly_yew::ToastViewer;
use wasm_bindgen_futures::spawn_local;
use yew::Context;
use yew::{function_component, Callback, MouseEvent};
use yew::{html, html_nested, Html, Properties};
use yew_nested_router::prelude::{Switch as RouterSwitch, Target};
use yew_nested_router::Router;
use yew_oauth2::oauth2::LocationRedirect;
use yew_oauth2::oauth2::{use_auth_agent, OAuth2};
use yew_oauth2::prelude::oauth2::Config;
use yew_oauth2::prelude::Failure;
use yew_oauth2::prelude::NotAuthenticated;
use yew_oauth2::prelude::{Authenticated, OAuth2Operations};

use crate::graphql::settings::{ResponseData, SettingsAuthentication};
use crate::graphql::{query_anonymous, settings, Settings};
use crate::pages::adder::Adder;

#[derive(Debug, Default, Clone, PartialEq, Eq, Target)]
pub enum AppRoute {
    Secure,
    Add,
    LoginRedirect,
    Login,
    NotFound,
    #[default]
    Home,
}

#[derive(Debug)]
pub struct App {
    oauth2_config: Option<Config>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub config: Config,
}

fn switch_main(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Home => html! {<h1>{"Home"}</h1>},
        AppRoute::Secure => html! {<h1>{"Secure"}</h1>},
        AppRoute::NotFound => html! {<h1>{"Not Found"}</h1>},
        #[allow(clippy::let_unit_value)]
        AppRoute::Add => html! {<Adder/>},
        AppRoute::LoginRedirect => html! {<h1>{"Login redirect"}</h1>},
        AppRoute::Login => html! {<h1>{"Login"}</h1>},
    }
}
fn switch_unauthenticated(switch: AppRoute) -> Html {
    match switch {
        AppRoute::Home => html! {  <p> { "You need to log in" } </p>},
        _ => html!(<LocationRedirect logout_href="/" />),
    }
}

#[derive(Debug)]
pub enum AppMessage {
    AuthenticationData(Config),
}

impl yew::Component for App {
    type Message = AppMessage;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            oauth2_config: None,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::AuthenticationData(config) => {
                self.oauth2_config = Some(config);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(config) = self.oauth2_config.clone() {
            html! {
                <MainOAuth2 {config}/>
            }
        } else {
            html! {
                <h1>{"Fetching"}</h1>
            }
        }
    }
    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let scope = ctx.link().clone();
            spawn_local(async move {
                let result =
                    query_anonymous::<Settings, _>(scope.clone(), settings::Variables {}).await;
                match result {
                    Ok(ResponseData {
                        authentication:
                            SettingsAuthentication {
                                auth_url,
                                client_id,
                                token_url,
                            },
                    }) => {
                        scope.send_message(AppMessage::AuthenticationData(Config {
                            client_id,
                            auth_url,
                            token_url,
                        }));
                    }
                    Err(err) => error!("Error on server {err:?}"),
                }
            });
        }
    }
}

#[function_component(MainOAuth2)]
fn main_oauth2(props: &Props) -> Html {
    let oauth2_config = &props.config;
    html! {
        <OAuth2 config={oauth2_config.clone()}>
            <Router<AppRoute> default={AppRoute::Home}>
                <MainPage/>
            </Router<AppRoute>>
        </OAuth2>
    }
}

#[function_component(MainPage)]
fn main_page() -> Html {
    html! {
        <BackdropViewer>
            <ToastViewer>
                <Failure>{"Fail"}</Failure>
                <Authenticated>
                    <Page sidebar={html_nested! {<PageSidebar><AuthenticatedSidebar/></PageSidebar>}}>
                      //<ToastViewer/>
                      //logo={logo}
                        <RouterSwitch<AppRoute>
                            render = { switch_main}
                        />
                    </Page>
                </Authenticated>
                <NotAuthenticated>
                    <Page sidebar={html_nested! {<PageSidebar><NotAuthenticatedSidebar/></PageSidebar>}}>
                        <RouterSwitch<AppRoute>
                            render = {switch_unauthenticated}
                        />
                    </Page>
                </NotAuthenticated>
            </ToastViewer>
        </BackdropViewer>
    }
}
#[function_component(AuthenticatedSidebar)]
fn authenticated_sidebar() -> Html {
    let agent = use_auth_agent().expect("Requires OAuth2Context component in parent hierarchy");
    let logout = Callback::from(move |_: MouseEvent| {
        if let Err(err) = agent.logout() {
            log::warn!("Failed to logout: {err}");
        }
    });
    html! {
        <Nav>
            <NavRouterItem<AppRoute> to={AppRoute::Home}>{"Start"}</NavRouterItem<AppRoute>>
            <NavRouterItem<AppRoute> to={AppRoute::Login}>{"Login"}</NavRouterItem<AppRoute>>
            <NavRouterItem<AppRoute> to={AppRoute::Add}>{"Add"}</NavRouterItem<AppRoute>>
            <NavRouterItem<AppRoute> to={AppRoute::Secure}>{"Secure"}</NavRouterItem<AppRoute>>
            <span onclick={logout}><NavItem>{"Logout"}</NavItem></span>
        </Nav>
    }
}
#[function_component(NotAuthenticatedSidebar)]
fn not_authenticated_sidebar() -> Html {
    let agent = use_auth_agent().expect("Requires OAuth2Context component in parent hierarchy");
    let login = Callback::from(move |_: MouseEvent| {
        if let Err(err) = agent.start_login() {
            log::warn!("Failed to start login: {err}");
        }
    });
    html! {
        <Nav>
            <span onclick={login}><NavItem>{"Login"}</NavItem></span>
        </Nav>
    }
}
