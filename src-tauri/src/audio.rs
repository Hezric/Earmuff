#![allow(non_snake_case)]
use crate::{
    core::{AudioBackend, Endpoint},
    runtime::Message,
};
use std::{ptr, sync::mpsc::Sender};
use windows::{
    core::{implement, Result, PCWSTR, PWSTR},
    Win32::{
        Devices::{
            DeviceAndDriverInstallation::*,
            FunctionDiscovery::{PKEY_Device_FriendlyName, PKEY_Device_InstanceId},
        },
        Foundation::PROPERTYKEY,
        Media::Audio::{Endpoints::*, *},
        System::Com::{StructuredStorage::*, *},
        UI::Shell::PropertiesSystem::IPropertyStore,
    },
};

#[implement(IMMNotificationClient)]
struct DeviceNotifications(Sender<Message>);
impl IMMNotificationClient_Impl for DeviceNotifications_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> Result<()> {
        let _ = self.0.send(Message::DevicesChanged);
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> Result<()> {
        let _ = self.0.send(Message::DevicesChanged);
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> Result<()> {
        let _ = self.0.send(Message::DevicesChanged);
        Ok(())
    }
    fn OnDefaultDeviceChanged(&self, flow: EDataFlow, role: ERole, _: &PCWSTR) -> Result<()> {
        if flow == eRender && role == eConsole {
            let _ = self.0.send(Message::DevicesChanged);
        }
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> Result<()> {
        let _ = self.0.send(Message::DevicesChanged);
        Ok(())
    }
}
#[implement(IAudioEndpointVolumeCallback)]
struct VolumeNotifications(Sender<Message>);
impl IAudioEndpointVolumeCallback_Impl for VolumeNotifications_Impl {
    fn OnNotify(&self, _: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        let _ = self.0.send(Message::VolumeChanged);
        Ok(())
    }
}

// COM interfaces and their subscriptions stay on the audio worker's MTA.
pub struct WindowsAudio {
    enumerator: IMMDeviceEnumerator,
    devices_callback: IMMNotificationClient,
    volume_callback: IAudioEndpointVolumeCallback,
    current: Option<(String, IAudioEndpointVolume)>,
}
impl WindowsAudio {
    pub fn new(sender: Sender<Message>) -> Result<Self> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let devices_callback: IMMNotificationClient =
                DeviceNotifications(sender.clone()).into();
            let volume_callback: IAudioEndpointVolumeCallback = VolumeNotifications(sender).into();
            enumerator.RegisterEndpointNotificationCallback(&devices_callback)?;
            Ok(Self {
                enumerator,
                devices_callback,
                volume_callback,
                current: None,
            })
        }
    }
    fn detach(&mut self) {
        if let Some((_, volume)) = self.current.take() {
            unsafe {
                let _ = volume.UnregisterControlChangeNotify(&self.volume_callback);
            }
        }
    }
    fn endpoint(&self, id: &str) -> std::result::Result<&IAudioEndpointVolume, String> {
        self.current
            .as_ref()
            .filter(|(current, _)| current == id)
            .map(|(_, v)| v)
            .ok_or_else(|| "Default output changed".into())
    }
    fn enumerate(&self) -> Result<Vec<Endpoint>> {
        unsafe {
            let collection = self
                .enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let mut devices = Vec::new();
            for index in 0..collection.GetCount()? {
                let device = collection.Item(index)?;
                let id = take_string(device.GetId()?)?;
                let store = device.OpenPropertyStore(STGM_READ)?;
                let name = string_property(&store, &PKEY_Device_FriendlyName)
                    .unwrap_or_else(|_| id.clone());
                let instance = string_property(&store, &PKEY_Device_InstanceId)
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("SWD\\MMDEVAPI\\{id}"));
                let factor = store.GetValue(&PKEY_AudioEndpoint_FormFactor)?;
                let form = PropVariantToUInt32(&factor).ok();
                let display = form == Some(DigitalAudioDisplayDevice.0 as u32);
                let headphones =
                    form.is_some_and(|v| v == Headphones.0 as u32 || v == Headset.0 as u32);
                devices.push(Endpoint {
                    id,
                    name,
                    software: software_ancestor(&instance),
                    display,
                    headphones,
                });
            }
            devices.sort_by_key(|device| device.name.to_lowercase());
            Ok(devices)
        }
    }
    fn connect_default(&mut self) -> Result<Option<String>> {
        unsafe {
            let device = match self.enumerator.GetDefaultAudioEndpoint(eRender, eConsole) {
                Ok(device) => device,
                Err(error) if error.code().0 as u32 == 0x80070490 => {
                    self.detach();
                    return Ok(None);
                }
                Err(error) => return Err(error),
            };
            let id = take_string(device.GetId()?)?;
            if self.current.as_ref().is_none_or(|(old, _)| *old != id) {
                self.detach();
                let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
                // Subscribe before the guard reads volume, covering a change during attachment.
                volume.RegisterControlChangeNotify(&self.volume_callback)?;
                self.current = Some((id.clone(), volume));
            }
            Ok(Some(id))
        }
    }
}
impl AudioBackend for WindowsAudio {
    fn active_devices(&mut self) -> std::result::Result<Vec<Endpoint>, String> {
        self.enumerate().map_err(|e| e.to_string())
    }
    fn default_output(&mut self) -> std::result::Result<Option<String>, String> {
        self.connect_default().map_err(|e| e.to_string())
    }
    fn volume(&mut self, id: &str) -> std::result::Result<f32, String> {
        unsafe {
            self.endpoint(id)?
                .GetMasterVolumeLevelScalar()
                .map(|v| v * 100.0)
                .map_err(|e| e.to_string())
        }
    }
    fn set_volume(&mut self, id: &str, percent: f32) -> std::result::Result<(), String> {
        unsafe {
            let current = self
                .enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .and_then(|device| take_string(device.GetId()?))
                .map_err(|e| e.to_string())?;
            if current != id {
                return Err("Default output changed before enforcement".into());
            }
            self.endpoint(id)?
                .SetMasterVolumeLevelScalar(percent / 100.0, ptr::null())
                .map_err(|e| e.to_string())
        }
    }
}
impl Drop for WindowsAudio {
    fn drop(&mut self) {
        self.detach();
        unsafe {
            let _ = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self.devices_callback);
        }
    }
}
unsafe fn take_string(value: PWSTR) -> Result<String> {
    let result = value.to_string();
    CoTaskMemFree(Some(value.0.cast()));
    Ok(result?)
}
unsafe fn string_property(store: &IPropertyStore, key: &PROPERTYKEY) -> Result<String> {
    let value = store.GetValue(key)?;
    take_string(PropVariantToStringAlloc(&value)?)
}
fn software_ancestor(instance: &str) -> bool {
    let id: Vec<u16> = instance.encode_utf16().chain(Some(0)).collect();
    let mut node = 0;
    unsafe {
        if CM_Locate_DevNodeW(&mut node, PCWSTR(id.as_ptr()), CM_LOCATE_DEVNODE_NORMAL)
            != CR_SUCCESS
        {
            return false;
        }
        for _ in 0..32 {
            let mut buffer = [0u16; 512];
            if CM_Get_Device_IDW(node, &mut buffer, 0) != CR_SUCCESS {
                break;
            }
            let length = buffer.iter().position(|v| *v == 0).unwrap_or(buffer.len());
            if String::from_utf16_lossy(&buffer[..length])
                .to_ascii_uppercase()
                .starts_with("ROOT\\")
            {
                return true;
            }
            // Stop at the first bus device; physical USB/PCI devices also have ROOT ancestors.
            if !String::from_utf16_lossy(&buffer[..length])
                .to_ascii_uppercase()
                .starts_with("SWD\\")
            {
                return false;
            }
            let mut parent = 0;
            if CM_Get_Parent(&mut parent, node, 0) != CR_SUCCESS {
                break;
            }
            node = parent;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires Windows audio hardware; read-only device enumeration"]
    fn enumerate_only() {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok().unwrap();
        }
        {
            let (sender, _) = std::sync::mpsc::channel();
            let mut audio = WindowsAudio::new(sender).unwrap();
            for device in audio.active_devices().unwrap() {
                assert!(!device.id.is_empty());
                println!(
                    "{} | software={} display={}",
                    device.name, device.software, device.display
                );
            }
        }
        unsafe {
            CoUninitialize();
        }
    }
}
