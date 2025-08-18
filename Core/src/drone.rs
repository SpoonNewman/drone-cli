use crate::{transport::Transport, DroneError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DroneState { Disarmed, Armed, Flying, Landing }

pub struct Drone<T: Transport> {
    transport: T,
    state: DroneState,
}

impl<T: Transport> Drone<T> {
    pub fn new(transport: T) -> Self {
        Self { transport, state: DroneState::Disarmed }
    }

    pub fn state(&self) -> DroneState { self.state }

    pub async fn connect(&mut self) -> Result<()> {
        self.transport.connect().await
    }

    pub async fn arm(&mut self) -> Result<()> {
        match self.state {
            DroneState::Disarmed => {
                self.transport.send(b"ARM").await?;
                self.state = DroneState::Armed;
                Ok(())
            }
            _ => Err(DroneError::State(format!("cannot arm from {:?}", self.state))),
        }
    }

    pub async fn takeoff(&mut self, altitude_m: f32) -> Result<()> {
        match self.state {
            DroneState::Armed => {
                let cmd = format!("TAKEOFF:{altitude_m}");
                self.transport.send(cmd.as_bytes()).await?;
                self.state = DroneState::Flying;
                Ok(())
            }
            _ => Err(DroneError::State("takeoff requires Armed".into())),
        }
    }

    pub async fn land(&mut self) -> Result<()> {
        match self.state {
            DroneState::Flying => {
                self.transport.send(b"LAND").await?;
                self.state = DroneState::Landing;
                Ok(())
            }
            _ => Err(DroneError::State("land requires Flying".into())),
        }
    }

    pub async fn status(&mut self) -> Result<String> {
        self.transport.send(b"STATUS").await?;
        let resp = self.transport.recv().await?;
        Ok(format!("state={:?} transport_resp={}", self.state, String::from_utf8_lossy(&resp)))
    }
}
