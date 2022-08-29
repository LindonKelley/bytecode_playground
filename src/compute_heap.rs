use std::collections::HashMap;
use std::num::NonZeroU64;

pub struct Heap {
    // this implementation makes no attempt to reclaim old allocation indices,
    // and simply increments the counter until it overflows, then returns a HeapError::Allocation
    // for any following allocation attempts
    counter: NonZeroU64,
    reference_map: HashMap<ObjectReference, Object>
}

impl Heap {
    pub fn new() -> Self {
        Self {
            counter: NonZeroU64::new(1).unwrap(),
            reference_map: HashMap::new()
        }
    }

    pub fn allocate(&mut self, children_length: usize, data_length: usize) -> HeapResult<ObjectReference> {
        let obj = Object::new(children_length, data_length);
        let obj_ref = ObjectReference(self.get_and_increment_counter()?);
        self.reference_map.insert(obj_ref.clone(), obj);
        Ok(obj_ref)
    }

    pub fn increment_stack_references(&mut self, obj_ref: &ObjectReference) -> HeapResult<ObjectReference> {
        let obj = self.get_mut_object(&obj_ref)?;
        obj.stack_references = obj.stack_references.checked_add(1).ok_or(HeapError::StackReferenceError)?;
        Ok(obj_ref.clone())
    }

    pub fn decrement_stack_references(&mut self, obj_ref: ObjectReference) -> HeapResult<()> {
        let obj = self.get_mut_object(&obj_ref)?;
        obj.stack_references = obj.stack_references.checked_sub(1).ok_or(HeapError::StackReferenceError)?;
        Ok(())
    }

    fn get_object(&self, obj_ref: &ObjectReference) -> HeapResult<&Object> {
        self.reference_map.get(obj_ref).ok_or(HeapError::ObjectNotFound)
    }

    fn get_mut_object(&mut self, obj_ref: &ObjectReference) -> HeapResult<&mut Object> {
        self.reference_map.get_mut(obj_ref).ok_or(HeapError::ObjectNotFound)
    }

    fn get_and_increment_counter(&mut self) -> HeapResult<NonZeroU64> {
        let n = self.counter;
        self.counter = NonZeroU64::new(self.counter.get() + 1).ok_or(HeapError::Allocation)?;
        Ok(n)
    }

    pub fn set_child(&mut self, parent: &ObjectReference, index: usize, child: Option<&ObjectReference>) -> HeapResult<()> {
        self.get_mut_object(parent)?.set_child(index, child)
    }

    pub fn get_child(&self, parent: &ObjectReference, index: usize) -> HeapResult<Option<ObjectReference>> {
        self.get_object(parent)?.get_child(index)
    }

    pub fn get_data_slice(&self, obj_ref: &ObjectReference, start: usize, length: usize) -> HeapResult<&[u8]> {
        self.get_object(obj_ref)?.get_data_slice(start, length)
    }

    pub fn get_mut_data_slice(&mut self, obj_ref: &ObjectReference, start: usize, length: usize) -> HeapResult<&mut [u8]> {
        self.get_mut_object(obj_ref)?.get_mut_data_slice(start, length)
    }

    pub fn collect_garbage(&mut self) {
        let mut scanning_statuses = HashMap::with_capacity(self.reference_map.len());
        let mut root_objects = Vec::new();
        for (obj_ref, obj) in &self.reference_map {
            scanning_statuses.insert(obj_ref.clone(), (IsGarbage::Yes, obj));
            if obj.stack_references > 0 {
                root_objects.push(obj_ref);
            }
        }
        for root_obj in root_objects {
            self.sift_garbage(root_obj, &mut scanning_statuses);
        }
        let mut garbage_bin = Vec::new();
        for (obj_ref, (status, _)) in scanning_statuses {
            if status == IsGarbage::Yes {
                garbage_bin.push(obj_ref);
            }
        }
        for garbage in garbage_bin {
            self.reference_map.remove(&garbage);
        }
    }

    fn sift_garbage(&self, obj_ref: &ObjectReference, scanning_statuses: &mut HashMap<ObjectReference, (IsGarbage, &Object)>) {
        let (status, obj) = scanning_statuses.get_mut(obj_ref).unwrap();
        if *status == IsGarbage::Yes {
            *status = IsGarbage::No;
            for child_ref in obj.children.iter().flatten() {
                self.sift_garbage(child_ref, scanning_statuses);
            }
        }
    }
}

#[derive(Eq, PartialEq)]
enum IsGarbage {
    Yes,
    No
}

#[derive(Debug)]
pub enum HeapError {
    Allocation,
    ObjectNotFound,
    StackReferenceError,
    ChildIndexOutOfBounds,
    IllegalNullObjectReferenceUsage,
    OutOfBoundsObjectDataAccess
}

pub type HeapResult<T> = Result<T, HeapError>;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ObjectReference(NonZeroU64);

impl ObjectReference {
    pub fn new_option(n: u64) -> Option<Self> {
        Some(Self(NonZeroU64::new(n)?))
    }

    pub fn new_result(n: u64) -> HeapResult<Self> {
        Self::new_option(n).ok_or(HeapError::IllegalNullObjectReferenceUsage)
    }
}

impl From<ObjectReference> for u64 {
    fn from(v: ObjectReference) -> Self {
        v.0.get()
    }
}

#[derive(Debug)]
pub struct Object {
    /// a count of stack references to this object
    stack_references: u16,
    /// all objects this object refers to
    children: Box<[Option<ObjectReference>]>,
    /// data contained within this object
    data: Box<[u8]>
}

impl Object {
    fn new(children_length: usize, data_length: usize) -> Self {
        Self {
            stack_references: 1,
            children: vec![None; children_length].into_boxed_slice(),
            data: vec![0; data_length].into_boxed_slice()
        }
    }

    fn set_child(&mut self, index: usize, child: Option<&ObjectReference>) -> HeapResult<()> {
        *self.children.get_mut(index).ok_or(HeapError::ChildIndexOutOfBounds)? = child.cloned();
        Ok(())
    }

    fn get_child(&self, index: usize) -> HeapResult<Option<ObjectReference>> {
        Ok(self.children.get(index).ok_or(HeapError::ChildIndexOutOfBounds)?.clone())
    }

    fn get_data_slice(&self, start: usize, length: usize) -> HeapResult<&[u8]> {
        self.data.get(start..start+length).ok_or(HeapError::OutOfBoundsObjectDataAccess)
    }

    fn get_mut_data_slice(&mut self, start: usize, length: usize) -> HeapResult<&mut [u8]> {
        self.data.get_mut(start..start+length).ok_or(HeapError::OutOfBoundsObjectDataAccess)
    }
}
