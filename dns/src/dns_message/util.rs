pub fn sanitize_name(name: &str) -> Result<String, String> {
    let sanitized_name = name
        .trim()
        .chars()
        .filter(|c| c.is_ascii())
        .collect::<String>();

    if sanitized_name.is_empty() {
        return Err(format!(
            "Name must be valid ASCII string, instead received {name}"
        ));
    }

    Ok(sanitized_name)
}

pub fn name_to_vec8(name: &str) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![b'\0'; name.len() + 2];

    let mut buf_offset = 0;

    for label in name.split('.') {
        buf[buf_offset] = label.len() as u8;
        buf_offset += 1;

        for ch in label.chars() {
            buf[buf_offset] = ch as u8;
            buf_offset += 1;
        }
    }

    buf
}

pub fn is_label_size(b: &u8) -> bool {
    *b <= 63
}

pub fn is_null_byte(b: &u8) -> bool {
    *b == b'\0'
}

pub fn is_pointer(b: &u16) -> bool {
    *b & 0xc000 == 0xc000
}

pub fn get_pointer_offset(b: &u16) -> usize {
    return (*b & 0x3fff) as usize - 12;
}
