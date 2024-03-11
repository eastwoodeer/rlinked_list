use std::cell::UnsafeCell;
use std::marker::PhantomPinned;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

struct LinksInner<T: ?Sized> {
    prev: NonNull<T>,
    next: NonNull<T>,
    _pin: PhantomPinned,
}

pub struct Links<T: ?Sized>(UnsafeCell<MaybeUninit<LinksInner<T>>>);

impl<T: ?Sized> Links<T> {
    pub fn new() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }
}

pub trait Adapter {
    type EntryType: ?Sized;

    fn to_links(obj: &Self::EntryType) -> &Links<Self::EntryType>;
}

pub struct List<A: Adapter + ?Sized> {
    head: Option<NonNull<A::EntryType>>,
}

impl<A: Adapter + ?Sized> List<A> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn insert_only_entry(&mut self, obj: &A::EntryType) {
        let obj_ptr = NonNull::from(obj);
        let obj_inner = unsafe { &mut *A::to_links(obj).0.get() };

        obj_inner.write(LinksInner {
            prev: obj_ptr,
            next: obj_ptr,
            _pin: PhantomPinned,
        });

        self.head = Some(obj_ptr);
    }

    pub fn push_back(&mut self, obj: &A::EntryType) {
        if let Some(head) = self.head {
            unsafe {
                self.insert_after(self.inner_ref(head).prev, obj);
            }
        } else {
            self.insert_only_entry(obj);
        }
    }

    pub unsafe fn push_front(&mut self, obj: &A::EntryType) {
        if let Some(head) = self.head {
            self.insert_before(head, obj);
        } else {
            self.insert_only_entry(obj);
        }
    }

    pub fn remove(&mut self, entry: &A::EntryType) {
        let inner = unsafe { self.inner_ref(NonNull::from(entry)) };
        let next = inner.next;
        let prev = inner.prev;

        let inner = unsafe { &mut *A::to_links(entry).0.get() };
        unsafe { inner.assume_init_drop() };

        if core::ptr::eq(next.as_ptr(), entry) {
            self.head = None;
        } else {
            unsafe {
                self.inner_mut(prev).next = next;
                self.inner_mut(next).prev = prev;
            }

            if core::ptr::eq(self.head.unwrap().as_ptr(), entry) {
                self.head = Some(next);
            }
        }
    }

    pub fn insert_after(&mut self, existing: NonNull<A::EntryType>, new: &A::EntryType) {
        let new_inner = unsafe { &mut *A::to_links(new).0.get() };
        let existing_inner = unsafe { self.inner_mut(existing) };
        let next = existing_inner.next;

        new_inner.write(LinksInner {
            prev: existing,
            next,
            _pin: PhantomPinned,
        });

        existing_inner.next = NonNull::from(new);
        unsafe {
            self.inner_mut(next).prev = NonNull::from(new);
        }
    }

    pub fn insert_before(&mut self, existing: NonNull<A::EntryType>, new: &A::EntryType) {
        unsafe { self.insert_after(self.inner_ref(existing).prev, new) }

        if self.head.unwrap() == existing {
            self.head = Some(NonNull::from(existing));
        }
    }

    unsafe fn inner_mut(&self, ptr: NonNull<A::EntryType>) -> &mut LinksInner<A::EntryType> {
        unsafe { (*A::to_links(ptr.as_ref()).0.get()).assume_init_mut() }
    }

    unsafe fn inner_ref(&self, ptr: NonNull<A::EntryType>) -> &LinksInner<A::EntryType> {
        unsafe { (*A::to_links(ptr.as_ref()).0.get()).assume_init_ref() }
    }
}

pub struct TaskList {
    pub id: usize,
    pub task_links: Links<TaskList>,
}

impl Adapter for TaskList {
    type EntryType = Self;

    fn to_links(obj: &Self::EntryType) -> &Links<Self::EntryType> {
        &obj.task_links
    }
}

fn main() {
    println!("Hello, world!");

    let t1 = TaskList {
        id: 1,
        task_links: Links::new(),
    };

    let t2 = TaskList {
        id: 2,
        task_links: Links::new(),
    };

    let mut list: List<TaskList> = List::<TaskList>::new();
    list.push_back(&t1);
    list.push_back(&t2);

    unsafe {
        println!(
            "list.first: {}, list.first.next: {}, list.first.next.next: {}",
            list.head.unwrap().as_ref().id,
            (*(*list.head.unwrap().as_ref().task_links.0.get()).as_ptr())
                .next
                .as_ref()
                .id,
            (*(*TaskList::to_links(&t2).0.get()).as_ptr())
                .next
                .as_ref()
                .id
        );
    }

    list.remove(&t2);

    unsafe {
        println!(
            "list.first: {}, list.first.next: {}, list.first.next.next: {}",
            list.head.unwrap().as_ref().id,
            (*(*list.head.unwrap().as_ref().task_links.0.get()).as_ptr())
                .next
                .as_ref()
                .id,
            (*(*TaskList::to_links(&t2).0.get()).as_ptr())
                .next
                .as_ref()
                .id
        );
    }

    list.remove(&t1);

    unsafe {
        println!(
            "list.first: {}, list.first.next: {}, list.first.next.next: {}",
            list.head.unwrap().as_ref().id,
            (*(*list.head.unwrap().as_ref().task_links.0.get()).as_ptr())
                .next
                .as_ref()
                .id,
            (*(*TaskList::to_links(&t2).0.get()).as_ptr())
                .next
                .as_ref()
                .id
        );
    }

}
