use std::{ffi::OsString, net::IpAddr, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{sleep, JoinHandle}, time::Duration};

use ipnetwork::IpNetwork;
use log::*;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use netstat::*;
use tokio::runtime::Runtime;
use anyhow::*;

use crate::{aws_iprange::{FakeIpRanges, IpPrefix}, models::Message};

pub struct ProcessWatcher {
    handle: Option<JoinHandle<Result<()>>>,
    close_flag: Arc<AtomicBool>,
}

impl ProcessWatcher {
    pub fn new() -> Self {
        Self {
            handle: None,
            close_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, process_name: &str, port: u16) -> Receiver<Message> {

        let (tx, rx) = std::sync::mpsc::channel::<Message>();
        let process_name = OsString::from(process_name);
        let close_flag = self.close_flag.clone();
        let handle = std::thread::spawn(move || Self::check_periodically(
            process_name,
            port,
            close_flag, tx));

        self.handle = Some(handle);

        rx
    }

    fn check_periodically(
        process_name: OsString,
        port: u16,
        close_flag: Arc<AtomicBool>,
        tx: Sender<Message>,
    ) -> Result<()> {
        let mut system = System::new_all();
        let check_timeout = Duration::from_secs(5);
        // let ip_range = AwsIpRange::new();
        let ip_range = FakeIpRanges::new();
        let rt = Runtime::new()?;
        let ip_ranges = rt.block_on(async { ip_range.get().await })?;
        let mut last_message = Message::Unknown;
        let mut process_id = None;
        sleep(check_timeout);

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
                    Self::send_message(&tx, &mut last_message, Message::ProcessRunning)?;

                    let ip_addrs = Self::find_process_ips(process_id.as_u32(), port)?;

                    if ip_addrs.is_empty() {
                        Self::send_message(&tx, &mut last_message, Message::ProcessNotListening)?;
                        sleep(check_timeout);
                        continue;
                    }
      
                    for ip_addr in &ip_addrs {
                        match Self::match_ip(&ip_ranges.prefixes, ip_addr)? {
                            Some(region) => {
                                Self::send_message(&tx, &mut last_message, Message::ProcessListening(region))?;
                            },
                            None => {
                                Self::send_message(&tx, &mut last_message, Message::ProcessNotListening)?;
                            },
                        }
                    }
                },
                None => {
                    Self::handle_process_stopped(&tx, &mut last_message)?;
                    
                },
            }

            sleep(check_timeout);
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
            let network: IpNetwork = prefix.ip_prefix.parse()?;
            if network.contains(*ip_addr) {
                return Ok(Some(prefix.region.clone()));
            }
        }
        Ok(None)
    }

    fn send_message(tx: &Sender<Message>, last_message: &mut Message, new_message: Message) -> Result<()> {

        let should_skip = new_message == Message::ProcessRunning
            && matches!(*last_message, Message::ProcessNotListening | Message::ProcessListening(_));

        if should_skip {
            return Ok(());
        }

        if *last_message != new_message {
            tx.send(new_message.clone())?;
            *last_message = new_message;
        }
        Ok(())
    }

    fn handle_process_stopped(tx: &Sender<Message>, last_message: &mut Message) -> Result<()> {
        let new_message = match last_message {
            Message::Unknown => Message::ProcessNotRunning,
            Message::ProcessNotRunning => return Ok(()),
            Message::ProcessRunning => Message::ProcesStopped,
            Message::ProcessNotListening => Message::ProcesStopped,
            Message::ProcessListening(_) => Message::ProcesStopped,
            Message::ProcesStopped => return Ok(()),
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
