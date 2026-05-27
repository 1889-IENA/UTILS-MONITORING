// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




mod S_Device_Scanner;
mod S_Input_Capturer;
mod S_Connection_Manager;




use anyhow::Result;
use log::info;
use std::sync::Arc;


use tokio::sync::{mpsc, Mutex};




use protocol::Default_Port;
use S_Connection_Manager::S_Connection_Manager as SC_Manager;
use S_Input_Capturer::S_Input_Capturer as SI_Capturer;




#[tokio::main]
async fn main() -> Result<()> {

    print!("\x1B[2J\x1B[1;1H");

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format_target(false)
    .init();

    info!("Monitoring Server | KVM Server Starting..");
    info!("Monitoring Server | Switch PC - Pause/Break");

    let Client_Address = std::env::args()

        .nth(1)

        .unwrap_or_else(|| {

            eprintln!("For Use: kvm-server <client-ip>");
            eprintln!("Example: kvm-server 192.168.1.100");

            std::process::exit(1);

        });

    let Client_Address_With_Port = if Client_Address.contains(':') {

        Client_Address

    } else {

        format!("{}:{}", Client_Address, Default_Port)

    };

    info!("Monitoring Server | Target Client Adress: {}", Client_Address_With_Port);

    let (Packet_Sender, Packet_Receiver) = mpsc::channel::<protocol::Packet>(512);

    let Shared_Connection_Manager = Arc::new(Mutex::new(

        SC_Manager::New(Client_Address_With_Port, Packet_Receiver).await?

    ));

    let Shared_Connection_Manager_Clone = Arc::clone(&Shared_Connection_Manager);

    tokio::spawn(async move {

        if let Err(Manager_Error) = Shared_Connection_Manager_Clone.lock().await.Run().await {

            log::error!("Monitoring Server | Connection Manager Error: {}", Manager_Error);

        }

    });

    let mut Input_Capturer_Instance = SI_Capturer::New(Packet_Sender).await?;

    Input_Capturer_Instance.Run().await?;

    Ok(())

}
