use std::io::{
    Read,
};
use thiserror::Error;

/// A reader that removes a limited set of supported comment types
/// from text input.
///
/// Supported comment types are:
///  - Line comments: from `//` to the next newlines
///  - Block comments: begin with `/*` as the first non-whitespace characters
///    on a line, and end with `*/` as the last non-whitespace characters on a
///    line.
pub struct StripComments<R: Read>
{
    reader: R,
    buf: Vec<u8>,
    offset: usize,
}

#[derive(Debug,Error)]
#[error("Unterminated block comment at line {0}")]
pub struct UnterminatedCommentError(usize);

impl<R: Read> StripComments<R>
{
    pub fn new(reader: R) -> Self
    {
        Self {
            reader,
            buf: Vec::new(),
            offset: 0,
        }
    }

    fn do_send(&mut self, buffer: &mut [u8]) -> std::io::Result<usize>
    {
        let remaining = self.buf.len() - self.offset;

        if remaining <= buffer.len()
        {
            buffer[0..remaining].copy_from_slice(&self.buf[self.offset..]);
            self.offset = 0;
            self.buf.clear();
            Ok(remaining)
        }
        else
        {
            buffer.copy_from_slice(&self.buf[self.offset .. self.offset+buffer.len()]);
            self.offset += buffer.len();
            Ok(buffer.len())
        }
    }
}

impl<R: Read> Read for StripComments<R>
{
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize>
    {
        if self.offset < self.buf.len()
        {
            // We've got data buffered, so just send it
            self.do_send(buffer)
        }
        else
        {
            // Nothing buffered, so we need to fill it up first
            let mut input = String::new();
            self.reader.read_to_string(&mut input)?;

            let mut inside_block_comment = 0;

            for (num, line) in input.split('\n').enumerate()
            {
                // split() removes the delimiters (i.e. newlines) so we need to
                // put them back, without introducing any spurious extra ones
                if num != 0 && inside_block_comment == 0
                {
                    self.buf.push(b'\n');
                }

                let trimmed = line.trim();

                if trimmed.starts_with("/*")
                {
                    inside_block_comment = num + 1;
                }
                if inside_block_comment != 0
                {
                    if trimmed.ends_with("*/")
                    {
                        inside_block_comment = 0;
                    }
                    continue;
                }

                if let Some(idx) = line.find("//")
                {
                    self.buf.extend_from_slice(line[0..idx].as_bytes());
                }
                else
                {
                    self.buf.extend_from_slice(line.as_bytes());
                }
            }

            if inside_block_comment != 0
            {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, UnterminatedCommentError(inside_block_comment)))
            }
            else
            {
                self.do_send(buffer)
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use stringreader::StringReader;

    // Newlines are explicitly insert in all these to preserve and make visible
    // what would otherwise be trailing whitespace.

    fn do_test(input: &str, expected: &str)
    {
        let input = StringReader::new(input);
        let mut reader = StripComments::new(input);

        let mut vec = Vec::new();
        reader.read_to_end(&mut vec).unwrap();

        assert_eq!(String::from_utf8(vec).unwrap(), expected);
    }

    #[test]
    fn single_line_comments()
    {
        let input: &str = "\
                aaa\n\
                bbb //foo\n\
                //blah\n\
                ccc\n\
                ";

        let output: &str = "\
                aaa\n\
                bbb \n\
                \n\
                ccc\n\
                ";

        do_test(input, output);
    }

    #[test]
    fn block_comments()
    {
        let input: &str = "\
                aaa\n\
                /* bbb\n\
                    ccc\n\
                    ddd */\n\
                eee\n\
                ";

        let output: &str = "\
                aaa\n\
                \n\
                eee\n\
                ";

        do_test(input, output);
    }

    #[test]
    fn block_inside_single_line()
    {
        let input = "\
                aaa\n\
                // blah /*\n\
                bbb\n\
                ";

        let output = "\
                aaa\n\
                \n\
                bbb\n\
                ";

        do_test(input, output);
    }

    #[test]
    fn single_inside_block()
    {
        let input = "\
                aaa\n\
                /* bbb\n\
                // ccc */\n\
                ddd\n\
                ";

        let output = "\
                aaa\n\
                \n\
                ddd\n\
                ";

        do_test(input, output);
    }
}