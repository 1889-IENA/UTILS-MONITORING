// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use anyhow::{Context, Result};
use evdev::{Device, InputEventKind, Key, RelativeAxisType};
use log::info;


use tokio::sync::mpsc;




use protocol::{Control_Message, Input_Event, Mouse_Button, Packet, Scroll_Direction};
use crate::S_Device_Scanner::S_Scan_Input_Devices;




const Pause_Break_Key_Code: u16 = 119;




pub struct S_Input_Capturer {

    Keyboard_Device: Device,
    Mouse_Device: Device,
    Packet_Sender: mpsc::Sender<Packet>,
    Is_Controlling_Remote: bool,

}




impl S_Input_Capturer {

    pub async fn New(Packet_Sender: mpsc::Sender<Packet>) -> Result<Self> {

        let Found_Devices = S_Scan_Input_Devices()
            .context("Monitoring Server | Device Scan Failed")?;

        let Keyboard_Device = Device::open(&Found_Devices.Keyboard_Path)
            .context("Monitoring Server | Keyboard Setup Failed")?;

        let Mouse_Device = Device::open(&Found_Devices.Mouse_Path)
            .context("Monitoring Server | Mouse Setup Failed")?;

        Ok(Self {

            Keyboard_Device,
            Mouse_Device,
            Packet_Sender,
            Is_Controlling_Remote: false,

        })

    }




    pub async fn Run(&mut self) -> Result<()> {

        info!("Monitoring Server | Input Event Loop Starting..");

        let Keyboard_Raw_Fd = {

            use std::os::unix::io::AsRawFd;
            self.Keyboard_Device.as_raw_fd()

        };

        let Mouse_Raw_Fd = {

            use std::os::unix::io::AsRawFd;
            self.Mouse_Device.as_raw_fd()
            
        };

        loop {

            let Keyboard_Ready = Self::Wait_Fd_Readable(Keyboard_Raw_Fd);
            let Mouse_Ready    = Self::Wait_Fd_Readable(Mouse_Raw_Fd);

            tokio::select! {

                _ = Keyboard_Ready => {

                    let Keyboard_Events = self.Keyboard_Device
                        .fetch_events()
                        .context("Monitoring Server | Keyboard Events Failed")?
                        .collect::<Vec<_>>();

                    for Raw_Event in Keyboard_Events {

                        self.Handle_Keyboard_Event(Raw_Event).await?;

                    }

                }

                _ = Mouse_Ready => {

                    let Mouse_Events = self.Mouse_Device
                        .fetch_events()
                        .context("Monitoring Server | Mouse Events Failed")?
                        .collect::<Vec<_>>();

                    for Raw_Event in Mouse_Events {

                        self.Handle_Mouse_Event(Raw_Event).await?;

                    }

                }

            }

        }

    }




    async fn Wait_Fd_Readable(File_Descriptor: i32) {

        tokio::io::unix::AsyncFd::new(File_Descriptor)
            .expect("AsyncFd olusturulamadi")
            .readable()
            .await
            .expect("readable bekleme hatasi")
            .retain_ready();

    }




    async fn Handle_Keyboard_Event(&mut self, Raw_Event: evdev::InputEvent) -> Result<()> {

        let Key_Code = match Raw_Event.kind() {

            InputEventKind::Key(Key_From_Event) => Key_From_Event.code(),
            _ => return Ok(()),

        };

        let Event_Value = Raw_Event.value();

        if Key_Code == Pause_Break_Key_Code && Event_Value == 1 {

            self.Toggle_Remote_Control().await?;
            return Ok(());

        }

        if !self.Is_Controlling_Remote {

            return Ok(());

        }

        let Packet_To_Send = match Event_Value {

            1 => Packet::Input(Input_Event::Key_Press { Key_Code }),
            0 => Packet::Input(Input_Event::Key_Release { Key_Code }),
            _ => return Ok(()),

        };

        self.Send_Packet(Packet_To_Send).await

    }




    async fn Handle_Mouse_Event(&mut self, Raw_Event: evdev::InputEvent) -> Result<()> {

        if !self.Is_Controlling_Remote {

            return Ok(());

        }

        let Packet_To_Send = match Raw_Event.kind() {

            InputEventKind::RelAxis(Axis) => {

                match Axis {

                    RelativeAxisType::REL_X => {

                        Some(Packet::Input(Input_Event::Mouse_Move {

                            Delta_X: Raw_Event.value(),
                            Delta_Y: 0,

                        }))

                    }

                    RelativeAxisType::REL_Y => {

                        Some(Packet::Input(Input_Event::Mouse_Move {

                            Delta_X: 0,
                            Delta_Y: Raw_Event.value(),

                        }))

                    }

                    RelativeAxisType::REL_WHEEL => {

                        Some(Packet::Input(Input_Event::Mouse_Scroll {

                            Direction: Scroll_Direction::Vertical,
                            Amount: Raw_Event.value(),

                        }))

                    }

                    RelativeAxisType::REL_HWHEEL => {

                        Some(Packet::Input(Input_Event::Mouse_Scroll {

                            Direction: Scroll_Direction::Horizontal,
                            Amount: Raw_Event.value(),

                        }))

                    }

                    _ => None,

                }

            }

            InputEventKind::Key(Key_From_Event) => {

                let Mouse_Button_Option = match Key_From_Event {

                    Key::BTN_LEFT   => Some(Mouse_Button::Left),
                    Key::BTN_RIGHT  => Some(Mouse_Button::Right),
                    Key::BTN_MIDDLE => Some(Mouse_Button::Middle),
                    Key::BTN_SIDE   => Some(Mouse_Button::Side { Button_Index: 0 }),
                    Key::BTN_EXTRA  => Some(Mouse_Button::Side { Button_Index: 1 }),
                    _               => None,

                };

                if let Some(Button) = Mouse_Button_Option {

                    match Raw_Event.value() {

                        1 => Some(Packet::Input(Input_Event::Mouse_Button_Press { Button })),
                        0 => Some(Packet::Input(Input_Event::Mouse_Button_Release { Button })),
                        _ => None,

                    }

                } else {

                    None

                }

            }

            _ => None,

        };

        if let Some(Packet_To_Send) = Packet_To_Send {

            self.Send_Packet(Packet_To_Send).await?;

        }

        Ok(())

    }




    async fn Toggle_Remote_Control(&mut self) -> Result<()> {

        self.Is_Controlling_Remote = !self.Is_Controlling_Remote;

        if self.Is_Controlling_Remote {

            info!("Monitoring Server | Other PC Controlling By You");
            
            self.Send_Packet(Packet::Control(Control_Message::Take_Control)).await?;

            if let Err(Capture_Error) = self.Keyboard_Device.grab() {

                log::warn!("Monitoring Server | Failed to grab keyboard: {}", Capture_Error);

            }
            if let Err(Capture_Error) = self.Mouse_Device.grab() {

                log::warn!("Monitoring Server | Failed to grab mouse: {}", Capture_Error);

            }

        } else {

            info!("Monitoring Server | This PC Controlling By You");
            self.Send_Packet(Packet::Control(Control_Message::Release_Control)).await?;

            if let Err(Capture_Error) = self.Keyboard_Device.ungrab() {

                log::warn!("Monitoring Server | Failed to ungrab keyboard: {}", Capture_Error);

            }

            if let Err(Capture_Error) = self.Mouse_Device.ungrab() {

                log::warn!("Monitoring Server | Failed to ungrab mouse: {}", Capture_Error);

            }

        }

        Ok(())

    }




    async fn Send_Packet(&self, Packet_To_Send: Packet) -> Result<()> {

        self.Packet_Sender
            .send(Packet_To_Send)
            .await
            .context("Monitoring Server | Queue List Add Failed")

    }

}
