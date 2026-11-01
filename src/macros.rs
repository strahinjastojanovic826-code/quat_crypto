#[macro_export]
macro_rules! quat_vec {
    ($($q:expr),* $(,)?) => {
        {
            let mut buffer = $crate::QuatBuffer::new();
            $(
                buffer.push($q);
            )*
            buffer
        }
    };
}