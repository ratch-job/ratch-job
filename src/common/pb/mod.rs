// Automatically generated mod.rs
pub mod data_object;

#[cfg(test)]
mod tests {
    use crate::common::pb::data_object::JobDo;
    use quick_protobuf::BytesReader;

    #[test]
    fn data_to_job_do() {
        let data: Vec<u8> = vec![
            97, 10, 23, 120, 120, 108, 45, 106, 111, 98, 45, 101, 120, 101, 99, 117, 116, 111, 114,
            45, 115, 97, 109, 112, 108, 101, 18, 3, 120, 120, 108, 26, 23, 120, 120, 108, 45, 106,
            111, 98, 45, 101, 120, 101, 99, 117, 116, 111, 114, 45, 115, 97, 109, 112, 108, 101,
            34, 4, 65, 85, 84, 79, 50, 34, 10, 26, 104, 116, 116, 112, 58, 47, 47, 49, 57, 50, 46,
            49, 54, 56, 46, 50, 46, 49, 51, 51, 58, 57, 57, 57, 57, 47, 16, 246, 145, 128, 191, 6,
        ];
        let mut reader = BytesReader::from_bytes(&data);
        match reader.read_message::<JobDo>(&data) {
            Ok(job_do) => {
                println!("job_do:{:?}", job_do);
            }
            Err(e) => {
                println!("err:{:?}", e);
            }
        }
    }
}
