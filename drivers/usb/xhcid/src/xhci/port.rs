use common::io::{Io, Mmio};
use tock_registers::{register_bitfields};
use tock_registers::registers::{ReadWrite, ReadOnly, WriteOnly};
use tock_registers::interfaces::{Readable, Writeable};
use power_states::process_register_block;

use crate::xhci::power_manager;

// RO - read-only
// ROS - read-only sticky
// RW - read/write
// RWS - read/write sticky
// RW1CS - read/write-1-to-clear sticky
// RW1S - read/write-1-to-set
// Sticky register values may preserve values through chip hardware reset
use log::info;

register_bitfields![
    u32, 
    pub PortFlags [
        CCS OFFSET(0) NUMBITS(1), // ROS
        PED OFFSET(1) NUMBITS(1), // RW1CS
        RSVD_2 OFFSET(2) NUMBITS(1), // Rsvd
        OCA OFFSET(3) NUMBITS(1), // RO
        PR OFFSET(4) NUMBITS(1), // RW1S
        PLS_0 OFFSET(5) NUMBITS(1), // RWS
        PLS_1 OFFSET(6) NUMBITS(1), // RWS
        PLS_2 OFFSET(7) NUMBITS(1), // RWS
        PLS_3 OFFSET(8) NUMBITS(1), // RWS
        PP OFFSET(9) NUMBITS(1), // RWS
        SPEED_0 OFFSET(10) NUMBITS(1), // ROS
        SPEED_1 OFFSET(11) NUMBITS(1), // ROS
        SPEED_2 OFFSET(12) NUMBITS(1), // ROS
        SPEED_3 OFFSET(13) NUMBITS(1), // ROS
        PIC_AMB OFFSET(14) NUMBITS(1), // RWS
        PIC_GRN OFFSET(15) NUMBITS(1), // RWS
        LWS OFFSET(16) NUMBITS(1), // RW
        CSC OFFSET(17) NUMBITS(1), // RW1CS      
        PEC OFFSET(18) NUMBITS(1), // RW1CS
        WRC OFFSET(19) NUMBITS(1), // RW1CS
        OCC OFFSET(20) NUMBITS(1), // RW1CS
        PRC OFFSET(21) NUMBITS(1), // RW1CS
        PLC OFFSET(22) NUMBITS(1), // RW1CS
        CEC OFFSET(23) NUMBITS(1), // RW1CS
        CAS OFFSET(24) NUMBITS(1), // RO
        WCE OFFSET(25) NUMBITS(1), // RWS
        WDE OFFSET(26) NUMBITS(1), // RWS
        WOE OFFSET(27) NUMBITS(1), // RWS
        RSVD_28 OFFSET(28) NUMBITS(1), // Rsvd
        RSVD_29 OFFSET(29) NUMBITS(1), // Rsvd
        DR OFFSET(30) NUMBITS(1), // RO
        WPR OFFSET(31) NUMBITS(1)  // RW1S    
    ]
];

/*
#[repr(C)]
#[process_register_block(
    peripheral_name = "Nrf5xTemp",
    register_base_addr = 0x4000C000,
    states = [
        Off => [Reading],
        Reading => [Off],
    ]
)]
struct RegisterBlock {
    /// Start temperature measurement
    /// Address: 0x000 - 0x004
    #[RegAttributes([Off], StateChange(Reading, [Task::ENABLE::SET]))]
    pub task_start: WriteOnly<u32, Task::Register>,
    /// Stop temperature measurement
    /// Address: 0x004 - 0x008
    #[RegAttributes([Reading], StateChange(Off, [Task::ENABLE::SET]))]
    pub task_stop: WriteOnly<u32, Task::Register>,
*/

/*
Off On(Disconnected)
On(Disconnected) 
On(Connected(Disabled))


*/

impl power_manager::SyncState for PortRegisters<Off> {
    type SyncStateEnum = PortStore;

    fn sync_state(self) -> Self::SyncStateEnum {
        unimplemented!();
       info!("sync state - PortRegisters<Off>");
        self.into()
        }
}

impl power_manager::SyncState for PortRegisters<Disconnected> {
    type SyncStateEnum = PortStore;

    fn sync_state(self) -> Self::SyncStateEnum {
        //info!("sync state - PortRegisters<Disconnected>");
        //info!("portsc address {:p}", &self.portsc);
        // print each bitfield value:


        // info!("portsc CCS: {}", self.portsc.reg.is_set(PortFlags::CCS));
        // info!("portsc CSC: {}", self.portsc.reg.is_set(PortFlags::CSC));
        // info!("portsc PR: {}", self.portsc.reg.is_set(PortFlags::PR));
        // info!("portsc PED: {}", self.portsc.reg.is_set(PortFlags::PED));
        // info!("portsc PP: {}", self.portsc.reg.is_set(PortFlags::PP));
        // info!("portsc SPEED0: {}", self.portsc.reg.is_set(PortFlags::SPEED_0));
        // info!("portsc SPEED1: {}", self.portsc.reg.is_set(PortFlags::SPEED_1));
        // info!("portsc SPEED2: {}", self.portsc.reg.is_set(PortFlags::SPEED_2));
        // info!("portsc SPEED3: {}", self.portsc.reg.is_set(PortFlags::SPEED_3));
        // info!("portsc PLS0: {}", self.portsc.reg.is_set(PortFlags::PLS_0));
        // info!("portsc PLS1: {}", self.portsc.reg.is_set(PortFlags::PLS_1));
        // info!("portsc PLS2: {}", self.portsc.reg.is_set(PortFlags::PLS_2));
        // info!("portsc PLS3: {}", self.portsc.reg.is_set(PortFlags::PLS_3));

        // info!("------------------------------------------");
        if self.portsc.reg.is_set(PortFlags::CCS) {
            // info!("xhcid portsc sync state - PortRegisters<Disconnected> - CCS is set");
            // clear flag
            self.portsc.reg.modify(PortFlags::CSC::CLEAR);
           unsafe {  PortStore::Disabled(core::mem::transmute::<_, PortRegisters<Disabled>>(self)) }
        } else {
            self.into()
        }
    }
}

impl power_manager::SyncState for PortRegisters<Reset> {
    type SyncStateEnum = PortStore;

    fn sync_state(self) -> Self::SyncStateEnum {
        //info!("sync state - PortRegisters<Reset>");
        if !self.portsc.reg.is_set(PortFlags::PRC) {
            // clear flag
            self.portsc.reg.modify(PortFlags::PRC::SET);
            unsafe { PortStore::Enabled(core::mem::transmute::<_, PortRegisters<Enabled>>(self)) }
        } else {
            self.into()
        }
    }
}

impl power_manager::SyncState for PortRegisters<Enabled> {
    type SyncStateEnum = PortStore;

    fn sync_state(self) -> Self::SyncStateEnum {
        // may go to diabled if pec (port error) or may go to disconnected with ccs flag set
        //info!("sync state - PortRegisters<Enabled>");
        if self.portsc.reg.is_set(PortFlags::PEC) {
            //info!("xhcid portsc sync state - PortRegisters<Enabled> - PEC is set");
            // clear flag
            self.portsc.reg.modify(PortFlags::PEC::CLEAR);
            unsafe { PortStore::Disabled(core::mem::transmute::<_, PortRegisters<Disabled>>(self)) }
        } else if self.portsc.reg.is_set(PortFlags::CCS) {
            //info!("xhcid portsc sync state - PortRegisters<Enabled> - CCS is set");
            // clear flag
            self.portsc.reg.modify(PortFlags::CSC::CLEAR);
            unsafe { PortStore::Disconnected(core::mem::transmute::<_, PortRegisters<Disconnected>>(self)) }
        } else {
            self.into()
        }
    }
}

impl power_manager::SyncState for PortRegisters<Disabled> {
    type SyncStateEnum = PortStore;

    fn sync_state(self) -> Self::SyncStateEnum {
        //info!("sync state - PortRegisters<Disabled>");
        if self.portsc.reg.is_set(PortFlags::CCS) {
           // info!("xhcid portsc sync state - PortRegisters<Disabled> - CCS is set");
            // clear flag
            self.portsc.reg.modify(PortFlags::CSC::CLEAR);
            unsafe { PortStore::Disconnected(core::mem::transmute::<_, PortRegisters<Disconnected>>(self)) }
        } else {
            self.into()
        }
    }
}

impl PortStore {
    pub fn get_state(&self) {
        //info!("PortStore get_state");
        // self.read_state();
       let val = match self {
            PortStore::Off(_) => {
            //    info!("PortStore is Off");
                return;
            }
            PortStore::Disconnected(port) => port.portsc.reg.get(),
            PortStore::Disabled(port) => port.portsc.reg.get(),
            PortStore::Reset(port) => port.portsc.reg.get(),
            PortStore::Enabled(port) => port.portsc.reg.get(),
       };

        // info!("PortStore read_state: portsc value: 0x{:08x}", val);
        // info!("PortStore read_state: CCS: {}", (val >> 0) & 1);
        // info!("PortStore read_state: PED: {}", (val >> 1) & 1);
        // info!("PortStore read_state: OCA: {}", (val >> 3) & 1);
        // info!("PortStore read_state: PR: {}", (val >> 4) & 1);
        // info!("PortStore read_state: PLS: {}", (val >> 5) & 0b1111);
        // info!("PortStore read_state: PP: {}", (val >> 9) & 1);
        // info!("PortStore read_state: SPEED: {}", (val >> 10) & 0b1111);
        // info!("PortStore read_state: PIC_AMB: {}", (val >> 14) & 1);
        // info!("PortStore read_state: PIC_GRN: {}", (val >> 15) & 1);
        // info!("PortStore read_state: LWS: {}", (val >> 16) & 1);
        // info!("PortStore read_state: CSC: {}", (val >> 17) & 1);
        // info!("PortStore read_state: PEC: {}", (val >> 18) & 1);
        // info!("PortStore read_state: WRC: {}", (val >> 19) & 1);
        // info!("PortStore read_state: OCC: {}", (val >> 20) & 1);
        // info!("PortStore read_state: PRC: {}", (val >> 21) & 1);
        // info!("PortStore read_state: PLC: {}", (val >> 22) & 1);
        // info!("PortStore read_state: Reserved: {}", (val >> 23) & 0b1111);
        // info!("PortStore read_state: CAS: {}", (val >> 24) & 1);
        // info!("PortStore read_state: WCE: {}", (val >> 25) & 1);
        // info!("PortStore read_state: WDE: {}", (val >> 26) & 1);
        // info!("PortStore read_state: WOE: {}", (val >> 27) & 1);
        // info!("PortStore read_state: DR: {}", (val >> 30) & 1);
        // info!("PortStore read_state: WPR: {}", (val >> 31) & 1);

    }
}

pub trait Resetable {
    fn into_reset(self) -> PortRegisters<Reset>;
}

impl Resetable for PortRegisters<Disabled> {
    fn into_reset(self) -> PortRegisters<Reset> {
        StepReset::into_reset(self)
    }
}

impl Resetable for PortRegisters<Enabled> {
    fn into_reset(self) -> PortRegisters<Reset> {
        StepReset::into_reset(self)
    
    }
}

// undefined in off / reset
pub trait GetState {
    fn state(&self) -> u8;
}

impl GetState for PortRegisters<Enabled> {
    fn state(&self) -> u8 {
        (self.portsc.reg.get() >> 5 & 0b1111) as u8
    }
}
impl GetState for PortRegisters<Disabled> {
    fn state(&self) -> u8 {
        (self.portsc.reg.get() >> 5 & 0b1111) as u8
    }
}
impl GetState for PortRegisters<Disconnected> {
    fn state(&self) -> u8 {
        (self.portsc.reg.get() >> 5 & 0b1111) as u8
    }
}


#[repr(C)]
#[process_register_block(
    peripheral_name = "Port",
    register_base_addr = 0x16440,
    states = [
        Off => [Disconnected],
        Disconnected => [Disabled, Off], 
        Reset => [Enabled, Off],
        Enabled => [Reset, Disabled, Off, Disconnected],
        Disabled => [Reset, Off, Disconnected]
    ]
)]
pub struct Port {
    // This has write one to clear fields, do not expose it, handle writes carefully!
    #[RegAttributes([Disconnected, Disabled, Reset, Enabled], StateChange(Off, [PortFlags::PP::CLEAR]))]
    #[RegAttributes([Off], StateChange(Disconnected, [PortFlags::PP::SET]))]

    #[RegAttributes([Disabled, Enabled], StateChange(Reset, [PortFlags::PP::SET]))]
    pub portsc: ReadWrite<u32, PortFlags::Register>,
    pub portpmsc: ReadWrite<u32>,
    pub portli: ReadWrite<u32>,
    pub porthlpmc: ReadWrite<u32>,
}

// bitflags! {
//     pub struct PortFlags: u32 {
//         const CCS = 1 << 0; // ROS
//         const PED = 1 << 1; // RW1CS
//         const RSVD_2 = 1 << 2; // RsvdZ
//         const OCA = 1 << 3; // RO
//         const PR =  1 << 4; // RW1S
//         const PLS_0 = 1 << 5; // RWS
//         const PLS_1 = 1 << 6; // RWS
//         const PLS_2 = 1 << 7; // RWS
//         const PLS_3 = 1 << 8; // RWS
//         const PP =  1 << 9; // RWS
//         const SPEED_0 =  1 << 10; // ROS
//         const SPEED_1 =  1 << 11; // ROS
//         const SPEED_2 =  1 << 12; // ROS
//         const SPEED_3 =  1 << 13; // ROS
//         const PIC_AMB = 1 << 14; // RWS
//         const PIC_GRN = 1 << 15; // RWS
//         const LWS = 1 << 16; // RW
//         const CSC = 1 << 17; // RW1CS
//         const PEC = 1 << 18; // RW1CS
//         const WRC = 1 << 19; // RW1CS
//         const OCC = 1 << 20; // RW1CS
//         const PRC = 1 << 21; // RW1CS
//         const PLC = 1 << 22; // RW1CS
//         const CEC = 1 << 23; // RW1CS
//         const CAS = 1 << 24; // RO
//         const WCE = 1 << 25; // RWS
//         const WDE = 1 << 26; // RWS
//         const WOE = 1 << 27; // RWS
//         const RSVD_28 = 1 << 28; // RsvdZ
//         const RSVD_29 = 1 << 29; // RsvdZ
//         const DR =  1 << 30; // RO
//         const WPR = 1 << 31; // RW1S
//     }
// }

// #[repr(C, packed)]
// pub struct Port {
//     // This has write one to clear fields, do not expose it, handle writes carefully!
//     portsc: Mmio<u32>,
//     pub portpmsc: Mmio<u32>,
//     pub portli: Mmio<u32>,
//     pub porthlpmc: Mmio<u32>,
// }

impl <S: State> PortRegisters<S> {
    pub fn read_state(&self) {
        let val = self.portsc.reg.get();

        info!("PortStore read_state: portsc value: 0x{:08x}", val);
        info!("PortStore read_state: CCS: {}", (val >> 0) & 1);
        info!("PortStore read_state: PED: {}", (val >> 1) & 1);
        info!("PortStore read_state: OCA: {}", (val >> 3) & 1);
        info!("PortStore read_state: PR: {}", (val >> 4) & 1);
        info!("PortStore read_state: PLS: {}", (val >> 5) & 0b1111);
        info!("PortStore read_state: PP: {}", (val >> 9) & 1);
        info!("PortStore read_state: SPEED: {}", (val >> 10) & 0b1111);
        info!("PortStore read_state: PIC_AMB: {}", (val >> 14) & 1);
        info!("PortStore read_state: PIC_GRN: {}", (val >> 15) & 1);
        info!("PortStore read_state: LWS: {}", (val >> 16) & 1);
        info!("PortStore read_state: CSC: {}", (val >> 17) & 1);
        info!("PortStore read_state: PEC: {}", (val >> 18) & 1);
        info!("PortStore read_state: WRC: {}", (val >> 19) & 1);
        info!("PortStore read_state: OCC: {}", (val >> 20) & 1);
        info!("PortStore read_state: PRC: {}", (val >> 21) & 1);
        info!("PortStore read_state: PLC: {}", (val >> 22) & 1);
        info!("PortStore read_state: Reserved: {}", (val >> 23) & 0b1111);
        info!("PortStore read_state: CAS: {}", (val >> 24) & 1);
        info!("PortStore read_state: WCE: {}", (val >> 25) & 1);
        info!("PortStore read_state: WDE: {}", (val >> 26) & 1);
        info!("PortStore read_state: WOE: {}", (val >> 27) & 1);
        info!("PortStore read_state: DR: {}", (val >> 30) & 1);
        info!("PortStore read_state: WPR: {}", (val >> 31) & 1);

    }
    pub fn read(&self) -> u32 {
        // panic!("we are a not working xhci");
        // self.portsc.get()
        unimplemented!()
    }

   // pub fn clear_csc(&mut self) {
   //     info!("xhcid portsc clear csc");
// //       self.portsc
// //           .write((self.flags_preserved() | PortFlags::CSC).bits());
   //     unimplemented!()
   // }

   // pub fn clear_prc(&mut self) {
   //     info!("xhcid portsc clear prc");
   // //    self.portsc
   // //        .write((self.flags_preserved() | PortFlags::PRC).bits());
   // }

   // pub fn set_pr(&mut self) {
   //     info!("xhcid portsc set pr");
   //     // self.portsc.modify(PortFlags::PR::SET);
   // }

    pub fn speed(&self) -> u8 {
        ((self.portsc.reg.get() & (0b1111 << 10)) >> 10) as u8
    }

    // pub fn flags(&self) -> PortFlags {
    //     info!("xhcid portsc flags");
    //    PortFlags::from_bits_truncate(self.read())
    //     unimplemented!()
    // }

    // Read only preserved flags
    //pub fn flags_preserved(&self) -> PortFlags {
    //    info!("xhcid portsc flags_preserved");
    //    // RO(S) and RW(S) bits should be preserved
    //    // RW1S and RW1CS bits should not
    //  //  let preserved = PortFlags::CCS
    //  //      | PortFlags::OCA
    //  //      | PortFlags::PLS_0
    //  //      | PortFlags::PLS_1
    //  //      | PortFlags::PLS_2
    //  //      | PortFlags::PLS_3
    //  //      | PortFlags::PP
    //  //      | PortFlags::SPEED_0
    //  //      | PortFlags::SPEED_1
    //  //      | PortFlags::SPEED_2
    //  //      | PortFlags::SPEED_3
    //  //      | PortFlags::PIC_AMB
    //  //      | PortFlags::PIC_GRN
    //  //      | PortFlags::WCE
    //  //      | PortFlags::WDE
    //  //      | PortFlags::WOE
    //  //      | PortFlags::DR;

    //  //  self.flags() & preserved
    //    unimplemented!()
    //}
}

