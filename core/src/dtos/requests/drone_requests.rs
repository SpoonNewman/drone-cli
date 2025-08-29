use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DroneLandRequestDTO {

}

impl DroneLandRequestDTO {
    pub fn new() -> anyhow::Result<Self> { 
        Ok(Self {

        })
    }
}