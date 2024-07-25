use esp_hal::{analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess}, prelude::nb};

pub trait AdcExtension {
    // Copied from the adc header
    fn read_oneshot<PIN: AdcChannel>(&mut self, _pin: &mut AdcPin<PIN, impl RegisterAccess>) -> nb::Result<u16, ()>;
}

impl<'d, ADCI> AdcExtension for Adc<'d, ADCI> {
    fn read_oneshot<PIN: AdcChannel>(&mut self, _pin: &mut AdcPin<PIN, impl RegisterAccess>) -> nb::Result<u16, ()>
    {
        <Adc<'d, ADCI>>::read_oneshot::<PIN>(self, _pin)
    }
}