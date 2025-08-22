use crate::{transport::Transport, DroneError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DroneState { Disarmed, Armed, Flying, Landing }

// TODO: Big Refactor: This code assumes that Drone service is going to use SBUS (serial) to interact with the Flight Controller. This is
// not what we want. Instead we need to simply send HTTP(S) requests to a certain address. *BEAR THIS IN MIND* for the following statement
// This code is using a generic T where T conforms to the shape of `Transport`, i.e. it has all the properties listed in Transport. 
// This is problematic because `Transport` assumes only 3 properties, e.g. connect, recv, send. That's not what we need to for HTTP
// interaction.

// TODO: Followup Discussion: DroneState is confusing. Not sure whether we want to actually track the drone state or not although philosophically I
// agree with this. 
// Reasoning: the CLI workflow is 1) it starts up, 2) it receives arguments, 3) it executes some actions, 4) it ends.
// The only time it would be relevant to care about the state is during the execution of actions and I don't foresee a usecase where we
// make a change to the state of the drone, evaluate the state, and make an additional change to the state before the CLI execution ends.
// The CLI is a very quick one-and-done type of thing and tracking the state doens't seem relevant, never mind that we would effectively need a
// database to store the state. State in the CLI is lost as soon as execution is finished without having a database.
// Summary: DroneState is confusing.

// TODO: None of this can be used. See above statements. Make a new Drone service that is able to do GET/POST requests with and without headers
// The methods that handle performing the GET/POST requets should be generic because we're going to use them in every command in different ways
// and expect different shapes of data to be sent and received.
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
