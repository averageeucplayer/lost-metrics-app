use std::{ffi::OsString, net::IpAddr, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use ipnetwork::IpNetwork;
use log::*;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use netstat::*;
use tokio::runtime::Runtime;
use anyhow::*;

use crate::{aws_iprange::{FakeIpRanges, IpPrefix}, models::ProcessState};

pub struct ProcessWatcher {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>,
    check_interval: Duration
}

impl ProcessWatcher {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
            check_interval
        }
    }

    pub fn start(&mut self, process_name: &str, port: u16) -> Receiver<ProcessState> {

        let (tx, rx) = std::sync::mpsc::channel::<ProcessState>();
        let process_name = OsString::from(process_name);
        let close_flag = self.close_flag.clone();
        let check_interval = self.check_interval.clone();
        let handle = std::thread::spawn(move || Self::check_periodically(
            process_name,
            port,
            close_flag,
            tx,
            check_interval));

        self.handle = Some(handle);

        rx
    }

    fn check_periodically(
        process_name: OsString,
        port: u16,
        close_flag: Arc<AtomicBool>,
        tx: Sender<ProcessState>,
        check_interval: Duration
    ) -> Result<()> {
        let mut system = System::new_all();
        // let ip_range = AwsIpRange::new();
        let ip_range = FakeIpRanges::new();
        let rt = Runtime::new()?;
        let ip_ranges = rt.block_on(async { ip_range.get().await })?;
        let mut last_message = ProcessState::Unknown;
        let mut process_id = None;
        sleep(check_interval);

        while !close_flag.load(Ordering::Relaxed) {

            match process_id {
                Some(id) => {
                    let size = system.refresh_processes_specifics(
                        sysinfo::ProcessesToUpdate::Some(&vec![id]),
                        true,
                        ProcessRefreshKind::nothing());
                    if size == 0 {
                        process_id = None;
                    }
                },
                None => {
                    system.refresh_processes_specifics(ProcessesToUpdate::All, false, ProcessRefreshKind::everything());
                    let processes: Vec<_> = system.processes_by_name(&process_name).collect();
                    process_id = processes.first().map(|p| p.pid());
                },
            }

            match process_id {
                Some(process_id) => {
                    Self::send_message(&tx, &mut last_message, ProcessState::ProcessRunning)?;

                    let ip_addrs = Self::find_process_ips(process_id.as_u32(), port)?;

                    if ip_addrs.is_empty() {
                        Self::send_message(&tx, &mut last_message, ProcessState::ProcessNotListening)?;
                        sleep(check_interval);
                        continue;
                    }
      
                    for ip_addr in &ip_addrs {
                        match Self::match_ip(&ip_ranges.prefixes, ip_addr)? {
                            Some(region) => {
                                Self::send_message(&tx, &mut last_message, ProcessState::ProcessListening(region))?;
                            },
                            None => {
                                Self::send_message(&tx, &mut last_message, ProcessState::ProcessNotListening)?;
                            },
                        }
                    }
                },
                None => {
                    Self::handle_process_stopped(&tx, &mut last_message)?;
                    
                },
            }

            sleep(check_interval);
        }

        Ok(())
    }

    fn find_process_ips(process_id: u32, port: u16) -> Result<Vec<IpAddr>> {
        let address_family_flags = AddressFamilyFlags::IPV4;
        let proto = ProtocolFlags::TCP;

        let sockets = get_sockets_info(address_family_flags, proto)
            .ok()
            .unwrap_or_default();

        let ip_addrs = sockets.into_iter()
            .find(|socket| socket.associated_pids.contains(&process_id))
            .into_iter()
            .filter_map(|info| {
                if let ProtocolSocketInfo::Tcp(tcp) = info.protocol_socket_info {
                    (tcp.remote_port == port).then(|| tcp.remote_addr)
                } else {
                    None
                }
            })
            .collect();

        Ok(ip_addrs)
    }

    fn match_ip(prefixes: &[IpPrefix], ip_addr: &IpAddr) -> Result<Option<String>> {
        for prefix in prefixes {
            println!("{:?} {:?}", prefix, ip_addr);
            let network: IpNetwork = prefix.ip_prefix.parse()?;
            if network.contains(*ip_addr) {
                return Ok(Some(prefix.region.clone()));
            }
        }
        Ok(None)
    }

    fn send_message(tx: &Sender<ProcessState>, last_message: &mut ProcessState, new_message: ProcessState) -> Result<()> {

        let should_skip = new_message == ProcessState::ProcessRunning
            && matches!(*last_message, ProcessState::ProcessNotListening | ProcessState::ProcessListening(_));

        if should_skip {
            return Ok(());
        }

        if *last_message != new_message {
            tx.send(new_message.clone())?;
            *last_message = new_message;
        }
        Ok(())
    }

    fn handle_process_stopped(tx: &Sender<ProcessState>, last_message: &mut ProcessState) -> Result<()> {
        let new_message = match last_message {
            ProcessState::Unknown => ProcessState::ProcessNotRunning,
            ProcessState::ProcessNotRunning => return Ok(()),
            ProcessState::ProcessRunning => ProcessState::ProcesStopped,
            ProcessState::ProcessNotListening => ProcessState::ProcesStopped,
            ProcessState::ProcessListening(_) => ProcessState::ProcesStopped,
            ProcessState::ProcesStopped => return Ok(()),
        };

        Self::send_message(tx, last_message, new_message)
    }

    pub fn is_running(&self) -> bool {
        !self.close_flag.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.close_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|err| anyhow::anyhow!("{:?}", err))??;
        }

        Ok(())
    }
}
