
use crate::rcc::Ccdr;
use crate::stm32::{DMA1, DMAMUX1};
pub use crate::stm32::dmamux1::ccr::DMAREQ_ID_A as DMAREQ_ID;


#[derive(Copy, Clone)]
pub enum Stream {
    One = 0,
    Two = 1,
}

pub struct Dma {
    dma: DMA1,
    dma_mux: DMAMUX1,
}

impl Dma {
    pub fn dma(dma: DMA1, mux: DMAMUX1, ccdr: &Ccdr) -> Self {
        ccdr.rb.ahb1enr.modify(|_, w| w.dma1en().set_bit());
        Dma {dma: dma, dma_mux: mux}
    }

    pub fn configure_m2p_stream(&mut self,
            stream: Stream,
            source_address: u32,
            dest_address: u32,
            trigger: DMAREQ_ID)
    {
        // Wait for the DMA stream to disable before modifying it.
        self.dma.st[stream as usize].cr.modify(|_, w| w.en().clear_bit());
        while self.dma.st[stream as usize].cr.read().en().bit_is_set() {};

        // Configure the peripheral and memory address of the transfer.
        self.dma.st[stream as usize].par.write(|w| w.pa().bits(dest_address));
        self.dma.st[stream as usize].m0ar.write(|w| w.m0a().bits(source_address));

        // Indicate that this DMA transfer is a single transfer.
        self.dma.st[stream as usize].ndtr.write(|w| w.ndt().bits(1));

        // Configure the triggering DMA request source.
        self.dma_mux.ccr[stream as usize].modify(|_, w| w.dmareq_id().variant(trigger));

        self.dma.st[stream as usize].cr.modify(|_, w|
                w.pl().medium()
                 .circ().enabled()
                 .msize().bits32()
                 .minc().fixed()
                 .mburst().single()
                 .psize().bits32()
                 .pinc().fixed()
                 .pburst().single()
                 .dbm().disabled()
                 .dir().memory_to_peripheral()
                 .pfctrl().dma()
        );

        // Disable direct mode for the FIFO on this channel.
        self.dma.st[stream as usize].fcr.modify(|_, w| w.dmdis().clear_bit());

        // Enable the DMA stream.
        self.dma.st[stream as usize].cr.modify(|_, w| w.en().set_bit());
    }
}
