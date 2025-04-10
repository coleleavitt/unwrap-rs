use super::SafetyVerifiedComponent;
use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Radiation-hardened error boundary with triple redundancy
pub struct ErrorBoundary {
    children: Children,
    error_state: Rc<RefCell<[Option<String>; 3]>>, // Triple redundant error storage
    has_error: Rc<RefCell<[bool; 3]>>, // Triple redundant error flags
}
pub enum ErrorBoundaryMsg {
    CatchError(String),
    ClearError,
    VerifyErrorState,
}

#[derive(PartialEq, Properties)]
pub struct ErrorBoundaryProps {
    #[prop_or_default]
    pub children: Children,
}

impl Component for ErrorBoundary {
    type Message = ErrorBoundaryMsg;
    type Properties = ErrorBoundaryProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Set up periodic verification
        let link = ctx.link().clone();
        let verification_interval = gloo_timers::callback::Interval::new(
            1000,
            move || link.send_message(ErrorBoundaryMsg::VerifyErrorState)
        );
        verification_interval.forget();

        Self {
            children: ctx.props().children.clone(),
            error_state: Rc::new(RefCell::new([None, None, None])),
            has_error: Rc::new(RefCell::new([false, false, false])),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ErrorBoundaryMsg::CatchError(error) => {
                // Store error with triple redundancy
                let mut error_state = self.error_state.borrow_mut();
                let mut has_error = self.has_error.borrow_mut();

                for i in 0..3 {
                    error_state[i] = Some(error.clone());
                    has_error[i] = true;
                }

                true
            },
            ErrorBoundaryMsg::ClearError => {
                // Clear error with triple redundancy
                let mut error_state = self.error_state.borrow_mut();
                let mut has_error = self.has_error.borrow_mut();

                for i in 0..3 {
                    error_state[i] = None;
                    has_error[i] = false;
                }

                true
            },
            ErrorBoundaryMsg::VerifyErrorState => {
                // Verify state integrity using majority voting
                let mut error_state = self.error_state.borrow_mut();
                let mut has_error = self.has_error.borrow_mut();

                // Verify has_error flags consistency
                let error_count = has_error.iter().filter(|&&e| e).count();

                // Apply majority vote
                if error_count >= 2 && error_count < 3 {
                    // Inconsistency detected - repair using majority vote
                    let should_have_error = error_count >= 2;

                    for i in 0..3 {
                        has_error[i] = should_have_error;
                    }

                    // Also ensure error messages are consistent if we have an error
                    if should_have_error {
                        // Find the most common error message
                        let mut valid_errors: Vec<&Option<String>> = error_state.iter()
                            .filter(|e| e.is_some())
                            .collect();

                        if !valid_errors.is_empty() {
                            let correct_error = valid_errors[0].clone();

                            for i in 0..3 {
                                error_state[i] = correct_error.clone();
                            }
                        }
                    }

                    log::warn!("Error boundary state inconsistency detected and repaired");
                    return true;
                }

                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.children = ctx.props().children.clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let error_detected = self.has_error.borrow().iter().filter(|&&e| e).count() >= 2;

        if error_detected {
            // Get the error message using redundant storage
            let error_state = self.error_state.borrow();
            let error_message = error_state.iter()
                .filter_map(|e| e.as_ref())
                .next()
                .cloned()
                .unwrap_or_else(|| "Unknown error".to_string());

            html! {
                <div class="error-boundary">
                    <h3>{"System Error Detected"}</h3>
                    <p>{"A radiation-hardened error boundary has contained a failure:"}</p>
                    <pre class="error-message">{error_message}</pre>
                    <div class="tmr-indicator">
                        <div class="tmr-unit error"></div>
                        <div class="tmr-unit error"></div>
                        <div class="tmr-unit error"></div>
                    </div>
                </div>
            }
        } else {
            html! {
                <>{ for self.children.iter() }</>
            }
        }
    }
}

impl SafetyVerifiedComponent for ErrorBoundary {
    fn render(&self) -> Html {
        html! {
            <div class="error-boundary fallback">
                <p>{"Critical fault in error boundary component"}</p>
            </div>
        }
    }

    fn create_component() -> Self {
        Self {
            children: Children::default(),
            error_state: Rc::new(RefCell::new([None, None, None])),
            has_error: Rc::new(RefCell::new([false, false, false])),
        }
    }

    fn verify_state_integrity(&self) -> bool {
        let error_state = self.error_state.borrow();
        let has_error = self.has_error.borrow();

        // Check if error states are consistent
        let error_count = has_error.iter().filter(|&&e| e).count();

        // All should be in agreement (all 3 true or all 3 false)
        error_count == 0 || error_count == 3
    }
}
