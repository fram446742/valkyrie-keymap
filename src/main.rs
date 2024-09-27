use std::collections::HashMap;
use std::mem::zeroed;
// For the audio
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
// For the keyboard hook
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, GetAsyncKeyState, GetKeyState, SendInput, SetWindowsHookExW, INPUT,
    INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYEVENTF_UNICODE, VK_CAPITAL, VK_CONTROL, VK_LSHIFT, VK_LWIN,
    VK_MENU, VK_RSHIFT, VK_RWIN, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
};

// The key mapper struct
struct KeyMapper {
    lowercase_map: HashMap<char, char>,
    uppercase_map: HashMap<char, char>,
}

// The key mapper implementation
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

// The global variables
static mut KEY_MAPPER: Option<KeyMapper> = None;
static mut IS_MAPPING_ENABLED: bool = true;
static mut IS_TOGGLE_PROCESSED: bool = false;
// Load the sound data
static ACTIVATED_SOUND: &[u8] = include_bytes!("sounds/mixkit-big-fire-spell-burning-1332.wav");
static DEACTIVATED_SOUND: &[u8] = include_bytes!("sounds/mixkit-powerful-air-whooshes-3220.wav");

// The main menu
static MENU: &str = "
ᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞ
ᚨ                            𖤍 𖤍     ᛤᚪᛚᛯᛉᚱᛂᚯ     𖤍 𖤍                               ᚺ
ᛒ               𖤍 𖤍   Welcome to the Runic Keyboard mapper!   𖤍 𖤍                   ᛁ
ᚲ      This application will map the English alphabet to the Valkyrie lang runes.   ᚴ
ᚦ                   Press Ctrl + Alt + M to toggle the mapping.                     ᛘ
ᛅ               Press RShift + (number) to use the custom symbology.                ᛐ
ᛞ                  Press Ctrl + Alt + Q to exit the application.                    ᛖ
ᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞᚺᛁᚴᛘᛐᛖᚾᛜᛩᛶᛃᛋᛄᚢᛡᚳ×ᛣᛇᚨᛒᚲᚦᛅᚠᛞ
";

// The keyboard hook
unsafe extern "system" fn keyboard_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        if w_param == WM_KEYDOWN as WPARAM || w_param == WM_KEYUP as WPARAM {
            let vk_code = kb_struct.vkCode as u8 as char;

            // Check if the Left Shift key is pressed
            let is_left_shift_pressed = (GetAsyncKeyState(VK_LSHIFT) & 0x8000u16 as i16) != 0;
            // Check if the Right Shift key is pressed
            let is_right_shift_pressed = (GetAsyncKeyState(VK_RSHIFT) & 0x8000u16 as i16) != 0;
            // Check if the Ctrl is pressed
            let is_ctrl_pressed = (GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16) != 0;
            // Check if the Alt is pressed
            let is_alt_pressed = (GetAsyncKeyState(VK_MENU) & 0x8000u16 as i16) != 0;

            // Check for Ctrl + Alt + M combination
            if let Some(value) = toggle_runes(is_ctrl_pressed, is_alt_pressed, vk_code, w_param) {
                return value;
            }

            println!("Key: {}", vk_code);
            println!("Left Shift: {}", is_left_shift_pressed);
            println!("Right Shift: {}", is_right_shift_pressed);
            println!("Ctrl: {}", is_ctrl_pressed);
            println!("Alt: {}", is_alt_pressed);

            // Check for Ctrl + Alt + Q combination
            process_exit_command(is_ctrl_pressed, is_alt_pressed, vk_code, w_param);

            // Check if the key is mapped
            if let Some(value) = process_key_mapping_event(
                vk_code,
                is_left_shift_pressed,
                is_right_shift_pressed,
                w_param,
            ) {
                return value;
            }
        }
    }
    CallNextHookEx(null_mut(), code, w_param, l_param)
}

// Process the key mapping event
fn process_key_mapping_event(
    vk_code: char,
    is_left_shift_pressed: bool,
    is_right_shift_pressed: bool,
    w_param: usize,
) -> Option<isize> {
    unsafe {
        // Verificar si la tecla Shift derecha está presionada y el carácter es un número
        if is_left_shift_pressed && vk_code.is_digit(10) {
            return None; // Permitir que el evento original sea procesado
        }

        if let Some(ref key_mapper) = KEY_MAPPER {
            if let Some(mapped_key) = key_mapper.map_key(vk_code, is_left_shift_pressed || is_right_shift_pressed) {
                if w_param == WM_KEYDOWN as WPARAM {
                    let unicode_value = mapped_key as u32;

                    if unicode_value <= 0xFFFF {
                        // For BMP characters (code point ≤ 0xFFFF)
                        let mut input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: zeroed(),
                        };

                        let ki = input.u.ki_mut();

                        ki.wVk = 0;
                        ki.wScan = mapped_key as u16;
                        ki.dwFlags = KEYEVENTF_UNICODE;
                        ki.time = 0;
                        ki.dwExtraInfo = 0;
                        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
                    } else {
                        // For extended Unicode (code points > 0xFFFF), send surrogate pairs
                        let high_surrogate = ((unicode_value - 0x10000) / 0x400 + 0xD800) as u16;
                        let low_surrogate = ((unicode_value - 0x10000) % 0x400 + 0xDC00) as u16;

                        // Send the high surrogate
                        let mut high_input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: zeroed(),
                        };
                        let high_ki = high_input.u.ki_mut();
                        high_ki.wVk = 0;
                        high_ki.wScan = high_surrogate;
                        high_ki.dwFlags = KEYEVENTF_UNICODE;
                        high_ki.time = 0;
                        high_ki.dwExtraInfo = 0;
                        SendInput(1, &mut high_input, std::mem::size_of::<INPUT>() as i32);

                        // Send the low surrogate
                        let mut low_input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: zeroed(),
                        };
                        let low_ki = low_input.u.ki_mut();
                        low_ki.wVk = 0;
                        low_ki.wScan = low_surrogate;
                        low_ki.dwFlags = KEYEVENTF_UNICODE;
                        low_ki.time = 0;
                        low_ki.dwExtraInfo = 0;
                        SendInput(1, &mut low_input, std::mem::size_of::<INPUT>() as i32);
                    }
                    return Some(1); // Block the original event
                }
            }
        }
        None
    }
}

// Process the exit command
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

// Toggle the runes mapping
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

                sound_thread(IS_MAPPING_ENABLED);
            } else if w_param == WM_KEYUP as WPARAM {
                IS_TOGGLE_PROCESSED = false;
            }
            return Some(1); // Block the original event
        }
        None
    }
}

// Play the sound
fn sound_thread(is_mapping_thread: bool) {
    if is_mapping_thread {
        println!("Runes Awakened! You’ve been blessed by the ancient spirits 🔥🐦‍🔥");
        print!("\x1B[1A\x1B[2K"); // Move cursor up one line and clear the line
    } else {
        println!("Runes Slumbering. The ancient spirits are resting... 💨❄️");
        print!("\x1B[1A\x1B[2K"); // Move cursor up one line and clear the line
    }
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        match is_mapping_thread {
            true => {
                let cursor = Cursor::new(ACTIVATED_SOUND);
                let source = Decoder::new(cursor).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            }
            false => {
                let cursor = Cursor::new(DEACTIVATED_SOUND);
                let source = Decoder::new(cursor).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            }
        }
    });
}

// Generate the key mapping
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
    // key_mapper.add_mapping('+', '᛭', '᛭');
    key_mapper.add_mapping('1', '1', '𖤍');
    key_mapper.add_mapping('2', '2', '♅');
    key_mapper.add_mapping('3', '3', '↟');
    key_mapper.add_mapping('4', '4', '↡');
    key_mapper.add_mapping('5', '5', '↠');
    key_mapper.add_mapping('6', '6', '↞');
    key_mapper.add_mapping('7', '7', '𒌐');
    key_mapper.add_mapping('8', '8', '𖤓');
    key_mapper.add_mapping('9', '9', '☽');
    key_mapper.add_mapping('0', '0', '🕈'); // ⴵ
    key_mapper.add_mapping('À', 'ⴵ', '∞'); // ⴵ

    key_mapper
}

// The main function
fn main() {
    println!("{MENU}");

    sound_thread(true);

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
