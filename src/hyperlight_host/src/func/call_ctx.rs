use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterValue, ReturnType, ReturnValue,
};
use tracing::{instrument, Span};

use super::guest_dispatch::call_function_on_guest;
use crate::{MultiUseSandbox, Result, SingleUseSandbox};
/// A context for calling guest functions.
///
/// Takes ownership of an existing `MultiUseSandbox`.
/// Once created, guest function calls may be made through this and only this context
/// until it is converted back to the `MultiUseSandbox` from which it originated.
///
/// Upon this conversion,the memory associated with the `MultiUseSandbox` it owns will be reset to the state it was in before
/// this context was created.
///
/// Calls made through this context will cause state to be retained across calls, until such time as the context
/// is converted back to a `MultiUseSandbox`
///
/// If dropped, the `MultiUseSandbox` from which it came will be also be dropped as it is owned by the
/// `MultiUseGuestCallContext` until it is converted back to a `MultiUseSandbox`
///
#[derive(Debug)]
pub struct MultiUseGuestCallContext<'a> {
    sbox: MultiUseSandbox<'a>,
}

impl<'a> MultiUseGuestCallContext<'a> {
    /// Take ownership  of a `MultiUseSandbox` and
    /// return a new `MultiUseGuestCallContext` instance.
    ///     
    #[instrument(skip_all, parent = Span::current())]
    pub fn start(sbox: MultiUseSandbox<'a>) -> Self {
        Self { sbox }
    }

    /// Call the guest function called `func_name` with the given arguments
    /// `args`, and expect the return value have the same type as
    /// `func_ret_type`.
    ///
    /// Every call to a guest function through this method will be made with the same "context"
    /// meaning that the guest state resulting from any previous call will be present/osbservable
    /// by the guest funcation called.
    ///
    /// If you want  to reset state, call `finish()` on this `MultiUseGuestCallContext`
    /// and get a new one from the resulting `MultiUseSandbox`
    #[instrument(err(Debug),skip(self, args),parent = Span::current())]
    pub fn call(
        &mut self,
        func_name: &str,
        func_ret_type: ReturnType,
        args: Option<Vec<ParameterValue>>,
    ) -> Result<ReturnValue> {
        // we are guaranteed to be holding a lock now, since `self` can't
        // exist without doing so. Since GuestCallContext is effectively
        // !Send (and !Sync), we also don't need to worry about
        // synchronization

        call_function_on_guest(&mut self.sbox, func_name, func_ret_type, args)
    }

    /// Close out the context and get back the internally-stored
    /// `MultiUseSandbox`. Future contexts opened by the returned sandbox
    /// will have guest state restored.
    #[instrument(err(Debug), skip(self), parent = Span::current())]
    pub fn finish(mut self) -> Result<MultiUseSandbox<'a>> {
        self.sbox.restore_state()?;
        Ok(self.sbox)
    }
    /// Close out the context and get back the internally-stored
    /// `MultiUseSandbox`.
    ///
    /// Note that this method is pub(crate) and does not reset the state of the
    /// sandbox.
    ///
    /// It is intended to be used when evolving a MutliUseSandbox to a new state
    /// and is not intended to be called publicly. It allows the state of the guest to be altered
    /// during the eveolution of one sandbox state to another, enabling the new state created
    /// to be captured and stored in the Sandboxes state stack.
    ///
    pub(crate) fn finish_no_reset(self) -> MultiUseSandbox<'a> {
        self.sbox
    }
}

/// A context for calling guest functions. Can only be created from an existing
/// `SingleUseSandbox`, and once created, guest functions against that sandbox
/// can be made from this until it is dropped.
#[derive(Debug)]
pub struct SingleUseGuestCallContext<'a> {
    sbox: SingleUseSandbox<'a>,
}

impl<'a> SingleUseGuestCallContext<'a> {
    /// Take ownership  of a `SingleUseSandbox` and
    /// return a new `SingleUseGuestCallContext` instance.
    ///     
    #[instrument(skip_all, parent = Span::current())]
    pub(crate) fn start(sbox: SingleUseSandbox<'a>) -> Self {
        Self { sbox }
    }

    /// Call the guest function called `func_name` with the given arguments
    /// `args`, and expect the return value have the same type as
    /// `func_ret_type`.
    ///
    /// Once the call is complete, the 'SingleUseSandbox' will no longer be useable and a new one will need to be created.
    ///
    /// Rather than call this method directly, consider using the `call_guest_function_by_name` method on the `SingleUseSandbox`

    #[instrument(err(Debug),skip(self, args),parent = Span::current())]
    pub(crate) fn call(
        mut self,
        func_name: &str,
        func_ret_type: ReturnType,
        args: Option<Vec<ParameterValue>>,
    ) -> Result<ReturnValue> {
        self.call_internal(func_name, func_ret_type, args)
    }

    // Internal call function that takes a mutable reference to self
    // This function allows a SingleUseMultiGuestCallContext to be used to make multiple calls to guest functions
    // before it is no longer usable.
    #[instrument(skip_all, parent = Span::current())]
    fn call_internal(
        &mut self,
        func_name: &str,
        func_ret_type: ReturnType,
        args: Option<Vec<ParameterValue>>,
    ) -> Result<ReturnValue> {
        // We are guaranteed to be holding a lock now, since `self` can't
        // exist without doing so. since GuestCallContext is effectively
        // !Send (and !Sync), we also don't need to worry about
        // synchronization

        call_function_on_guest(&mut self.sbox, func_name, func_ret_type, args)
    }

    /// This function allows for a `SingleUseSandbox` to be used to make multiple calls to guest functions before it is dropped.
    ///
    /// The function is passed a callback function that it then callsd with a reference to a 'SingleUseMultiGuestCallContext'
    /// that can be used to make multiple calls to guest functions.
    ///
    pub fn call_from_func<
        Fn: FnOnce(&mut SingleUseMultiGuestCallContext) -> Result<ReturnValue>,
    >(
        self,
        f: Fn,
    ) -> Result<ReturnValue> {
        let mut ctx = SingleUseMultiGuestCallContext::new(self);
        f(&mut ctx)
    }
}

/// A context for making multiple calls to guest functions in a SingleUseSandbox. Can only be created
/// from an existing SingleUseGuestCallContext using the `call_using_closure` method.
/// Once created, calls to guest functions may be made through this context until it is dropped.
/// Once dropped the underlying `SingleUseGuestCallContext` and associated `SingleUseSandbox` will be dropped
pub struct SingleUseMultiGuestCallContext<'a> {
    call_context: SingleUseGuestCallContext<'a>,
}

impl<'a> SingleUseMultiGuestCallContext<'a> {
    fn new(call_context: SingleUseGuestCallContext<'a>) -> Self {
        Self { call_context }
    }

    /// Call the guest function called `func_name` with the given arguments
    pub fn call(
        &mut self,
        func_name: &str,
        func_ret_type: ReturnType,
        args: Option<Vec<ParameterValue>>,
    ) -> Result<ReturnValue> {
        self.call_context
            .call_internal(func_name, func_ret_type, args)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::sync_channel;
    use std::thread::{self, JoinHandle};

    use hyperlight_common::flatbuffer_wrappers::function_types::{
        ParameterValue, ReturnType, ReturnValue,
    };
    use hyperlight_testing::simple_guest_as_string;

    use crate::func::call_ctx::SingleUseMultiGuestCallContext;
    use crate::sandbox_state::sandbox::EvolvableSandbox;
    use crate::sandbox_state::transition::Noop;
    use crate::{
        GuestBinary, HyperlightError, MultiUseSandbox, Result, SingleUseSandbox,
        UninitializedSandbox,
    };

    fn new_uninit() -> Result<UninitializedSandbox> {
        let path = simple_guest_as_string().map_err(|e| {
            HyperlightError::Error(format!("failed to get simple guest path ({e:?})"))
        })?;
        UninitializedSandbox::new(GuestBinary::FilePath(path), None, None, None)
    }

    /// Test to create a `SingleUseSandbox`, then call several guest
    /// functions sequentially.
    #[test]
    fn singleusesandbox_single_call() {
        let calls = [
            (
                "StackAllocate",
                ReturnType::Int,
                Some(vec![ParameterValue::Int(1)]),
                ReturnValue::Int(1),
            ),
            (
                "CallMalloc",
                ReturnType::Int,
                Some(vec![ParameterValue::Int(200)]),
                ReturnValue::Int(200),
            ),
        ];

        for call in calls.iter() {
            let sbox: SingleUseSandbox = new_uninit().unwrap().evolve(Noop::default()).unwrap();
            let ctx = sbox.new_call_context();
            let res = ctx.call(call.0, call.1, call.2.clone()).unwrap();
            assert_eq!(call.3, res);
        }
    }

    #[test]
    fn singleusesandbox_multi_call() {
        let calls = [
            (
                "StackAllocate",
                ReturnType::Int,
                Some(vec![ParameterValue::Int(1)]),
                ReturnValue::Int(1),
            ),
            (
                "CallMalloc",
                ReturnType::Int,
                Some(vec![ParameterValue::Int(200)]),
                ReturnValue::Int(200),
            ),
        ];

        let sbox: SingleUseSandbox = new_uninit().unwrap().evolve(Noop::default()).unwrap();
        let ctx = sbox.new_call_context();

        let callback_closure = |ctx: &mut SingleUseMultiGuestCallContext| {
            let mut res: ReturnValue = ReturnValue::Int(0);
            for call in calls.iter() {
                res = ctx
                    .call(call.0, call.1, call.2.clone())
                    .expect("failed to call guest function");
                assert_eq!(call.3, res);
            }
            Ok(res)
        };

        let res = ctx.call_from_func(callback_closure).unwrap();
        assert_eq!(calls.last().unwrap().3, res);
    }

    /// Test to create a `MultiUseSandbox`, then call several guest functions
    /// on it across different threads.
    ///
    /// This test works by passing messages between threads using Rust's
    /// [mpsc crate](https://doc.rust-lang.org/std/sync/mpsc). Details of this
    /// interaction are as follows.
    ///
    /// One thread acts as the receiver (AKA: consumer) and owns the
    /// `MultiUseSandbox`. This receiver fields requests from N senders
    /// (AKA: producers) to make batches of calls.
    ///
    /// Upon receipt of a message to execute a batch, a new
    /// `MultiUseGuestCallContext` is created in the receiver thread from the
    /// existing `MultiUseSandbox`, and the batch is executed.
    ///
    /// After the batch is complete, the `MultiUseGuestCallContext` is done
    /// and it is converted back to the underlying `MultiUseSandbox`
    #[test]
    fn test_multi_call_multi_thread() {
        let (snd, recv) = sync_channel::<Vec<TestFuncCall>>(0);

        // create new receiver thread and on it, begin listening for
        // requests to execute batches of calls
        let recv_hdl = thread::spawn(move || {
            let mut sbox: MultiUseSandbox = new_uninit().unwrap().evolve(Noop::default()).unwrap();
            while let Ok(calls) = recv.recv() {
                let mut ctx = sbox.new_call_context();
                for call in calls {
                    let res = ctx
                        .call(call.func_name.as_str(), call.ret_type, call.params)
                        .unwrap();
                    assert_eq!(call.expected_ret, res);
                }
                sbox = ctx.finish().unwrap();
            }
        });

        // create new sender threads
        let send_handles: Vec<JoinHandle<()>> = (0..10)
            .map(|i| {
                let sender = snd.clone();
                thread::spawn(move || {
                    let calls: Vec<TestFuncCall> = vec![
                        TestFuncCall {
                            func_name: "StackAllocate".to_string(),
                            ret_type: ReturnType::Int,
                            params: Some(vec![ParameterValue::Int(i + 1)]),
                            expected_ret: ReturnValue::Int(i + 1),
                        },
                        TestFuncCall {
                            func_name: "CallMalloc".to_string(),
                            ret_type: ReturnType::Int,
                            params: Some(vec![ParameterValue::Int(i + 2)]),
                            expected_ret: ReturnValue::Int(i + 2),
                        },
                    ];
                    sender.send(calls).unwrap();
                })
            })
            .collect();

        for hdl in send_handles {
            hdl.join().unwrap();
        }
        // after all sender threads are done, drop the sender itself
        // so the receiver thread can exit. then, ensure the receiver
        // thread has exited.
        drop(snd);
        recv_hdl.join().unwrap();
    }

    pub struct TestSandbox<'a> {
        sandbox: MultiUseSandbox<'a>,
    }

    impl<'a> TestSandbox<'a> {
        pub fn new() -> Self {
            let sbox: MultiUseSandbox = new_uninit().unwrap().evolve(Noop::default()).unwrap();
            Self { sandbox: sbox }
        }
        pub fn call_add_to_static_multiple_times(mut self, i: i32) -> Result<TestSandbox<'a>> {
            let mut ctx = self.sandbox.new_call_context();
            let mut sum: i32 = 0;
            for n in 0..i {
                let result = ctx.call(
                    "AddToStatic",
                    ReturnType::Int,
                    Some(vec![ParameterValue::Int(n)]),
                );
                sum += n;
                println!("{:?}", result);
                let result = result.unwrap();
                assert_eq!(result, ReturnValue::Int(sum));
            }
            let result = ctx.finish();
            assert!(result.is_ok());
            self.sandbox = result.unwrap();
            Ok(self)
        }

        pub fn call_add_to_static(mut self, i: i32) -> Result<()> {
            for n in 0..i {
                let result = self.sandbox.call_guest_function_by_name(
                    "AddToStatic",
                    ReturnType::Int,
                    Some(vec![ParameterValue::Int(n)]),
                );
                println!("{:?}", result);
                let result = result.unwrap();
                assert_eq!(result, ReturnValue::Int(n));
            }
            Ok(())
        }
    }

    #[test]
    fn ensure_multiusesandbox_multi_calls_dont_reset_state() {
        let sandbox = TestSandbox::new();
        let result = sandbox.call_add_to_static_multiple_times(5);
        assert!(result.is_ok());
    }

    #[test]
    fn ensure_multiusesandbox_single_calls_do_reset_state() {
        let sandbox = TestSandbox::new();
        let result = sandbox.call_add_to_static(5);
        assert!(result.is_ok());
    }

    struct TestFuncCall {
        func_name: String,
        ret_type: ReturnType,
        params: Option<Vec<ParameterValue>>,
        expected_ret: ReturnValue,
    }
}
