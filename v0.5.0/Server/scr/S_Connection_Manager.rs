// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use anyhow::{Context, Result};
use log::{info, warn};


use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};




use protocol::{Serialize_Packet, Packet};




const Reconnect_Delay_Seconds: u64 = 3;




pub struct S_Connection_Manager {

    Client_Address: String,
    Packet_Receiver: mpsc::Receiver<Packet>,

}




impl S_Connection_Manager {

    pub async fn New(

        Client_Address: String,
        Packet_Receiver: mpsc::Receiver<Packet>,

    ) -> Result<Self> {

        Ok(Self {

            Client_Address,
            Packet_Receiver,

        })

    }




    pub async fn Run(&mut self) -> Result<()> {

        loop {

            info!("Monitoring Server | Connecting Client: {}", self.Client_Address);

            match self.Connect_And_Stream().await {

                Ok(()) => {

                    info!("Monitoring Server | Connection Safely Closed");
                    break;

                }

                Err(Connection_Error) => {

                    warn!(

                        "Monitoring Server | Connection Error: {}. {} After Trying Again",
                        Connection_Error, Reconnect_Delay_Seconds

                    );

                    sleep(Duration::from_secs(Reconnect_Delay_Seconds)).await;

                }

            }

        }

        Ok(())

    }




    async fn Connect_And_Stream(&mut self) -> Result<()> {

        let mut Tcp_Stream = TcpStream::connect(&self.Client_Address)
            .await
            .context("Monitoring Server | TCP Connection Failed")?;

        info!("Monitoring Server | TCP Connected: {}", self.Client_Address);

        Tcp_Stream
            .set_nodelay(true)
            .context("TCP_NODELAY Failed")?;

        loop {
            let Packet_To_Send = match self.Packet_Receiver.recv().await {
                Some(Packet_To_Send) => Packet_To_Send,
                None => {
                    info!("Monitoring Server | Channel Closed");
                    return Ok(());
                }
            };

            let Serialized_Bytes = Serialize_Packet(&Packet_To_Send)
                .context("Monitoring Server | Serialization Failed")?;

            Tcp_Stream
                .write_all(&Serialized_Bytes)
                .await
                .context("Monitoring Server | Data Not Sending - You Have Connection Bro??")?;
        }
        
    }

}
