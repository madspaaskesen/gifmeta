use std::collections::HashMap;

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

/// Parses a key=value CSV string like `"1=50,2=100"` into a HashMap<usize, u16>.
/// Returns an error if any part of the string fails to parse.
pub fn parse_keyval_csv(input: &str) -> Result<HashMap<usize, u16>, String> {
    let mut map = HashMap::new();
    for part in input.split(',') {
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| format!("Invalid format: '{}'", part))?;
        let key = k
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("Invalid frame index: '{}'", k))?;
        let val = v
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("Invalid delay value: '{}'", v))?;
        map.insert(key, val);
    }
    Ok(map)
}
