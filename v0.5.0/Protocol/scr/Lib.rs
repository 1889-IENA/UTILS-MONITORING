// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use serde::{Deserialize, Serialize};




pub const Default_Port: u16 = 57000;




pub const Protocol_Magic: u32 = 0x4B564D53;




pub const Protocol_Version: u8 = 1;




#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mouse_Button {

    Left,
    Right,
    Middle,
    Side { Button_Index: u8 },

}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scroll_Direction {

    Vertical,
    Horizontal,

}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input_Event {

    Mouse_Move {

        Delta_X: i32,
        Delta_Y: i32,

    },

    Mouse_Button_Press {

        Button: Mouse_Button,

    },

    Mouse_Button_Release {

        Button: Mouse_Button,

    },

    Mouse_Scroll {

        Direction: Scroll_Direction,
        Amount: i32,

    },

    Key_Press {

        Key_Code: u16,

    },

    Key_Release {

        Key_Code: u16,

    },

}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Control_Message {

    Take_Control,
    Release_Control,
    Ping,
    Pong,
    
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {

    Input(Input_Event),
    Control(Control_Message),

}




pub fn Serialize_Packet(Packet_To_Serialize: &Packet) -> Result<Vec<u8>, bincode::Error> {

    let Payload = bincode::serialize(Packet_To_Serialize)?;
    let Payload_Length = Payload.len() as u32;
    let mut Buffer = Vec::with_capacity(9 + Payload.len());

    Buffer.extend_from_slice(&Protocol_Magic.to_be_bytes());
    Buffer.push(Protocol_Version);
    Buffer.extend_from_slice(&Payload_Length.to_be_bytes());
    Buffer.extend_from_slice(&Payload);

    Ok(Buffer)

}




pub fn Deserialize_Packet(Raw_Payload: &[u8]) -> Result<Packet, bincode::Error> {

    bincode::deserialize(Raw_Payload)

}




pub fn Parse_Frame_Header(Header_Bytes: &[u8; 9]) -> Option<(u32, u8, u32)> {

    let Magic = u32::from_be_bytes([
        Header_Bytes[0],
        Header_Bytes[1],
        Header_Bytes[2],
        Header_Bytes[3],
    ]);

    let Version = Header_Bytes[4];

    let Payload_Length = u32::from_be_bytes([
        Header_Bytes[5],
        Header_Bytes[6],
        Header_Bytes[7],
        Header_Bytes[8],
    ]);

    if Magic != Protocol_Magic {

        return None;

    }

    Some((Magic, Version, Payload_Length))

}
