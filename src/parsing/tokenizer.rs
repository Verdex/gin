
use crate::array_pattern::*;
use super::data::{Token, TMeta};

pub fn tokenize( input : &str ) -> Result<Vec<Token>, String> {
    Err("TODO".into())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_comment() -> Result<(), MatchError> {
        let input = r#"#this is a comment
                        false
        "#;
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 4 );
        assert_eq!( output[2].start, 43 );
        assert_eq!( output[2].end,  47);

        let name = match &output[2].item {
            InternalToken::Bool(n) => *n,
            _ => panic!("not bool"),
        };

        assert_eq!( name, false );

        Ok(())
    }

    #[test]
    fn should_parse_whitespace() -> Result<(), MatchError> {
        let input = "      \n\t\rfalse";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 2 );
        assert_eq!( output[1].start, 9 );
        assert_eq!( output[1].end,  13);

        let name = match &output[1].item {
            InternalToken::Bool(n) => *n,
            _ => panic!("not bool"),
        };

        assert_eq!( name, false );

        Ok(())
    }

    #[test]
    fn should_parse_string() -> Result<(), MatchError> {
        fn t(input : &str, expected : &str) -> Result<(), MatchError> {
            let output = internal_tokenize(input)?;

            assert_eq!( output.len(), 1 );
            assert_eq!( output[0].start, 0 );
            assert_eq!( output[0].end, input.len() - 1 );

            let value = match &output[0].item {
                InternalToken::String(n) => n.clone(),
                _ => panic!("not string"),
            };

            println!("{}", value);
            assert_eq!( value, expected );
            Ok(())
        }

        t(r#""string input""#, "string input")?;
        t(r#""string \n input""#, "string \n input")?;
        t(r#""string \r input""#, "string \r input")?;
        t(r#""string \0 input""#, "string \0 input")?;
        t(r#""string \t input""#, "string \t input")?;
        t(r#""string \\ input""#, "string \\ input")?;
        t(r#""string \" input""#, "string \" input")?;

        Ok(())
    }

    #[test]
    fn should_parse_numbers() -> Result<(), MatchError> {
        fn t(input : &str, expected : f64) -> Result<(), MatchError> {
            let output = internal_tokenize(input)?;

            assert_eq!( output.len(), 1 );
            assert_eq!( output[0].start, 0 );
            assert_eq!( output[0].end, input.len() - 1 );

            let value = match &output[0].item {
                InternalToken::Number(n) => *n,
                _ => panic!("not number"),
            };

            assert_eq!( value, expected );
            Ok(())
        }

        t("0", 0.0)?;
        t("0.0", 0.0)?;
        t("1E1", 1E1)?;
        t("1e1", 1e1)?;
        t("+1.0", 1.0)?;
        t("-1.0", -1.0)?;
        t("1E+1", 1E+1)?;
        t("1e+1", 1e+1)?;
        t("1234.5678", 1234.5678)?;
        t("1234.5678E-90", 1234.5678E-90)?;
        t("1234.5678e-90", 1234.5678e-90)?;
        t("1234.5678e-901", 1234.5678e-901)?;
        t("1234", 1234.0)?;

        Ok(())
    }

    #[test]
    fn should_parse_boolean_starting_lower_symbol() -> Result<(), MatchError> {
        let input = "false_";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::LowerSymbol(n) => n.clone(),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( name, "false_" );

        Ok(())
    }

    #[test]
    fn should_parse_false() -> Result<(), MatchError> {
        let input = "false";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::Bool(n) => *n,
            _ => panic!("not bool"),
        };

        assert_eq!( name, false );

        Ok(())
    }

    #[test]
    fn should_parse_true() -> Result<(), MatchError> {
        let input = "true";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::Bool(n) => *n,
            _ => panic!("not bool"),
        };

        assert_eq!( name, true );

        Ok(())
    }

    #[test]
    fn should_parse_lower_symbol() -> Result<(), MatchError> {
        let input = "lower_symbol";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::LowerSymbol(n) => n.clone(),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( name, "lower_symbol" );

        Ok(())
    }

    #[test]
    fn should_parse_single_lower_symbol() -> Result<(), MatchError> {
        let input = "l";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::LowerSymbol(n) => n.clone(),
            _ => panic!("not lower symbol"),
        };

        assert_eq!( name, "l" );

        Ok(())
    }

    #[test]
    fn should_parse_upper_symbol() -> Result<(), MatchError> {
        let input = "UpperSymbol";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::UpperSymbol(n) => n.clone(),
            _ => panic!("not upper symbol"),
        };

        assert_eq!( name, "UpperSymbol" );

        Ok(())
    }

    #[test]
    fn should_parse_single_upper_symbol() -> Result<(), MatchError> {
        let input = "U";
        let output = internal_tokenize(input)?;

        assert_eq!( output.len(), 1 );
        assert_eq!( output[0].start, 0 );
        assert_eq!( output[0].end, input.len() - 1 );

        let name = match &output[0].item {
            InternalToken::UpperSymbol(n) => n.clone(),
            _ => panic!("not upper symbol"),
        };

        assert_eq!( name, "U" );

        Ok(())
    }
}
