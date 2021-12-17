use std::{
    any::{type_name, Any},
    collections::HashMap,
};

#[cfg(test)]
mod tests {
    use crate::app::event_system::EventQueue;

    #[test]
    fn event() {
        #[derive(PartialEq, Clone, Debug)]
        enum MyEvent {
            Hello,
            World,
        }

        let mut queue = EventQueue::default();
        queue.notify(MyEvent::Hello);
        queue.notify(MyEvent::World);

        assert_eq!(queue.next(), Some(MyEvent::World));
        assert_eq!(queue.next(), Some(MyEvent::Hello));
    }
}

#[derive(Default)]
pub struct EventQueue {
    events: HashMap<String, Box<dyn Any>>,
}
impl EventQueue {
    pub fn notify<T: 'static>(&mut self, event: T) {
        match self.events.get_mut(type_name::<T>()) {
            Some(list) => {
                let list = list.downcast_mut::<Vec<T>>().unwrap();
                list.push(event);
            }
            None => {
                self.events
                    .insert(type_name::<T>().into(), Box::new(vec![event]));
            }
        }
    }
    fn next<T: 'static + Clone>(&mut self) -> Option<T> {
        match self.events.get_mut(type_name::<T>()) {
            Some(list) => list.downcast_mut::<Vec<T>>()?.pop(),
            None => None,
        }
    }
    pub fn read_all<T: 'static + Clone>(&mut self) -> std::vec::IntoIter<T> {
        match self.events.get_mut(type_name::<T>()) {
            Some(list) => {
                if let Some(list) = list.downcast_mut::<Vec<T>>() {
                    list.drain(0..).collect::<Vec<T>>().into_iter()
                } else {
                    Vec::new().into_iter()
                }
            }
            None => Vec::new().into_iter(),
        }
    }
}
