// v0.5.0




#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]




use anyhow::{Context, Result};
use evdev::{Device, Key};
use std::path::PathBuf;
use log::info;




pub struct S_Found_Devices {

    pub Keyboard_Path: PathBuf,
    pub Mouse_Path: PathBuf,
    
}




pub fn S_Scan_Input_Devices() -> Result<S_Found_Devices> {

    info!("Monitoring Server | Input Devices Scanning..");

    let mut Keyboard_Path: Option<PathBuf> = None;
    let mut Mouse_Path: Option<PathBuf> = None;

    for Directory_Entry in std::fs::read_dir("/dev/input")? {

        let Directory_Entry = Directory_Entry?;
        let Entry_Path = Directory_Entry.path();

        if !Entry_Path.to_string_lossy().contains("event") {

            continue;

        }

        let Current_Device = match Device::open(&Entry_Path) {

            Ok(Current_Device) => Current_Device,
            Err(_) => continue,

        };

        let Device_Name = Current_Device.name().unwrap_or("Isimsiz").to_string();

        if Is_Keyboard(&Current_Device) && Keyboard_Path.is_none() {

            info!("Monitoring Server | Keyboard Founded: {} ({})", Device_Name, Entry_Path.display());
            Keyboard_Path = Some(Entry_Path.clone());

        }

        if Is_Mouse(&Current_Device) && Mouse_Path.is_none() {

            info!("Monitoring Server | Mouse Founded: {} ({})", Device_Name, Entry_Path.display());
            Mouse_Path = Some(Entry_Path.clone());

        }

        if Keyboard_Path.is_some() && Mouse_Path.is_some() {

            break;

        }

    }


    let Keyboard_Path = Keyboard_Path
        .context("Monitoring Server | Keyboard Not Found - Control This Path? '/dev/input' ")?;

    let Mouse_Path = Mouse_Path
        .context("Monitoring Server | Mouse Not Found - Control This Path? '/dev/input' ")?;

    Ok(S_Found_Devices {

        Keyboard_Path,
        Mouse_Path,

    })

}




fn Is_Keyboard(Device_To_Check: &Device) -> bool {

    let Supported_Keys = match Device_To_Check.supported_keys() {

        Some(Supported_Keys) => Supported_Keys,
        None => return false,

    };

    let Has_Letter_Keys = Supported_Keys.contains(Key::KEY_A)
        && Supported_Keys.contains(Key::KEY_Z)
        && Supported_Keys.contains(Key::KEY_SPACE);

    Has_Letter_Keys

}




fn Is_Mouse(Device_To_Check: &Device) -> bool {

    use evdev::RelativeAxisType;

    let Supported_Relative_Axes = match Device_To_Check.supported_relative_axes() {

        Some(Supported_Relative_Axes) => Supported_Relative_Axes,
        None => return false,

    };

    let Has_XY_Axes = Supported_Relative_Axes.contains(RelativeAxisType::REL_X)
        && Supported_Relative_Axes.contains(RelativeAxisType::REL_Y);

    Has_XY_Axes

}
