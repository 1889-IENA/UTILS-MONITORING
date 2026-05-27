// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use anyhow::{Context, Result};
use log::{debug, info};


use tokio::sync::mpsc;




use protocol::{Control_Message, Input_Event, Mouse_Button, Packet, Scroll_Direction};




const Uinput_Device_Path: &str = "/dev/uinput";




pub struct C_Virtural_Input {

    Virtual_Keyboard: Virtual_Keyboard,
    Virtual_Mouse: Virtual_Mouse,

}




struct Virtual_Keyboard {

    Device_File: std::fs::File,

}




struct Virtual_Mouse {

    Device_File: std::fs::File,

}




impl C_Virtural_Input {

    pub async fn New() -> Result<Self> {

        info!("Monitoring Client | Virtual Input Device Creating..");

        let Virtual_Keyboard = Virtual_Keyboard::Create()
            .context("Monitoring Client | Virtual Keyboard Create Failed — Root Permission May Required")?;

        let Virtual_Mouse = Virtual_Mouse::Create()
            .context("Monitoring Client | Virtual Mouse Create Failed — Root Permission May Required")?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Monitoring Client | Virtual Input Device Ready");

        Ok(Self {

            Virtual_Keyboard,
            Virtual_Mouse,

        })

    }




    pub async fn Run(mut self, mut Packet_Receiver: mpsc::Receiver<Packet>) -> Result<()> {

        info!("Monitoring Client | Packet Processing Loop Starting..");

        let mut Is_Active = false;

        while let Some(Received_Packet) = Packet_Receiver.recv().await {

            match Received_Packet {

                Packet::Control(Control_Message_Value) => {

                    self.Handle_Control_Message(Control_Message_Value, &mut Is_Active)?;

                }

                Packet::Input(Input_Event_Value) => {

                    if Is_Active {

                        self.Handle_Input_Event(Input_Event_Value)?;

                    }

                }

            }

        }

        info!("Monitoring Client | Packet Channel Closed");
        Ok(())

    }




    fn Handle_Control_Message(

        &mut self,
        Message: Control_Message,
        Is_Active: &mut bool,
        
    ) -> Result<()> {

        match Message {

            Control_Message::Take_Control => {

                *Is_Active = true;
                info!("Monitoring Client | Control Taken — Input Injection Active");

            }

            Control_Message::Release_Control => {

                *Is_Active = false;
                info!("Monitoring Client | Control Released — Input Injection Stopped");

            }

            Control_Message::Ping => {

                debug!("Monitoring Client | Ping Received");

            }

            Control_Message::Pong => {

                debug!("Monitoring Client | Pong Received");

            }

        }

        Ok(())

    }




    fn Handle_Input_Event(&mut self, Event: Input_Event) -> Result<()> {

        match Event {

            Input_Event::Key_Press { Key_Code } => {

                self.Virtual_Keyboard.Send_Key_Event(Key_Code, 1)?;

            }

            Input_Event::Key_Release { Key_Code } => {

                self.Virtual_Keyboard.Send_Key_Event(Key_Code, 0)?;

            }

            Input_Event::Mouse_Move { Delta_X, Delta_Y } => {

                if Delta_X != 0 {

                    self.Virtual_Mouse.Send_Relative_Event(0x00, Delta_X)?;

                }

                if Delta_Y != 0 {

                    self.Virtual_Mouse.Send_Relative_Event(0x01, Delta_Y)?;

                }

                self.Virtual_Mouse.Send_Sync_Event()?;

            }

            Input_Event::Mouse_Button_Press { Button } => {

                let Button_Code = Mouse_Button_To_Code(Button);
                self.Virtual_Mouse.Send_Key_Event(Button_Code, 1)?;
                self.Virtual_Mouse.Send_Sync_Event()?;

            }

            Input_Event::Mouse_Button_Release { Button } => {

                let Button_Code = Mouse_Button_To_Code(Button);
                self.Virtual_Mouse.Send_Key_Event(Button_Code, 0)?;
                self.Virtual_Mouse.Send_Sync_Event()?;

            }

            Input_Event::Mouse_Scroll { Direction, Amount } => {

                let Axis_Code = match Direction {

                    Scroll_Direction::Vertical => 0x08,
                    Scroll_Direction::Horizontal => 0x06,

                };

                self.Virtual_Mouse.Send_Relative_Event(Axis_Code, Amount)?;
                self.Virtual_Mouse.Send_Sync_Event()?;

            }

        }

        Ok(())

    }

}




fn Mouse_Button_To_Code(Button: Mouse_Button) -> u16 {

    match Button {
        Mouse_Button::Left => 0x110,
        Mouse_Button::Right => 0x111,
        Mouse_Button::Middle => 0x112,
        Mouse_Button::Side { Button_Index: 0 } => 0x113,
        Mouse_Button::Side { Button_Index: _ } => 0x114,
    }

}




impl Virtual_Keyboard {

    fn Create() -> Result<Self> {

        use std::fs::OpenOptions;

        let Device_File = OpenOptions::new()
            .write(true)
            .open(Uinput_Device_Path)
            .context("Monitoring Client | Uinput Open Failed")?;

        let Raw_File_Descriptor = std::os::unix::io::AsRawFd::as_raw_fd(&Device_File);

        unsafe {

            Setup_Uinput_Keyboard(Raw_File_Descriptor)?;

        }

        Ok(Self { Device_File })

    }




    fn Send_Key_Event(&mut self, Key_Code: u16, Event_Value: i32) -> Result<()> {

        use std::os::unix::io::AsRawFd;

        let Raw_Fd = self.Device_File.as_raw_fd();

        unsafe {

            Write_Uinput_Event(Raw_Fd, 0x01, Key_Code, Event_Value)?;
            Write_Uinput_Event(Raw_Fd, 0x00, 0x00, 0)?;

        }

        Ok(())

    }

}




impl Virtual_Mouse {

    fn Create() -> Result<Self> {

        use std::fs::OpenOptions;

        let Device_File = OpenOptions::new()
            .write(true)
            .open(Uinput_Device_Path)
            .context("Monitoring Client | Uinput Open Failed")?;

        let Raw_File_Descriptor = std::os::unix::io::AsRawFd::as_raw_fd(&Device_File);

        unsafe {

            Setup_Uinput_Mouse(Raw_File_Descriptor)?;

        }

        Ok(Self { Device_File })

    }




    fn Send_Relative_Event(&mut self, Axis_Code: u16, Event_Value: i32) -> Result<()> {

        use std::os::unix::io::AsRawFd;

        unsafe {

            Write_Uinput_Event(self.Device_File.as_raw_fd(), 0x02, Axis_Code, Event_Value)?;

        }

        Ok(())

    }




    fn Send_Key_Event(&mut self, Button_Code: u16, Event_Value: i32) -> Result<()> {

        use std::os::unix::io::AsRawFd;

        unsafe {

            Write_Uinput_Event(self.Device_File.as_raw_fd(), 0x01, Button_Code, Event_Value)?;

        }

        Ok(())

    }




    fn Send_Sync_Event(&mut self) -> Result<()> {

        use std::os::unix::io::AsRawFd;

        unsafe {

            Write_Uinput_Event(self.Device_File.as_raw_fd(), 0x00, 0x00, 0)?;

        }

        Ok(())

    }

}




#[repr(C)]
struct Uinput_Event {

    Time_Seconds: i64,
    Time_Microseconds: i64,
    Event_Type: u16,
    Event_Code: u16,
    Event_Value: i32,

}




#[repr(C)]
struct Uinput_Setup {

    Device_Id: Uinput_Id,
    Device_Name: [u8; 80],
    Force_Feedback_Effects_Max: u32,

}




#[repr(C)]
struct Uinput_Id {

    Bus_Type: u16,
    Vendor_Id: u16,
    Product_Id: u16,
    Version: u16,

}




unsafe fn Write_Uinput_Event(

    File_Descriptor: i32,
    Event_Type: u16,
    Event_Code: u16,
    Event_Value: i32,

) -> Result<()> {

    let Input_Event_Struct = Uinput_Event {

        Time_Seconds: 0,
        Time_Microseconds: 0,

        Event_Type,
        Event_Code,
        Event_Value,

    };

    let Event_Bytes = std::slice::from_raw_parts(

        &Input_Event_Struct as *const Uinput_Event as *const u8,
        std::mem::size_of::<Uinput_Event>(),

    );

    Write_Event_Bytes_To_Fd(File_Descriptor, Event_Bytes)

}




unsafe fn Write_Event_Bytes_To_Fd(File_Descriptor: i32, Event_Bytes: &[u8]) -> Result<()> {

    let Bytes_Written = libc::write(

        File_Descriptor,
        Event_Bytes.as_ptr() as *const libc::c_void,
        Event_Bytes.len(),

    );

    if Bytes_Written < 0 {

        return Err(anyhow::anyhow!(

            "Monitoring Client | Uinput Event Write Failed: {}",

            std::io::Error::last_os_error()

        ));

    }

    Ok(())

}




unsafe fn Setup_Uinput_Keyboard(File_Descriptor: i32) -> Result<()> {

    const Ioctl_Set_Event_Bit: u64 = 0x40045564;
    const Ioctl_Set_Key_Bit: u64 = 0x40045565;
    const Ioctl_Device_Setup: u64 = 0x405c5503;
    const Ioctl_Device_Create: u64 = 0x5501;

    const Event_Type_Sync: u64 = 0x00;
    const Event_Type_Key: u64 = 0x01;

    Run_Ioctl(File_Descriptor, Ioctl_Set_Event_Bit, Event_Type_Sync)?;
    Run_Ioctl(File_Descriptor, Ioctl_Set_Event_Bit, Event_Type_Key)?;

    for Key_Code in 0u64..=255 {

        let _ = Run_Ioctl(File_Descriptor, Ioctl_Set_Key_Bit, Key_Code);

    }

    let mut Device_Setup = Uinput_Setup {

        Device_Id: Uinput_Id {

            Bus_Type: 0x03,
            Vendor_Id: 0x1234,
            Product_Id: 0x5678,
            Version: 1,

        },

        Device_Name: [0u8; 80],
        Force_Feedback_Effects_Max: 0,

    };

    let Keyboard_Name = b"KVM-Virtual-Keyboard";
    Device_Setup.Device_Name[..Keyboard_Name.len()].copy_from_slice(Keyboard_Name);

    let Setup_Bytes = std::slice::from_raw_parts(

        &Device_Setup as *const Uinput_Setup as *const u8,
        std::mem::size_of::<Uinput_Setup>(),

    );

    Run_Ioctl_With_Pointer(File_Descriptor, Ioctl_Device_Setup, Setup_Bytes.as_ptr())?;
    Run_Ioctl(File_Descriptor, Ioctl_Device_Create, 0)?;

    Ok(())

}




unsafe fn Setup_Uinput_Mouse(File_Descriptor: i32) -> Result<()> {

    const Ioctl_Set_Event_Bit: u64 = 0x40045564;
    const Ioctl_Set_Key_Bit: u64 = 0x40045565;
    const Ioctl_Set_Relative_Bit: u64 = 0x40045566;
    const Ioctl_Device_Setup: u64 = 0x405c5503;
    const Ioctl_Device_Create: u64 = 0x5501;

    const Event_Type_Sync: u64 = 0x00;
    const Event_Type_Key: u64 = 0x01;
    const Event_Type_Relative: u64 = 0x02;

    Run_Ioctl(File_Descriptor, Ioctl_Set_Event_Bit, Event_Type_Sync)?;
    Run_Ioctl(File_Descriptor, Ioctl_Set_Event_Bit, Event_Type_Key)?;
    Run_Ioctl(File_Descriptor, Ioctl_Set_Event_Bit, Event_Type_Relative)?;

    for Button_Code in [0x110u64, 0x111, 0x112, 0x113, 0x114] {

        Run_Ioctl(File_Descriptor, Ioctl_Set_Key_Bit, Button_Code)?;

    }

    for Relative_Axis in 0u64..=8 {

        let _ = Run_Ioctl(File_Descriptor, Ioctl_Set_Relative_Bit, Relative_Axis);

    }

    let mut Device_Setup = Uinput_Setup {

        Device_Id: Uinput_Id {

            Bus_Type: 0x03,
            Vendor_Id: 0x1234,
            Product_Id: 0x5679,
            Version: 1,

        },

        Device_Name: [0u8; 80],
        Force_Feedback_Effects_Max: 0,

    };

    let Mouse_Name = b"KVM-Virtual-Mouse";
    Device_Setup.Device_Name[..Mouse_Name.len()].copy_from_slice(Mouse_Name);

    let Setup_Bytes = std::slice::from_raw_parts(

        &Device_Setup as *const Uinput_Setup as *const u8,
        std::mem::size_of::<Uinput_Setup>(),

    );

    Run_Ioctl_With_Pointer(File_Descriptor, Ioctl_Device_Setup, Setup_Bytes.as_ptr())?;
    Run_Ioctl(File_Descriptor, Ioctl_Device_Create, 0)?;

    Ok(())

}




unsafe fn Run_Ioctl(File_Descriptor: i32, Request: u64, Argument: u64) -> Result<()> {

    let Return_Value = libc::ioctl(File_Descriptor, Request, Argument);

    if Return_Value < 0 {

        return Err(anyhow::anyhow!(

            "Monitoring Client | Ioctl Failed: Request=0x{:x}, Error={}",

            Request,
            std::io::Error::last_os_error()

        ));

    }

    Ok(())

}




unsafe fn Run_Ioctl_With_Pointer(

    File_Descriptor: i32,
    Request: u64,
    Pointer: *const u8,

) -> Result<()> {

    let Return_Value = libc::ioctl(File_Descriptor, Request, Pointer);

    if Return_Value < 0 {

        return Err(anyhow::anyhow!(

            "Monitoring Client | Ioctl Pointer Failed: Request=0x{:x}, Error={}",

            Request,
            std::io::Error::last_os_error()

        ));
        
    }

    Ok(())

}
