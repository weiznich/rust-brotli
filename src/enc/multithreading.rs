#![cfg(not(feature="no-stdlib"))]
use core::mem;
use std;
use core::marker::PhantomData;
use std::thread::{
    JoinHandle,
};
use alloc::{SliceWrapper, Allocator};
use enc::BrotliAlloc;
use enc::BrotliEncoderParams;
use enc::threading::{
  CompressMulti,
  SendAlloc,
  InternalSendAlloc,
  BatchSpawnable,
  Joinable,
  Owned,
  OwnedRetriever,
  CompressionThreadResult,
  InternalOwned,
  BrotliEncoderThreadError,
  AnyBoxConstructor,
  PoisonedThreadError,
  ReadGuard,
};

use std::sync::RwLock;


pub struct MultiThreadedJoinable<T:Send+'static, U:Send+'static>(JoinHandle<T>, PhantomData<U>);

impl<T:Send+'static, U:Send+'static+AnyBoxConstructor> Joinable<T, U> for MultiThreadedJoinable<T, U> {
  fn join(self) -> Result<T, U> {
      match self.0.join() {
          Ok(t) => Ok(t),
          Err(e) => Err(<U as AnyBoxConstructor>::new(e)),
      }
  }
}
pub struct MultiThreadedOwnedRetriever<U:Send+'static>(std::sync::Arc<RwLock<U>>);

impl<U:Send+'static> OwnedRetriever<U> for MultiThreadedOwnedRetriever<U> {
  fn view(&self) -> Result<ReadGuard<U>, PoisonedThreadError> {
      match self.0.read() {
          Ok(u) => Ok(ReadGuard::<U>(u)),
          Err(_) => Err(PoisonedThreadError::default()),
      }
  }
  fn unwrap(self) -> Result<U, PoisonedThreadError> {
    match std::sync::Arc::try_unwrap(self.0) {
      Ok(rwlock) => match rwlock.into_inner() {
        Ok(u) => Ok(u),
        Err(_) => Err(PoisonedThreadError::default()),
      },
      Err(_) => Err(PoisonedThreadError::default()),
    }
  }
}


#[derive(Default)]
pub struct MultiThreadedSpawner{}
fn thread_adapter<T:Send+'static, F: Fn(usize, usize, &U, Alloc) -> T+Send+'static, Alloc:BrotliAlloc+Send+'static, U:Send+Sync+'static>(index: usize, num_threads: usize, locked_input:std::sync::Arc<RwLock<U>>, alloc:Alloc, f:F) -> T {
  f(index, num_threads, &*locked_input.view().unwrap(), alloc)
}


fn spawn_work<T:Send+'static, F: Fn(usize, usize, &U, Alloc) -> T+Send+'static, Alloc:BrotliAlloc+Send+'static, U:Send+Sync+'static>(index: usize, num_threads: usize, locked_input:std::sync::Arc<RwLock<U>>, alloc:Alloc, f:F) -> std::thread::JoinHandle<T>
where <Alloc as Allocator<u8>>::AllocatedMemory:Send+'static {
  std::thread::spawn(move || thread_adapter(index, num_threads, locked_input, alloc, f))
}
/*
impl<T:Send+'static, Alloc:BrotliAlloc+Send+'static, U:Send+'static> BatchSpawnable<T, Alloc, U> for MultiThreadedSpawner
where <Alloc as Allocator<u8>>::AllocatedMemory:Send+'static {
  type JoinHandle = MultiThreadedJoinable<T, Alloc>;
  type FinalJoinHandle = MultiThreadedOwnedRetriever<U>;
    fn batch_spawn<F: Fn(usize, usize, &U, Alloc) -> T>(
    &mut self,
    input: &mut Owned<U>,
    alloc_per_thread:&mut [SendAlloc<T, Alloc, Self::JoinHandle>],
    f: F,
    ) -> Self::FinalJoinHandle {
      let num_threads = alloc_per_thread.len();
      let locked_input = MultiThreadedOwnedRetriever::<U>(mem::replace(input, Owned(InternalOwned::Borrowed)).unwrap());
      for (index, work) in alloc_per_thread.iter_mut().enumerate() {
        let alloc = work.replace_with_default();
        let ret = spawn_work(index, num_threads, locked_input, alloc);
        *work = SendAlloc(InternalSendAlloc::Join(MultiThreadedJoinable{result:Ok(ret)}));
      }
      locked_input
    }
}
*/
/*

pub fn compress_multi<Alloc:BrotliAlloc+Send+'static,
                      SliceW: SliceWrapper<u8>+Send+'static> (
  params:&BrotliEncoderParams,
  owned_input: &mut Owned<SliceW>,
  output: &mut [u8],
  alloc_per_thread:&mut [SendAlloc<CompressionThreadResult<Alloc>,
                                   Alloc,
                                   <MultiThreadedSpawner as BatchSpawnable<CompressionThreadResult<Alloc>,Alloc, SliceW>>::JoinHandle>],
) -> Result<usize, BrotliEncoderThreadError> where <Alloc as Allocator<u8>>::AllocatedMemory: Send {
  CompressMulti(params, owned_input, output, alloc_per_thread, MultiThreadedSpawner::default())
}
*/                      
