use std::vec;
use std::ops::Deref;

pub fn iter<S>(container: Vec<S>) -> PathSetIter<S> where S: Deref<Target = str> {
    PathSetIter {
        iter: container.into_iter(),
        current_strings: vec![],
    }
}

pub struct PathSetIter<S> where S: Deref<Target = str> {
    iter: vec::IntoIter<S>,
    current_strings: Vec<String>
}

impl<S> Iterator for PathSetIter<S> where S: Deref<Target = str> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(s) = self.current_strings.pop() {
                return Some(s);
            }

            let next_str = match self.iter.next() {
                Some(s) => s,
                None    => return None,
            };

            self.current_strings.extend(next_str.split(':').map(|s| s.to_string()).rev());
        }
    }
}
