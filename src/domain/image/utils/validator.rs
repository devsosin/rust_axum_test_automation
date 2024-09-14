use regex::Regex;

pub fn validation_image(file_name: &str) -> bool {
    let image_regex = Regex::new(r#"^.*\.(jpg|jpeg|png|webp|avif|svg)$"#).unwrap();
    image_regex.is_match(file_name)
}
