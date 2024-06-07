use crate::func::exports::get_os_page_size;
#[cfg(feature = "function_call_metrics")]
use crate::histogram_vec_observe;
use crate::hypervisor::handlers::{MemAccessHandlerWrapper, OutBHandlerWrapper};
#[cfg(target_os = "windows")]
use crate::hypervisor::hyperv_windows::HypervWindowsDriver;
use crate::hypervisor::Hypervisor;
use crate::mem::ptr::RawPtr;
#[cfg(feature = "function_call_metrics")]
use crate::sandbox::metrics::SandboxMetric::GuestFunctionCallDurationMicroseconds;
use crate::sandbox::WrapperGetter;
use crate::sandbox_state::sandbox::Sandbox;
use crate::{log_then_return, new_error, HyperlightError, Result};
use crossbeam::atomic::AtomicCell;
use crossbeam_channel::{Receiver, Sender};
#[cfg(target_os = "linux")]
use libc::{c_void, pthread_self, siginfo_t};
use rand::Rng;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tracing::instrument;
use tracing::Span;
#[cfg(target_os = "linux")]
use vmm_sys_util::signal::{register_signal_handler, SIGRTMIN};
#[cfg(target_os = "windows")]
use windows::Win32::System::Hypervisor::WHV_PARTITION_HANDLE;

/// Trait to indicate that a type contains to/from Hypervisor handler receivers/transmitters.
pub trait ToFromRxTx {
    /// Get the transmitter for vCPU actions to the handler
    fn get_to_handler_tx(&self) -> Sender<VCPUAction>;
    /// Set the transmitter to send messages to the handler
    fn set_to_handler_tx(&mut self, tx: Sender<VCPUAction>);
    /// Drop the transmitter to send messages to the handler
    /// This is useful to forcefully terminate a vCPU
    fn drop_to_handler_tx(&mut self);

    /// Get the receiver for messages from the handler
    fn get_from_handler_rx(&self) -> Receiver<HandlerMsg>;
    /// Set the receiver to receive messages from the handler
    fn set_from_handler_rx(&mut self, rx: Receiver<HandlerMsg>);

    /// Get the transmitter for messages from the handler
    fn get_from_handler_tx(&self) -> Sender<HandlerMsg>;
    /// Set the transmitter for messages from the handler
    fn set_from_handler_tx(&mut self, tx: Sender<HandlerMsg>);
}

/// `VCPUActions` enumerates the
/// possible actions that a Hypervisor
/// handler can execute.
pub enum VCPUAction {
    /// Initialise the vCPU
    Initialise(InitArgs),
    /// Execute the vCPU until a HLT instruction
    DispatchCallFromHost(DispatchArgs),
}

/// `HandlerMsg` is structure used by the Hypervisor
/// handler to indicate that the Hypervisor Handler has
/// finished performing an action (i.e., `DispatchCallFromHost`, or
/// `Initialise`).
pub enum HandlerMsg {
    FinishedVCPUAction,
    Error(HyperlightError),
}

/// Arguments to initialise the vCPU
pub struct InitArgs {
    peb_addr: RawPtr,
    seed: u64,
    page_size: u32,
    outb_handle_fn: OutBHandlerWrapper,
    mem_access_fn: MemAccessHandlerWrapper,
}

impl InitArgs {
    /// Create a new `InitArgs` instance
    pub fn new(
        peb_addr: RawPtr,
        seed: u64,
        page_size: u32,
        outb_handle_fn: OutBHandlerWrapper,
        mem_access_fn: MemAccessHandlerWrapper,
    ) -> Self {
        Self {
            peb_addr,
            seed,
            page_size,
            outb_handle_fn,
            mem_access_fn,
        }
    }
}

/// Arguments to execute the vCPU
pub struct DispatchArgs {
    function_name: String,
    dispatch_func_addr: RawPtr,
    outb_handle_fn: OutBHandlerWrapper,
    mem_access_fn: MemAccessHandlerWrapper,
}

impl DispatchArgs {
    /// Create a new `DispatchArgs` instance
    pub fn new(
        function_name: String,
        dispatch_func_addr: RawPtr,
        outb_handle_fn: OutBHandlerWrapper,
        mem_access_fn: MemAccessHandlerWrapper,
    ) -> Self {
        Self {
            function_name,
            dispatch_func_addr,
            outb_handle_fn,
            mem_access_fn,
        }
    }
}

// For Windows, we don't mess with the thread ID, we just need to set the partition handle
#[cfg(target_os = "windows")]
thread_local! {
    pub static PARTITION_HANDLE: Arc<Mutex<WHV_PARTITION_HANDLE>> =
        Arc::new(Mutex::new(WHV_PARTITION_HANDLE::default()));
}

/// Sets up a Hypervisor 'handler', designed to listen to
/// messages to execute a specific action, such as:
/// - `initialise` resources,
/// - `dispatch_call_from_host` in the vCPU, and
/// - `terminate_execution` of the vCPU.
///
/// The execution of an action within the handler has to
/// be paired w/ a call to from_handler_tx.recv() to
/// synchronise the completion of the action. Otherwise,
/// the main thread will proceed prior to finishing the
/// required action, and you may encounter unexpected unset
/// pointers or other issues.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn start_hypervisor_handler(
    hv: Arc<Mutex<Box<dyn Hypervisor>>>,
    (from_handler_tx, to_handler_rx): (Sender<HandlerMsg>, Receiver<VCPUAction>),
) -> crate::Result<JoinHandle<Result<()>>> {
    let hv_clone = hv.clone();

    // Other than running initialization and code execution, the handler thread also handles
    // cancellation. When we need to cancel the execution there are 2 possible cases
    // we have to deal with depending on if the vCPU is currently running or not.
    //
    // 1. If the vCPU is executing, then we need to cancel the execution.
    // 2. If the vCPU is not executing, then we need to signal to the thread
    // that it should exit the loop.
    //
    // For the first case, on Linux, we send a signal to the thread running the
    // vCPU to interrupt it and cause an EINTR error on the underlying VM run call.
    //
    // For the second case, we set a flag that is checked on each iteration of the run loop
    // and if it is set to true then the loop will exit.

    // On Linux, we have another problem to deal with. The way we terminate a running vCPU
    // (case 1 above) is to send a signal to the thread running the vCPU to interrupt it.
    //
    // There is a possibility that the signal is sent and received just before the thread
    // calls run on the vCPU (between the check on the cancelled_run variable and the call to run)
    // - see this StackOverflow question for more details
    // https://stackoverflow.com/questions/25799667/fixing-race-condition-when-sending-signal-to-interrupt-system-call)
    //
    // To solve this, we need to keep sending the signal until we know that the spawned thread
    // knows it should cancel the execution. To do this, we will use another `AtomicCell` and `Arc`
    // to communicate between the main thread and the handler thread running the vCPU.
    // This variable will be set when the thread has received the instruction to cancel the
    // execution and will be checked in the code which sends the signal to terminate to know
    // there is no longer a needed to send a signal. Again, we will create this on the main
    // thread the first time we enter this function and TLS its. Then, create a clone which we
    // will move to the handler thread which will then place it in its TLS so that it
    // is accessible in the run loop if we re-enter the function.

    {
        let mut hv_lock = hv.lock().unwrap();

        hv_lock.set_termination_status(false);

        #[cfg(target_os = "linux")]
        {
            hv_lock.set_run_cancelled(false);
        }
    }

    // On Windows, we have a slightly different problem. Windows supports an API that allows us
    // to cancel the execution of a vCPU, but using that from here would require us to expose
    // an API from the `WindowsHypervisor` implementation, because, in the thread spawned,
    // we move the mutable self reference into the thread closure, which it makes it painful to
    // be able to call such a cancel function from the main thread since we would need
    // the mutable reference to call `run` we would not also be able to have a reference to call
    // cancel. As Windows is happy for us to interact with the VM from different threads
    // the solution we are going to use is to get the partition handle from the `WindowsHypervisor`
    // implementation, and then use that handle in a call to the cancel function from the
    // main thread. We only need the partition handle to call the cancel function as we are always
    // ever creating a single vCPU in a partition, and, therefore, can default the other parameters
    // to 0.
    #[cfg(target_os = "windows")]
    {
        PARTITION_HANDLE.with(|elem| {
            let mut partition_handle_lock = elem.lock().unwrap();
            *partition_handle_lock = {
                // To get the partition handle, we need to downcast the Hypervisor trait object to a
                // `WindowsHypervisorDriver`.
                let hv_lock = hv.lock().unwrap();
                let hyperv_windows_driver: &HypervWindowsDriver =
                    match hv_lock.as_any().downcast_ref::<HypervWindowsDriver>() {
                        Some(b) => b,
                        None => {
                            log_then_return!("Expected a WindowsHypervisorDriver");
                        }
                    };
                hyperv_windows_driver.get_partition_hdl()
            };

            Ok(())
        })?;
    }

    let join_handle = {
        thread::spawn(move || -> Result<()> {
            for action in to_handler_rx.clone() {
                match action {
                    VCPUAction::Initialise(args) => {
                        let mut hv_lock = hv_clone.lock().unwrap();

                        // Reset termination status from a possible previous execution
                        hv_lock.set_termination_status(false);
                        #[cfg(target_os = "linux")]
                        {
                            hv_lock.set_run_cancelled(false);
                        }

                        #[cfg(target_os = "linux")]
                        {
                            // We cannot use the Killable trait, so we get the `pthread_t` via a libc
                            // call.

                            let thread_id = unsafe { pthread_self() };
                            hv_lock.set_thread_id(thread_id);

                            // Register a signal handler to cancel the execution of the vCPU on Linux.
                            // On Windows, we don't need to do anything as we can just call the cancel
                            // function.

                            extern "C" fn handle_signal(_: i32, _: *mut siginfo_t, _: *mut c_void) {
                            }
                            match register_signal_handler(SIGRTMIN(), handle_signal) {
                                Ok(_) => {}
                                Err(e) => panic!("failed to register signal handler: {:?}", e),
                            }
                        }

                        match hv_lock.initialise(
                            args.peb_addr,
                            args.seed,
                            args.page_size,
                            args.outb_handle_fn,
                            args.mem_access_fn,
                        ) {
                            Ok(_) => {
                                from_handler_tx
                                    .send(HandlerMsg::FinishedVCPUAction)
                                    .map_err(|_| {
                                        HyperlightError::HypervisorHandlerCommunicationFailure()
                                    })?;
                            }
                            Err(e) => {
                                from_handler_tx.send(HandlerMsg::Error(e)).map_err(|_| {
                                    HyperlightError::HypervisorHandlerCommunicationFailure()
                                })?;
                            }
                        }
                    }
                    VCPUAction::DispatchCallFromHost(args) => {
                        let mut hv_lock = hv_clone.lock().unwrap();

                        // Reset termination status from a possible previous execution
                        hv_lock.set_termination_status(false);
                        #[cfg(target_os = "linux")]
                        {
                            hv_lock.set_run_cancelled(false);
                        }

                        log::info!("Dispatching call from host: {}", args.function_name);

                        let res = {
                            #[cfg(feature = "function_call_metrics")]
                            {
                                let start = std::time::Instant::now();
                                let result = hv_lock.dispatch_call_from_host(
                                    args.dispatch_func_addr,
                                    args.outb_handle_fn,
                                    args.mem_access_fn,
                                );
                                histogram_vec_observe!(
                                    &GuestFunctionCallDurationMicroseconds,
                                    &[args.function_name.as_str()],
                                    start.elapsed().as_micros() as f64
                                );
                                result
                            }

                            #[cfg(not(feature = "function_call_metrics"))]
                            hv_lock.dispatch_call_from_host(
                                args.dispatch_func_addr,
                                args.outb_handle_fn,
                                args.mem_access_fn,
                            )
                        };

                        match res {
                            Ok(_) => {
                                from_handler_tx
                                    .send(HandlerMsg::FinishedVCPUAction)
                                    .map_err(|_| {
                                        HyperlightError::HypervisorHandlerCommunicationFailure()
                                    })?;
                            }
                            Err(e) => {
                                from_handler_tx.send(HandlerMsg::Error(e)).map_err(|_| {
                                    HyperlightError::HypervisorHandlerCommunicationFailure()
                                })?;
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    };

    Ok(join_handle)
}

/// Try `join` on `HypervisorHandler` thread for `max_execution_time` duration.
/// - Before attempting a join, this function checks if execution isn't already finished.
/// Note: This function call takes ownership of the `JoinHandle`.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn try_join_hypervisor_handler_thread<T>(sbox: &mut T) -> Result<()>
where
    T: Sandbox,
{
    let hv_wrapper = sbox.get_hypervisor_wrapper_mut();
    let timeout = hv_wrapper.max_execution_time;
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() >= timeout {
            log_then_return!(HyperlightError::Error(
                "Failed to join Hypervisor handler thread".to_string()
            ));
        }

        if let Some(handle) = sbox.get_hypervisor_handler_thread_mut().take() {
            if !handle.is_finished() {
                match handle.join() {
                    Ok(Ok(())) => return Ok(()),
                    Ok(Err(e)) => {
                        log_then_return!(e);
                    }
                    Err(e) => {
                        log_then_return!(new_error!("{:?}", e));
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

/// Tries to kill the Hypervisor Handler Thread.
///
/// All a Hypervisor Handler Thread does is continuously listen for messages. So, to finish the
/// thread's execution, we just need to `drop` the `to_handler_tx` channel. Once the thread realizes
/// that there are no more active senders, it will finish its execution.
///
/// To drop the `to_handler_tx` channel, we need to call `drop_to_handler_tx` on the `Hypervisor`, which
/// requires us to acquire a lock. This lock shouldn't hang, but, to be extra safe and avoid crashing,
/// we only try to acquire the lock for `max_execution_time` duration. If we can't acquire the lock in that
/// time, we will be leaking a thread.
#[instrument(err(Debug), skip_all, parent = Span::current(), level = "Trace")]
pub(crate) fn kill_hypervisor_handler_thread<T>(sbox: &mut T) -> Result<()>
where
    T: Sandbox,
{
    {
        let mut hv_lock = sbox
            .get_hypervisor_wrapper_mut()
            .try_get_hypervisor_lock_for_max_execution_time()?;
        hv_lock.drop_to_handler_tx();
    }

    try_join_hypervisor_handler_thread(sbox)
}

/// Terminate the execution of the hypervisor handler
///
/// This function is intended to be called after a guest function called has
/// timed-out (i.e., `from_handler_rx.recv_timeout(max_execution_time).is_err()`).
///
/// It is possible that, even after we timed-out, the guest function execution will
/// finish. If that is the case, this function is fundamentally a NOOP, because it
/// will restore the memory snapshot to the last state, and then re-initialise the
/// accidentally terminated vCPU.
///
/// This function, usually, will return one of the following HyperlightError's
/// - `ExecutionCanceledByHost` if the execution was successfully terminated, or
/// - `HypervisorHandlerExecutionCancelAttemptOnFinishedExecution` if the execution
///  finished while we tried to terminate it.
///
/// Hence, common usage of this function would be to match on the result. If you get a
/// `HypervisorHandlerExecutionCancelAttemptOnFinishedExecution`, you can safely ignore
/// retrieve the return value from shared memory.
#[allow(clippy::too_many_arguments)]
pub(crate) fn terminate_hypervisor_handler_execution_and_reinitialise<HvMemMgrT: WrapperGetter>(
    wrapper_getter: &mut HvMemMgrT,
    (to_handler_tx, from_handler_rx): (Sender<VCPUAction>, Receiver<HandlerMsg>),
    max_execution_time: Duration,
    termination_status: Arc<AtomicCell<bool>>,
    outb_hdl: OutBHandlerWrapper,
    mem_access_hdl: MemAccessHandlerWrapper,
    #[cfg(target_os = "linux")] thread_id: u64,
    #[cfg(target_os = "linux")] run_cancelled: Arc<AtomicCell<bool>>,
    #[cfg(target_os = "linux")] max_wait_for_cancellation: Duration,
) -> Result<HyperlightError> {
    let mem_mgr = wrapper_getter.get_mgr_wrapper_mut().unwrap_mgr_mut();

    // we allow this unused mut because it is only used on Linux
    #[allow(unused_mut)]
    let mut host_failed_to_cancel_guest_execution_sending_signals = 0;

    match crate::hypervisor::terminate_execution(
        max_execution_time,
        termination_status,
        #[cfg(target_os = "linux")]
        run_cancelled,
        #[cfg(target_os = "linux")]
        thread_id,
        #[cfg(target_os = "linux")]
        max_wait_for_cancellation,
    ) {
        Ok(_) => {}
        Err(e) => {
            match e {
                #[cfg(target_os = "linux")]
                HyperlightError::HostFailedToCancelGuestExecutionSendingSignals(val) => {
                    host_failed_to_cancel_guest_execution_sending_signals = val;
                }
                // ^^^ If we get here, this means that execution finished
                // while we tried to `terminate_execution`. We know this is
                // the case because, if we had successfully terminated execution,
                // the handler thread would have given us a `ExecutionCanceledByHost`.
                _ => return Err(e),
            }
        }
    };

    mem_mgr.restore_state_from_last_snapshot()?;

    // Receive again to get `ExecutionCanceledByHost` (or other)
    // error from handler thread
    let res = match from_handler_rx.recv_timeout(Duration::from_millis(1000)) {
        Ok(msg) => match msg {
            HandlerMsg::Error(e) => e,
            _ => new_error!("Unexpected response message from hypervisor handler"),
        },
        Err(_) => {
            // If we get here, this means that execution finished
            // while we tried to `terminate_execution`. We know this is
            // the case because, if we had successfully terminated execution,
            // the handler thread would have given us a `ExecutionCanceledByHost`.

            if host_failed_to_cancel_guest_execution_sending_signals != 0 {
                #[cfg(target_os = "linux")]
                {
                    HyperlightError::HostFailedToCancelGuestExecutionSendingSignals(
                        host_failed_to_cancel_guest_execution_sending_signals,
                    )
                }
                #[cfg(target_os = "windows")]
                {
                    new_error!("Failed to cancel guest execution")
                    // This is unreachable
                }
            } else {
                HyperlightError::HypervisorHandlerExecutionCancelAttemptOnFinishedExecution()
            }
        }
    };

    let seed = {
        let mut rng = rand::thread_rng();
        rng.gen::<u64>()
    };

    let peb_addr = {
        let peb_u64 = u64::try_from(mem_mgr.layout.peb_address)?;
        RawPtr::from(peb_u64)
    };
    let page_size = u32::try_from(get_os_page_size())?;

    to_handler_tx
        .send(VCPUAction::Initialise(InitArgs::new(
            peb_addr,
            seed,
            page_size,
            outb_hdl,
            mem_access_hdl,
        )))
        .map_err(|_| HyperlightError::HypervisorHandlerCommunicationFailure())?;

    // wait for hypervisor to finish re-initialising
    from_handler_rx
        .recv()
        .map_err(|_| HyperlightError::HypervisorHandlerCommunicationFailure())?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crossbeam::atomic::AtomicCell;
    use std::sync::Arc;
    use std::time::Duration;

    use crate::sandbox_state::sandbox::EvolvableSandbox;
    use crate::sandbox_state::transition::MutatingCallback;
    use crate::{
        is_hypervisor_present, GuestBinary, HyperlightError, MultiUseSandbox, UninitializedSandbox,
    };
    use hyperlight_common::flatbuffer_wrappers::function_types::{ParameterValue, ReturnType};
    use hyperlight_testing::simple_guest_as_string;

    use crate::hypervisor::hypervisor_handler::terminate_hypervisor_handler_execution_and_reinitialise;
    use crate::HyperlightError::HypervisorHandlerExecutionCancelAttemptOnFinishedExecution;

    use crate::sandbox::WrapperGetter;
    use crate::Result;

    fn create_multi_use_sandbox() -> MultiUseSandbox<'static> {
        if !is_hypervisor_present() {
            panic!("Panic on create_multi_use_sandbox because no hypervisor is present");
        }
        let usbox = UninitializedSandbox::new(
            GuestBinary::FilePath(simple_guest_as_string().expect("Guest Binary Missing")),
            None,
            None,
            None,
        )
        .unwrap();

        usbox.evolve(MutatingCallback::from(init)).unwrap()
    }

    fn init(_: &mut UninitializedSandbox) -> Result<()> {
        Ok(())
    }

    #[test]
    fn create_1000_sandboxes() {
        for _ in 0..1000 {
            create_multi_use_sandbox();
        }
    }

    #[test]
    fn terminate_execution_then_call_another_function() -> Result<()> {
        let mut sandbox = create_multi_use_sandbox();

        let res = sandbox.call_guest_function_by_name("Spin", ReturnType::Void, None);

        assert!(res.is_err());

        match res.err().unwrap() {
            HyperlightError::ExecutionCanceledByHost() => {}
            _ => panic!("Expected ExecutionTerminated error"),
        }

        let res = sandbox.call_guest_function_by_name(
            "Echo",
            ReturnType::String,
            Some(vec![ParameterValue::String("a".to_string())]),
        );

        assert!(res.is_ok());

        Ok(())
    }

    #[test]
    fn terminate_execution_of_an_already_finished_function_then_call_another_function() -> Result<()>
    {
        let call_echo = |sandbox: &mut MultiUseSandbox| {
            let res = sandbox.call_guest_function_by_name(
                "Echo",
                ReturnType::String,
                Some(vec![ParameterValue::String("a".to_string())]),
            );
            assert!(res.is_ok());
        };

        let mut sandbox = create_multi_use_sandbox();
        call_echo(&mut sandbox);

        // this simulates what would happen if a function call timed-out,
        // but was being cancelled
        {
            let (outb_hdl, mem_access_hdl) = {
                let hv_wrapper = sandbox.get_hv_mut();
                (
                    hv_wrapper.outb_hdl.clone(),
                    hv_wrapper.mem_access_hdl.clone(),
                )
            };

            let (to_handler_tx, from_handler_rx) = {
                let hv_lock = sandbox.get_hv_mut().get_hypervisor_lock()?;
                (hv_lock.get_to_handler_tx(), hv_lock.get_from_handler_rx())
            };

            #[cfg(target_os = "linux")]
            let thread_id = {
                let hv_lock = sandbox.get_hv_mut().get_hypervisor_lock()?;
                hv_lock.get_thread_id()
            };

            match terminate_hypervisor_handler_execution_and_reinitialise(
                &mut sandbox,
                (to_handler_tx.clone(), from_handler_rx.clone()),
                Duration::from_millis(1000),
                Arc::new(AtomicCell::new(true)),
                outb_hdl,
                mem_access_hdl,
                #[cfg(target_os = "linux")]
                thread_id,
                #[cfg(target_os = "linux")]
                Arc::new(AtomicCell::new(false)),
                #[cfg(target_os = "linux")]
                Duration::from_millis(100),
            )? {
                HypervisorHandlerExecutionCancelAttemptOnFinishedExecution() => {}
                #[cfg(target_os = "linux")]
                HyperlightError::HostFailedToCancelGuestExecutionSendingSignals(_) => {}
                _ => panic!("Expected error demonstrating execution wasn't cancelled properly"),
            }
        }

        call_echo(&mut sandbox);

        Ok(())
    }
}