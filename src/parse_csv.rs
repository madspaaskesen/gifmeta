/// Parses a comma-separated string into a vector of values.
/// Used for parsing frame numbers or delay values from CLI.
///
/// # Example
/// ```
/// let values = parse_csv::<u16>("10,20,30").unwrap();
/// assert_eq!(values, vec![10, 20, 30]);
/// ```
pub fn parse_csv<T: std::str::FromStr>(input: &str) -> Result<Vec<T>, String> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<T>()
                .map_err(|_| format!("Invalid value in list: '{}'", s))
        })
        .collect()
}
