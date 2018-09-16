pub fn iter<C>(container: C) -> PathSetIter<C::IntoIter>
where
    C::Item: AsRef<str>,
    C: IntoIterator,
{
    PathSetIter {
        iter: container.into_iter(),
        current_strings: vec![],
    }
}

pub struct PathSetIter<I>
where
    I::Item: AsRef<str>,
    I: Iterator,
{
    iter: I,
    current_strings: Vec<String>,
}

impl<I> Iterator for PathSetIter<I>
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(s) = self.current_strings.pop() {
                return Some(s);
            }

            let next_item = match self.iter.next() {
                Some(s) => s,
                None => return None,
            };

            let next_str = next_item.as_ref();
            self.current_strings
                .extend(next_str.split(':').map(|s| s.to_owned()).rev());
        }
    }
}
