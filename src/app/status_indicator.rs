use super::SafetyVerifiedComponent;
use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Status indicator showing system health with triple redundancy
pub struct StatusIndicator {
    health_status: [bool; 3],
    last_check: Rc<RefCell<[u64; 3]>>,
    blink_state: Rc<RefCell<bool>>,
}

pub enum StatusIndicatorMsg {
    ToggleBlink,
    UpdateStatusChecks,
}

#[derive(PartialEq, Properties, Clone)]
pub struct StatusIndicatorProps {
    pub health_status: [bool; 3],
}

impl Component for StatusIndicator {
    type Message = StatusIndicatorMsg;
    type Properties = StatusIndicatorProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Set up blink interval for status indicator
        let link = ctx.link().clone();
        let blink_interval = gloo_timers::callback::Interval::new(
            500,
            move || link.send_message(StatusIndicatorMsg::ToggleBlink)
        );
        blink_interval.forget();

        // Set up status check interval
        let link = ctx.link().clone();
        let check_interval = gloo_timers::callback::Interval::new(
            2000,
            move || link.send_message(StatusIndicatorMsg::UpdateStatusChecks)
        );
        check_interval.forget();

        Self {
            health_status: ctx.props().health_status,
            last_check: Rc::new(RefCell::new([0, 0, 0])),
            blink_state: Rc::new(RefCell::new(false)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            StatusIndicatorMsg::ToggleBlink => {
                // Toggle blink state for visual indication
                let mut blink = self.blink_state.borrow_mut();
                *blink = !*blink;
                true
            },
            StatusIndicatorMsg::UpdateStatusChecks => {
                // Update timestamp of checks for staleness detection
                let now = js_sys::Date::now() as u64;

                let mut checks = self.last_check.borrow_mut();
                for i in 0..3 {
                    if self.health_status[i] {
                        checks[i] = now;
                    }
                }

                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.health_status = ctx.props().health_status;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let blink_active = *self.blink_state.borrow();
        let now = js_sys::Date::now() as u64;
        let last_check = self.last_check.borrow();

        // Check for stale health checks (over 5 seconds old)
        let mut health_status = self.health_status;
        for i in 0..3 {
            if now - last_check[i] > 5000 {
                health_status[i] = false;
            }
        }

        // Count healthy subsystems for status determination
        let healthy_count = health_status.iter().filter(|&&h| h).count();

        let (status_text, status_class) = match healthy_count {
            3 => ("All Systems Nominal", ""),
            2 => ("Degraded Redundancy", "degraded"),
            1 => ("Critical Redundancy Loss", "error"),
            0 => ("System Failure", "error"),
            _ => unreachable!(),
        };

        html! {
            <div class="status-indicator">
                <div class={format!("status-dot {}", status_class)} style={if blink_active && healthy_count < 2 { "opacity: 0.5" } else { "" }}></div>
                <span class="status-text">{status_text}</span>
                <div class="tmr-indicator">
                    {
                        (0..3).map(|i| {
                            let class = if health_status[i] {
                                "tmr-unit"
                            } else if healthy_count == 2 {
                                "tmr-unit degraded"
                            } else {
                                "tmr-unit error"
                            };

                            html! { <div class={class}></div> }
                        }).collect::<Html>()
                    }
                </div>
            </div>
        }
    }
}

impl SafetyVerifiedComponent for StatusIndicator {
    fn render(&self) -> Html {
        html! {
            <div class="status-indicator fallback">
                <div class="status-dot error"></div>
                <span class="status-text">{"Indicator Failure"}</span>
            </div>
        }
    }

    fn create_component() -> Self {
        Self {
            health_status: [true, true, true],
            last_check: Rc::new(RefCell::new([0, 0, 0])),
            blink_state: Rc::new(RefCell::new(false)),
        }
    }

    fn verify_state_integrity(&self) -> bool {
        true // Basic implementation, could be expanded
    }
}
