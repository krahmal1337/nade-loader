use std::ffi::CString;
use std::path::PathBuf;
use std::process::Command;
use crate::error::LauncherError;

#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, HWND},
    System::Threading::{OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_TERMINATE},
    UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId, PostMessageW, WM_CLOSE},
};

const CSGO_WINDOW_CLASS: &str = "Valve001";
const SYNCHRONIZE_ACCESS: u32 = 0x00100000;
const WAIT_OBJECT_0: u32 = 0;
const SKEET_ALLOC_ADDR: usize = 0x43310000;
const SKEET_ALLOC_SIZE: usize = 0x2FC000;
const SKEET_ALLOC2_SIZE: usize = 0x1000;

#[derive(serde::Serialize)]
pub struct InstalledGames {
    pub cs2_legacy_branch: bool,
    pub csgo_standalone: bool,
}

#[cfg(windows)]
pub fn get_steam_install_path() -> Option<PathBuf> {
    let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey("Software\\Valve\\Steam") {
        if let Ok(steam_path) = key.get_value::<String, _>("SteamPath") {
            return Some(PathBuf::from(steam_path));
        }
    }
    let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Wow6432Node\\Valve\\Steam") {
        if let Ok(steam_path) = key.get_value::<String, _>("InstallPath") {
            return Some(PathBuf::from(steam_path));
        }
    }
    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Valve\\Steam") {
        if let Ok(steam_path) = key.get_value::<String, _>("InstallPath") {
            return Some(PathBuf::from(steam_path));
        }
    }
    None
}

#[cfg(not(windows))]
pub fn get_steam_install_path() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
pub fn steam_install_dir() -> Option<PathBuf> {
    let path = get_steam_install_path()?;
    path.join("steam.exe").exists().then_some(path)
}

#[cfg(not(windows))]
pub fn steam_install_dir() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
pub fn restart_csgo(appid: i32) -> Result<(), LauncherError> {
    close_csgo_if_running()?;

    let steam_dir =
        steam_install_dir().ok_or_else(|| LauncherError::System("failed to find Steam install path".to_string()))?;
    let steam = steam_dir.join("steam.exe");
    
    let protocol_string = match appid {
        730 => "steam://launch/730//-beta%20csgo_legacy".to_string(),
        _ => format!("steam://launch/{}/dialog", appid),
    };

    Command::new(&steam)
        .args([&protocol_string, "-steam", "-insecure", "-novid"])
        .current_dir(&steam_dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| LauncherError::System(format!("failed to launch {}: {error}", steam.display())))
}

#[cfg(not(windows))]
pub fn restart_csgo(_appid: i32) -> Result<(), LauncherError> {
    Ok(())
}

#[cfg(windows)]
pub fn close_csgo_if_running() -> Result<(), LauncherError> {
    let Some(window) = find_csgo_window() else {
        return Ok(());
    };

    let mut process_id = 0;
    unsafe {
        GetWindowThreadProcessId(window, &mut process_id);
    }

    unsafe {
        let _ = PostMessageW(window, WM_CLOSE, 0, 0);
    }

    if process_id == 0 {
        std::thread::sleep(std::time::Duration::from_millis(1400));
        return Ok(());
    }

    let process = unsafe { OpenProcess(SYNCHRONIZE_ACCESS | PROCESS_TERMINATE, 0, process_id) };
    if process.is_null() {
        std::thread::sleep(std::time::Duration::from_millis(1400));
        return Ok(());
    }

    let closed = unsafe { WaitForSingleObject(process, 3500) == WAIT_OBJECT_0 };
    if !closed {
        unsafe {
            TerminateProcess(process, 0);
            let _ = WaitForSingleObject(process, 2000);
        }
    }

    unsafe {
        CloseHandle(process);
    }

    std::thread::sleep(std::time::Duration::from_millis(700));
    Ok(())
}

#[cfg(not(windows))]
pub fn close_csgo_if_running() -> Result<(), LauncherError> {
    Ok(())
}

#[cfg(windows)]
fn find_csgo_window() -> Option<HWND> {
    let class_name = wide_null(CSGO_WINDOW_CLASS);
    let window = unsafe { FindWindowW(class_name.as_ptr(), std::ptr::null()) };
    (!window.is_null()).then_some(window)
}

#[cfg(windows)]
pub fn find_csgo_pid() -> Option<u32> {
    let window = find_csgo_window()?;
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(window, &mut pid) };
    (pid != 0).then_some(pid)
}

#[cfg(not(windows))]
pub fn find_csgo_pid() -> Option<u32> {
    None
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

// ─── Win32 injection helpers (extern) ────────────────────────────────
#[cfg(windows)]
#[allow(non_snake_case)]
#[repr(C)]
struct MODULEENTRY32W {
    dwSize: u32,
    th32ModuleID: u32,
    th32ProcessID: u32,
    glblcntUsage: u32,
    proccntUsage: u32,
    modBaseAddr: *mut u8,
    modBaseSize: u32,
    hModule: *mut core::ffi::c_void,
    szModule: [u16; 256],
    szExePath: [u16; 260],
}

#[cfg(windows)]
const TH32CS_SNAPMODULE: u32 = 0x0000_0008;
#[cfg(windows)]
const MEM_COMMIT: u32 = 0x0000_1000;
#[cfg(windows)]
const MEM_RESERVE: u32 = 0x0000_2000;
#[cfg(windows)]
const MEM_RELEASE: u32 = 0x0000_8000;
#[cfg(windows)]
const PAGE_READWRITE: u32 = 0x04;
#[cfg(windows)]
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
#[cfg(windows)]
const DONT_RESOLVE_DLL_REFERENCES: u32 = 0x0000_0001;
#[cfg(windows)]
const PROCESS_CREATE_THREAD: u32 = 0x0002;
#[cfg(windows)]
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
#[cfg(windows)]
const PROCESS_VM_OPERATION: u32 = 0x0008;
#[cfg(windows)]
const PROCESS_VM_WRITE: u32 = 0x0020;
#[cfg(windows)]
const PROCESS_VM_READ: u32 = 0x0010;

#[cfg(windows)]
extern "system" {
    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> HANDLE;
    fn Module32FirstW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> i32;
    fn Module32NextW(hSnapshot: HANDLE, lpme: *mut MODULEENTRY32W) -> i32;
    fn GetModuleHandleA(lpModuleName: *const u8) -> *mut core::ffi::c_void;
    fn GetProcAddress(hModule: *mut core::ffi::c_void, lpProcName: *const u8) -> *mut core::ffi::c_void;
    fn LoadLibraryExW(lpLibFileName: *const u16, hFile: *mut core::ffi::c_void, dwFlags: u32) -> *mut core::ffi::c_void;
    fn FreeLibrary(hLibModule: *mut core::ffi::c_void) -> i32;
    fn GetSystemDirectoryW(lpBuffer: *mut u16, uSize: u32) -> u32;
    fn GetProcessId(hProcess: HANDLE) -> u32;
    fn VirtualAllocEx(hProcess: HANDLE, lpAddress: *const core::ffi::c_void, dwSize: usize, flAllocationType: u32, flProtect: u32) -> *mut core::ffi::c_void;
    fn VirtualFreeEx(hProcess: HANDLE, lpAddress: *const core::ffi::c_void, dwSize: usize, dwFreeType: u32) -> i32;
    fn VirtualProtectEx(hProcess: HANDLE, lpAddress: *const core::ffi::c_void, dwSize: usize, flNewProtect: u32, lpflOldProtect: *mut u32) -> i32;
    fn WriteProcessMemory(hProcess: HANDLE, lpBaseAddress: *const core::ffi::c_void, lpBuffer: *const core::ffi::c_void, nSize: usize, lpNumberOfBytesWritten: *mut usize) -> i32;
    fn CreateRemoteThread(hProcess: HANDLE, lpThreadAttributes: *const core::ffi::c_void, dwStackSize: usize, lpStartAddress: extern "system" fn() -> i32, lpParameter: *const core::ffi::c_void, dwCreationFlags: u32, lpThreadId: *mut u32) -> HANDLE;
    fn GetExitCodeThread(hThread: HANDLE, lpExitCode: *mut u32) -> i32;
}

#[cfg(windows)]
fn get_module_base(pid: u32, name: &[u16]) -> Option<*mut u8> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);
        if snap.is_null() {
            return None;
        }
        let mut me = std::mem::MaybeUninit::<MODULEENTRY32W>::zeroed().assume_init();
        me.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;
        let mut base = None;
        if Module32FirstW(snap, &mut me) != 0 {
            loop {
                if me.szModule.iter().zip(name.iter()).all(|(a, b)| *a == *b) {
                    base = Some(me.modBaseAddr);
                    break;
                }
                if Module32NextW(snap, &mut me) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        base
    }
}

#[cfg(windows)]
fn restore_nt_open_file(process: HANDLE) {
    eprintln!("[inject] restoring NtOpenFile (EAC bypass)...");
    unsafe {
        let ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
        if ntdll.is_null() {
            eprintln!("[inject] ntdll not found in local process");
            return;
        }
        let local_ntopen = GetProcAddress(ntdll, b"NtOpenFile\0".as_ptr());
        if local_ntopen.is_null() {
            eprintln!("[inject] NtOpenFile not found in local ntdll");
            return;
        }
        let pid = GetProcessId(process);
        let remote_ntdll = get_module_base(pid, &wide_null("ntdll.dll"));
        let Some(remote_base) = remote_ntdll else {
            eprintln!("[inject] remote ntdll not found in PID {pid}");
            return;
        };
        let target = remote_base.wrapping_add((local_ntopen as usize).wrapping_sub(ntdll as usize));
        eprintln!("[inject] local NtOpenFile={local_ntopen:p}, remote ntdll={remote_base:p}, target={target:p}");

        let mut orig = [0u8; 5];
        let sys_path = {
            let mut buf = [0u16; 260];
            GetSystemDirectoryW(buf.as_mut_ptr(), 260);
            let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
            let mut path: Vec<u16> = buf[..len].to_vec();
            path.extend_from_slice(&wide_null("\\ntdll.dll"));
            path
        };

        let fresh = LoadLibraryExW(sys_path.as_ptr(), std::ptr::null_mut(), DONT_RESOLVE_DLL_REFERENCES);
        if !fresh.is_null() {
            let p_fn = GetProcAddress(fresh, b"NtOpenFile\0".as_ptr());
            if !p_fn.is_null() {
                std::ptr::copy_nonoverlapping(p_fn as *const u8, orig.as_mut_ptr(), 5);
                eprintln!("[inject] read {} fresh NtOpenFile bytes", orig.len());
            }
            FreeLibrary(fresh);
        }

        let orig_dword = *(orig.as_ptr() as *const u32);
        if orig_dword == 0 {
            eprintln!("[inject] fresh NtOpenFile bytes are zero, skipping restore");
            return;
        }

        let mut old_prot = 0u32;
        if VirtualProtectEx(process, target as *const _, 5, PAGE_EXECUTE_READWRITE, &mut old_prot) != 0 {
            let ok = WriteProcessMemory(process, target as *const _, orig.as_ptr() as *const _, 5, std::ptr::null_mut());
            VirtualProtectEx(process, target as *const _, 5, old_prot, &mut old_prot);
            eprintln!("[inject] NtOpenFile restore: WriteProcessMemory={}", if ok != 0 { "OK" } else { "FAILED" });
        } else {
            eprintln!("[inject] VirtualProtectEx failed for NtOpenFile restore");
        }
    }
}

#[cfg(windows)]
pub fn inject_dll(pid: u32, dll_path: &str, skeet: bool) -> Result<(), LauncherError> {
    eprintln!("[inject] opening PID {pid} for injection (skeet={skeet})");
    const ACCESS: u32 = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ;

    unsafe {
        let process = OpenProcess(ACCESS, 0, pid);
        if process.is_null() {
            eprintln!("[inject] OpenProcess failed for PID {pid}");
            return Err(LauncherError::System("failed to open target process".to_string()));
        }
        eprintln!("[inject] OpenProcess OK");

        if skeet {
            eprintln!("[inject] skeet VAC bypass: restoring NtOpenFile in remote process");
            let ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
            if !ntdll.is_null() {
                let local_ntopen = GetProcAddress(ntdll, b"NtOpenFile\0".as_ptr());
                if !local_ntopen.is_null() {
                    let mut orig = [0u8; 5];
                    std::ptr::copy_nonoverlapping(local_ntopen as *const u8, orig.as_mut_ptr(), 5);
                    let mut written = 0usize;
                    WriteProcessMemory(process, local_ntopen as *const _, orig.as_ptr() as *const _, 5, &mut written);
                    eprintln!("[inject] NtOpenFile restore: wrote {written} bytes");
                }
            }

            eprintln!("[inject] skeet pre-allocation: 0x{SKEET_ALLOC_ADDR:x} size 0x{SKEET_ALLOC_SIZE:x}");
            let alloc1 = VirtualAllocEx(process, SKEET_ALLOC_ADDR as *const _, SKEET_ALLOC_SIZE, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
            eprintln!("[inject] pre-alloc 1: {:?}", if alloc1.is_null() { "FAILED" } else { "OK" });

            eprintln!("[inject] skeet pre-allocation 2: null addr size 0x{SKEET_ALLOC2_SIZE:x}");
            let alloc2 = VirtualAllocEx(process, std::ptr::null(), SKEET_ALLOC2_SIZE, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
            eprintln!("[inject] pre-alloc 2: {:?}", if alloc2.is_null() { "FAILED" } else { "OK" });
        } else {
            restore_nt_open_file(process);
        }

        let path_cstring = CString::new(dll_path.as_bytes())
            .map_err(|_| LauncherError::System("DLL path contains null byte".to_string()))?;
        let path_bytes = path_cstring.as_bytes_with_nul();
        let path_len = path_bytes.len();
        eprintln!("[inject] VirtualAllocEx({path_len} bytes): {dll_path}");
        let remote_path = VirtualAllocEx(process, std::ptr::null(), path_len, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        if remote_path.is_null() {
            eprintln!("[inject] VirtualAllocEx failed");
            CloseHandle(process);
            return Err(LauncherError::System("VirtualAllocEx failed".to_string()));
        }
        eprintln!("[inject] remote memory allocated at {remote_path:p}");

        let written = WriteProcessMemory(process, remote_path, path_bytes.as_ptr() as *const _, path_len, std::ptr::null_mut());
        if written == 0 {
            eprintln!("[inject] WriteProcessMemory failed");
            VirtualFreeEx(process, remote_path, 0, MEM_RELEASE);
            CloseHandle(process);
            return Err(LauncherError::System("WriteProcessMemory failed".to_string()));
        }
        eprintln!("[inject] WriteProcessMemory OK");

        let kernel32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
        if kernel32.is_null() {
            eprintln!("[inject] failed to get kernel32 handle");
            VirtualFreeEx(process, remote_path, 0, MEM_RELEASE);
            CloseHandle(process);
            return Err(LauncherError::System("failed to get kernel32".to_string()));
        }

        let loadlib = GetProcAddress(kernel32, b"LoadLibraryA\0".as_ptr());
        if loadlib.is_null() {
            eprintln!("[inject] failed to find LoadLibraryA");
            VirtualFreeEx(process, remote_path, 0, MEM_RELEASE);
            CloseHandle(process);
            return Err(LauncherError::System("failed to find LoadLibraryA".to_string()));
        }
        eprintln!("[inject] LoadLibraryA at {loadlib:p}");

        let loadlib_fn: extern "system" fn() -> i32 = std::mem::transmute(loadlib);
        eprintln!("[inject] CreateRemoteThread...");
        let thread = CreateRemoteThread(process, std::ptr::null(), 0, loadlib_fn, remote_path as *const _, 0, std::ptr::null_mut());
        if thread.is_null() {
            eprintln!("[inject] CreateRemoteThread failed");
            VirtualFreeEx(process, remote_path, 0, MEM_RELEASE);
            CloseHandle(process);
            return Err(LauncherError::System("CreateRemoteThread failed".to_string()));
        }
        eprintln!("[inject] waiting for remote thread to finish...");
        WaitForSingleObject(thread, 0xFFFFFFFF);

        let mut exit_code = 0u32;
        GetExitCodeThread(thread, &mut exit_code);
        eprintln!("[inject] remote thread exit code: {exit_code}");
        VirtualFreeEx(process, remote_path, 0, MEM_RELEASE);
        CloseHandle(thread);
        CloseHandle(process);

        if exit_code == 0 {
            eprintln!("[inject] LoadLibrary returned 0 — injection failed");
            return Err(LauncherError::System("DLL failed to load in target process".to_string()));
        }

        eprintln!("[inject] injection successful!");
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn inject_dll(_pid: u32, _dll_path: &str, _skeet: bool) -> Result<(), LauncherError> {
    Err(LauncherError::System("injection not supported on this platform".to_string()))
}


