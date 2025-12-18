/// Helper para imprimir número hexadecimal
fn print_hex(console: &mut drivers::video::Console, mut n: usize) {
    if n == 0 {
        console.write_str("0");
        drivers::legacy::serial::print("0");
        return;
    }

    let mut buffer = [0u8; 16];
    let mut i = 0;

    while n > 0 {
        let digit = n % 16;
        buffer[i] = if digit < 10 {
            b'0' + digit as u8
        } else {
            b'a' + (digit - 10) as u8
        };
        n /= 16;
        i += 1;
    }

    // Reverter e imprimir
    while i > 0 {
        i -= 1;
        let c = buffer[i] as char;
        console.write_char(c);
        drivers::legacy::serial::write_byte(buffer[i]);
    }
}
