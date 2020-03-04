use crate::{
    agents::{NotificationBus, NotificationSender},
    app,
    components::NeqAssign,
};
use yew::{
    agent::{Dispatched, Dispatcher},
    prelude::*,
};

pub struct Navbar {
    link: ComponentLink<Self>,
    notification_bus: Dispatcher<NotificationBus>,
    props: Props,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub on_navevent: Callback<app::NavEvent>,
}

#[derive(Debug)]
pub enum Msg {
    NavEvent(app::NavEvent),
    RestorePreviousScenario,
    LoadFromUrl,
    UserGuide,
    About,
    ReportIssue,
    Share,
    Settings,
}

impl NotificationSender for Navbar {
    fn notification_bus(&mut self) -> &mut Dispatcher<NotificationBus> {
        &mut self.notification_bus
    }
}

impl Component for Navbar {
    type Properties = Props;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            notification_bus: NotificationBus::dispatcher(),
            props,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NavEvent(nav_event) => {
                self.props.on_navevent.emit(nav_event);
                false
            }
            unhandled => {
                self.notif_error(format!("{:?} not implemented yet.", unhandled));
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                        <p>{ "Templatr" }</p>
                    </a>

                    <a role="button" class="navbar-burger burger" aria-label="menu" aria-expanded="false" data-target="navbarBasicExample">
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </a>
                </div>

                <div id="navbarBasicExample" class="navbar-menu">
                    <div class="navbar-start">
                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                                { "Scenario" }
                            </a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::LoadFromUrl)>
                                    { "Load from URL" }
                                </a>
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::RestorePreviousScenario)>
                                    { "Restore a previous scenario" }
                                </a>
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::NavEvent(app::NavEvent::UnloadScenario))>
                                    { "Unload the workspace" }
                                </a>
                                <hr class="navbar-divider" />
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::NavEvent(app::NavEvent::LoadFromLocalStorage))>
                                    { "Reload from local storage" }
                                </a>
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::NavEvent(app::NavEvent::LoadDebugScenario))>
                                    { "Load a debug scenario" }
                                </a>
                            </div>
                        </div>

                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">
                                { "Help" }
                            </a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::UserGuide)>
                                    { "User guide" }
                                </a>
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::About)>
                                    { "About" }
                                </a>
                                <a class="navbar-item" onclick=self.link.callback(|_| Msg::ReportIssue)>
                                    { "Report an issue" }
                                </a>
                            </div>
                        </div>
                    </div>

                    <div class="navbar-end">
                        <div class="navbar-item">
                            <div class="buttons">
                                <a class="button is-primary" onclick=self.link.callback(|_| Msg::Share)>
                                    <strong>{ "Share" }</strong>
                                </a>
                                <a class="button is-light"  onclick=self.link.callback(|_| Msg::Settings)>
                                    { "Settings" }
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
