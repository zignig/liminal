//! All the auth things
//! 

mod id_comms;

use id_comms::IdentityApi;

use anyhow::Result;

// use irpc;
// use rcan;

// The auth object
pub struct AuthService{ 
    indentity: IdentityApi,

}

impl AuthService { 
    pub async fn new() -> Result<Self> {
        info!("Create new auth service");
        Ok(Self{
            indentity: IdentityApi::new(None).await?,
        })
    }
}
