use crate::Register;
use core::mem;
use generic_array::{ArrayLength, GenericArray};
use hal::blocking::{i2c, spi};
use hal::digital::OutputPin;

/// Marker trait for mode
pub(crate) trait MpuMode {}
/// Accelerometer + Gyroscope + Temperature Sensor
pub struct Imu;
impl MpuMode for Imu {}

/// Magnetometer + Accelerometer + Gyroscope + Temperature Sensor
pub struct Marg;
impl MpuMode for Marg {}

/// Generic interface for communicating with sensor
pub trait Interface<E> {
    fn read_many<N>(&mut self, reg: Register) -> Result<GenericArray<u8, N>, E>
        where N: ArrayLength<u8>;

    fn write(&mut self, reg: Register, val: u8) -> Result<(), E>;
}

struct I2c<I2C>(I2C, u8);

impl<SPI, NCS, E> Interface<E> for (SPI, NCS)
    where SPI: spi::Write<u8, Error = E> + spi::Transfer<u8, Error = E>,
          NCS: OutputPin
{
    fn read_many<N>(&mut self, reg: Register) -> Result<GenericArray<u8, N>, E>
        where N: ArrayLength<u8>
    {
        let mut buffer: GenericArray<u8, N> = unsafe { mem::zeroed() };
        {
            let slice: &mut [u8] = &mut buffer;
            slice[0] = reg.read_address();
            self.1.set_low();
            self.0.transfer(slice)?;
            self.1.set_high();
        }

        Ok(buffer)
    }

    fn write(&mut self, reg: Register, val: u8) -> Result<(), E> {
        self.1.set_low();
        self.0.write(&[reg.write_address(), val])?;
        self.1.set_high();
        Ok(())
    }
}

impl<I2C, E> Interface<E> for I2c<I2C>
    where I2C: i2c::Write<Error = E>
              + i2c::Read<Error = E>
              + i2c::WriteRead<Error = E>
{
    fn read_many<N>(&mut self, reg: Register) -> Result<GenericArray<u8, N>, E>
        where N: ArrayLength<u8>
    {
        let mut buffer: GenericArray<u8, N> = unsafe { mem::zeroed() };
        self.0.write_read(self.1, &[reg.read_address()], &mut buffer)?;
        Ok(buffer)
    }

    fn write(&mut self, reg: Register, val: u8) -> Result<(), E> {
        self.0.write(self.1, &[reg.read_address(), val])?;
        Ok(())
    }
}
