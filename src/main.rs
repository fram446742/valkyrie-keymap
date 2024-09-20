use std::collections::HashMap;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, GetAsyncKeyState, GetKeyState, SendInput, SetWindowsHookExW, INPUT,
    INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYEVENTF_UNICODE, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU,
    VK_RWIN, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
};

struct KeyMapper {
    lowercase_map: HashMap<char, char>,
    uppercase_map: HashMap<char, char>,
}

impl KeyMapper {
    fn new() -> Self {
        KeyMapper {
            lowercase_map: HashMap::new(),
            uppercase_map: HashMap::new(),
        }
    }

    fn add_mapping(&mut self, char: char, mapped_lower: char, mapped_upper: char) {
        self.lowercase_map.insert(char, mapped_lower);
        self.uppercase_map.insert(char, mapped_upper);
    }

    fn is_modifier_key_pressed() -> bool {
        unsafe {
            GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0
                || GetAsyncKeyState(VK_MENU) & 0x8000u16 as i16 != 0
                || GetAsyncKeyState(VK_LWIN) & 0x8000u16 as i16 != 0
                || GetAsyncKeyState(VK_RWIN) & 0x8000u16 as i16 != 0
        }
    }

    fn is_caps_lock_on() -> bool {
        unsafe { (GetKeyState(VK_CAPITAL) & 0x0001) != 0 }
    }

    fn map_key(&self, key: char, is_shift_pressed: bool) -> Option<char> {
        if !unsafe { IS_MAPPING_ENABLED } || Self::is_modifier_key_pressed() {
            return None;
        }

        let is_uppercase = is_shift_pressed ^ Self::is_caps_lock_on();

        if is_uppercase {
            self.uppercase_map.get(&key).cloned()
        } else {
            self.lowercase_map.get(&key).cloned()
        }
    }
}

static mut KEY_MAPPER: Option<KeyMapper> = None;
static mut IS_MAPPING_ENABLED: bool = true;
static mut IS_TOGGLE_PROCESSED: bool = false;
static START: &str = "
8b,dPPYba, 88       88 8b,dPPYba,   ,adPPYba, ,adPPYba,  
88P'   \"Y8 88       88 88P'   `\"8a a8P_____88 I8[    \"\"  
88         88       88 88       88 8PP\"\"\"\"\"\"\"  `\"Y8ba,   
88         \"8a,   ,a88 88       88 \"8b,   ,aa aa    ]8I  
88          `\"YbbdP'Y8 88       88  `\"Ybbd8\"' `\"YbbdP\"'
";

unsafe extern "system" fn keyboard_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        if w_param == WM_KEYDOWN as WPARAM || w_param == WM_KEYUP as WPARAM {
            let vk_code = kb_struct.vkCode as u8 as char;

            // Check if the Shift key is pressed
            let is_shift_pressed = (GetAsyncKeyState(0x10) & 0x8000u16 as i16) != 0;
            // Check if the Ctrl is pressed
            let is_ctrl_pressed = (GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16) != 0;
            // Check if the Alt is pressed
            let is_alt_pressed = (GetAsyncKeyState(VK_MENU) & 0x8000u16 as i16) != 0;

            // Check for Ctrl + Alt + M combination
            if let Some(value) = toggle_runes(is_ctrl_pressed, is_alt_pressed, vk_code, w_param) {
                return value;
            }

            // Check for Ctrl + Alt + Q combination
            process_exit_command(is_ctrl_pressed, is_alt_pressed, vk_code, w_param);

            // Check for easteregg combination
            if let Some(value) =
                process_easter_egg(is_ctrl_pressed, is_alt_pressed, vk_code, w_param)
            {
                return value;
            }

            // Check if the key is mapped
            if let Some(value) = process_key_mapping_event(vk_code, is_shift_pressed, w_param) {
                return value;
            }
        }
    }
    CallNextHookEx(null_mut(), code, w_param, l_param)
}

fn process_key_mapping_event(
    vk_code: char,
    is_shift_pressed: bool,
    w_param: usize,
) -> Option<isize> {
    unsafe {
        if let Some(ref key_mapper) = KEY_MAPPER {
            if let Some(mapped_key) = key_mapper.map_key(vk_code, is_shift_pressed) {
                if w_param == WM_KEYDOWN as WPARAM {
                    let mut input = INPUT {
                        type_: INPUT_KEYBOARD,
                        u: std::mem::zeroed(),
                    };
                    let ki = input.u.ki_mut();
                    ki.wVk = 0; // No virtual key code is needed for Unicode input
                    ki.wScan = mapped_key as u16; // Set the Unicode character to be sent
                    ki.dwFlags = KEYEVENTF_UNICODE; // Use Unicode event flag
                    ki.time = 0;
                    ki.dwExtraInfo = 0;
                    SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                }
                return Some(1); // Block the original event
            }
        }
        None
    }
}

fn process_easter_egg(
    is_ctrl_pressed: bool,
    is_alt_pressed: bool,
    vk_code: char,
    w_param: usize,
) -> Option<isize> {
    if is_ctrl_pressed && is_alt_pressed && vk_code == 'Y' {
        if w_param == WM_KEYDOWN as WPARAM {
            println!("{}", START);
        }
        return Some(1); // Block the original event
    }
    None
}

fn process_exit_command(
    is_ctrl_pressed: bool,
    is_alt_pressed: bool,
    vk_code: char,
    w_param: usize,
) {
    if is_ctrl_pressed && is_alt_pressed && vk_code == 'Q' {
        if w_param == WM_KEYDOWN as WPARAM {
            // Exit the application
            println!("Exiting the application...");
            std::process::exit(0);
        }
    }
}

fn toggle_runes(
    is_ctrl_pressed: bool,
    is_alt_pressed: bool,
    vk_code: char,
    w_param: usize,
) -> Option<isize> {
    unsafe {
        if is_ctrl_pressed && is_alt_pressed && vk_code == 'M' {
            if w_param == WM_KEYDOWN as WPARAM && !IS_TOGGLE_PROCESSED {
                IS_MAPPING_ENABLED = !IS_MAPPING_ENABLED;
                IS_TOGGLE_PROCESSED = true;
                println!("Mapping toggled: {}", IS_MAPPING_ENABLED);
            } else if w_param == WM_KEYUP as WPARAM {
                IS_TOGGLE_PROCESSED = false;
            }
            return Some(1); // Block the original event
        }
        None
    }
}

fn generate_mapping() -> KeyMapper {
    let mut key_mapper = KeyMapper::new();
    key_mapper.add_mapping('A', 'ᚨ', 'ᚪ');
    key_mapper.add_mapping('B', 'ᛒ', 'ᛔ');
    key_mapper.add_mapping('C', 'ᚲ', 'ᛈ');
    key_mapper.add_mapping('D', 'ᚦ', 'ᚣ'); // ᚮ
    key_mapper.add_mapping('E', 'ᛅ', 'ᚯ'); // ᛑ
    key_mapper.add_mapping('F', 'ᚠ', 'ᚡ');
    key_mapper.add_mapping('G', 'ᛞ', 'ᛥ');
    key_mapper.add_mapping('H', 'ᚺ', 'ᚻ');
    key_mapper.add_mapping('I', 'ᛁ', 'ᛂ');
    key_mapper.add_mapping('J', 'ᚴ', 'ᚵ');
    key_mapper.add_mapping('K', 'ᛘ', 'ᛯ');
    key_mapper.add_mapping('L', 'ᛐ', 'ᛚ');
    key_mapper.add_mapping('M', 'ᛖ', 'ᛗ');
    key_mapper.add_mapping('N', 'ᚾ', 'ᚬ');
    key_mapper.add_mapping('O', 'ᛜ', 'ᛟ');
    key_mapper.add_mapping('P', 'ᛩ', 'ᚹ');
    key_mapper.add_mapping('Q', 'ᛶ', 'ᚿ'); // ᛃ
    key_mapper.add_mapping('R', 'ᛃ', 'ᚱ'); //
    key_mapper.add_mapping('S', 'ᛋ', 'ᛊ');
    key_mapper.add_mapping('T', 'ᛄ', 'ᛏ');
    key_mapper.add_mapping('U', 'ᚢ', 'ᚤ');
    key_mapper.add_mapping('V', 'ᛡ', 'ᛤ');
    key_mapper.add_mapping('W', 'ᚳ', 'ᛠ');
    key_mapper.add_mapping('X', '×', 'ᚷ');
    key_mapper.add_mapping('Y', 'ᛣ', 'ᛉ');
    key_mapper.add_mapping('Z', 'ᛇ', 'ᛢ');

    key_mapper
}

fn main() {
    println!("Welcome to the Runic Keyboard!");
    println!("This application will map the English alphabet to the Valkyrie lang runes.");
    println!("Press Ctrl + Alt + M to toggle the mapping.");
    println!("Press Ctrl + Alt + Q to exit the application.");
    unsafe {
        let key_mapper = generate_mapping();

        KEY_MAPPER = Some(key_mapper);

        let h_instance = GetModuleHandleW(null_mut());
        let _ = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), h_instance, 0);

        let mut msg = std::mem::zeroed();
        while winapi::um::winuser::GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
            winapi::um::winuser::TranslateMessage(&msg);
            winapi::um::winuser::DispatchMessageW(&msg);
        }
    }
}
