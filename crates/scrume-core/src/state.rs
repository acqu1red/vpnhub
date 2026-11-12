use crate::bundle::VerifiedBundle;
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Unenrolled,
    Ready,
    Connecting,
    Connected,
    Disconnecting,
    Recovering,
    Expired,
    Revoked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    StartTunnel { generation: u64 },
    StopAndRecover,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    #[error("CONNECT_NOT_ALLOWED")]
    NotAllowed,
    #[error("PROFILE_EXPIRED")]
    Expired,
    #[error("STALE_OPERATION")]
    Stale,
}

pub struct Machine {
    state: State,
    bundle: Option<VerifiedBundle>,
    generation: u64,
    terminal_after_cleanup: State,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            state: State::Unenrolled,
            bundle: None,
            generation: 0,
            terminal_after_cleanup: State::Ready,
        }
    }
}

impl Machine {
    pub fn state(&self) -> State {
        self.state
    }

    /// Call only after the verified bundle and rollback floor are committed atomically.
    /// Live bundle replacement requires a controller transaction; it is not implicit here.
    pub fn enroll(&mut self, bundle: VerifiedBundle, now: u64) -> Result<(), StateError> {
        if !matches!(
            self.state,
            State::Unenrolled | State::Ready | State::Expired | State::Revoked
        ) {
            return Err(StateError::NotAllowed);
        }
        if !bundle.valid_at(now) {
            return Err(StateError::Expired);
        }
        self.bundle = Some(bundle);
        self.state = State::Ready;
        Ok(())
    }

    pub fn connect(&mut self, now: u64) -> Result<Effect, StateError> {
        if !self.bundle.as_ref().is_some_and(|b| b.valid_at(now)) {
            return Err(StateError::Expired);
        }
        if matches!(self.state, State::Connecting | State::Connected) {
            return Ok(Effect::None);
        }
        if self.state != State::Ready {
            return Err(StateError::NotAllowed);
        }
        self.generation = self.generation.wrapping_add(1);
        self.state = State::Connecting;
        Ok(Effect::StartTunnel {
            generation: self.generation,
        })
    }

    pub fn boot(&mut self, now: u64) -> Result<Effect, StateError> {
        if self
            .bundle
            .as_ref()
            .is_some_and(|b| b.bundle().policy.auto_connect)
        {
            self.connect(now)
        } else {
            Ok(Effect::None)
        }
    }

    /// Internal controller event, deliberately absent from IPC Command.
    pub fn healthy(&mut self, generation: u64, now: u64) -> Result<(), StateError> {
        if generation != self.generation || self.state != State::Connecting {
            return Err(StateError::Stale);
        }
        if !self.bundle.as_ref().is_some_and(|b| b.valid_at(now)) {
            return Err(StateError::Expired);
        }
        self.state = State::Connected;
        Ok(())
    }

    pub fn disconnect(&mut self) -> Effect {
        if matches!(self.state, State::Connecting | State::Connected) {
            self.generation = self.generation.wrapping_add(1);
            self.terminal_after_cleanup = State::Ready;
            self.state = State::Disconnecting;
            Effect::StopAndRecover
        } else {
            Effect::None
        }
    }

    pub fn tick(&mut self, now: u64) -> Effect {
        if self.bundle.as_ref().is_some_and(|b| !b.valid_at(now))
            && !matches!(self.state, State::Revoked | State::Expired)
        {
            self.invalidate(State::Expired)
        } else {
            Effect::None
        }
    }

    pub fn revoke(&mut self) -> Effect {
        self.invalidate(State::Revoked)
    }

    fn invalidate(&mut self, target: State) -> Effect {
        self.generation = self.generation.wrapping_add(1);
        self.bundle = None;
        if matches!(
            self.state,
            State::Connecting | State::Connected | State::Disconnecting | State::Recovering
        ) {
            self.terminal_after_cleanup = target;
            self.state = State::Recovering;
            Effect::StopAndRecover
        } else {
            self.state = target;
            Effect::None
        }
    }

    pub fn child_crashed(&mut self) -> Effect {
        if matches!(self.state, State::Connecting | State::Connected) {
            self.generation = self.generation.wrapping_add(1);
            self.terminal_after_cleanup = State::Ready;
            self.state = State::Recovering;
            Effect::StopAndRecover
        } else {
            Effect::None
        }
    }

    /// Called only once controller proves its owned network changes are recovered.
    pub fn recovered(&mut self) -> Result<(), StateError> {
        if !matches!(self.state, State::Recovering | State::Disconnecting) {
            return Err(StateError::NotAllowed);
        }
        self.state = self.terminal_after_cleanup;
        Ok(())
    }
}
