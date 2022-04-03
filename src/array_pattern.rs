
#[derive(Debug)]
pub enum MatchError {
    Error(usize),
    ErrorEndOfFile,
    Fatal(usize), 
    FatalEndOfFile,
}

macro_rules! alt {
    ($matcher_name:ident<$life:lifetime> : $t_in:ty => $t_out:ty = $($m:ident)|+ => |$name:ident| $b:block) => {
        fn $matcher_name<$life>(input : &mut (impl Iterator<Item = (usize, $t_in)> + Clone)) -> Result<$t_out, MatchError> {

            let mut _error : Option<MatchError> = None;

            $(
                match $m(input) {
                    Ok(v) => { 
                        let $name = v;
                        let ret = $b;
                        return Ok(ret); 
                    },
                    e @ Err(MatchError::Fatal(_)) => { return e; },
                    e @ Err(MatchError::FatalEndOfFile) => { return e; },
                    Err(e @ MatchError::Error(_)) => { _error = Some(e); },
                    Err(e @ MatchError::ErrorEndOfFile) => { _error = Some(e); },
                }

            )*
        
            Err(_error.unwrap())
        }
    };
    ($matcher_name:ident<$life:lifetime> : $t:ty = $($m:ident)|+ => |$name:ident| $b:block) => {
        alt!($matcher_name<$life> : $t => $t = $($m)|+ => |$name| $b);
    };
    ($matcher_name:ident : $t:ty = $($m:ident)|+ => |$name:ident| $b:block) => {
        alt!($matcher_name<'a> : $t => $t = $($m)|+ => |$name| $b);
    };
    ($matcher_name:ident : $t_in:ty => $t_out:ty = $($m:ident)|+ => |$name:ident| $b:block) => {
        alt!($matcher_name<'a> : $t_in => $t_out = $($m)|+ => |$name| $b);
    };
    ($matcher_name:ident<$life:lifetime> : $t_in:ty => $t_out:ty = $($m:ident)|+) => {
        alt!($matcher_name<$life> : $t_in => $t_out = $($m)|+ => |x| { x });
    };
    ($matcher_name:ident<$life:lifetime> : $t:ty = $($m:ident)|+) => {
        alt!($matcher_name<$life> : $t => $t = $($m)|+ => |x| { x });
    };
    ($matcher_name:ident : $t:ty = $($m:ident)|+) => {
        alt!($matcher_name<'a> : $t => $t = $($m)|+ => |x| { x });
    };
    ($matcher_name:ident : $t_in:ty => $t_out:ty = $($m:ident)|+) => {
        alt!($matcher_name<'a> : $t_in => $t_out = $($m)|+ => |x| { x });
    };
}

macro_rules! group { 
    ($matcher_name:ident<$life:lifetime> : $t_in:ty => $t_out:ty = |$input:ident| $b:block) => {
        fn $matcher_name<$life>($input : &mut (impl Iterator<Item = (usize, $t_in)> + Clone)) -> Result<$t_out, MatchError> {
            $b
        }
    };
    ($matcher_name:ident<$life:lifetime> : $t:ty = |$input:ident| $b:block) => {
        group!($matcher_name<$life>: $t => $t = |$input| $b);
    };
    ($matcher_name:ident: $t:ty = |$input:ident| $b:block) => {
        group!($matcher_name<'a>: $t => $t = |$input| $b);
    };
    ($matcher_name:ident: $t_in:ty => $t_out:ty = |$input:ident| $b:block) => {
        group!($matcher_name<'a>: $t_in => $t_out = |$input| $b);
    };
}

macro_rules! pred {
    ($matcher_name:ident<$life:lifetime> : $t_in:ty => $t_out:ty = |$item:ident| $predicate:expr => $b:block) => {
        fn $matcher_name<$life>(input : &mut (impl Iterator<Item = (usize, $t_in)> + Clone)) -> Result<$t_out, MatchError> {

            let mut rp = input.clone();

            let p = |$item:$t_in| $predicate;
            
            match input.next() {
                Some((_, c)) if p(c) => {
                    let $item = c;
                    let ret = $b;
                    Ok(ret)
                },
                Some((i, _)) => { 
                    std::mem::swap(&mut rp, input);
                    Err(MatchError::Error(i))
                },
                None => {
                    std::mem::swap(&mut rp, input);
                    Err(MatchError::ErrorEndOfFile)
                },
            } 
        }
    };
    ($matcher_name:ident : $t_in:ty => $t_out:ty = |$item:ident| $predicate:expr => $b:block) => {
        pred!($matcher_name<'a> : $t_in => $t_out = |$item| $predicate => $b);
    };
    ($matcher_name:ident<$life:lifetime> : $t:ty = |$item:ident| $predicate:expr) => {
        pred!($matcher_name<$life> : $t => $t = |$item| $predicate => { $item });
    };
    ($matcher_name:ident : $t:ty = |$item:ident| $predicate:expr) => {
        pred!($matcher_name<'a> : $t => $t = |$item| $predicate => { $item });
    };
}

macro_rules! seq {
    (err, $rp:ident, $input:ident, $start:ident, $end:ident, $n:ident <= $matcher:ident, $($rest:tt)*) => {
        let v = $matcher($input)?;
        let $n = v.item;
        $start = v.start;
        $end = v.end;
        seq!(fatal, $rp, $input, $start, $end, $($rest)*);
    };

    (fatal, $rp:ident, $input:ident, $start:ident, $end:ident, $n:ident <= $matcher:ident, $($rest:tt)*) => {
        let $n = match $matcher($input) {
            Ok(v) => {
                if $end < v.end {
                    $end = v.end;
                }
                v.item
            },
            Err(MatchError::Fatal(i)) => return Err(MatchError::Fatal(i)),
            Err(MatchError::Error(i)) => return Err(MatchError::Fatal(i)),
            Err(MatchError::FatalEndOfFile) => return Err(MatchError::FatalEndOfFile),
            Err(MatchError::ErrorEndOfFile) => return Err(MatchError::FatalEndOfFile),
        };
        seq!(fatal, $rp, $input, $start, $end, $($rest)*);
    };

    (err, $rp:ident, $input:ident, $start:ident, $end:ident, $n:ident <= $p:pat, $($rest:tt)*) => {
        #[allow(unreachable_patterns)]
        let $n = match $input.next() {
            Some((i, item @ $p)) => {
                $start = i;
                $end = i;
                item
            },
            Some((i, _)) => {
                std::mem::swap(&mut $rp, $input); 
                return Err(MatchError::Error(i)); 
            },
            _ => { 
                std::mem::swap(&mut $rp, $input); 
                return Err(MatchError::ErrorEndOfFile); 
            },
        };
        seq!(fatal, $rp, $input, $start, $end, $($rest)*);
    };

    (fatal, $rp:ident, $input:ident, $start:ident, $end:ident, $n:ident <= $p:pat, $($rest:tt)*) => {
        #[allow(unreachable_patterns)]
        let $n = match $input.next() {
            Some((i, item @ $p)) => {
                $end = i;
                item
            },
            Some((i, _)) => {
                std::mem::swap(&mut $rp, $input); 
                return Err(MatchError::Fatal(i));  
            },
            _ => { 
                std::mem::swap(&mut $rp, $input); 
                return Err(MatchError::FatalEndOfFile);  
            },
        };
        seq!(fatal, $rp, $input, $start, $end, $($rest)*);
    };

    ($mode:ident, $rp:ident, $input:ident, $start:ident, $end:ident, $b:block) => {
        let item = $b;
        return Ok( Success { start: $start, end: $end, item: item } );
    };

    ($matcher_name:ident<$life:lifetime> : $in_t:ty => $out_t:ty = $($rest:tt)*) => {
        fn $matcher_name<$life>(input : &mut (impl Iterator<Item = (usize, $in_t)> + Clone)) -> Result<Success<$out_t>, MatchError> {
            let mut _rp = input.clone();
            let mut _start : usize = 0;
            let mut _end : usize = 0;
            seq!(err, _rp, input, _start, _end, $($rest)*);
        }
    };

    (zero_or_more ~ $matcher_name:ident<$life:lifetime> : $in_t:ty => $out_t:ty = $($rest:tt)*) => {
        fn $matcher_name<$life>(input : &mut (impl Iterator<Item = (usize, $in_t)> + Clone)) -> Result<Success<Vec<$out_t>>, MatchError> {

            fn matcher<$life>(input : &mut (impl Iterator<Item = (usize, $in_t)> + Clone)) -> Result<Success<$out_t>, MatchError> {
                let mut _rp = input.clone();
                let mut _start : usize = 0;
                let mut _end : usize = 0;
                seq!(err, _rp, input, _start, _end, $($rest)*);
            }

            let mut ret = vec![];

            let mut result = matcher(input);
            let mut _start = 0;
            let mut _end = 0;
            match result {
                Ok(s) => { 
                    _start = s.start;
                    _end = s.end;
                    ret.push(s.item);
                },
                Err(MatchError::Error(i)) => { return Ok(Success{ item: ret, start: i, end: i }); },
                Err(MatchError::ErrorEndOfFile) => { return Ok(Success{ item: ret, start: 0, end: 0 }); },
                Err(MatchError::Fatal(i)) => { return Err(MatchError::Fatal(i)); },
                Err(MatchError::FatalEndOfFile) => { return Err(MatchError::FatalEndOfFile); },
            }

            loop {
                result = matcher(input);
                match result {
                    Ok(s) => { 
                        _end = s.end;
                        ret.push(s.item);
                    },
                    Err(MatchError::Error(_)) => { break; },
                    Err(MatchError::ErrorEndOfFile) => { break; },
                    Err(MatchError::Fatal(i)) => { return Err(MatchError::Fatal(i)); },
                    Err(MatchError::FatalEndOfFile) => { return Err(MatchError::FatalEndOfFile); },
                }
            }

            Ok(Success{ item: ret, start: _start, end: _end })
        }
    };

    (maybe ~ $matcher_name:ident<$life:lifetime> : $in_t:ty => $out_t:ty = $($rest:tt)*) => {
        fn $matcher_name<$life>(input : &mut (impl Iterator<Item = (usize, $in_t)> + Clone)) -> Result<Success<Option<$out_t>>, MatchError> {
            let mut _rp = input.clone();
            let mut _start : usize = 0;
            let mut _end : usize = 0;
            let mut matcher = || { seq!(err, _rp, input, _start, _end, $($rest)*); };
            let result = matcher();
            match result {
                Ok(Success{ item, start, end }) => Ok(Success{ item: Some(item), start, end }),
                Err(MatchError::Error(i)) => Ok(Success{ item: None, start: i, end: i }),
                Err(MatchError::ErrorEndOfFile) => Ok(Success{ item: None, start: 0, end: 0 }),
                Err(MatchError::Fatal(i)) => Err(MatchError::Fatal(i)),
                Err(MatchError::FatalEndOfFile) => Err(MatchError::FatalEndOfFile),
            }
        }
    };
}

#[cfg(test)]
mod test { 
    use super::*;

    #[test]
    fn alt_should_match() -> Result<(), MatchError> {
        pred!(even : u8 = |x| x % 2 == 0);
        pred!(odd : u8 = |x| x % 2 == 1);
        alt!(even_or_odd : u8 = even | odd);

        let v : Vec<u8> = vec![3, 3];
        let mut i = v.into_iter().enumerate();

        let o = even_or_odd(&mut i)?;

        assert_eq!( o, 3 );

        Ok(())
    }

    #[test]
    fn alt_should_not_match() {
        pred!(even : u8 = |x| x % 2 == 0);
        pred!(five : u8 = |x| x == 5);
        alt!(even_or_five : u8 = even | five);

        let v : Vec<u8> = vec![3, 3];
        let mut i = v.into_iter().enumerate();

        let o = even_or_five(&mut i);

        assert!( matches!(o, Err(MatchError::Error(_))) );
    }

    #[test]
    fn alt_should_handle_lifetime() -> Result<(), MatchError> {
        struct Input(u8);
        
        pred!(even<'a> : &'a Input = |x| x.0 % 2 == 0);
        pred!(odd<'a> : &'a Input = |x| x.0 % 2 == 1);
        alt!(even_or_odd<'a> : &'a Input = even | odd);

        let v : Vec<Input> = vec![Input(3), Input(3)];
        let mut i = v.iter().enumerate();

        let o = even_or_odd(&mut i)?;

        assert_eq!( o.0, 3 );

        Ok(())
    }

    #[test]
    fn alt_should_handle_different_output_type() -> Result<(), MatchError> {
        struct Output(u8);
        
        pred!(even : u8 => Output = |x| x % 2 == 0 => { Output(x) });
        pred!(odd : u8 => Output = |x| x % 2 == 1 => { Output(x) });
        alt!(even_or_odd : u8 => Output = even | odd);

        let v : Vec<u8> = vec![3, 3];
        let mut i = v.into_iter().enumerate();

        let o = even_or_odd(&mut i)?;

        assert_eq!( o.0, 3 );

        Ok(())
    }
    
    #[test]
    fn alt_should_handle_different_output_type_with_lifetime() -> Result<(), MatchError> {
        struct Input(u8);
        struct Output<'a>(&'a Input);
        
        pred!(even<'a> : &'a Input => Output<'a> = |x| x.0 % 2 == 0 => { Output(x) });
        pred!(odd<'a> : &'a Input => Output<'a> = |x| x.0 % 2 == 1 => { Output(x) });
        alt!(even_or_odd<'a> : &'a Input => Output<'a> = even | odd);

        let v : Vec<Input> = vec![Input(3), Input(3)];
        let mut i = v.iter().enumerate();

        let o = even_or_odd(&mut i)?;

        assert_eq!( o.0.0, 3 );

        Ok(())
    }

    #[test]
    fn pred_should_match() -> Result<(), MatchError> {
        pred!(even : u8 = |x| x % 2 == 0);

        let v : Vec<u8> = vec![2, 3];
        let mut i = v.into_iter().enumerate();

        let o = even(&mut i)?;

        assert_eq!( o, 2 );

        Ok(())
    }

    #[test]
    fn pred_should_not_match() {
        pred!(even : u8 = |x| x % 2 == 0);

        let v : Vec<u8> = vec![3, 2];
        let mut i = v.into_iter().enumerate();

        let o = even(&mut i);

        assert!( matches!( o, Err(MatchError::Error(_)) ) );
    }

    #[test]
    fn pred_should_handle_lifetime() -> Result<(), MatchError> {
        struct Input(u8);
        
        pred!(even<'a> : &'a Input = |x| x.0 % 2 == 0);

        let v : Vec<Input> = vec![Input(2), Input(3)];
        let mut i = v.iter().enumerate();

        let o = even(&mut i)?;

        assert_eq!( o.0, 2 );

        Ok(())
    }

    #[test]
    fn pred_should_handle_output_block() -> Result<(), MatchError> {
        struct Output(u8);

        pred!(even : u8 => Output = |x| x % 2 == 0 => { Output(x + 1) });

        let v : Vec<u8> = vec![2, 3];
        let mut i = v.into_iter().enumerate();

        let o = even(&mut i)?;

        assert_eq!( o.0, 3 );

        Ok(())
    }

    #[test]
    fn pred_should_handle_output_block_with_lifetime() -> Result<(), MatchError> {
        struct Input(u8);
        struct Output<'a>(&'a Input);

        pred!(even<'a> : &'a Input => Output<'a> = |x| x.0 % 2 == 0 => { Output(x) });

        let v : Vec<Input> = vec![Input(2), Input(3)];
        let mut i = v.iter().enumerate();

        let o = even(&mut i)?;

        assert_eq!( o.0.0, 2 );

        Ok(())
    }
}