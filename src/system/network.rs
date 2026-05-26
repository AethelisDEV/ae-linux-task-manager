use sysinfo::Networks;

#[derive(Clone, Default)]
pub struct NetworkStats {
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub interfaces: Vec<NetworkInterfaceInfo>,
    pub rx_history: Vec<f32>, // Rolling history of Rx speed in KB/s
    pub tx_history: Vec<f32>, // Rolling history of Tx speed in KB/s
}

#[derive(Clone, Default)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub total_rx: u64,
    pub total_tx: u64,
}

impl NetworkStats {
    pub fn new() -> Self {
        Self {
            rx_history: vec![0.0; 60],
            tx_history: vec![0.0; 60],
            ..Default::default()
        }
    }

    pub fn update(&mut self, networks: &mut Networks) {
        networks.refresh();

        let mut total_rx_sec = 0;
        let mut total_tx_sec = 0;
        let mut total_rx = 0;
        let mut total_tx = 0;
        let mut ifaces = Vec::new();

        for (name, data) in networks.iter() {
            // Under sysinfo, received() / transmitted() returns the bytes sent/received since last refresh
            let rx = data.received();
            let tx = data.transmitted();
            let rx_tot = data.total_received();
            let tx_tot = data.total_transmitted();

            total_rx_sec += rx;
            total_tx_sec += tx;
            total_rx += rx_tot;
            total_tx += tx_tot;

            ifaces.push(NetworkInterfaceInfo {
                name: name.clone(),
                rx_bytes_sec: rx,
                tx_bytes_sec: tx,
                total_rx: rx_tot,
                total_tx: tx_tot,
            });
        }

        self.rx_bytes_sec = total_rx_sec;
        self.tx_bytes_sec = total_tx_sec;
        self.total_rx_bytes = total_rx;
        self.total_tx_bytes = total_tx;
        self.interfaces = ifaces;

        // Maintain rolling history (convert to KB/s for the graph)
        let rx_kb = (total_rx_sec as f32) / 1024.0;
        let tx_kb = (total_tx_sec as f32) / 1024.0;

        self.rx_history.remove(0);
        self.rx_history.push(rx_kb);

        self.tx_history.remove(0);
        self.tx_history.push(tx_kb);
    }
}
