pub fn resp_encode_array(list: &[String], write_buffer: &mut String) {
    write_buffer.push_str(&format!("*{}\r\n", list.len()));
    for ele in list {
        write_buffer.push_str(&format!("${}\r\n", ele.len()));
        write_buffer.push_str(&format!("{}\r\n", ele));
    }
}
