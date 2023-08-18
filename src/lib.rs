#[cfg(all(feature = "regex_types", feature = "strum"))]
pub mod size_config;
#[cfg(all(feature = "chrono"))]
pub mod chrono_types;

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
