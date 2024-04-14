/// Iterator of strings that concatenates input items into lines of a given maximum length
///
/// The length is given in **bytes**.
pub struct LineWrapper<const JOINER: char, Item: AsRef<str>, Iter: IntoIterator<Item = Item>> {
    line_length: usize,
    iter: Iter::IntoIter,
    buf: Option<String>,
}

impl<const JOINER: char, Item: AsRef<str>, Iter: Iterator<Item = Item>>
    LineWrapper<JOINER, Item, Iter>
{
    pub fn new(line_length: usize, iter: Iter) -> Self {
        let mut iter = iter.into_iter();
        LineWrapper {
            line_length,
            buf: iter.next().map(|item| {
                let mut buf = String::with_capacity(line_length);
                buf.push_str(item.as_ref());
                buf
            }),
            iter: iter,
        }
    }
}

impl<const JOINER: char, Item: AsRef<str>, Iter: Iterator<Item = Item>> Iterator
    for LineWrapper<JOINER, Item, Iter>
{
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let Some(buf) = &mut self.buf else {
            return None;
        };

        while let Some(item) = self.iter.next() {
            let item = item.as_ref();
            if buf.as_bytes().len() + JOINER.len_utf8() + item.as_bytes().len() <= self.line_length
            {
                buf.push(JOINER);
                buf.push_str(item);
            } else {
                // Line length exceeded; put the item aside for next call and return
                // the content of the current buffer
                let line = String::from(buf.as_str()); // Reallocate without the extra capacity
                buf.clear();
                buf.push_str(item);
                return Some(line);
            }
        }

        // No more items in the source iterator; return what remains in the buffer

        let mut buf = self.buf.take().unwrap(); // Can't panic, we already checked for None-ness
        buf.shrink_to_fit();
        Some(buf)
    }
}

#[test]
fn test_linewrapper() {
    let items = ["a", "ab", "cde", "f", "ghi", "jklm", "nopqr"];

    assert_eq!(
        LineWrapper::<' ', _, _>::new(3, items.into_iter()).collect::<Vec<_>>(),
        vec!["a", "ab", "cde", "f", "ghi", "jklm", "nopqr"]
    );

    assert_eq!(
        LineWrapper::<' ', _, _>::new(4, items.into_iter()).collect::<Vec<_>>(),
        vec!["a ab", "cde", "f", "ghi", "jklm", "nopqr"]
    );

    assert_eq!(
        LineWrapper::<' ', _, _>::new(5, items.into_iter()).collect::<Vec<_>>(),
        vec!["a ab", "cde f", "ghi", "jklm", "nopqr"]
    );

    assert_eq!(
        LineWrapper::<' ', _, _>::new(9, items.into_iter()).collect::<Vec<_>>(),
        vec!["a ab cde", "f ghi", "jklm", "nopqr"]
    );
}

#[test]
fn test_linewrapper_empty() {
    assert_eq!(
        LineWrapper::<' ', _, _>::new(3, Vec::<&str>::new().into_iter()).collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

#[test]
fn test_linewrapper_single() {
    assert_eq!(
        LineWrapper::<' ', _, _>::new(3, ["a"].into_iter()).collect::<Vec<_>>(),
        vec!["a"]
    );

    assert_eq!(
        LineWrapper::<' ', _, _>::new(3, ["abcde"].into_iter()).collect::<Vec<_>>(),
        vec!["abcde"]
    );
}
