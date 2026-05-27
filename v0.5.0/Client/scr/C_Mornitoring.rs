// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




mod C_Virtural_Input;
mod C_Packed_Listener;




use anyhow::Result;
use log::info;


use tokio::sync::mpsc;




use protocol::Default_Port;
use C_Packed_Listener::C_Packed_Listener as CP_Listener;
use C_Virtural_Input::C_Virtural_Input as CV_Input;




#[tokio::main]
async fn main() -> Result<()> {

    print!("\x1B[2J\x1B[1;1H");

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format_target(false)
    .init();

    info!("Monitoring Client | KVM Client Starting..");

    let Bind_Address = std::env::args()

        .nth(1)
        .unwrap_or_else(|| format!("0.0.0.0:{}", Default_Port));

    let Bind_Address_With_Port = if Bind_Address.contains(':') {

        Bind_Address

    } else {

        format!("{}:{}", Bind_Address, Default_Port)
        
    };

    info!("Monitoring Client | Listening: {}", Bind_Address_With_Port);

    let (Packet_Sender, Packet_Receiver) = mpsc::channel::<protocol::Packet>(512);

    let Virtual_Input = CV_Input::New().await?;

    tokio::spawn(async move {

        if let Err(Device_Error) = Virtual_Input.Run(Packet_Receiver).await {

            log::error!("Monitoring Client | Virtural Input Device Error: {}", Device_Error);

        }

    });

    let mut Listener = CP_Listener::New(Bind_Address_With_Port, Packet_Sender);

    Listener.Run().await?;

    Ok(())

}
