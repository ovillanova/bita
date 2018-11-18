#[derive(Debug)]
pub struct BaseConfig {
    pub force_create: bool,
}

#[derive(Debug)]
pub struct CompressConfig {
    pub base: BaseConfig,

    // Use stdin if input not given
    pub input: String,
    pub output: String,
    pub temp_file: String,
    pub hash_length: usize,
}
#[derive(Debug)]
pub struct UnpackConfig {
    pub base: BaseConfig,
}

#[derive(Debug)]
pub enum Config {
    Compress(CompressConfig),
    Unpack(UnpackConfig),
}
