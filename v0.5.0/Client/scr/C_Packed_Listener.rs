// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use anyhow::{Context, Result};
use log::{info, warn};


use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;




use protocol::{Deserialize_Packet, Parse_Frame_Header, Packet};




const Frame_Header_Size: usize = 9;
const Max_Packet_Size_Bytes: u32 = 1024 * 64;




pub struct C_Packed_Listener {

    Bind_Address: String,
    Packet_Sender: mpsc::Sender<Packet>,

}




impl C_Packed_Listener {

    pub fn New(Bind_Address: String, Packet_Sender: mpsc::Sender<Packet>) -> Self {

        Self {

            Bind_Address,
            Packet_Sender,

        }

    }




    pub async fn Run(&mut self) -> Result<()> {

        let Tcp_Listener = TcpListener::bind(&self.Bind_Address)

            .await
            .context("Monitoring Client | TCP Server Error")?;

        info!("Monitoring Client | TCP Server Waiting..");

        loop {

            let (Incoming_Stream, Remote_Address) = Tcp_Listener

                .accept()
                .await
                .context("Monitoring Client | TCP Server Not Accepted Connection")?;

            info!("Monitoring Client | TCP Server Connected: {}", Remote_Address);

            Incoming_Stream

                .set_nodelay(true)
                .context("TCP_NODELAY Error")?;

            let Packet_Sender_Clone = self.Packet_Sender.clone();

            tokio::spawn(async move {

                if let Err(Connection_Error) = Self::Handle_Connection(Incoming_Stream, Packet_Sender_Clone).await {
                    warn!("Monitoring Client | TCP Server Connection Error: {}", Connection_Error);
                }

                info!("Monitoring Client | Server Is Closed: {}", Remote_Address);

            });
        }
    }




    async fn Handle_Connection(

        mut Tcp_Stream: TcpStream,
        Packet_Sender: mpsc::Sender<Packet>,

    ) -> Result<()> {

        loop {

            let mut Header_Buffer = [0u8; Frame_Header_Size];

            Tcp_Stream

                .read_exact(&mut Header_Buffer)
                .await
                .context("Monitoring Client | Frame Header Not Readed")?;

            let (_Magic, _Version, Payload_Length) = Parse_Frame_Header(&Header_Buffer)
                .context("Monitoring Client | Invalid Frame Header — Magic Number Doesnt Match")?;

            if Payload_Length > Max_Packet_Size_Bytes {

                return Err(anyhow::anyhow!(

                    "Monitoring Client | Packed Too High: {} Byte (Max: {})",

                    Payload_Length,
                    Max_Packet_Size_Bytes

                ));

            }

            let mut Payload_Buffer = vec![0u8; Payload_Length as usize];

            Tcp_Stream
                .read_exact(&mut Payload_Buffer)
                .await
                .context("Monitoring Client | Payload Not Readed")?;

            let Decoded_Packet = Deserialize_Packet(&Payload_Buffer)
                .context("Monitoring Client | Deserialization failed")?;

            Packet_Sender
                .send(Decoded_Packet)
                .await
                .context("Monitoring Client | Queue List Add Failed")?;
    
        }

    }

}
