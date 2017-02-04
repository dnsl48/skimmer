use reader::Read;

use symbol::Symbol;
use symbol::Word;




pub const NO_STOPS:   [(Stop, Word); 0] = [];
pub const NO_ESCAPES: [Word; 0] = [];
pub const NO_QUOTES:  [(Quote, Word); 0] = [];
pub const NO_BRACES:  [(Brace, (Word, Word)); 0] = [];




pub struct Stop {
    pub quoted: bool,
    pub escaped: bool,
    pub greedy: bool,
    pub skip: bool
}



impl Stop {
    pub fn new () -> Stop {
        Stop { quoted: true, escaped: true, greedy: true, skip: true }
    }


    pub fn skip (mut self, skip: bool) -> Stop {
        self.skip = skip;
        self
    }


    pub fn greedy (mut self, greedy: bool) -> Stop {
        self.greedy = greedy;
        self
    }


    pub fn escaped (mut self, flag: bool) -> Stop {
        self.escaped = flag;
        self
    }


    pub fn quoted (mut self, flag: bool) -> Stop {
        self.quoted = flag;
        self
    }
}




pub struct Brace {
    pub escapes_stop: bool,
    pub quoted: bool,
    pub escaped: bool,
    pub is_stop: bool
}



impl Brace {
    pub fn new () -> Brace {
        Brace { escaped: true, quoted: true, is_stop: true, escapes_stop: true }
    }


    pub fn is_stop (mut self, flag: bool) -> Brace {
        self.is_stop = flag;
        self
    }


    pub fn escapes_stop (mut self, flag: bool) -> Brace {
        self.escapes_stop = flag;
        self
    }


    pub fn escaped (mut self, flag: bool) -> Brace {
        self.escaped = flag;
        self
    }


    pub fn quoted (mut self, flag: bool) -> Brace {
        self.quoted = flag;
        self
    }
}




pub struct Quote {
    pub escapes_stop: bool,
    pub escaped: bool,
    pub greedy: bool,
    pub is_stop: bool
}



impl Quote {
    pub fn new () -> Quote {
        Quote { escapes_stop: true, escaped: true, greedy: true, is_stop: true }
    }


    pub fn is_stop (mut self, flag: bool) -> Quote {
        self.is_stop = flag;
        self
    }


    pub fn escapes_stop (mut self, flag: bool) -> Quote {
        self.escapes_stop = flag;
        self
    }


    pub fn escaped (mut self, flag: bool) -> Quote {
        self.escaped = flag;
        self
    }


    pub fn greedy (mut self, flag: bool) -> Quote {
        self.greedy = flag;
        self
    }
}




pub fn skip_until<R: Read, S: Symbol> (reader: &mut R, symbols: &[S]) -> (usize, Option<(usize, usize)>) {
    let mut skipped = 0usize;

    loop {
        if !reader.has (1) { break; }

        for (idx, symbol) in symbols.iter ().enumerate () {
            if let Some (len) = symbol.read (reader) { return (skipped, Some ((idx, len))); }
        }

        skipped += reader.skip (1);
    }

    (skipped, None)
}




pub fn skip_while<R: Read, S: Symbol> (reader: &mut R, symbols: &[S]) -> (usize, usize) {
    let mut skipped: usize = 0;
    let mut chars: usize = 0;

    loop {
        let mut found = false;

        for symbol in symbols {
            if let Some (len) = symbol.read (reader) {
                skipped += reader.skip (len);
                chars += symbol.len_chars ();
                found = true;
                break;
            }
        }

        if found { continue; }

        break;
    }

    (skipped, chars)
}



pub fn scan_until<R: Read, S: Symbol> (reader: &mut R, symbols: &[S]) -> (usize, Option<(usize, usize)>) {
    scan_until_at (0, reader, symbols)
}


pub fn scan_until_at<R: Read, S: Symbol> (at: usize, reader: &mut R, symbols: &[S]) -> (usize, Option<(usize, usize)>) {
    let mut scanned = at;

    loop {
        if !reader.has (scanned + 1) { break; }

        for (idx, symbol) in symbols.iter ().enumerate () {
            if let Some (len) = symbol.read_at (scanned, reader) { return (scanned - at, Some ((idx, len))); }
        }

        scanned += 1;
    }

    (scanned - at, None)
}



pub fn scan_while<R: Read, S: Symbol> (reader: &mut R, symbols: &[S]) -> (usize, usize) {
    scan_while_at (0, reader, symbols)
}


pub fn scan_while_at<R: Read, S: Symbol> (at: usize, reader: &mut R, symbols: &[S]) -> (usize, usize) {
    let mut scanned: usize = at;
    let mut chars: usize = 0;

    loop {
        let mut found = false;

        for symbol in symbols {
            if let Some (len) = symbol.read_at (scanned, reader) {
                scanned += len;
                chars += symbol.len_chars ();
                found = true;
                break;
            }
        }

        if found { continue; }

        break;
    }

    (scanned - at, chars)
}



pub fn scan_one<R: Read, S: Symbol> (reader: &mut R, symbols: &[S]) -> Option<(usize, usize)> { scan_one_at (0, reader, symbols) }



pub fn scan_one_at<R: Read, S: Symbol> (at: usize, reader: &mut R, symbols: &[S]) -> Option<(usize, usize)> {
    for idx in 0 .. symbols.len () {
        if let Some (len) = symbols[idx].read_at (at, reader) {
            return Some ( (idx, len) )
        }
    }

    None
}




pub fn scan<Reader, StopSymbol, EscapeSymbol, QuoteSymbol, BraceOpenSymbol, BraceCloseSymbol> (
    reader: &mut Reader,
    stops: &[(Stop, StopSymbol)],
    escapes: &[EscapeSymbol],
    quotes: &[(Quote, QuoteSymbol)],
    braces: &[(Brace, (BraceOpenSymbol, BraceCloseSymbol))],
    braces_counters: &mut [usize]
) -> Option<(usize, usize)>
    where Reader: Read,
          StopSymbol: Symbol,
          EscapeSymbol: Symbol,
          QuoteSymbol: Symbol,
          BraceOpenSymbol: Symbol,
          BraceCloseSymbol: Symbol
{
    if braces_counters.len () != braces.len () { panic! ("Not equal amount of counters({}) to passed braces({})", braces_counters.len (), braces.len ()) }

    let mut quote_flag = false;
    let mut quote_index = 0usize;

    let mut escape_flag = false;
    let mut escape_loop = 0;

    let mut read_pos = 0usize;

    let mut stop_idx = 0usize;
    let mut stop_len = 0usize;

    let mut greedy_mode = false;

    for idx in 0 .. braces_counters.len () { braces_counters[idx] = 0; }

    loop {
        if !reader.has (read_pos + 1) { break; }

        if escape_flag && escape_loop == 0 {
            escape_loop += 1;
        } else {
            escape_flag = false;
            escape_loop = 0;
        }

        if let Some ( (idx, len) ) = _look_for_stop (reader, stops, read_pos) {
            read_pos += len;

            if escape_flag && stops[idx].0.escaped {
                escape_flag = false;
                escape_loop = 0;
                continue;
            }

            if quote_flag && stops[idx].0.quoted && quotes[quote_index].0.escapes_stop {
                continue;
            }

            let mut go_on = false;
            for bidx in 0 .. braces_counters.len () {
                if braces_counters[bidx] > 0 && braces[bidx].0.escapes_stop {
                    go_on = true;
                    break;
                }
            }
            if go_on {
                continue;
            }

            stop_idx = idx;
            stop_len += len;

            if stops[idx].0.greedy {
                greedy_mode = true;
                continue;
            }

            break;
        }


        if greedy_mode { break; }
        stop_len = 0;


        if let Some ( (_, len) ) = _look_for_symbols (reader, escapes, read_pos) {
            read_pos += len;
            escape_flag = !escape_flag;
            escape_loop = 0;
            continue;
        }


        if let Some ( (idx, len) ) = _look_for_quote (reader, quotes, read_pos) {
            read_pos += len;

            if escape_flag && quotes[idx].0.escaped {
                escape_flag = false;
                escape_loop = 0;

            } else if !quote_flag {
                quote_flag = true;
                quote_index = idx;

            } else if quote_index == idx {
                quote_flag = false;

                if quotes[idx].0.is_stop {
                    let mut go_on = false;
                    for bidx in 0 .. braces_counters.len () {
                        if braces_counters[bidx] > 0 && braces[bidx].0.escapes_stop {
                            go_on = true;
                            break;
                        }
                    }
                    if go_on { continue; }

                    if quotes[idx].0.greedy {
                        if let Some (len) = quotes[idx].1.read_at (read_pos, reader) {
                            read_pos += len;

                            quote_flag = true;
                            quote_index = idx;

                            continue;
                        }
                    }

                    break;
                }
            }

            continue;
        }


        if let Some ( (idx, is_open_found, len) ) = _look_for_braces (reader, braces, read_pos) {
            read_pos += len;

            if escape_flag && braces[idx].0.escaped {
                escape_flag = false;
                escape_loop = 0;

            } else if quote_flag && braces[idx].0.quoted {
                // nothing happens then
            } else if is_open_found {
                braces_counters[idx] += 1;
            } else {
                if braces_counters[idx] > 0 {
                    braces_counters[idx] -= 1;
                }

                if braces[idx].0.is_stop {
                    let mut go_on = false;
                    for bidx in 0 .. braces_counters.len () {
                        if braces_counters[bidx] > 0 && braces[bidx].0.escapes_stop {
                            go_on = true;
                            break;
                        }
                    }
                    if !go_on { break; }
                }
            }

            continue;
        }

        read_pos += 1;
    }


    let result_length = if stop_len > 0 {
        if stops[stop_idx].0.skip { read_pos - stop_len } else { read_pos }
    } else { read_pos };


    Some ((result_length, read_pos))
}




fn _look_for_stop<R: Read, S: Symbol> (reader: &mut R, stops: &[(Stop, S)], offset: usize) -> Option<(usize, usize)> {
    for (idx, &(_, ref stop)) in stops.iter ().enumerate () {
        if let Some (len) = stop.read_at (offset, reader) { return Some ( (idx, len) ); };
    }

    None
}




fn _look_for_quote<R: Read, S: Symbol> (reader: &mut R, quotes: &[(Quote, S)], offset: usize) -> Option<(usize, usize)> {
    for (idx, &(_, ref quote)) in quotes.iter ().enumerate () {
        if let Some (len) = quote.read_at (offset, reader) { return Some ( (idx, len) ); };
    }

    None
}




fn _look_for_braces<R: Read, OS: Symbol, CS: Symbol> (reader: &mut R, braces: &[(Brace, (OS, CS))], offset: usize) -> Option<(usize, bool, usize)> {
    for (idx, &(_, (ref open, ref close))) in braces.iter ().enumerate () {
        if let Some (len) = open.read_at (offset, reader) { return Some ( (idx, true, len) ); };

        if let Some (len) = close.read_at (offset, reader) { return Some ( (idx, false, len) ); };
    }

    None
}




fn _look_for_symbols<R, S> (reader: &mut R, symbols: &[S], offset: usize) -> Option<(usize, usize)>
    where R: Read,
          S: Symbol
{
    for (idx, ref symbol) in symbols.iter ().enumerate () {
        if let Some (len) = symbol.read_at (offset, reader) { return Some ( (idx, len) ) }
    }

    None
}





#[cfg(test)]
mod tests {
    use super::*;

    use reader::Read;
    use reader::SliceReader;
    use symbol::Char;
    // use symbol::Word;


    #[test]
    fn test_stop () {
        let stop = Stop::new ().skip (false).greedy (false).escaped (false).quoted (false);

        assert_eq! (false, stop.skip);
        assert_eq! (false, stop.greedy);
        assert_eq! (false, stop.escaped);
        assert_eq! (false, stop.quoted);


        let stop = Stop::new ().skip (true).greedy (true).escaped (true).quoted (true);

        assert! (stop.skip);
        assert! (stop.greedy);
        assert! (stop.escaped);
        assert! (stop.quoted);
    }



    #[test]
    fn test_brace () {
        let brace = Brace::new ()
            .is_stop (false)
            .escaped (false)
            .quoted (false)
            .escapes_stop (false);

        assert_eq! (false, brace.is_stop);
        assert_eq! (false, brace.escaped);
        assert_eq! (false, brace.quoted);
        assert_eq! (false, brace.escapes_stop);


        let brace = Brace::new ()
            .is_stop (true)
            .escaped (true)
            .quoted (true)
            .escapes_stop (true);

        assert! (brace.is_stop);
        assert! (brace.escaped);
        assert! (brace.quoted);
        assert! (brace.escapes_stop);
    }



    #[test]
    fn test_quote () {
        let quote = Quote::new ()
            .is_stop (false)
            .escaped (false)
            .greedy (false)
            .escapes_stop (false);

        assert_eq! (false, quote.is_stop);
        assert_eq! (false, quote.escaped);
        assert_eq! (false, quote.greedy);
        assert_eq! (false, quote.escapes_stop);


        let quote = Quote::new ()
            .is_stop (true)
            .escaped (true)
            .greedy (true)
            .escapes_stop (true);

        assert! (quote.is_stop);
        assert! (quote.escaped);
        assert! (quote.greedy);
        assert! (quote.escapes_stop);
    }



    #[test]
    fn test_scan_macro () {
        let string = r"Lorem( ipsum\) dolor') sit' amet) consectetur";

        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];
        let escapes = [Char::new (r"\".as_bytes ()).to_word ()];
        let quotes = [(Quote::new (), Char::new ("'".as_bytes ()).to_word ())];
        let braces = [(Brace::new ().is_stop (false), (Char::new ("(".as_bytes ()).to_word (), Char::new (")".as_bytes ()).to_word ()))];


        let mut reader = SliceReader::new (string.as_bytes ());

        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &quotes, &braces, &mut [0]) {
            assert_eq! (res, r"Lorem( ipsum\) dolor') sit' amet)".len ());
            assert_eq! (pos, r"Lorem( ipsum\) dolor') sit' amet) ".len ());
        } else { assert! (false, "Cannot parse the string"); }
    }



    #[test]
    fn test_scan_shallow_scrap () {
        let string = r"Lorem( ipsum\) dolor') sit' amet) consectetur";
        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];
        let escapes = [Char::new (r"\".as_bytes ()).to_word ()];
        let quotes = [(Quote::new (), Char::new ("'".as_bytes ()).to_word ())];
        let brace = [(Brace::new ().is_stop (false), (Char::new ("(".as_bytes ()).to_word (), Char::new (")".as_bytes ()).to_word ()))];

        let mut reader = SliceReader::new (string.as_bytes ());
        if let Some ((res, pos)) = scan (&mut reader, &stops, &escapes, &quotes, &brace, &mut [0]) {
            assert_eq! (res, r"Lorem( ipsum\) dolor') sit' amet)".len ());
            assert_eq! (pos, r"Lorem( ipsum\) dolor') sit' amet) ".len ());
        } else { assert! (false, "Cannot parse the string"); }
    }



    #[test]
    fn test_scan_stop () {
        let string = "one test  two";
        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];

        let mut reader = SliceReader::new (string.as_bytes ());

        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (res, 3);
            assert_eq! (pos, 4);

            assert_eq! (&reader.consume (pos)[..], "one ".as_bytes ());
        } else { assert! (false, "Cannot find the stop while scanning"); }


        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (4, res);
            assert_eq! (6, pos);

            assert_eq! (&reader.consume (pos)[..], "test  ".as_bytes ());
        } else { assert! (false, "Cannot find the stop while scanning"); }


        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (3, res);
            assert_eq! (3, pos);

            assert_eq! (&reader.consume (pos)[..], "two".as_bytes ());
        } else { assert! (false, "Cannot finish by EOF while scanning"); }
    }



    #[test]
    fn test_scan_escape () {
        let string = r"Lorem\ ip\sum  dolor";
        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];
        let escapes = [Char::new (r"\".as_bytes ()).to_word ()];


        let mut reader = SliceReader::new (string.as_bytes ());

        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (res, r"Lorem\ ip\sum".len ());
            assert_eq! (pos, r"Lorem\ ip\sum  ".len ());
        } else { assert! (false, "Cannot parse the string"); }


        let escapes = [Char::new ("<!-- ".as_bytes ()).to_word ()];
        let string = "Lorem<!--  ipsum<!-- <!--   dolor";

        let mut reader = SliceReader::new (string.as_bytes ());

        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (res, "Lorem<!--  ipsum<!-- <!-- ".len ());
            assert_eq! (pos, "Lorem<!--  ipsum<!-- <!--   ".len ());
        } else { assert! (false, "Cannot parse the string"); }


        let string = "Lorem<!-ipsum<!--  dolor";
        let escapes = [Char::new ("<!--".as_bytes ()).to_word ()];

        let mut reader = SliceReader::new (string.as_bytes ());

        if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &NO_QUOTES, &NO_BRACES, &mut []) {
            assert_eq! (res, "Lorem<!-ipsum<!-- ".len ());
            assert_eq! (pos, "Lorem<!-ipsum<!--  ".len ());
        } else { assert! (false, "Cannot parse the string"); }
    }



    #[test]
    fn test_scan_quote () {
        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];

        {
            let string = "Lorem' ipsum'  dolor";
            let quotes = [(Quote::new ().is_stop (false), Char::new ("'".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());

            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, "Lorem' ipsum'".len ());
                assert_eq! (pos, "Lorem' ipsum'  ".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }


        {
            let string = "Lorem' ipsum'  dolor";
            let quotes = [(Quote::new ().is_stop (true), Char::new ("'".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());

            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, "Lorem' ipsum'".len ());
                assert_eq! (pos, "Lorem' ipsum'".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }

        {
            let string = "Lorem' ipsum''s test'  dolor";
            let quotes = [(Quote::new ().is_stop (true).greedy (true), Char::new ("'".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());

            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, "Lorem' ipsum''s test'".len ());
                assert_eq! (pos, "Lorem' ipsum''s test'".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }


        {
            let string = "Lorem' ipsum''s test'  dolor";
            let quotes = [(Quote::new ().is_stop (false).greedy (true), Char::new ("'".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());

            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, "Lorem' ipsum''s test'".len ());
                assert_eq! (pos, "Lorem' ipsum''s test'  ".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }


        {
            let string = r"Lorem-- ip\--sum--  dolor";
            let escapes = [Char::new (r"\".as_bytes ()).to_word ()];
            let quotes = [(Quote::new ().is_stop (false), Char::new ("--".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());
            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, r"Lorem-- ip\--sum--".len ());
                assert_eq! (pos, r"Lorem-- ip\--sum--  ".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }


        {
            let string = r"Lorem\-- ipsum--  dolor";
            let escapes = [Char::new (r"\".as_bytes ()).to_word ()];
            let quotes = [(Quote::new (), Char::new ("--".as_bytes ()).to_word ())];

            let mut reader = SliceReader::new (string.as_bytes ());
            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &escapes, &quotes, &NO_BRACES, &mut []) {
                assert_eq! (res, r"Lorem\--".len ());
                assert_eq! (pos, r"Lorem\-- ".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }
    }



    #[test]
    fn test_scan_brace () {
        let stops = [(Stop::new (), Char::new (" ".as_bytes ()).to_word ())];

        {
            let string = "Lorem<!-- ipsum dolor <!-- sit --> amet --> consectetur";
            let braces = [(Brace::new (), (Char::new ("<!--".as_bytes ()).to_word (), Char::new ("-->".as_bytes ()).to_word ()))];

            let mut reader = SliceReader::new (string.as_bytes ());
            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [0]) {
                assert_eq! (res, "Lorem<!-- ipsum dolor <!-- sit --> amet -->".len ());
                assert_eq! (pos, "Lorem<!-- ipsum dolor <!-- sit --> amet -->".len ());
            } else { assert! (false, "Cannot parse the string"); }


            let braces = [(Brace::new ().is_stop (false), (Char::new ("<!--".as_bytes ()).to_word (), Char::new ("-->".as_bytes ()).to_word ()))];

            let mut reader = SliceReader::new (string.as_bytes ());
            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [0]) {
                assert_eq! (res, "Lorem<!-- ipsum dolor <!-- sit --> amet -->".len ());
                assert_eq! (pos, "Lorem<!-- ipsum dolor <!-- sit --> amet --> ".len ());
            } else { assert! (false, "Cannot parse the string"); }
        }


        {
            let string = "(Lorem)[ipsum](dolor[sit]amet)(consectertur[adipisicing)(elit(sed])do)";
            let braces = [
                (Brace::new (), (Char::new ("(".as_bytes ()).to_word (), Char::new (")".as_bytes ()).to_word ())),
                (Brace::new (), (Char::new ("[".as_bytes ()).to_word (), Char::new ("]".as_bytes ()).to_word ()))
            ];

            let mut reader = SliceReader::new (string.as_bytes ());
            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [0; 2]) {
                assert_eq! (res, "(Lorem)".len ());
                assert_eq! (pos, res);

                assert_eq! (&reader.consume (pos)[..], "(Lorem)".as_bytes ());
            } else { assert! (false, "Cannot parse the string"); }


            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [1; 2]) {
                assert_eq! (res, "[ipsum]".len ());
                assert_eq! (pos, res);

                assert_eq! (&reader.consume (pos)[..], "[ipsum]".as_bytes ());
            } else { assert! (false, "Cannot parse the string"); }


            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [2; 2]) {
                assert_eq! (res, "(dolor[sit]amet)".len ());
                assert_eq! (pos, res);

                assert_eq! (&reader.consume (pos)[..], "(dolor[sit]amet)".as_bytes ());
            } else { assert! (false, "Cannot parse the string"); }


            if let Some ( (res, pos) ) = scan (&mut reader, &stops, &NO_ESCAPES, &NO_QUOTES, &braces, &mut [3; 2]) {
                assert_eq! (res, "(consectertur[adipisicing)(elit(sed])do)".len ());
                assert_eq! (pos, res);

                assert_eq! (&reader.consume (pos)[..], "(consectertur[adipisicing)(elit(sed])do)".as_bytes ());
            } else { assert! (false, "Cannot parse the string"); }
        }
    }



    #[test]
    fn test_skip_until () {
        let src = r"Lorem( ipsum\) dolor') sit' amet) consectetur";

        let mut reader = SliceReader::new (src.as_bytes ());

        let (skipped, stopper) = skip_until (&mut reader, &[
            Char::new ("test".as_bytes ()).to_word (),
            Char::new ("(".as_bytes ()).to_word ()
        ]);

        assert_eq! (skipped, r"Lorem".len ());

        if let Some ( (stopper, len ) ) = stopper {
            assert_eq! (stopper, 1);
            assert_eq! (len, 1);
        } else { assert! (false, "Unexpected result!") }


        if let Some (bytes) = reader.slice (9) {
            assert_eq! (bytes, r"( ipsum\)".as_bytes ());
        } else { assert! (false, "Unexpected result!") }
    }



    #[test]
    fn test_skip_while () {
        let src = "    \t\t\t\tLorem";

        let mut reader = SliceReader::new (src.as_bytes ());

        let (skipped, chars) = skip_while (&mut reader, &[
            Char::new (" ".as_bytes ()).to_word (),
            Char::new ("\t".as_bytes ()).to_word ()
        ]);

        assert_eq! (skipped, "    \t\t\t\t".len ());
        assert_eq! (chars, 8);


        if let Some (bytes) = reader.slice (5) {
            assert_eq! (bytes, "Lorem".as_bytes ());
        } else { assert! (false, "Unexpected result!") }
    }


    #[test]
    fn test_scan_until () {
        let src = r"Lorem( ipsum\) dolor') sit' amet) consectetur";

        let mut reader = SliceReader::new (src.as_bytes ());

        let (scanned, stopper) = scan_until (&mut reader, &[
            Char::new ("test".as_bytes ()).to_word (),
            Char::new ("(".as_bytes ()).to_word ()
        ]);

        assert_eq! (scanned, r"Lorem".len ());

        if let Some ( (stopper, len ) ) = stopper {
            assert_eq! (stopper, 1);
            assert_eq! (len, 1);
        } else { assert! (false, "Unexpected result!") }


        if let Some (bytes) = reader.slice (5) {
            assert_eq! (bytes, "Lorem".as_bytes ());
        } else { assert! (false, "Unexpected result!") }
    }


    #[test]
    fn test_scan_while () {
        let src = "    \t\t\t\tLorem";

        let mut reader = SliceReader::new (src.as_bytes ());

        let (scanned, chars) = scan_while (&mut reader, &[
            Char::new (" ".as_bytes ()).to_word (),
            Char::new ("\t".as_bytes ()).to_word ()
        ]);

        assert_eq! (scanned, "    \t\t\t\t".len ());
        assert_eq! (chars, 8);

        if let Some (bytes) = reader.slice (4) {
            assert_eq! (bytes, "    ".as_bytes ());
        } else { assert! (false, "Unexpected result!") }
    }


    #[test]
    fn test_scan_one () {
        let src = "    \t\t\t\tLorem";

        let mut reader = SliceReader::new (src.as_bytes ());

        if let Some ( (idx, len) ) = scan_one (&mut reader, &[
            Char::new (" ".as_bytes ()).to_word (),
            Char::new ("\t".as_bytes ()).to_word ()
        ]) {
            assert_eq! (idx, 0);
            assert_eq! (len, 1);
        } else { assert! (false, "Unexpected result!") }

        if let Some ( _ ) = scan_one (&mut reader, &[
            Char::new ("a".as_bytes ()).to_word (),
            Char::new ("b".as_bytes ()).to_word ()
        ]) { assert! (false, "Unexpected result!") }
    }
}
