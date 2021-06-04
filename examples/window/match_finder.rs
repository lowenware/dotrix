use dotrix::{
    ecs::{ Const, Mut },
    services::{ Frame, Window },
    window::{ UserAttentionType },
};
use std::time::Duration;

pub struct MatchFinder {
    pub attention_type: UserAttentionType,
    pub estimated_time: f32,
    pub state: MatchFinderState,
    wait_ready_time: f32,
}

pub enum MatchFinderState {
    Idle,
    Searching(Option<Duration>),
    Stopping(),
    Ready(Duration),
}

impl MatchFinder {
    pub fn new() -> Self {
        Self {
            attention_type: UserAttentionType::Informational,
            estimated_time: 10.0,
            state: MatchFinderState::Idle,
            wait_ready_time: 10.0,
        }
    }

    pub fn start_searching(&mut self) {
        self.state = MatchFinderState::Searching(None);
    }

    pub fn stop_searching(&mut self) {
        self.state =  MatchFinderState::Stopping();
    }
}

pub fn update(finder: Mut<MatchFinder>, frame: Const<Frame>, window: Const<Window>) {
    match finder.state {
        MatchFinderState::Searching(from) => update_searching(from, finder, frame, window),
        MatchFinderState::Stopping() => update_stopping(finder, window),
        MatchFinderState::Ready(until) => update_ready(until, finder, frame),
        _ => {}
    }
}

fn update_searching(
    from: Option<Duration>,
    mut finder: Mut<MatchFinder>,
    frame: Const<Frame>,
    window: Const<Window>
) {
    if let Some(from) = from {
        let dur_secs = (frame.time() - from).as_secs_f32();
        if dur_secs > finder.estimated_time {
            let until = frame.time() + Duration::from_secs_f32(finder.wait_ready_time);
            window.request_attention(Some(finder.attention_type));
            println!("request attention {:?}", finder.attention_type);
            finder.state = MatchFinderState::Ready(until);
        }
    } else {
        finder.state = MatchFinderState::Searching(Some(frame.time()));
    }
}

fn update_stopping(mut finder: Mut<MatchFinder>, window: Const<Window>) {
    window.request_attention(None);
    finder.state =  MatchFinderState::Idle;
}

fn update_ready(until: Duration, mut finder: Mut<MatchFinder>, frame: Const<Frame>) {
    if frame.time() > until {
        finder.stop_searching();
    }
}
