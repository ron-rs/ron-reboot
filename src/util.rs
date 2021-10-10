use std::fmt;
use std::fmt::Formatter;

pub fn write_pretty_list<T>(
    f: &mut Formatter<'_>,
    mut i: impl Iterator<Item = T> + Clone,
    mut write_t: impl FnMut(&mut Formatter<'_>, T) -> fmt::Result,
) -> fmt::Result {
    let char_count = i.clone().count();
    if char_count == 0 {
        return write!(f, "<empty list>");
    }
    if char_count == 1 {
        return write_t(f, i.next().unwrap());
    }

    let i = &mut i;

    write!(f, "one of ")?;
    i.take(char_count - 2).try_for_each(|c| {
        write_t(f, c)?;
        write!(f, ", ")
    })?;
    write_t(f, i.next().unwrap())?;
    write!(f, " or ")?;
    write_t(f, i.next().unwrap())
}
