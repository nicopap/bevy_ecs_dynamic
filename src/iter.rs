use super::DynamicItem;

pub struct RoDynamicQueryIter<'w, 's> {
    foo: &'w (),
    bar: &'s (),
}
impl<'w, 's> Iterator for RoDynamicQueryIter<'w, 's> {
    type Item = DynamicItem<'w>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct DynamicQueryIter<'w, 's> {
    foo: &'w mut (),
    bar: &'s mut (),
}
impl<'w, 's> Iterator for DynamicQueryIter<'w, 's> {
    type Item = DynamicItem<'w>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
