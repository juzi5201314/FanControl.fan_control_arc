#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::alloc::{alloc, Layout};
use std::ffi::CStr;
use std::mem::{forget, size_of, MaybeUninit};
use std::os::raw::c_void;
use std::ptr::{null, null_mut};
use std::cell::RefCell;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use once_cell::unsync::OnceCell;
use rnet::{net, Net};
use sharedlib::{Lib, Symbol};
use windows_sys::Win32::Foundation::LUID;

use crate::bindings::{_ctl_application_id_t, _ctl_firmware_version_t, _ctl_init_flag_t_CTL_INIT_FLAG_USE_LEVEL_ZERO, _ctl_result_t_CTL_RESULT_SUCCESS, ctl_api_handle_t, ctl_device_adapter_handle_t, ctl_device_adapter_properties_t, ctl_init_args_t, ctl_init_flags_t, ctl_result_t, CTL_IMPL_MAJOR_VERSION, CTL_IMPL_MINOR_VERSION, ctl_temp_handle_t, ctl_temp_properties_t, _ctl_temp_sensors_t_CTL_TEMP_SENSORS_GPU, _ctl_temp_sensors_t_CTL_TEMP_SENSORS_MEMORY, ctl_fan_handle_t, ctl_fan_speed_units_t, _ctl_fan_speed_units_t_CTL_FAN_SPEED_UNITS_PERCENT, ctl_fan_speed_t, _ctl_fan_speed_units_t_CTL_FAN_SPEED_UNITS_MAX, _ctl_fan_speed_units_t_CTL_FAN_SPEED_UNITS_RPM, ctl_fan_properties_t};

// #> bindgen ./lib/igcl_api.h -o src/bindings.rs
mod bindings;

rnet::root!();

static TEMP_HANDLES: Lazy<RwLock<HashMap<Luid, TempHandleContainer>>> = Lazy::new(|| RwLock::new(HashMap::new()));

static LIB: once_cell::sync::OnceCell<Lib> = once_cell::sync::OnceCell::new();


pub struct TempHandleContainer {
    name: String,
    gpu_handle: ctl_temp_handle_t,
    vram_handle: ctl_temp_handle_t,
    fan_handle: ctl_fan_handle_t,
}

unsafe impl Sync for TempHandleContainer {}
unsafe impl Send for TempHandleContainer {}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Net)]
pub struct Luid {
    low: u32,
    high: i32,
}

#[derive(Clone, Eq, PartialEq, Net)]
pub struct Sensor {
    id: Luid,
    name: String,
}

#[net]
pub fn init_api() {
    _init_api()
}

fn _init_api() {
    unsafe {
        let lib = LIB.get_or_init(|| Lib::new("ControlLib.dll").unwrap());

        let mut ctl_init_args = ctl_init_args_t {
            Size: size_of::<ctl_init_args_t>() as u32,
            Version: 0,
            AppVersion: (CTL_IMPL_MAJOR_VERSION << 16u8) | (CTL_IMPL_MINOR_VERSION & 0x0000ffff),
            flags: _ctl_init_flag_t_CTL_INIT_FLAG_USE_LEVEL_ZERO as ctl_init_flags_t,
            SupportedVersion: 0,
            ApplicationUID: _ctl_application_id_t {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            },
        };
        let mut ctl_api_handle = MaybeUninit::uninit();

        let result = lib
            .find_func::<extern "C" fn(
                pInitDesc: *mut ctl_init_args_t,
                phAPIHandle: *mut ctl_api_handle_t,
            ) -> ctl_result_t, _>("ctlInit")
            .unwrap()
            .get()(&mut ctl_init_args, ctl_api_handle.as_mut_ptr());
        if result != _ctl_result_t_CTL_RESULT_SUCCESS {
            panic!("Can't initialize the API: {}", result);
        }
        let ctl_api_handle = ctl_api_handle.assume_init();

        let ctlEnumerateDevices = lib
            .find_func::<extern "C" fn(
                hAPIHandle: ctl_api_handle_t,
                pCount: *mut u32,
                phDevices: *mut ctl_device_adapter_handle_t,
            ) -> ctl_result_t, _>("ctlEnumerateDevices")
            .unwrap()
            .get();

        let mut count = 0;
        let mut ctl_device_adapter_handle =
            null::<ctl_device_adapter_handle_t>() as *mut ctl_device_adapter_handle_t;
        ctlEnumerateDevices(ctl_api_handle, &mut count, ctl_device_adapter_handle);
        let h_devices = malloc::<ctl_device_adapter_handle_t>(count as usize);
        ctlEnumerateDevices(ctl_api_handle, &mut count, h_devices);
        let h_devices = Vec::from_raw_parts(h_devices, count as usize, count as usize);

        for device in &h_devices {
            let mut p = ctl_device_adapter_properties_t {
                Size: size_of::<ctl_device_adapter_properties_t>() as u32,
                Version: 0,
                pDeviceID: alloc(Layout::new::<LUID>()) as *mut c_void,
                device_id_size: size_of::<LUID>() as u32,
                device_type: 0,
                supported_subfunction_flags: 0,
                driver_version: 0,
                firmware_version: _ctl_firmware_version_t {
                    major_version: 0,
                    minor_version: 0,
                    build_number: 0,
                },
                pci_vendor_id: 0,
                pci_device_id: 0,
                rev_id: 0,
                num_eus_per_sub_slice: 0,
                num_sub_slices_per_slice: 0,
                num_slices: 0,
                name: [0; 100],
                graphics_adapter_properties: 0,
                Frequency: 0,
                pci_subsys_id: 0,
                pci_subsys_vendor_id: 0,
                reserved: [0; 116],
            };

            let ctlGetDeviceProperties = lib
                .find_func::<extern "C" fn(
                    hDAhandle: ctl_device_adapter_handle_t,
                    pProperties: *mut ctl_device_adapter_properties_t,
                ) -> ctl_result_t, _>("ctlGetDeviceProperties")
                .unwrap()
                .get();
            ctlGetDeviceProperties(*device, &mut p);

            let ctlEnumTemperatureSensors = lib
                .find_func::<extern "C" fn(
                    hDAhandle: ctl_device_adapter_handle_t,
                    pCount: *mut u32,
                    phTemperature: *mut ctl_temp_handle_t,
                ) -> ctl_result_t, _>("ctlEnumTemperatureSensors")
                .unwrap()
                .get();

            let mut count = 0;
            ctlEnumTemperatureSensors(*device, &mut count, null_mut());
            let temp_handles = malloc::<ctl_temp_handle_t>(count as usize);
            ctlEnumTemperatureSensors(*device, &mut count, temp_handles);
            let temp_handles = Vec::from_raw_parts(temp_handles, count as usize, count as usize);

            let ctlTemperatureGetProperties = lib
                .find_func::<extern "C" fn(
                    hTemperature: ctl_temp_handle_t,
                    pProperties: *mut ctl_temp_properties_t,
                ) -> ctl_result_t, _>("ctlTemperatureGetProperties")
                .unwrap()
                .get();

            let ctlEnumFans = lib
                .find_func::<extern "C" fn(
                    hDAhandle: ctl_device_adapter_handle_t,
                    pCount: *mut u32,
                    phFan: *mut ctl_fan_handle_t,
                ) -> ctl_result_t, _>("ctlEnumFans")
                .unwrap()
                .get();

            let mut count = 0;
            ctlEnumFans(*device, &mut count, null_mut());
            assert!(count > 0);
            let fan_handle = malloc::<ctl_fan_handle_t>(count as usize);
            ctlEnumFans(*device, &mut count, fan_handle);
            let fan_handle = Vec::from_raw_parts(fan_handle, count as usize, count as usize);
            assert!(!fan_handle.is_empty());
            dbg!(fan_handle.len());

            TEMP_HANDLES.write().insert(Luid {
                    low: (*(p.pDeviceID as *mut LUID)).LowPart,
                    high: (*(p.pDeviceID as *mut LUID)).HighPart,
                }, TempHandleContainer {
                    name: CStr::from_ptr(p.name.as_ptr())
                        .to_str()
                        .unwrap()
                        .to_string(),
                    gpu_handle: *temp_handles.iter().find(|h| {
                        let mut ctl_temp_properties = ctl_temp_properties_t {
                            Size: size_of::<ctl_temp_properties_t>() as u32,
                            Version: 0,
                            type_: 0,
                            maxTemperature: 0.0,
                        };
                        let result = ctlTemperatureGetProperties(**h, &mut ctl_temp_properties);
                        assert_eq!(result, _ctl_result_t_CTL_RESULT_SUCCESS);
                        ctl_temp_properties.type_ == _ctl_temp_sensors_t_CTL_TEMP_SENSORS_GPU
                    }).unwrap(),
                    vram_handle: *temp_handles.iter().find(|h| {
                        let mut ctl_temp_properties = ctl_temp_properties_t {
                            Size: size_of::<ctl_temp_properties_t>() as u32,
                            Version: 0,
                            type_: 0,
                            maxTemperature: 0.0,
                        };
                        let result = ctlTemperatureGetProperties(**h, &mut ctl_temp_properties);
                        assert_eq!(result, _ctl_result_t_CTL_RESULT_SUCCESS);
                        ctl_temp_properties.type_ == _ctl_temp_sensors_t_CTL_TEMP_SENSORS_MEMORY
                    }).unwrap(),
                    fan_handle: fan_handle[0],
                });

            forget(temp_handles);
            forget(fan_handle);
        }

        forget(h_devices);
    }
}

#[net]
pub fn sensors() -> Vec<Sensor> {
    _sensors()
}

fn _sensors() -> Vec<Sensor> {
    TEMP_HANDLES.read().iter().map(|(id, c)| Sensor {
        id: *id,
        name: c.name.clone(),
    }).collect()
}

#[net]
pub fn get_gpu_temp(id: Luid) -> f64 {
    _get_gpu_temp(id)
}

fn _get_gpu_temp(id: Luid) -> f64 {
    let ctlTemperatureGetState = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hTemperature: ctl_temp_handle_t,
                pTemperature: *mut f64,
            ) -> ctl_result_t, _>("ctlTemperatureGetState")
            .unwrap()
            .get()
    };
    let mut temp = 0f64;
    let ret = ctlTemperatureGetState(TEMP_HANDLES.read().get(&id).unwrap().gpu_handle, &mut temp);
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
    temp
}

#[net]
pub fn get_vram_temp(id: Luid) -> f64 {
    _get_vram_temp(id)
}

fn _get_vram_temp(id: Luid) -> f64 {
    let ctlTemperatureGetState = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hTemperature: ctl_temp_handle_t,
                pTemperature: *mut f64,
            ) -> ctl_result_t, _>("ctlTemperatureGetState")
            .unwrap()
            .get()
    };
    let mut temp = 0f64;
    let ret = ctlTemperatureGetState(TEMP_HANDLES.read().get(&id).unwrap().vram_handle, &mut temp);
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
    temp
}

#[net]
pub fn _get_fan_speed(id: Luid) -> i32 {
    _get_fan_speed(id)
}

fn _get_fan_speed(id: Luid) -> i32 {
    let ctlFanGetState = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hFan: ctl_fan_handle_t,
                units: ctl_fan_speed_units_t,
                pSpeed: *mut i32,
            ) -> ctl_result_t, _>("ctlFanGetState")
            .unwrap()
            .get()
    };
    let mut speed = 0;
    let ret = ctlFanGetState(TEMP_HANDLES.read().get(&id).unwrap().fan_handle, _ctl_fan_speed_units_t_CTL_FAN_SPEED_UNITS_RPM, &mut speed);
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
    speed
}

#[net]
pub fn _set_fan_speed(id: Luid, speed: i32) {
    _set_fan_speed(id, speed)
}

fn _set_fan_speed(id: Luid, speed: i32) {
    let ctlFanSetFixedSpeedMode = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hFan: ctl_fan_handle_t,
                speed: *const ctl_fan_speed_t,
            ) -> ctl_result_t, _>("ctlFanSetFixedSpeedMode")
            .unwrap()
            .get()
    };
    let ret = ctlFanSetFixedSpeedMode(TEMP_HANDLES.read().get(&id).unwrap().fan_handle, &ctl_fan_speed_t {
        Size: size_of::<ctl_fan_speed_t>() as u32,
        Version: 0,
        speed,
        units: _ctl_fan_speed_units_t_CTL_FAN_SPEED_UNITS_RPM,
    });
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
}

#[net]
pub fn _get_fan_max_speed(id: Luid) -> i32 {
    _get_fan_max_speed(id)
}

fn _get_fan_max_speed(id: Luid) -> i32{
    let ctlFanGetProperties = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hFan: ctl_fan_handle_t,
                pProperties: *mut ctl_fan_properties_t,
            ) -> ctl_result_t, _>("ctlFanGetProperties")
            .unwrap()
            .get()
    };
    let mut p = ctl_fan_properties_t {
        Size: size_of::<ctl_fan_properties_t>() as u32,
        Version: 0,
        canControl: false,
        supportedModes: 0,
        supportedUnits: 0,
        maxRPM: 0,
        maxPoints: 0,
    };
    let ret = ctlFanGetProperties(TEMP_HANDLES.read().get(&id).unwrap().fan_handle, &mut p);
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
    p.maxRPM
}


#[net]
pub fn _set_fan_default(id: Luid) {
    _set_fan_default(id)
}

fn _set_fan_default(id: Luid) {
    let ctlFanSetDefaultMode = unsafe {
        LIB
            .get()
            .unwrap()
            .find_func::<extern "C" fn(
                hFan: ctl_fan_handle_t,
            ) -> ctl_result_t, _>("ctlFanSetDefaultMode")
            .unwrap()
            .get()
    };
    let ret = ctlFanSetDefaultMode(TEMP_HANDLES.read().get(&id).unwrap().fan_handle);
    assert_eq!(ret, _ctl_result_t_CTL_RESULT_SUCCESS);
}

fn malloc<T: Copy>(count: usize) -> *mut T {
    debug_assert!(
        size_of::<T>() > 0,
        "manually allocating a buffer of ZST is a very dangerous idea"
    );
    let mut vec = Vec::<T>::with_capacity(count);
    let ret = vec.as_mut_ptr();
    forget(vec);
    ret
}

#[test]
fn api_test() {
    _init_api();
    for s in _sensors() {
        dbg!(_get_gpu_temp(s.id));
        dbg!(_get_fan_max_speed(s.id));
        //_set_fan_speed(s.id, 100);
    }
}