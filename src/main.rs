use winapi::shared::ntdef::*;
use ntapi::ntobapi::*;
use ntapi::ntrtl::*;

fn main() {
    let dir = std::env::args().skip(1).next().unwrap_or("\\".to_owned());
    let result = enum_directory(&dir);
match result {
    Ok(objects) => {
        for (name, typename) in &objects {
            println!("{name} ({typename})");
        }
        println!("{} objects.", objects.len());
    },
    Err(status) => println!("Error: 0x{status:X}")
};
}

fn enum_directory(dir: &str) -> Result<Vec<(String, String)>, NTSTATUS> {
    let mut items = vec![];

    unsafe {
        let mut udir = UNICODE_STRING::default();
        let wdir = string_to_wstring(&dir);
        RtlInitUnicodeString(&mut udir, wdir.as_ptr());
        let mut dir_attr = OBJECT_ATTRIBUTES::default();
        InitializeObjectAttributes(&mut dir_attr, &mut udir, OBJ_CASE_INSENSITIVE, NULL, NULL);
        let mut hdir: HANDLE = NULL;
        match NtOpenDirectoryObject(&mut hdir, DIRECTORY_QUERY, &mut dir_attr) {
            0 => {
                const LEN: u32 = 1 << 16;
                let mut first = 1;
                let mut buffer: Vec<u8> = Vec::with_capacity(LEN as usize);
                let mut index = 0u32;
                let mut size: u32 = 0;
                loop {
                    let start = index;
                    if NtQueryDirectoryObject(hdir, buffer.as_mut_ptr().cast(), LEN, 0, first, &mut index, &mut size) < 0 {
                        break;
                    }
                    first = 0;
                    let mut obuffer = buffer.as_ptr() as *const OBJECT_DIRECTORY_INFORMATION;
                    for _ in 0..index - start {
                        let item = *obuffer;
                        let name = String::from_utf16_lossy(std::slice::from_raw_parts(item.Name.Buffer, (item.Name.Length / 2) as usize));
                        let typename = String::from_utf16_lossy(std::slice::from_raw_parts(item.TypeName.Buffer, (item.TypeName.Length / 2) as usize));
                        items.push((name, typename));
                        obuffer = obuffer.add(1);
                    }
                }
                Ok(items)
            },
            err => Err(err),
        }
    }
}

fn string_to_wstring(s: &str) -> Vec<u16> {
    let mut wstring: Vec<_> = s.encode_utf16().collect();
    wstring.push(0);    // null terminator
    wstring
}