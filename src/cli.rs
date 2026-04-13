use std::num::ParseFloatError;
use std::time::Duration;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = crate::APP_NAME, version, about)]
pub struct Cli {
    #[arg(short, long, default_value = "25m", value_parser = atodur)]
    pub focus: Option<Duration>,

    #[arg(short, long, default_value = "5m", value_parser = atodur)]
    pub short_break: Option<Duration>,

    #[arg(short, long, default_value = "15m", value_parser = atodur)]
    pub long_break: Option<Duration>,

    #[arg(short = 'L', long, default_value = "3", value_parser = |a: &str| a.parse::<u32>())]
    pub long_interval: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum CliArgumentError {
    #[error("This argument cannot be empty")]
    EmptyArgument,

    #[error("Failed parsing argument: {0}")]
    ParseError(String),
}

impl From<ParseFloatError> for CliArgumentError {
    fn from(e: ParseFloatError) -> Self {
        CliArgumentError::ParseError(e.to_string())
    }
}

fn atodur(arg: &str) -> Result<Duration, CliArgumentError> {
    let suffix = arg.chars().last();

    let secs = if let Some(suffix) = suffix {
        let time = &arg[..arg.len() - 1];
        match suffix {
            'h' => time.parse::<f32>()? * 3600 as f32,
            'm' => time.parse::<f32>()? * 60 as f32,
            's' => time.parse::<f32>()?,
            _ => arg.parse::<f32>()?,
        }
    } else {
        return Err(CliArgumentError::EmptyArgument);
    };

    Ok(Duration::from_secs_f32(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opt_valid() {
        let res = atodur("67h").unwrap();
        assert_eq!(res, Duration::from_hours(67));

        let res = atodur("67.5h").unwrap();
        assert_eq!(res, Duration::from_mins((67.5 * 60.0) as u64));

        let res = atodur("67m").unwrap();
        assert_eq!(res, Duration::from_mins(67));

        let res = atodur("67.5m").unwrap();
        assert_eq!(res, Duration::from_secs((67.5 * 60.0) as u64));

        let res = atodur("67s").unwrap();
        assert_eq!(res, Duration::from_secs(67));

        let res = atodur("67.5s").unwrap();
        assert_eq!(res, Duration::from_secs_f32(67.5));

        let res = atodur("67").unwrap();
        assert_eq!(res, Duration::from_secs(67));

        let res = atodur("67.5").unwrap();
        assert_eq!(res, Duration::from_secs_f32(67.5));
    }

    #[test]
    fn test_parse_opt_empty() {
        let res = atodur("");
        assert!(matches!(res, Err(CliArgumentError::EmptyArgument)));
    }

    #[test]
    fn test_parse_opt_invalid() {
        let res = atodur("six seven");
        assert!(matches!(res, Err(CliArgumentError::ParseError(_))));

        let res = atodur("six seven h");
        assert!(matches!(res, Err(CliArgumentError::ParseError(_))));

        let res = atodur("six seven m");
        assert!(matches!(res, Err(CliArgumentError::ParseError(_))));

        let res = atodur("six seven s");
        assert!(matches!(res, Err(CliArgumentError::ParseError(_))));
    }
}
